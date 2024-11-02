use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use lazy_static::lazy_static;
use target_lexicon::Triple;
use types::I64;

use crate::{
    data_types::{ScratchObject, ID_NUMBER},
    input_primitives::{Input, Ptr, ReturnValue},
    ins_shortcuts::{ins_mem_write_bool, ins_mem_write_f64, ins_mem_write_string},
};

lazy_static! {
    pub static ref MEMORY: Box<[ScratchObject]> =
        vec![ScratchObject::Number(0.0); 1024].into_boxed_slice();
}

pub struct Compiler {
    // pub json: JsonStruct,
    // pub project_dir: TempDir,
}

#[derive(Debug)]
pub enum ScratchBlock {
    WhenFlagClicked,
    SetVar(Ptr, Input),
    OpAdd(Input, Input),
    OpSub(Input, Input),
    OpMul(Input, Input),
    OpDiv(Input, Input),
}

struct CodeSprite {
    pub scripts: Vec<Vec<ScratchBlock>>,
}

impl Compiler {
    pub fn new() -> Self {
        // let file_bytes = std::fs::read(path).unwrap();

        // let loaded_project_dir = tempfile::TempDir::new().unwrap();

        // zip_extract::extract(
        //     std::io::Cursor::new(file_bytes),
        //     loaded_project_dir.path(),
        //     false,
        // )
        // .unwrap();

        // let json = std::fs::read_to_string(loaded_project_dir.path().join("project.json")).unwrap();
        // let json: JsonStruct = serde_json::from_str(&json).unwrap();

        Self {
            // json,
            // project_dir: loaded_project_dir,
        }
    }

    pub fn compile(&self) {
        // println!("{}", std::mem::size_of::<ScratchObject>());
        let mut builder = settings::builder();
        builder.set("opt_level", "speed").unwrap();
        let flags = settings::Flags::new(builder);

        let isa = match isa::lookup(Triple::host()) {
            Err(err) => panic!("Error looking up target: {}", err),
            Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
        };

        // let code_sprites = self.get_block_code();
        let code_sprites = vec![CodeSprite {
            scripts: vec![vec![
                ScratchBlock::WhenFlagClicked,
                ScratchBlock::SetVar(Ptr(0), Input::Obj(ScratchObject::Number(2.0))),
                ScratchBlock::SetVar(Ptr(1), Input::Obj(ScratchObject::Bool(true))),
                ScratchBlock::SetVar(Ptr(2), Input::Obj(ScratchObject::Bool(false))),
                ScratchBlock::SetVar(Ptr(3), Input::Obj(ScratchObject::String("Hey".to_owned()))),
                ScratchBlock::SetVar(
                    Ptr(4),
                    Input::Block(Box::new(ScratchBlock::OpAdd(
                        Input::Obj(ScratchObject::Number(2.0)),
                        Input::Block(Box::new(ScratchBlock::OpMul(
                            Input::Obj(ScratchObject::String("3.0".to_owned())),
                            Input::Obj(ScratchObject::Number(4.0)),
                        ))),
                    ))),
                ),
                ScratchBlock::SetVar(
                    Ptr(5),
                    Input::Block(Box::new(ScratchBlock::OpSub(
                        Input::Obj(ScratchObject::Number(2.0)),
                        Input::Block(Box::new(ScratchBlock::OpDiv(
                            Input::Obj(ScratchObject::Number(3.0)),
                            Input::Obj(ScratchObject::Number(4.0)),
                        ))),
                    ))),
                ),
                ScratchBlock::SetVar(
                    Ptr(6),
                    Input::Block(Box::new(ScratchBlock::OpAdd(
                        Input::Block(Box::new(ScratchBlock::OpAdd(
                            Input::Obj(ScratchObject::Bool(true)),
                            Input::Obj(ScratchObject::Bool(true)),
                        ))),
                        Input::Block(Box::new(ScratchBlock::OpMul(
                            Input::Obj(ScratchObject::String("3.0".to_owned())),
                            Input::Obj(ScratchObject::Number(4.0)),
                        ))),
                    ))),
                ),
            ]],
        }];
        for sprite in &code_sprites {
            for script in &sprite.scripts {
                let sig = Signature::new(CallConv::SystemV);
                let mut func = Function::with_name_signature(UserFuncName::default(), sig);

                let mut func_ctx = FunctionBuilderContext::new();
                let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

                let block = builder.create_block();
                builder.seal_block(block);

                builder.append_block_params_for_function_params(block);
                builder.switch_to_block(block);

                for block in script {
                    compile_block(block, &mut builder);
                }

                let ins = builder.ins();
                ins.return_(&[]);

                builder.finalize();

                println!("{}", func.display());

                let mut ctx = codegen::Context::for_function(func);
                let mut plane = ControlPlane::default();
                let code = ctx.compile(&*isa, &mut plane).unwrap();

                let mut buffer = memmap2::MmapOptions::new()
                    .len(code.code_buffer().len())
                    .map_anon()
                    .unwrap();

                buffer.copy_from_slice(code.code_buffer());

                let buffer = buffer.make_exec().unwrap();

                unsafe {
                    let code_fn: unsafe extern "sysv64" fn() = std::mem::transmute(buffer.as_ptr());

                    code_fn()
                }
            }
        }
    }
}

pub fn compile_block(
    block: &ScratchBlock,
    builder: &mut FunctionBuilder<'_>,
) -> Option<ReturnValue> {
    match block {
        ScratchBlock::WhenFlagClicked => {}
        ScratchBlock::SetVar(ptr, obj) => {
            match obj {
                Input::Obj(obj) => match obj {
                    ScratchObject::Number(num) => {
                        ins_mem_write_f64(builder, *ptr, *num);
                    }
                    ScratchObject::Bool(num) => {
                        ins_mem_write_bool(builder, *ptr, *num);
                    }
                    ScratchObject::String(string) => {
                        ins_mem_write_string(string, builder, *ptr);
                    }
                },
                Input::Block(block) => {
                    // compile block
                    let val = compile_block(block, builder).unwrap();
                    match val {
                        ReturnValue::Num(value) | ReturnValue::Bool(value) => {
                            let mem_ptr = builder.ins().iconst(
                                I64,
                                MEMORY.as_ptr() as i64
                                    + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
                            );
                            builder.ins().store(MemFlags::new(), value, mem_ptr, 8);

                            let id = builder.ins().iconst(I64, ID_NUMBER as i64);
                            builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
                        }
                        ReturnValue::Object((i1, i2, i3, i4)) => {
                            let mem_ptr = builder.ins().iconst(
                                I64,
                                MEMORY.as_ptr() as i64
                                    + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
                            );

                            builder.ins().store(MemFlags::new(), i1, mem_ptr, 0);
                            builder.ins().store(MemFlags::new(), i2, mem_ptr, 8);
                            builder.ins().store(MemFlags::new(), i3, mem_ptr, 16);
                            builder.ins().store(MemFlags::new(), i4, mem_ptr, 24);
                        }
                    }
                }
            };
        }
        ScratchBlock::OpAdd(a, b) => {
            let a = a.get_number(builder);
            let b = b.get_number(builder);
            let res = builder.ins().fadd(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpSub(a, b) => {
            let a = a.get_number(builder);
            let b = b.get_number(builder);
            let res = builder.ins().fsub(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpMul(a, b) => {
            let a = a.get_number(builder);
            let b = b.get_number(builder);
            let res = builder.ins().fmul(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpDiv(a, b) => {
            let a = a.get_number(builder);
            let b = b.get_number(builder);
            let res = builder.ins().fdiv(a, b);
            return Some(ReturnValue::Num(res));
        }
    }
    None
}

pub fn c_main() {
    // let arg1 = std::env::args().nth(1).unwrap();
    // println!("opening dir {arg1}");
    let compiler = Compiler::new();
    compiler.compile();

    // print memory
    for (i, obj) in MEMORY.iter().enumerate().take(10) {
        println!("{}: {:?}", i, obj);
    }
}
