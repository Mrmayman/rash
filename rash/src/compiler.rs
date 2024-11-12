use std::{collections::HashMap, sync::Mutex};

use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use lazy_static::lazy_static;
use target_lexicon::Triple;

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
    OpRandom(Input, Input),
    ControlIf(Input, Vec<ScratchBlock>),
    ControlIfElse(Input, Vec<ScratchBlock>, Vec<ScratchBlock>),
    ControlRepeat(Input, Vec<ScratchBlock>),
    ControlRepeatUntil(Input, Vec<ScratchBlock>),
}

#[derive(PartialEq, Eq, Debug)]
pub enum VarTypeChecked {
    Number,
    Bool,
    String,
    Unknown,
}

impl From<VarType> for VarTypeChecked {
    fn from(value: VarType) -> Self {
        match value {
            VarType::Number => Self::Number,
            VarType::Bool => Self::Bool,
            VarType::String => Self::String,
        }
    }
}

impl ScratchBlock {
    pub fn return_type(
        &self,
        variable_type_data: &HashMap<Ptr, VarType>,
    ) -> Option<VarTypeChecked> {
        match self {
            ScratchBlock::VarRead(ptr) => match variable_type_data.get(ptr) {
                Some(vartype) => Some((*vartype).into()),
                None => Some(VarTypeChecked::Unknown),
            },
            ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpDiv(_, _)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpRandom(_, _)
            | ScratchBlock::OpStrLen(_) => Some(VarTypeChecked::Number),
            ScratchBlock::OpStrJoin(_, _) => Some(VarTypeChecked::String),
            ScratchBlock::OpCmpGreater(_, _) | ScratchBlock::OpCmpLesser(_, _) => {
                Some(VarTypeChecked::Bool)
            }
            ScratchBlock::WhenFlagClicked
            | ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::ControlIf(_, _)
            | ScratchBlock::ControlIfElse(_, _, _)
            | ScratchBlock::ControlRepeat(_, _)
            | ScratchBlock::ControlRepeatUntil(_, _) => None,
        }
    }

    pub fn affects_var(
        &self,
        var_ptr: Ptr,
        variable_type_data: &HashMap<Ptr, VarType>,
    ) -> Option<VarTypeChecked> {
        match self {
            ScratchBlock::VarSet(ptr, input) => {
                if var_ptr == *ptr {
                    match input {
                        Input::Obj(scratch_object) => Some(scratch_object.get_type().into()),
                        Input::Block(scratch_block) => {
                            scratch_block.return_type(variable_type_data)
                        }
                    }
                } else {
                    None
                }
            }
            ScratchBlock::VarChange(ptr, _) => {
                if var_ptr == *ptr {
                    Some(VarTypeChecked::Number)
                } else {
                    None
                }
            }
            ScratchBlock::ControlIf(_, vec)
            | ScratchBlock::ControlRepeat(_, vec)
            | ScratchBlock::ControlRepeatUntil(_, vec) => vec
                .iter()
                .filter_map(|n| n.affects_var(var_ptr, variable_type_data))
                .last(),
            ScratchBlock::ControlIfElse(_, then, else_block) => {
                let then = then
                    .iter()
                    .map(|n| n.affects_var(var_ptr, variable_type_data))
                    .last()
                    .flatten();
                let else_block = else_block
                    .iter()
                    .map(|n| n.affects_var(var_ptr, variable_type_data))
                    .last()
                    .flatten();

                match (then, else_block) {
                    (None, None) => None,
                    (None, Some(n)) | (Some(n), None) => Some(n),
                    (Some(a), Some(b)) => (a == b).then_some(a),
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Number,
    Bool,
    String,
}

pub struct CodeSprite {
    pub scripts: Vec<Vec<ScratchBlock>>,
}

impl Compiler {
    // pub fn new() -> Self {
    //     // let file_bytes = std::fs::read(path).unwrap();

    //     // let loaded_project_dir = tempfile::TempDir::new().unwrap();

    //     // zip_extract::extract(
    //     //     std::io::Cursor::new(file_bytes),
    //     //     loaded_project_dir.path(),
    //     //     false,
    //     // )
    //     // .unwrap();

    //     // let json = std::fs::read_to_string(loaded_project_dir.path().join("project.json")).unwrap();
    //     // let json: JsonStruct = serde_json::from_str(&json).unwrap();

    //     Self {
    //         // json,
    //         // project_dir: loaded_project_dir,
    //     }
    // }

    pub fn compile(/*&self*/) {
        let mut builder = settings::builder();
        builder.set("opt_level", "speed").unwrap();
        // for setting in builder.iter() {
        //     println!("{setting:?}");
        // }
        let flags = settings::Flags::new(builder);

        let isa = match isa::lookup(Triple::host()) {
            Err(err) => panic!("Error looking up target: {err}"),
            Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
        };

        // let code_sprites = self.get_block_code();
        let code_sprites = vec![CodeSprite {
            scripts: vec![block_test::pi()],
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
            blocks::var::set(obj, builder, *ptr, variable_type_data, code_block, memory);
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
            blocks::var::change(input, builder, code_block, variable_type_data, *ptr, memory);
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
        ScratchBlock::ControlIfElse(condition, then, r#else) => {
            blocks::control::if_else(
                condition,
                builder,
                code_block,
                variable_type_data,
                memory,
                then,
                r#else,
            );
        }
        ScratchBlock::ControlRepeatUntil(input, vec) => {
            blocks::control::repeat_until(
                builder,
                code_block,
                input,
                variable_type_data,
                memory,
                vec,
            );
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
            return Some(blocks::op::str_len(
                input,
                builder,
                code_block,
                variable_type_data,
                memory,
            ));
        }
        ScratchBlock::OpRandom(a, b) => {
            return Some(blocks::op::random(
                a,
                b,
                builder,
                code_block,
                variable_type_data,
                memory,
            ))
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
