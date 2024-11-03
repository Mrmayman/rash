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
    callbacks,
    data_types::{self, ScratchObject, ID_NUMBER, ID_STRING},
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
    VarSet(Ptr, Input),
    VarRead(Ptr),
    OpAdd(Input, Input),
    OpSub(Input, Input),
    OpMul(Input, Input),
    OpDiv(Input, Input),
    OpJoin(Input, Input),
    ControlRepeat(Input, Vec<ScratchBlock>),
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
                // ScratchBlock::VarSet(Ptr(0), Input::Obj(ScratchObject::Number(2.0))),
                // ScratchBlock::VarSet(Ptr(1), Input::Obj(ScratchObject::Bool(true))),
                // ScratchBlock::VarSet(Ptr(2), Input::Obj(ScratchObject::Bool(false))),
                // ScratchBlock::VarSet(
                //     Ptr(3),
                //     Input::Obj(ScratchObject::String("192.0".to_owned())),
                // ),
                // ScratchBlock::VarSet(
                //     Ptr(4),
                //     Input::Block(Box::new(ScratchBlock::OpAdd(
                //         Input::Obj(ScratchObject::Number(2.0)),
                //         Input::Block(Box::new(ScratchBlock::OpMul(
                //             Input::Obj(ScratchObject::String("3.0".to_owned())),
                //             Input::Obj(ScratchObject::Number(4.0)),
                //         ))),
                //     ))),
                // ),
                // ScratchBlock::VarSet(
                //     Ptr(5),
                //     Input::Block(Box::new(ScratchBlock::OpSub(
                //         Input::Obj(ScratchObject::Number(2.0)),
                //         Input::Block(Box::new(ScratchBlock::OpDiv(
                //             Input::Obj(ScratchObject::Number(3.0)),
                //             Input::Obj(ScratchObject::Number(4.0)),
                //         ))),
                //     ))),
                // ),
                // ScratchBlock::VarSet(
                //     Ptr(6),
                //     Input::Block(Box::new(ScratchBlock::OpAdd(
                //         Input::Block(Box::new(ScratchBlock::OpAdd(
                //             Input::Obj(ScratchObject::Bool(true)),
                //             Input::Obj(ScratchObject::Bool(true)),
                //         ))),
                //         Input::Block(Box::new(ScratchBlock::VarRead(Ptr(3)))),
                //     ))),
                // ),
                ScratchBlock::ControlRepeat(
                    Input::Obj(ScratchObject::Number(100000.0)),
                    vec![ScratchBlock::VarSet(
                        Ptr(7),
                        // Input::Block(Box::new(ScratchBlock::OpJoin(
                        //     Input::Block(Box::new(ScratchBlock::VarRead(Ptr(7)))),
                        //     Input::Obj(ScratchObject::String("world".to_owned())),
                        // ))),
                        Input::Block(Box::new(ScratchBlock::OpAdd(
                            Input::Block(Box::new(ScratchBlock::VarRead(Ptr(7)))),
                            Input::Obj(ScratchObject::Bool(true)),
                        ))),
                    )],
                ),
            ]],
        }];
        for sprite in &code_sprites {
            for script in &sprite.scripts {
                let sig = Signature::new(CallConv::SystemV);
                let mut func = Function::with_name_signature(UserFuncName::default(), sig);

                let mut func_ctx = FunctionBuilderContext::new();
                let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

                let mut code_block = builder.create_block();
                // builder.seal_block(code_block);

                builder.append_block_params_for_function_params(code_block);
                builder.switch_to_block(code_block);

                for block in script {
                    compile_block(block, &mut builder, &mut code_block);
                }

                builder.seal_block(code_block);

                let ins = builder.ins();
                ins.return_(&[]);

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

                let ptr = buffer.as_ptr();
                let bytes = unsafe { std::slice::from_raw_parts(ptr, code.code_buffer().len()) };

                std::fs::write("func.bin", bytes).unwrap();

                // for (_i, byte) in bytes.iter().enumerate() {
                //     print!("{:#04x} ", byte);
                // }
                // println!();

                let buffer = buffer.make_exec().unwrap();

                unsafe {
                    let code_fn: unsafe extern "sysv64" fn() = std::mem::transmute(buffer.as_ptr());

                    let instant = std::time::Instant::now();
                    code_fn();
                    println!("Time: {:?}", instant.elapsed());
                }
            }
        }
    }
}

pub fn compile_block(
    block: &ScratchBlock,
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
) -> Option<ReturnValue> {
    match block {
        ScratchBlock::WhenFlagClicked => {}
        ScratchBlock::VarSet(ptr, obj) => {
            match obj {
                Input::Obj(obj) => {
                    ins_drop_obj(builder, *ptr);
                    match obj {
                        ScratchObject::Number(num) => {
                            ins_mem_write_f64(builder, *ptr, *num);
                        }
                        ScratchObject::Bool(num) => {
                            ins_mem_write_bool(builder, *ptr, *num);
                        }
                        ScratchObject::String(string) => {
                            ins_mem_write_string(string, builder, *ptr);
                        }
                    }
                }
                Input::Block(block) => {
                    // compile block
                    let val = compile_block(block, builder, code_block);
                    let val = val.unwrap();
                    ins_drop_obj(builder, *ptr);
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
                        ReturnValue::ObjectPointer(_value, slot) => {
                            let i1 = builder.ins().stack_load(I64, slot, 0);
                            let i2 = builder.ins().stack_load(I64, slot, 8);
                            let i3 = builder.ins().stack_load(I64, slot, 16);
                            let i4 = builder.ins().stack_load(I64, slot, 24);

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
            let a = a.get_number(builder, code_block);
            let b = b.get_number(builder, code_block);
            let res = builder.ins().fadd(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpSub(a, b) => {
            let a = a.get_number(builder, code_block);
            let b = b.get_number(builder, code_block);
            let res = builder.ins().fsub(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpMul(a, b) => {
            let a = a.get_number(builder, code_block);
            let b = b.get_number(builder, code_block);
            let res = builder.ins().fmul(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpDiv(a, b) => {
            let a = a.get_number(builder, code_block);
            let b = b.get_number(builder, code_block);
            let res = builder.ins().fdiv(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::VarRead(ptr) => {
            let func = builder.ins().iconst(I64, callbacks::var_read as i64);
            let sig = builder.import_signature({
                let mut sig = Signature::new(CallConv::SystemV);
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig
            });
            let mem_ptr = builder.ins().iconst(
                I64,
                MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
            );
            let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                4 * std::mem::size_of::<usize>() as u32,
                8,
            ));
            let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

            builder
                .ins()
                .call_indirect(sig, func, &[mem_ptr, stack_ptr]);

            return Some(ReturnValue::ObjectPointer(stack_ptr, stack_slot));

            // let obj = ScratchObject::Number(3.0);
            // let transmuted_obj: [usize; 4] = unsafe { std::mem::transmute(obj) };
            // let i1 = builder.ins().iconst(I64, transmuted_obj[0]);
            // let i2 = builder.ins().iconst(I64, transmuted_obj[1]);
            // let i3 = builder.ins().iconst(I64, transmuted_obj[2]);
            // let i4 = builder.ins().iconst(I64, transmuted_obj[3]);
            // return Some(ReturnValue::Object((i1, i2, i3, i4)));
        }
        ScratchBlock::OpJoin(a, b) => {
            // Get strings
            let (a, a_is_const) = a.get_string(builder, code_block);
            let (b, b_is_const) = b.get_string(builder, code_block);

            // Create stack slot for result
            let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                3 * std::mem::size_of::<i64>() as u32,
                0,
            ));
            let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

            // Call join_string function
            let func = builder.ins().iconst(I64, callbacks::op_join_string as i64);
            let sig = builder.import_signature({
                let mut sig = Signature::new(CallConv::SystemV);
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig
            });
            let a_is_const = builder.ins().iconst(I64, a_is_const as i64);
            let b_is_const = builder.ins().iconst(I64, b_is_const as i64);
            builder
                .ins()
                .call_indirect(sig, func, &[a, b, stack_ptr, a_is_const, b_is_const]);

            // Read resulting string
            let id = builder.ins().iconst(I64, ID_STRING as i64);
            let i1 = builder.ins().stack_load(I64, stack_slot, 0);
            let i2 = builder.ins().stack_load(I64, stack_slot, 8);
            let i3 = builder.ins().stack_load(I64, stack_slot, 16);

            return Some(ReturnValue::Object((id, i1, i2, i3)));
        }
        ScratchBlock::ControlRepeat(input, vec) => {
            let loop_block = builder.create_block();
            builder.append_block_param(loop_block, I64);
            let mut body_block = builder.create_block();
            builder.append_block_param(body_block, I64);
            let end_block = builder.create_block();

            let number = input.get_number(builder, code_block);
            let number = builder.ins().fcvt_to_sint(I64, number);

            let counter = builder.ins().iconst(I64, 0);
            builder.ins().jump(loop_block, &[counter]);
            builder.seal_block(*code_block);

            builder.switch_to_block(loop_block);
            // (counter < number)
            let counter = builder.block_params(loop_block)[0];
            let condition = builder.ins().icmp(IntCC::SignedLessThan, counter, number);

            // if (counter < number) jump to body_block else jump to end_block
            builder
                .ins()
                .brif(condition, body_block, &[counter], end_block, &[]);

            builder.switch_to_block(body_block);
            for block in vec {
                compile_block(block, builder, &mut body_block);
            }
            let counter = builder.block_params(body_block)[0];
            let incremented = builder.ins().iadd_imm(counter, 1);
            builder.ins().jump(loop_block, &[incremented]);
            builder.seal_block(body_block);
            builder.seal_block(loop_block);

            builder.switch_to_block(end_block);
            *code_block = end_block;
        }
    }
    None
}

fn ins_drop_obj(builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
    let func = builder.ins().iconst(I64, data_types::drop_obj as i64);
    let sig = builder.import_signature({
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I64));
        sig
    });
    let mem_ptr = builder.ins().iconst(
        I64,
        MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
    );

    builder.ins().call_indirect(sig, func, &[mem_ptr]);
}

pub fn c_main() {
    // let arg1 = std::env::args().nth(1).unwrap();
    // println!("opening dir {arg1}");

    print_func_addresses();

    let compiler = Compiler::new();
    compiler.compile();

    // print memory
    // for (i, obj) in MEMORY.iter().enumerate().take(10) {
    //     println!("{}: {:?}", i, obj);
    // }
}

fn print_func_addresses() {
    println!("var_read: {:X}", callbacks::var_read as usize);
    println!("op_join_string: {:X}", callbacks::op_join_string as usize);
    println!(
        "to_string_from_num: {:X}",
        data_types::to_string_from_num as usize
    );
    println!("to_string: {:X}", data_types::to_string as usize);
    println!(
        "to_string_from_bool: {:X}",
        data_types::to_string_from_bool as usize
    );
    println!("to_number: {:X}", data_types::to_number as usize);
}
