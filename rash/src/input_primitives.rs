use cranelift::prelude::*;
use types::F64;

use crate::{
    compiler::{compile_block, ScratchBlock},
    data_types::ScratchObject,
    ins_shortcuts::ins_call_to_num,
};

#[derive(Debug, Clone, Copy)]
pub struct Ptr(pub usize);

#[derive(Debug)]
pub enum Input {
    Obj(ScratchObject),
    Block(Box<ScratchBlock>),
}

impl Input {
    pub fn get_number(&self, builder: &mut FunctionBuilder<'_>) -> Value {
        match self {
            Input::Obj(scratch_object) => {
                let o = scratch_object.to_number();
                builder.ins().f64const(o)
            }
            Input::Block(scratch_block) => {
                let o = compile_block(scratch_block, builder).unwrap();
                o.get_number(builder)
            }
        }
    }
}

pub enum ReturnValue {
    Num(Value),
    Bool(Value),
    Object((Value, Value, Value, Value)),
}

impl ReturnValue {
    pub fn get_number(self, builder: &mut FunctionBuilder<'_>) -> Value {
        match self {
            ReturnValue::Num(value) => value,
            ReturnValue::Object((i1, i2, i3, i4)) => {
                let num = ins_call_to_num(builder, i1, i2, i3, i4);
                builder.inst_results(num)[0]
            }
            ReturnValue::Bool(value) => builder.ins().fcvt_from_sint(F64, value),
        }
    }
}
