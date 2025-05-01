use std::sync::Arc;

use cranelift::{
    codegen::{
        self,
        control::ControlPlane,
        ir::{Function, UserFuncName},
    },
    prelude::{
        isa::{self, CallConv, TargetIsa},
        settings,
        types::I64,
        AbiParam, Block, Configurable, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC,
        MemFlags, Signature,
    },
};
use target_lexicon::Triple;

use crate::{
    compiler::{Compiler, ScratchBlock, MEMORY},
    scheduler::ScratchThread,
};
use rash_render::SpriteId;

pub fn compile(
    script: &[ScratchBlock],
    id: SpriteId,
    num_args: usize,
    is_screen_refresh: bool,
) -> ScratchThread {
    println!();
    for block in script {
        println!("{}", block.format(0));
    }
    println!();

    let isa = get_isa();

    let mut func = create_function(&*isa);
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let jmp1_block = builder.create_block();
    builder.append_block_param(jmp1_block, I64);

    let jmp2_block = builder.create_block();
    builder.append_block_params_for_function_params(jmp2_block);
    builder.switch_to_block(jmp2_block);

    let jump_id = builder.block_params(jmp2_block)[0];

    let repeat_stack_ptr = builder.block_params(jmp2_block)[1];
    let args_ptr = builder.block_params(jmp2_block)[2];
    let script_ptr = builder.block_params(jmp2_block)[3];
    let graphics_ptr = builder.block_params(jmp2_block)[4];

    let is_called_as_refresh = builder.block_params(jmp2_block)[5];
    let child_thread_ptr = builder.block_params(jmp2_block)[6];

    // For a function to be screen-refresh capable
    // (pausable), it must both inherently be screen refresh
    // AND must be called by a screen refresh function.
    //
    // If a pausable function is called by a non-pausable
    // function then it will run as non-pausable.
    let is_called_as_refresh = if is_screen_refresh {
        is_called_as_refresh
    } else {
        builder.ins().iconst(I64, 0)
    };

    let mut args_list = Vec::new();
    for _ in 0..num_args {
        let i1 = builder.ins().load(I64, MemFlags::new(), args_ptr, 0);
        let i2 = builder.ins().load(I64, MemFlags::new(), args_ptr, 8);
        let i3 = builder.ins().load(I64, MemFlags::new(), args_ptr, 16);
        let i4 = builder.ins().load(I64, MemFlags::new(), args_ptr, 24);

        args_list.push([i1, i2, i3, i4]);
    }

    builder.ins().jump(jmp1_block, &[jump_id]);

    let code_block = builder.create_block();
    builder.switch_to_block(code_block);

    let lock = MEMORY.lock().unwrap();

    let mut compiler = Compiler::new(
        code_block,
        &mut builder,
        script,
        &lock,
        repeat_stack_ptr,
        script_ptr,
        graphics_ptr,
        args_list,
        id,
        is_screen_refresh,
        is_called_as_refresh,
        child_thread_ptr,
    );

    compiler
        .cache
        .init(&mut builder, &mut compiler.constants, &lock);

    compiler.break_points.push(code_block);

    for block in script {
        compiler.compile_block(block, &mut builder);
    }

    compiler
        .cache
        .save(&mut builder, &mut compiler.constants, &lock);

    let return_value = builder.ins().iconst(I64, -1);
    builder.ins().return_(&[return_value]);

    prepare_screen_refresh_points(&compiler, &mut builder, jmp1_block);

    builder.seal_all_blocks();
    builder.finalize();

    println!("{}", func.display());

    compile_ir(func, &isa, id, compiler.is_screen_refresh)
}

fn compile_ir(
    func: Function,
    isa: &Arc<dyn TargetIsa>,
    id: SpriteId,
    is_screen_refresh: bool,
) -> ScratchThread {
    let mut ctx = codegen::Context::for_function(func);
    let mut plane = ControlPlane::default();
    ctx.optimize(isa.as_ref(), &mut plane).unwrap();

    let code = ctx.compile(&**isa, &mut plane).unwrap();

    // TODO: Implement arguments
    ScratchThread::new(code.code_buffer(), id, is_screen_refresh)
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
        builder.ins().brif(cmp, *point, &[], jmp1_block, &[param]);
    }

    builder.switch_to_block(jmp1_block);
    let return_value = builder.ins().iconst(I64, -1);
    builder.ins().return_(&[return_value]);
}

fn get_isa() -> Arc<dyn TargetIsa> {
    let mut builder = settings::builder();
    builder.set("opt_level", "speed").unwrap();
    let flags = settings::Flags::new(builder);

    match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {err}"),
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
    }
}

fn create_function(isa: &dyn TargetIsa) -> Function {
    let mut sig = Signature::new(CallConv::triple_default(isa.triple()));
    sig.params.push(AbiParam::new(I64)); // Jump ID
    sig.params.push(AbiParam::new(I64)); // Repeat Stack
    sig.params.push(AbiParam::new(I64)); // Args pointer
    sig.params.push(AbiParam::new(I64)); // Scripts
    sig.params.push(AbiParam::new(I64)); // RunState
    sig.params.push(AbiParam::new(I64)); // Is Screen Refresh?
    sig.params.push(AbiParam::new(I64)); // Child Thread (*mut Option<ScratchThread>)
    sig.returns.push(AbiParam::new(I64));
    Function::with_name_signature(UserFuncName::default(), sig)
}
