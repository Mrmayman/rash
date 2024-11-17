use std::sync::MutexGuard;

use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use target_lexicon::Triple;

use crate::{
    compiler::{Compiler, ScratchBlock, MEMORY},
    data_types::ScratchObject,
};

fn run(program: &[ScratchBlock], memory: &[ScratchObject]) {
    let mut builder = settings::builder();
    builder.set("opt_level", "speed").unwrap();
    let flags = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {err}"),
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
    };

    let sig = Signature::new(CallConv::SystemV);
    let mut func = Function::with_name_signature(UserFuncName::default(), sig);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let code_block = builder.create_block();

    builder.append_block_params_for_function_params(code_block);
    builder.switch_to_block(code_block);

    let mut compiler = Compiler::new(code_block, &mut builder, program);
    compiler
        .cache
        .init(&mut builder, memory, &mut compiler.constants);

    for block in program {
        compiler.compile_block(block, &mut builder, memory);
    }

    compiler
        .cache
        .save(&mut builder, &mut compiler.constants, memory);

    builder.seal_all_blocks();

    let ins = builder.ins();
    ins.return_(&[]);

    builder.finalize();

    // println!("{}", func.display());

    let mut ctx = codegen::Context::for_function(func);
    let mut plane = ControlPlane::default();
    ctx.optimize(isa.as_ref(), &mut plane).unwrap();

    let code = ctx.compile(&*isa, &mut plane).unwrap();

    let mut buffer = memmap2::MmapOptions::new()
        .len(code.code_buffer().len())
        .map_anon()
        .unwrap();

    buffer.copy_from_slice(code.code_buffer());

    // Machine code dump
    // let ptr = buffer.as_ptr();
    // let bytes = unsafe { std::slice::from_raw_parts(ptr, code.code_buffer().len()) };
    // for (_i, byte) in bytes.iter().enumerate() {
    //     print!("{:#04x} ", byte);
    // }
    // println!();
    // std::fs::write("func.bin", bytes).unwrap();

    let buffer = buffer.make_exec().unwrap();

    unsafe {
        let code_fn: unsafe extern "sysv64" fn() = std::mem::transmute(buffer.as_ptr());
        code_fn();
    }
}

#[allow(unused)]
pub fn run_code<'a>(code: Vec<ScratchBlock>) -> MutexGuard<'a, Box<[ScratchObject]>> {
    let mut memory = MEMORY.lock().unwrap();
    *memory = vec![ScratchObject::Number(0.0); 256].into_boxed_slice();
    run(&code, &memory);
    memory
}
