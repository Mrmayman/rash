use std::{collections::HashMap, path::Path};

use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use lazy_static::lazy_static;
use target_lexicon::Triple;
use tempfile::TempDir;
use types::I64;

use crate::json::JsonStruct;

#[derive(Clone, Copy)]
pub enum FunctionSignature {
    None(extern "C" fn()),
}

lazy_static! {
    static ref FUNCTIONS: HashMap<String, FunctionSignature> =
        HashMap::from_iter([("hi".to_owned(), FunctionSignature::None(hi))]);
}

fn get_function(name: &str) -> Option<FunctionSignature> {
    FUNCTIONS.get(name).cloned()
}

#[no_mangle]
extern "C" fn hi() {
    println!("Hello from rust!");
}

pub struct Compiler {
    pub json: JsonStruct,
    pub project_dir: TempDir,
}

impl Compiler {
    pub fn new(path: &Path) -> Self {
        let file_bytes = std::fs::read(path).unwrap();

        let loaded_project_dir = tempfile::TempDir::new().unwrap();

        zip_extract::extract(
            std::io::Cursor::new(file_bytes),
            loaded_project_dir.path(),
            false,
        )
        .unwrap();

        let json = std::fs::read_to_string(loaded_project_dir.path().join("project.json")).unwrap();
        let json: JsonStruct = serde_json::from_str(&json).unwrap();

        Self {
            json,
            project_dir: loaded_project_dir,
        }
    }
}

pub fn c_main() {
    let arg1 = std::env::args().nth(1).unwrap();
    println!("opening dir {arg1}");
    let compiler = Compiler::new(Path::new(&arg1));

    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(I64));
    sig.returns.push(AbiParam::new(I64));

    let mut func = Function::with_name_signature(UserFuncName::default(), sig);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
    let sig = builder.import_signature(Signature::new(CallConv::SystemV));

    let block = builder.create_block();
    builder.seal_block(block);

    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);

    let arg = builder.block_params(block)[0];
    let plus_one = builder.ins().iadd_imm(arg, 1);

    let call_func = get_function("hi").unwrap();
    match call_func {
        FunctionSignature::None(func) => {
            let func = builder.ins().iconst(I64, func as i64);
            builder.ins().call_indirect(sig, func, &[]);
        }
    }
    builder.ins().return_(&[plus_one]);

    builder.finalize();

    println!("{}", func.display());

    let builder = settings::builder();
    let flags = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {}", err),
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
    };

    let mut ctx = codegen::Context::for_function(func);
    let mut plane = ControlPlane::default();
    let code = ctx.compile(&*isa, &mut plane).unwrap();

    let mut buffer = memmap2::MmapOptions::new()
        .len(code.code_buffer().len())
        .map_anon()
        .unwrap();

    buffer.copy_from_slice(code.code_buffer());

    let buffer = buffer.make_exec().unwrap();

    let x = unsafe {
        let code_fn: unsafe extern "sysv64" fn(usize, *const i64) -> usize =
            std::mem::transmute(buffer.as_ptr());

        code_fn(69, hi as *const i64)
    };

    println!("out: {}", x);
}
