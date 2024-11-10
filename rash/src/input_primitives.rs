use std::{collections::HashMap, sync::Mutex};

use codegen::ir::StackSlot;
use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler::{compile_block, ScratchBlock, VarType},
    data_types::ScratchObject,
    ins_shortcuts::{
        ins_call_to_num, ins_call_to_num_with_decimal_check, ins_create_string_stack_slot,
    },
    ARITHMETIC_NAN_CHECK,
};

pub static STRINGS_TO_DROP: Mutex<Vec<[i64; 3]>> = Mutex::new(Vec::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ptr(pub usize);

impl Ptr {
    pub fn constant(&self, builder: &mut FunctionBuilder<'_>, memory: &[ScratchObject]) -> Value {
        builder
            .ins()
            .iconst(I64, unsafe { memory.as_ptr().add(self.0) } as i64)
    }
}

/// The input to a [`ScratchBlock`]
///
/// This can be either a [`ScratchObject`] object (number/string/bool)
/// or a [`ScratchBlock`] with another block inside.
///
/// This is used to represent the inputs to a block in the Scratch program.
///
/// # Examples
/// ```no_run
/// ScratchBlock::SetVar(Ptr(0), Input::new_num(5.0));
/// ScratchBlock::ChangeVar(
///     Ptr(1),
///     Input::new_block(
///         ScratchBlock::Add(Input::new_num(5.0), Input::new_num(3.0))
///     )
/// );
/// ```
#[derive(Debug)]
pub enum Input {
    Obj(ScratchObject),
    Block(Box<ScratchBlock>),
}

impl From<ScratchObject> for Input {
    fn from(obj: ScratchObject) -> Self {
        Input::Obj(obj)
    }
}

impl From<f64> for Input {
    fn from(num: f64) -> Self {
        Input::Obj(ScratchObject::Number(num))
    }
}

impl From<bool> for Input {
    fn from(b: bool) -> Self {
        Input::Obj(ScratchObject::Bool(b))
    }
}

impl From<ScratchBlock> for Input {
    fn from(block: ScratchBlock) -> Self {
        Input::Block(Box::new(block))
    }
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
        memory: &[ScratchObject],
    ) -> Value {
        let mut num = match self {
            Input::Obj(scratch_object) => {
                let o = scratch_object.convert_to_number();
                builder.ins().f64const(o)
            }
            Input::Block(scratch_block) => {
                let o = compile_block(
                    scratch_block,
                    builder,
                    code_block,
                    variable_type_data,
                    memory,
                )
                .unwrap();
                o.get_number(builder)
            }
        };
        if ARITHMETIC_NAN_CHECK {
            let is_not_nan = builder.ins().fcmp(FloatCC::Ordered, num, num);
            let zero_value = builder.ins().f64const(0.0);
            num = builder.ins().select(is_not_nan, num, zero_value);
        }

        num
    }

    pub fn get_string(
        &self,
        builder: &mut FunctionBuilder<'_>,
        code_block: &mut Block,
        variable_type_data: &mut HashMap<Ptr, VarType>,
        memory: &[ScratchObject],
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
                println!("Getting string {scratch_object:?}");
                let string = scratch_object.convert_to_string();

                let bytes: [i64; 3] = unsafe { std::mem::transmute(string) };
                STRINGS_TO_DROP.lock().unwrap().push(bytes);

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
                let o = compile_block(
                    scratch_block,
                    builder,
                    code_block,
                    variable_type_data,
                    memory,
                )
                .unwrap();
                (o.get_string(builder), false)
            }
        }
    }

    pub fn get_bool(
        &self,
        builder: &mut FunctionBuilder<'_>,
        code_block: &mut Block,
        variable_type_data: &mut HashMap<Ptr, VarType>,
        memory: &[ScratchObject],
    ) -> Value {
        match self {
            Input::Obj(scratch_object) => {
                let b = scratch_object.convert_to_bool() as i64;
                builder.ins().iconst(I64, b)
            }
            Input::Block(scratch_block) => {
                let b = compile_block(
                    scratch_block,
                    builder,
                    code_block,
                    variable_type_data,
                    memory,
                )
                .unwrap();
                b.get_bool(builder)
            }
        }
    }

    pub fn get_number_with_decimal_check(
        &self,
        builder: &mut FunctionBuilder<'_>,
        code_block: &mut Block,
        variable_type_data: &mut HashMap<Ptr, VarType>,
        memory: &[ScratchObject],
    ) -> (Value, Value) {
        match self {
            Input::Obj(scratch_object) => {
                let (n, b) = scratch_object.convert_to_number_with_decimal_check();
                let n = builder.ins().f64const(n);
                let b = builder.ins().iconst(I64, b as i64);
                (n, b)
            }
            Input::Block(scratch_block) => {
                let o = compile_block(
                    scratch_block,
                    builder,
                    code_block,
                    variable_type_data,
                    memory,
                )
                .unwrap();
                match o {
                    ReturnValue::Num(value) => (value, builder.ins().iconst(I64, 0)),
                    ReturnValue::Object((i1, i2, i3, i4)) => {
                        ins_call_to_num_with_decimal_check(builder, i1, i2, i3, i4)
                    }
                    ReturnValue::Bool(value) => (
                        builder.ins().fcvt_from_sint(F64, value),
                        builder.ins().iconst(I64, 0),
                    ),
                    ReturnValue::ObjectPointer(_value, slot) => {
                        let i1 = builder.ins().stack_load(I64, slot, 0);
                        let i2 = builder.ins().stack_load(I64, slot, 8);
                        let i3 = builder.ins().stack_load(I64, slot, 16);
                        let i4 = builder.ins().stack_load(I64, slot, 24);

                        ins_call_to_num_with_decimal_check(builder, i1, i2, i3, i4)
                    }
                }
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
            ReturnValue::ObjectPointer(_value, slot) => {
                let i1 = builder.ins().stack_load(I64, slot, 0);
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
        match self {
            ReturnValue::Num(value) => {
                let func = builder
                    .ins()
                    .iconst(I64, callbacks::types::to_string_from_num as i64);

                let stack_ptr = ins_create_string_stack_slot(builder);

                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(F64));
                    sig.params.push(AbiParam::new(I64));
                    sig
                });
                builder.ins().call_indirect(sig, func, &[value, stack_ptr]);
                stack_ptr
            }
            ReturnValue::Object((i1, i2, i3, i4)) => get_string_from_obj(builder, i1, i2, i3, i4),
            ReturnValue::Bool(value) => {
                let func = builder
                    .ins()
                    .iconst(I64, callbacks::types::to_string_from_bool as i64);

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
            ReturnValue::ObjectPointer(_value, slot) => {
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

    fn get_bool(&self, builder: &mut FunctionBuilder<'_>) -> Value {
        match self {
            ReturnValue::Num(value) => {
                // (*n != 0.0 && !n.is_nan()) as i64
                let zero = builder.ins().f64const(0.0);
                let nan = builder.ins().f64const(f64::NAN);
                let is_not_zero = builder.ins().fcmp(FloatCC::NotEqual, *value, zero);
                let is_not_nan = builder.ins().fcmp(FloatCC::NotEqual, *value, nan);
                builder.ins().band(is_not_zero, is_not_nan)
            }
            ReturnValue::Bool(value) => *value,
            ReturnValue::Object((i1, i2, i3, i4)) => {
                let func = builder.ins().iconst(I64, callbacks::types::to_bool as i64);

                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig
                });
                let ins = builder
                    .ins()
                    .call_indirect(sig, func, &[*i1, *i2, *i3, *i4]);
                builder.inst_results(ins)[0]
            }
            ReturnValue::ObjectPointer(_value, stack_slot) => {
                let i1 = builder.ins().stack_load(I64, *stack_slot, 0);
                let i2 = builder.ins().stack_load(I64, *stack_slot, 8);
                let i3 = builder.ins().stack_load(I64, *stack_slot, 16);
                let i4 = builder.ins().stack_load(I64, *stack_slot, 24);

                let func = builder.ins().iconst(I64, callbacks::types::to_bool as i64);

                let sig = builder.import_signature({
                    let mut sig = Signature::new(CallConv::SystemV);
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.params.push(AbiParam::new(I64));
                    sig.returns.push(AbiParam::new(I64));
                    sig
                });
                let ins = builder.ins().call_indirect(sig, func, &[i1, i2, i3, i4]);
                builder.inst_results(ins)[0]
            }
        }
    }
}

fn get_string_from_obj(
    builder: &mut FunctionBuilder<'_>,
    i1: Value,
    i2: Value,
    i3: Value,
    i4: Value,
) -> Value {
    let func = builder
        .ins()
        .iconst(I64, callbacks::types::to_string as i64);

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
