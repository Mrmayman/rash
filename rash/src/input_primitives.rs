use std::collections::HashMap;

use codegen::ir::StackSlot;
use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I32, I64};

use crate::{
    compiler::{compile_block, ScratchBlock, VarType},
    data_types::{self, ScratchObject},
    ins_shortcuts::{ins_call_to_num, ins_create_string_stack_slot},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ptr(pub usize);

#[derive(Debug)]
pub enum Input {
    Obj(ScratchObject),
    Block(Box<ScratchBlock>),
}

impl Input {
    pub fn new_num(num: f64) -> Self {
        Input::Obj(ScratchObject::Number(num))
    }

    pub fn new_block(block: ScratchBlock) -> Self {
        Input::Block(Box::new(block))
    }

    pub fn get_number(
        &self,
        builder: &mut FunctionBuilder<'_>,
        code_block: &mut Block,
        variable_type_data: &mut HashMap<Ptr, VarType>,
    ) -> Value {
        match self {
            Input::Obj(scratch_object) => {
                let o = scratch_object.to_number();
                builder.ins().f64const(o)
            }
            Input::Block(scratch_block) => {
                let o =
                    compile_block(scratch_block, builder, code_block, variable_type_data).unwrap();
                println!("compiling block: {:?}", o);
                o.get_number(builder)
            }
        }
    }

    pub fn get_string(
        &self,
        builder: &mut FunctionBuilder<'_>,
        code_block: &mut Block,
        variable_type_data: &mut HashMap<Ptr, VarType>,
    ) -> (Value, bool) {
        match self {
            Input::Obj(scratch_object) => {
                // Create a stack slot to store the string
                let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    3 * std::mem::size_of::<i64>() as u32,
                    0,
                ));
                let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

                // Transmute the String into a [i64; 3] array
                let string = scratch_object.to_str();
                let bytes: [i64; 3] = unsafe { std::mem::transmute(string) };

                let val1 = builder.ins().iconst(I64, bytes[0]);
                let val2 = builder.ins().iconst(I64, bytes[1]);
                let val3 = builder.ins().iconst(I64, bytes[2]);

                // Store the values in the stack slot
                builder.ins().stack_store(val1, stack_slot, 0);
                builder.ins().stack_store(val2, stack_slot, 8);
                builder.ins().stack_store(val3, stack_slot, 16);

                (stack_ptr, true)
            }
            Input::Block(scratch_block) => {
                let o =
                    compile_block(scratch_block, builder, code_block, variable_type_data).unwrap();
                (o.get_string(builder), false)
            }
        }
    }
}

#[derive(Debug)]
pub enum ReturnValue {
    Num(Value),
    Bool(Value),
    Object((Value, Value, Value, Value)),
    ObjectPointer(Value, StackSlot),
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
            ReturnValue::ObjectPointer(value, slot) => {
                // Get 4 i64 from pointer

                // The "tag" of a Rust enum is surprisingly i32 but aligned as i64
                // So we must load an i32 and convert it to i64
                let i1 = builder.ins().stack_load(I32, slot, 0);
                let i1 = builder.ins().sextend(I64, i1);

                let i2 = builder.ins().stack_load(I64, slot, 8);
                let i3 = builder.ins().stack_load(I64, slot, 16);
                let i4 = builder.ins().stack_load(I64, slot, 24);

                // Convert the object to number
                let num = ins_call_to_num(builder, i1, i2, i3, i4);
                builder.inst_results(num)[0]
            }
        }
    }

    pub fn get_string(self, builder: &mut FunctionBuilder<'_>) -> Value {
        // println!("get_string {self:?}");
        match self {
            ReturnValue::Num(value) => {
                let func = builder
                    .ins()
                    .iconst(I64, data_types::to_string_from_num as i64);

                let stack_ptr = ins_create_string_stack_slot(builder);

                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(F64));
                    sig.params.push(AbiParam::new(I64));
                    sig
                });
                builder.ins().call_indirect(sig, func, &[value, stack_ptr]);
                // let results = builder.inst_results(num);
                stack_ptr
            }
            ReturnValue::Object((i1, i2, i3, i4)) => get_string_from_obj(builder, i1, i2, i3, i4),
            ReturnValue::Bool(value) => {
                let func = builder
                    .ins()
                    .iconst(I64, data_types::to_string_from_bool as i64);

                let stack_ptr = ins_create_string_stack_slot(builder);

                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig
                });
                builder.ins().call_indirect(sig, func, &[value, stack_ptr]);
                // let results = builder.inst_results(num);
                stack_ptr
            }
            ReturnValue::ObjectPointer(value, slot) => {
                // read 4 i64 from pointer
                let i1 = builder.ins().stack_load(I64, slot, 0);
                let i2 = builder.ins().stack_load(I64, slot, 8);
                let i3 = builder.ins().stack_load(I64, slot, 16);
                let i4 = builder.ins().stack_load(I64, slot, 24);

                get_string_from_obj(builder, i1, i2, i3, i4)
            }
        }
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, ReturnValue::Bool(_))
    }
}

fn get_string_from_obj(
    builder: &mut FunctionBuilder<'_>,
    i1: Value,
    i2: Value,
    i3: Value,
    i4: Value,
) -> Value {
    let func = builder.ins().iconst(I64, data_types::to_string as i64);

    let stack_ptr = ins_create_string_stack_slot(builder);

    let sig = builder.import_signature({
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig
    });
    builder
        .ins()
        .call_indirect(sig, func, &[i1, i2, i3, i4, stack_ptr]);
    // let results = builder.inst_results(num);
    stack_ptr
}
