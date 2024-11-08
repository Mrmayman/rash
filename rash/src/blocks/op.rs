use std::collections::HashMap;

use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler::VarType,
    data_types::{ScratchObject, ID_STRING},
    input_primitives::{Input, Ptr},
};

pub fn str_join(
    a: &Input,
    b: &Input,
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    memory: &[ScratchObject],
) -> (Value, Value, Value, Value) {
    // Get strings
    let (a, a_is_const) = a.get_string(builder, code_block, variable_type_data, memory);
    let (b, b_is_const) = b.get_string(builder, code_block, variable_type_data, memory);

    // Create stack slot for result
    let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        3 * std::mem::size_of::<i64>() as u32,
        0,
    ));
    let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

    // Call join_string function
    let func = builder.ins().iconst(I64, callbacks::op_str_join as i64);
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
    (id, i1, i2, i3)
}

pub fn modulo(
    a: &Input,
    b: &Input,
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    memory: &[ScratchObject],
) -> Value {
    let a = a.get_number(builder, code_block, variable_type_data, memory);
    let b = b.get_number(builder, code_block, variable_type_data, memory);
    let div = builder.ins().fdiv(a, b);

    // Step 1: Truncate the division to an integer (simulates `floor` for positive values)
    let trunc_div = builder.ins().fcvt_to_sint(I64, div);
    let trunc_div = builder.ins().fcvt_from_sint(F64, trunc_div);

    // Step 2: Check if truncation needs adjustment for negative values
    // If `trunc_div > div`, we adjust by subtracting 1 to simulate `floor`
    let needs_adjustment = builder.ins().fcmp(FloatCC::GreaterThan, trunc_div, div);
    let tmp = builder.ins().f64const(-1.0);
    let adjustment = builder.ins().fadd(trunc_div, tmp);
    let floor_div = builder
        .ins()
        .select(needs_adjustment, adjustment, trunc_div);

    // Step 3: Calculate the decimal part and modulo as before
    let decimal_part = builder.ins().fsub(div, floor_div);
    let modulo = builder.ins().fmul(decimal_part, b);
    modulo
}
