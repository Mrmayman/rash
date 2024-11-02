use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I32, I64};

use crate::{
    compiler::{compile_block, ScratchBlock},
    data_types::{self, ScratchObject},
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
                println!("compiling block: {:?}", scratch_block);
                let o = compile_block(scratch_block, builder).unwrap();
                o.get_number(builder)
            }
        }
    }

    pub fn get_string(&self, builder: &mut FunctionBuilder<'_>) -> (Value, Value, Value) {
        match self {
            Input::Obj(scratch_object) => {
                let o = scratch_object.to_string();
                let bytes: [i64; 3] = unsafe { std::mem::transmute(o) };
                let val1 = builder.ins().iconst(I64, bytes[0]);
                let val2 = builder.ins().iconst(I64, bytes[1]);
                let val3 = builder.ins().iconst(I64, bytes[2]);
                (val1, val2, val3)
            }
            Input::Block(scratch_block) => {
                let o = compile_block(scratch_block, builder).unwrap();
                o.get_string(builder)
            }
        }
    }
}

pub enum ReturnValue {
    Num(Value),
    Bool(Value),
    Object((Value, Value, Value, Value)),
    ObjectPointer(Value),
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
            ReturnValue::ObjectPointer(value) => {
                // get 4 i64 from pointer
                let i1 = builder.ins().load(I32, MemFlags::new(), value, 0);
                let i1 = builder.ins().sextend(I64, i1);
                let i2 = builder.ins().load(I64, MemFlags::new(), value, 8);
                let i3 = builder.ins().load(I64, MemFlags::new(), value, 16);
                let i4 = builder.ins().load(I64, MemFlags::new(), value, 24);
                let num = ins_call_to_num(builder, i1, i2, i3, i4);
                builder.inst_results(num)[0]
            }
        }
    }

    pub fn get_string(self, builder: &mut FunctionBuilder<'_>) -> (Value, Value, Value) {
        match self {
            ReturnValue::Num(value) => {
                let func = builder
                    .ins()
                    .iconst(I64, data_types::to_string_from_num as i64);
                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(F64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig
                });
                let num = builder.ins().call_indirect(sig, func, &[value]);
                let results = builder.inst_results(num);
                (results[1], results[2], results[3])
            }
            ReturnValue::Object((i1, i2, i3, i4)) => {
                let func = builder.ins().iconst(I64, data_types::to_string as i64);
                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig
                });
                let num = builder.ins().call_indirect(sig, func, &[i1, i2, i3, i4]);
                let results = builder.inst_results(num);
                (results[1], results[2], results[3])
            }
            ReturnValue::Bool(value) => {
                let func = builder
                    .ins()
                    .iconst(I64, data_types::to_string_from_bool as i64);
                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig
                });
                let num = builder.ins().call_indirect(sig, func, &[value]);
                let results = builder.inst_results(num);
                (results[1], results[2], results[3])
            }
            ReturnValue::ObjectPointer(value) => {
                // read 4 i64 from pointer
                let i1 = builder.ins().load(I64, MemFlags::new(), value, 0);
                let i2 = builder.ins().load(I64, MemFlags::new(), value, 8);
                let i3 = builder.ins().load(I64, MemFlags::new(), value, 16);
                let i4 = builder.ins().load(I64, MemFlags::new(), value, 24);

                let func = builder.ins().iconst(I64, data_types::to_string as i64);
                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig
                });
                let num = builder.ins().call_indirect(sig, func, &[i1, i2, i3, i4]);
                let results = builder.inst_results(num);
                (results[1], results[2], results[3])
            }
        }
    }
}
