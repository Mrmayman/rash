use std::sync::MutexGuard;

use cranelift::{
    codegen::{
        self,
        control::ControlPlane,
        ir::{Function, UserFuncName},
    },
    prelude::{
        isa::{self, CallConv},
        settings,
        types::I64,
        AbiParam, Configurable, FunctionBuilder, FunctionBuilderContext, InstBuilder, Signature,
    },
};
use rash_render::SpriteId;
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

    let mut sig = Signature::new(CallConv::triple_default(isa.triple()));
    sig.params.push(AbiParam::new(I64));
    sig.returns.push(AbiParam::new(I64));
    let mut func = Function::with_name_signature(UserFuncName::default(), sig);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let code_block = builder.create_block();

    builder.append_block_params_for_function_params(code_block);
    builder.switch_to_block(code_block);
    let vec_ptr = builder.block_params(code_block)[0];
    let zero = builder.ins().iconst(I64, 0);
    let mut compiler = Compiler::new(
        code_block,
        &mut builder,
        program,
        memory,
        vec_ptr,
        zero,
        zero,
        Vec::new(),
        SpriteId(0),
        false,
        zero,
        zero,
    );
    compiler
        .cache
        .init(&mut builder, &mut compiler.constants, memory);

    for block in program {
        compiler.compile_block(block, &mut builder);
    }

    compiler
        .cache
        .save(&mut builder, &mut compiler.constants, memory);

    builder.seal_all_blocks();

    let minus_one = builder.ins().iconst(I64, -1);
    builder.ins().return_(&[minus_one]);

    builder.finalize();

    println!("{}", func.display());

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
        let code_fn: unsafe extern "C" fn(*mut Vec<i64>) = std::mem::transmute(buffer.as_ptr());
        let mut stack = Vec::new();
        code_fn(&mut stack);
    }
}

/// Simple headless runner for the JIT, limited in scope.
/// Takes in an array of ScratchBlock operations, compiles and executes them.
///
/// This is only used for the test-suite.
///
/// Doesn't support:
/// - Screen refresh (pausable functions)
/// - Custom blocks (calling other functions)
/// - Graphical or audio operations
/// - Environment operations (input/sensing)
/// - Timer operations
///
/// # Safety
/// As long as the compiler is functioning correctly,
/// this will be safe, as the machine code under correct
/// circumstances would function correctly.
#[allow(unused)]
pub fn run_code<'a>(code: &[ScratchBlock]) -> MutexGuard<'a, Box<[ScratchObject]>> {
    let mut memory = MEMORY.lock().unwrap();
    *memory = vec![ScratchObject::Number(0.0); 65536].into_boxed_slice();
    run(code, &memory);
    memory
}
