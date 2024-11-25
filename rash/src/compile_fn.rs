use std::sync::Arc;

use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::{CallConv, TargetIsa};
use target_lexicon::Triple;
use types::I64;

use crate::{
    compiler::{Compiler, ScratchBlock, MEMORY},
    scheduler::{ScratchThread, SpriteId},
};

pub fn compile(script: &[ScratchBlock], id: SpriteId) -> ScratchThread {
    let isa = get_isa();

    let mut func = create_function();
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let jmp1_block = builder.create_block();
    builder.append_block_param(jmp1_block, I64);

    let jmp2_block = builder.create_block();
    builder.append_block_params_for_function_params(jmp2_block);
    builder.switch_to_block(jmp2_block);
    let param = builder.block_params(jmp2_block)[0];
    let repeat_stack_ptr = builder.block_params(jmp2_block)[1];
    builder.ins().jump(jmp1_block, &[param]);

    let code_block = builder.create_block();
    builder.switch_to_block(code_block);

    let lock = MEMORY.lock().unwrap();

    let mut compiler = Compiler::new(code_block, &mut builder, script, &lock, repeat_stack_ptr);

    compiler
        .cache
        .init(&mut builder, &lock, &mut compiler.constants);

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

    compile_ir(func, &isa, id)
}

fn compile_ir(func: Function, isa: &Arc<dyn TargetIsa>, id: SpriteId) -> ScratchThread {
    let mut ctx = codegen::Context::for_function(func);
    let mut plane = ControlPlane::default();
    ctx.optimize(isa.as_ref(), &mut plane).unwrap();

    let code = ctx.compile(&**isa, &mut plane).unwrap();

    ScratchThread::new(code.code_buffer(), id)
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

fn create_function() -> Function {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(I64));
    sig.params.push(AbiParam::new(I64));
    sig.returns.push(AbiParam::new(I64));
    Function::with_name_signature(UserFuncName::default(), sig)
}
