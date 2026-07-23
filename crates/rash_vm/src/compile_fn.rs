use std::sync::Arc;

use cranelift::{
    codegen::{
        self, CompiledCode, FinalizedRelocTarget,
        binemit::Reloc,
        control::ControlPlane,
        ir::{
            ExternalName, Function, LibCall, StackSlotData, StackSlotKind, UserFuncName, types::I8,
        },
    },
    prelude::{
        AbiParam, Block, Configurable, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC,
        MemFlags, Signature,
        isa::{CallConv, TargetIsa},
        settings,
        types::I64,
    },
};
use smol_str::SmolStr;

use crate::{
    callbacks::{self, declare_callbacks},
    compiler::{Compiler, FuncMap, ScratchBlock},
    data_types::ScratchObject,
    graphics::SpriteId,
    runtime::ScratchThread,
};

pub fn compile(
    script: &[ScratchBlock],
    memory: &[ScratchObject],
    id: SpriteId,
    num_args: usize,
    is_screen_refresh: bool,
) -> (ScratchThread, Vec<SmolStr>) {
    println!();
    for block in script {
        println!("{}", block.format(0));
    }
    println!();

    let isa = get_isa();

    let (mut func, call_conv) = create_function(&*isa);
    let func_map = declare_callbacks(&mut func);
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let jmp1_block = builder.create_block();
    builder.append_block_param(jmp1_block, I64);

    let jmp2_block = builder.create_block();
    builder.append_block_params_for_function_params(jmp2_block);
    builder.switch_to_block(jmp2_block);

    let fn_args = builder.block_params(jmp2_block);
    let jump_id = fn_args[0];

    let repeat_stack_ptr = fn_args[1];
    let args_ptr = fn_args[2];
    let script_ptr = fn_args[3];
    let graphics_ptr = fn_args[4];

    let child_thread_ptr = fn_args[6];

    // For a function to be screen-refresh capable
    // (pausable), it must both inherently be screen refresh
    // AND must be called by a screen refresh function.
    //
    // If a pausable function is called by a non-pausable
    // function then it will run as non-pausable.
    let is_called_as_refresh = if is_screen_refresh {
        fn_args[5]
    } else {
        builder.ins().iconst(I8, 0)
    };

    let mut args_list = Vec::new();
    for _ in 0..num_args {
        let i1 = builder.ins().load(I64, MemFlags::new(), args_ptr, 0);
        let i2 = builder.ins().load(I64, MemFlags::new(), args_ptr, 8);
        let i3 = builder.ins().load(I64, MemFlags::new(), args_ptr, 16);
        let i4 = builder.ins().load(I64, MemFlags::new(), args_ptr, 24);

        args_list.push([i1, i2, i3, i4]);
    }

    let temp_slot4 = create_main_slot(&mut builder);
    builder.ins().jump(jmp1_block, &[jump_id.into()]);

    let mut compiler = Compiler::new(
        false,
        &mut builder,
        script,
        memory,
        repeat_stack_ptr,
        script_ptr,
        graphics_ptr,
        args_list,
        id,
        is_screen_refresh,
        is_called_as_refresh,
        child_thread_ptr,
        temp_slot4,
        func_map,
        call_conv,
    );

    for block in script {
        compiler.compile_block(block, &mut builder);
    }

    compiler
        .cache
        .save(&mut builder, &mut compiler.constants, compiler.memory);

    let return_value = builder.ins().iconst(I64, -1);
    builder.ins().return_(&[return_value]);

    prepare_screen_refresh_points(&compiler, &mut builder, jmp1_block);

    builder.seal_all_blocks();
    builder.finalize();

    if std::env::var("RASH_PRINT_IR").is_ok_and(|n| n == "1" || n.eq_ignore_ascii_case("true")) {
        println!("{}", func.display());
    }

    (
        compile_ir(
            func,
            &compiler.func_store.func_map,
            &isa,
            id,
            compiler.is_screen_refresh,
        ),
        compiler.static_strings,
    )
}

pub fn create_main_slot(
    builder: &mut FunctionBuilder<'_>,
) -> (cranelift::prelude::Value, codegen::ir::StackSlot) {
    let slot = builder.create_sized_stack_slot(StackSlotData {
        kind: StackSlotKind::ExplicitSlot,
        size: 4 * std::mem::size_of::<i64>() as u32,
        align_shift: 0,
        key: None,
    });
    let stack_addr = builder.ins().stack_addr(I64, slot, 0);
    (stack_addr, slot)
}

fn compile_ir(
    func: Function,
    func_map: &FuncMap,
    isa: &Arc<dyn TargetIsa>,
    id: SpriteId,
    is_screen_refresh: bool,
) -> ScratchThread {
    let mut ctx = codegen::Context::for_function(func);
    let mut plane = ControlPlane::default();
    ctx.optimize(isa.as_ref(), &mut plane).unwrap();

    let code = ctx.compile(&**isa, &mut plane).unwrap();

    ScratchThread::new(code, func_map, id, is_screen_refresh)
}

fn prepare_screen_refresh_points(
    compiler: &Compiler<'_>,
    builder: &mut FunctionBuilder<'_>,
    mut jmp1_block: Block,
) {
    for (i, point) in compiler.break_points.iter().enumerate() {
        builder.switch_to_block(jmp1_block);
        let param = builder.block_params(jmp1_block)[0];
        let cmp = builder.ins().icmp_imm(IntCC::Equal, param, i as i64);
        jmp1_block = builder.create_block();
        builder.append_block_param(jmp1_block, I64);
        builder
            .ins()
            .brif(cmp, *point, &[], jmp1_block, &[param.into()]);
    }

    builder.switch_to_block(jmp1_block);
    let return_value = builder.ins().iconst(I64, -1);
    builder.ins().return_(&[return_value]);
}

pub fn get_isa() -> Arc<dyn TargetIsa> {
    let mut builder = settings::builder();
    builder.set("opt_level", "speed").unwrap();
    let flags = settings::Flags::new(builder);

    let isa_builder = cranelift_native::builder()
        .unwrap_or_else(|msg| panic!("host machine not supported: {msg}"));

    isa_builder.finish(flags).unwrap()
}

fn create_function(isa: &dyn TargetIsa) -> (Function, CallConv) {
    let call_conv = CallConv::triple_default(isa.triple());
    let mut sig = Signature::new(call_conv);
    sig.params.push(AbiParam::new(I64)); // Jump ID
    sig.params.push(AbiParam::new(I64)); // Repeat Stack
    sig.params.push(AbiParam::new(I64)); // Args pointer
    sig.params.push(AbiParam::new(I64)); // Scripts
    sig.params.push(AbiParam::new(I64)); // RunState
    sig.params.push(AbiParam::new(I8)); // Is Screen Refresh?
    sig.params.push(AbiParam::new(I64)); // Child Thread (*mut Option<ScratchThread>)
    sig.returns.push(AbiParam::new(I64));
    (
        Function::with_name_signature(UserFuncName::default(), sig),
        call_conv,
    )
}

pub fn prepare_buffer(code: &CompiledCode, buffer: &memmap2::MmapMut, func_map: &FuncMap) {
    let start_ptr = buffer.as_ptr();
    let end_ptr = unsafe { start_ptr.add(buffer.len()) };

    for reloc in code.buffer.relocs() {
        let FinalizedRelocTarget::ExternalName(target_name) = &reloc.target else {
            continue;
        };
        let target_addr: usize = match target_name {
            ExternalName::LibCall(LibCall::FloorF64) => callbacks::op::floor as *const () as usize,
            ExternalName::LibCall(other) => {
                panic!("unhandled libcall: {other:?}")
            }
            ExternalName::User(name_ref) => {
                let name = func_map.get_by_left(name_ref).unwrap();
                let funcs = &callbacks::FUNCS;
                let func = funcs.get(name).unwrap();
                *func as usize
            }
            _ => panic!("unhandled external name: {target_name:?}"),
        };

        let offset = reloc.offset as usize;
        assert!(
            offset < buffer.len(),
            "Relocation offset {offset} is out of mmap bounds!"
        );
        let at = unsafe { start_ptr.add(offset) };

        // Safety: THIS SHIT IS COMPLETELY UNSAFE, you're on your own lol
        match reloc.kind {
            // A lot of this is lifted straight from Cranelift's own JIT backend
            Reloc::X86CallPCRel4 | Reloc::X86PCRel4 => {
                assert!(
                    offset + 4 <= buffer.len(),
                    "Not enough space for i32 relocation"
                );
                // Don't recompute this by hand, the +/-4 adjustment for
                // "PC-relative to next instruction" is already folded into
                // reloc.addend (it comes in as -4), so it's just:
                let what = (target_addr as isize).wrapping_add(reloc.addend as isize);
                let pcrel_long = what.wrapping_sub(at as isize);
                let pcrel = i32::try_from(pcrel_long)
                    .expect("x86 PC-relative call target out of +/-2GB range!");
                unsafe { std::ptr::write_unaligned(at as *mut i32, pcrel) };
            }
            Reloc::Abs8 => {
                assert!(
                    offset + 8 <= buffer.len(),
                    "Not enough space for u64 relocation"
                );
                let what = (target_addr as u64).wrapping_add(reloc.addend as u64);
                unsafe { std::ptr::write_unaligned(at as *mut u64, what) };
            }
            Reloc::Arm64Call => {
                assert!(
                    offset + 4 <= buffer.len(),
                    "Not enough space for ARM64 relocation"
                );
                let what = (target_addr as isize).wrapping_add(reloc.addend as isize);
                let pc = at as isize;
                // AArch64 PC-relative branches are relative to
                // the branch instruction's own address — no
                // +8 pipeline quirk like legacy A32/T32 has.
                let byte_offset = what.wrapping_sub(pc);
                assert_eq!(byte_offset & 0b11, 0, "misaligned arm64 call target");

                let word_offset = byte_offset >> 2;
                assert!(
                    (-(1 << 25)..(1 << 25)).contains(&word_offset),
                    "arm64 call target out of +/-128MB range"
                );

                unsafe {
                    let instr = std::ptr::read_unaligned(at.cast::<u32>());
                    let imm26 = (word_offset as u32) & 0x03FF_FFFF;
                    let patched = (instr & 0xFC00_0000) | imm26; // keep opcode bits [31:26]
                    std::ptr::write_unaligned(at as *mut u32, patched);
                }
            }
            other => panic!("unhandled reloc kind {other:?} for a libcall"),
        }
    }

    unsafe {
        if !clear_cache::clear_cache(start_ptr, end_ptr) {
            eprintln!("WARNING: Failed to clear_cache");
        }
    };
}
