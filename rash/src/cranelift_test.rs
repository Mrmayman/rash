use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use lazy_static::lazy_static;
use target_lexicon::Triple;
use types::I64;

use crate::data_types::{ScratchObject, ID_BOOL, ID_NUMBER, ID_STRING};

lazy_static! {
    pub static ref MEMORY: Box<[ScratchObject]> =
        vec![ScratchObject::Number(0.0); 1024].into_boxed_slice();
}

// #[no_mangle]
// extern "C" fn set_var(b1: i64, b2: i64, b3: i64, b4: i64) {
//     println!("Hello from rust!");
// }

pub struct Compiler {
    // pub json: JsonStruct,
    // pub project_dir: TempDir,
}

#[derive(Debug, Clone, Copy)]
pub struct Ptr(pub usize);

#[derive(Debug)]
pub enum ScratchBlock {
    WhenFlagClicked,
    SetVar(Ptr, ScratchObject),
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
                ScratchBlock::SetVar(Ptr(0), ScratchObject::Number(2.0)),
                ScratchBlock::SetVar(Ptr(1), ScratchObject::Bool(true)),
                ScratchBlock::SetVar(Ptr(2), ScratchObject::Bool(false)),
                ScratchBlock::SetVar(Ptr(3), ScratchObject::String("Hey".to_owned())),
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
                    match block {
                        ScratchBlock::WhenFlagClicked => {}
                        ScratchBlock::SetVar(ptr, obj) => {
                            let obj = if let ScratchObject::Pointer(ptr) = obj {
                                &MEMORY[*ptr]
                            } else {
                                obj
                            };
                            match obj {
                                ScratchObject::Number(num) => {
                                    ins_mem_write_f64(&mut builder, *ptr, *num);
                                }
                                ScratchObject::Bool(num) => {
                                    ins_mem_write_bool(&mut builder, *ptr, *num);
                                }
                                ScratchObject::String(string) => {
                                    let string = string.clone();

                                    // Transmute the String into a [i64; 4] array
                                    let arr: [i64; 3] = unsafe { std::mem::transmute(string) };
                                    let i1 = builder.ins().iconst(I64, arr[0]);
                                    let i2 = builder.ins().iconst(I64, arr[1]);
                                    let i3 = builder.ins().iconst(I64, arr[2]);

                                    let mem_ptr = builder.ins().iconst(
                                        I64,
                                        MEMORY.as_ptr() as i64
                                            + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
                                    );

                                    builder.ins().store(MemFlags::new(), i1, mem_ptr, 8);
                                    builder.ins().store(MemFlags::new(), i2, mem_ptr, 16);
                                    builder.ins().store(MemFlags::new(), i3, mem_ptr, 24);

                                    let id = builder.ins().iconst(I64, ID_STRING as i64);
                                    builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
                                }
                                _ => todo!(),
                            };
                        }
                    }
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

    /*fn get_block_code(&self) -> Vec<CodeSprite> {
        let mut code_sprites = Vec::new();
        for sprite in &self.json.targets {
            let mut code_sprite = CodeSprite {
                scripts: Vec::new(),
            };

            let orphaned_blocks = sprite
                .get_hat_blocks()
                .iter()
                .map(|(id, _)| (*id).clone())
                .collect::<Vec<_>>();

            for id in orphaned_blocks {
                let mut blocks = Vec::new();
                if let Some(block) = sprite.blocks.get(&id) {
                    match block {
                        crate::json::JsonBlock::Block { block } => {
                            self.collect_block(&id, &sprite, &mut blocks);
                        }
                        crate::json::JsonBlock::Array(vec) => todo!(),
                    }
                }
                println!("{blocks:#?}");
                code_sprite.scripts.push(blocks);
            }
            code_sprites.push(code_sprite);
        }
        code_sprites
    }

    pub fn collect_block(&self, id: &str, target: &Target, blocks: &mut Vec<ScratchBlock>) {
        let block = target.blocks.get(id).unwrap();
        match block {
            crate::json::JsonBlock::Block { block } => {
                // println!("Compiling block: {}", block.opcode);
                match block.opcode.as_str() {
                    "event_whenflagclicked" => {
                        blocks.push(ScratchBlock::WhenFlagClicked);
                    }
                    "data_setvariableto" => {
                        println!("{block:#?}");
                        // TODO: Implement setvar
                        blocks.push(ScratchBlock::SetVar(Ptr(0), ScratchObject::Number(2.0)));
                    }
                    _ => {
                        eprintln!("Unknown block: {}", block.opcode);
                    }
                }
                if let Some(next) = block.next.as_ref() {
                    self.collect_block(next, target, blocks);
                }
            }
            crate::json::JsonBlock::Array(_vec) => todo!(),
        }
    }*/
}

fn ins_mem_write_bool(builder: &mut FunctionBuilder<'_>, ptr: Ptr, num: bool) {
    let mem_ptr = builder.ins().iconst(
        I64,
        MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
    );

    let num = builder.ins().iconst(I64, num as i64);
    builder.ins().store(MemFlags::new(), num, mem_ptr, 8);

    let id = builder.ins().iconst(I64, ID_BOOL as i64);
    builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
}

fn ins_mem_write_f64(builder: &mut FunctionBuilder<'_>, ptr: Ptr, num: f64) {
    let mem_ptr = builder.ins().iconst(
        I64,
        MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
    );

    // write to the ptr
    let num = builder.ins().f64const(num);
    builder.ins().store(MemFlags::new(), num, mem_ptr, 8);

    let id = builder.ins().iconst(I64, ID_NUMBER as i64);
    builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
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
