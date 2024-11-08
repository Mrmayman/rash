use std::{collections::HashMap, sync::Mutex};

use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use lazy_static::lazy_static;
use target_lexicon::Triple;
use types::{F64, I64};

use crate::{
    block_test, blocks, callbacks,
    data_types::ScratchObject,
    input_primitives::{Input, Ptr, ReturnValue},
};

lazy_static! {
    pub static ref MEMORY: Mutex<Box<[ScratchObject]>> =
        Mutex::new(vec![ScratchObject::Number(0.0); 1024].into_boxed_slice());
}

pub struct Compiler {
    // pub json: JsonStruct,
    // pub project_dir: TempDir,
}

#[allow(unused)]
#[derive(Debug)]
pub enum ScratchBlock {
    WhenFlagClicked,
    VarSet(Ptr, Input),
    VarChange(Ptr, Input),
    VarRead(Ptr),
    OpAdd(Input, Input),
    OpSub(Input, Input),
    OpMul(Input, Input),
    OpDiv(Input, Input),
    OpStrJoin(Input, Input),
    OpMod(Input, Input),
    OpStrLen(Input),
    OpCmpGreater(Input, Input),
    OpCmpLesser(Input, Input),
    ControlIf(Input, Vec<ScratchBlock>),
    ControlIfElse(Input, Vec<ScratchBlock>, Vec<ScratchBlock>),
    ControlRepeat(Input, Vec<ScratchBlock>),
    ControlRepeatUntil(Input, Vec<ScratchBlock>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum VarType {
    Number,
    Bool,
    String,
}

pub struct CodeSprite {
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
            scripts: vec![block_test::str_ops()],
        }];
        for sprite in &code_sprites {
            for script in &sprite.scripts {
                let sig = Signature::new(CallConv::SystemV);
                let mut func = Function::with_name_signature(UserFuncName::default(), sig);

                let mut func_ctx = FunctionBuilderContext::new();
                let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

                let mut code_block = builder.create_block();

                builder.append_block_params_for_function_params(code_block);
                builder.switch_to_block(code_block);

                let mut variable_type_data: HashMap<Ptr, VarType> = HashMap::new();

                let lock = MEMORY.lock().unwrap();
                for block in script {
                    compile_block(
                        block,
                        &mut builder,
                        &mut code_block,
                        &mut variable_type_data,
                        &lock,
                    );
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

                    let instant = std::time::Instant::now();
                    code_fn();
                    println!("Time: {:?}", instant.elapsed());
                    println!("Types: {variable_type_data:?}");
                }
            }
        }
    }
}

pub fn compile_block(
    block: &ScratchBlock,
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    memory: &[ScratchObject],
) -> Option<ReturnValue> {
    match block {
        ScratchBlock::WhenFlagClicked => {}
        ScratchBlock::VarSet(ptr, obj) => {
            blocks::var::set(obj, builder, ptr, variable_type_data, code_block, memory);
        }
        ScratchBlock::OpAdd(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data, memory);
            let b = b.get_number(builder, code_block, variable_type_data, memory);
            let res = builder.ins().fadd(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpSub(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data, memory);
            let b = b.get_number(builder, code_block, variable_type_data, memory);
            let res = builder.ins().fsub(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpMul(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data, memory);
            let b = b.get_number(builder, code_block, variable_type_data, memory);
            let res = builder.ins().fmul(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpDiv(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data, memory);
            let b = b.get_number(builder, code_block, variable_type_data, memory);
            let res = builder.ins().fdiv(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpMod(a, b) => {
            let modulo = blocks::op::modulo(a, b, builder, code_block, variable_type_data, memory);
            return Some(ReturnValue::Num(modulo));
        }
        ScratchBlock::VarRead(ptr) => {
            return Some(blocks::var::read(builder, *ptr, variable_type_data, memory));
        }
        ScratchBlock::OpStrJoin(a, b) => {
            let obj = blocks::op::str_join(a, b, builder, code_block, variable_type_data, memory);
            return Some(ReturnValue::Object(obj));
        }
        ScratchBlock::ControlRepeat(input, vec) => {
            blocks::control::repeat(builder, input, code_block, variable_type_data, vec, memory);
        }
        ScratchBlock::VarChange(ptr, input) => {
            blocks::var::change(input, builder, code_block, variable_type_data, ptr, memory);
        }
        ScratchBlock::ControlIf(input, vec) => {
            blocks::control::if_statement(
                input,
                builder,
                code_block,
                variable_type_data,
                vec,
                memory,
            );
        }
        ScratchBlock::ControlIfElse(input, vec, vec1) => {
            let input = input.get_bool(builder, code_block, variable_type_data, memory);
            let mut inside_block = builder.create_block();
            let mut else_block = builder.create_block();
            let end_block = builder.create_block();

            builder
                .ins()
                .brif(input, inside_block, &[], else_block, &[]);
            builder.seal_block(*code_block);

            builder.switch_to_block(inside_block);
            for block in vec {
                compile_block(
                    block,
                    builder,
                    &mut inside_block,
                    variable_type_data,
                    memory,
                );
            }
            builder.ins().jump(end_block, &[]);
            builder.seal_block(inside_block);

            builder.switch_to_block(else_block);
            for block in vec1 {
                compile_block(block, builder, &mut else_block, variable_type_data, memory);
            }
            builder.ins().jump(end_block, &[]);
            builder.seal_block(else_block);

            builder.switch_to_block(end_block);
            *code_block = end_block;
        }
        ScratchBlock::ControlRepeatUntil(input, vec) => {
            let loop_block = builder.create_block();
            let mut body_block = builder.create_block();
            let end_block = builder.create_block();
            builder.ins().jump(loop_block, &[]);
            builder.seal_block(*code_block);

            builder.switch_to_block(loop_block);
            let condition = input.get_bool(builder, code_block, variable_type_data, memory);
            builder
                .ins()
                .brif(condition, end_block, &[], body_block, &[]);

            builder.switch_to_block(body_block);
            for block in vec {
                compile_block(block, builder, &mut body_block, variable_type_data, memory);
            }
            builder.ins().jump(loop_block, &[]);
            builder.seal_block(body_block);
            builder.seal_block(loop_block);

            builder.switch_to_block(end_block);
            *code_block = end_block;
        }
        ScratchBlock::OpCmpGreater(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data, memory);
            let b = b.get_number(builder, code_block, variable_type_data, memory);
            let res = builder.ins().fcmp(FloatCC::GreaterThan, a, b);
            return Some(ReturnValue::Bool(res));
        }
        ScratchBlock::OpCmpLesser(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data, memory);
            let b = b.get_number(builder, code_block, variable_type_data, memory);
            let res = builder.ins().fcmp(FloatCC::LessThan, a, b);
            return Some(ReturnValue::Bool(res));
        }
        ScratchBlock::OpStrLen(input) => {
            let (input, is_const) =
                input.get_string(builder, code_block, variable_type_data, memory);

            let func = builder.ins().iconst(I64, callbacks::op_str_len as i64);
            let sig = builder.import_signature({
                let mut sig = Signature::new(CallConv::SystemV);
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig.returns.push(AbiParam::new(I64));
                sig
            });

            let is_const = builder.ins().iconst(I64, is_const as i64);
            let inst = builder.ins().call_indirect(sig, func, &[input, is_const]);

            let res = builder.inst_results(inst)[0];
            let res = builder.ins().fcvt_from_sint(F64, res);

            return Some(ReturnValue::Num(res));
        }
    }
    None
}

pub fn print_func_addresses() {
    println!("var_read: {:X}", callbacks::var_read as usize);
    println!("op_str_join: {:X}", callbacks::op_str_join as usize);
    println!(
        "to_string_from_num: {:X}",
        callbacks::types::to_string_from_num as usize
    );
    println!("to_string: {:X}", callbacks::types::to_string as usize);
    println!(
        "to_string_from_bool: {:X}",
        callbacks::types::to_string_from_bool as usize
    );
    println!("to_number: {:X}", callbacks::types::to_number as usize);
    println!("drop_obj: {:X}", callbacks::types::drop_obj as usize);
}
