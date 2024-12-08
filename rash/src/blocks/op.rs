use cranelift::prelude::*;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler::Compiler,
    data_types::ID_STRING,
    input_primitives::{Input, ReturnValue},
};

pub fn str_join(
    compiler: &mut Compiler,
    a: &Input,
    b: &Input,
    builder: &mut FunctionBuilder<'_>,
) -> (Value, Value, Value, Value) {
    // Get strings
    let (a, a_is_const) = a.get_string(compiler, builder);
    let (b, b_is_const) = b.get_string(compiler, builder);

    // Create stack slot for result
    let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        3 * std::mem::size_of::<i64>() as u32,
        0,
    ));
    let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

    // Call join_string function
    let a_is_const = compiler.constants.get_int(i64::from(a_is_const), builder);
    let b_is_const = compiler.constants.get_int(i64::from(b_is_const), builder);

    compiler.call_function(
        builder,
        callbacks::op_str_join as usize,
        &[I64, I64, I64, I64, I64],
        &[],
        &[a, b, stack_ptr, a_is_const, b_is_const],
    );
    // Read resulting string
    let id = compiler.constants.get_int(ID_STRING, builder);
    let i1 = builder.ins().stack_load(I64, stack_slot, 0);
    let i2 = builder.ins().stack_load(I64, stack_slot, 8);
    let i3 = builder.ins().stack_load(I64, stack_slot, 16);
    (id, i1, i2, i3)
}

pub fn modulo(
    compiler: &mut Compiler,
    a: &Input,
    b: &Input,
    builder: &mut FunctionBuilder<'_>,
) -> Value {
    let a = a.get_number(compiler, builder);
    let b = b.get_number(compiler, builder);

    // let div = a / b;
    // let modulo = (div - floor(div)) * b;

    let div = builder.ins().fdiv(a, b);

    let floor_div = floor_call(div, compiler, builder);

    let decimal_part = builder.ins().fsub(div, floor_div);
    let modulo = builder.ins().fmul(decimal_part, b);
    modulo
}

/// Calls the rust [`f64::floor`] function.
fn floor_call(n: Value, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) -> Value {
    let ins = compiler.call_function(builder, f64::floor as usize, &[F64], &[F64], &[n]);
    builder.inst_results(ins)[0]
}

pub fn str_len(
    compiler: &mut Compiler,
    input: &Input,
    builder: &mut FunctionBuilder<'_>,
) -> ReturnValue {
    let (input, is_const) = input.get_string(compiler, builder);
    let is_const = compiler.constants.get_int(i64::from(is_const), builder);

    let inst = compiler.call_function(
        builder,
        callbacks::op_str_len as usize,
        &[I64, I64],
        &[I64],
        &[input, is_const],
    );
    let res = builder.inst_results(inst)[0];
    let res = builder.ins().fcvt_from_sint(F64, res);
    ReturnValue::Num(res)
}

pub fn random(
    compiler: &mut Compiler,
    a: &Input,
    b: &Input,
    builder: &mut FunctionBuilder<'_>,
) -> ReturnValue {
    let (a, a_is_decimal) = a.get_number_with_decimal_check(compiler, builder);
    let (b, b_is_decimal) = b.get_number_with_decimal_check(compiler, builder);

    let is_decimal = builder.ins().bor(a_is_decimal, b_is_decimal);

    let inst = compiler.call_function(
        builder,
        callbacks::op_random as usize,
        &[F64, F64, I64],
        &[F64],
        &[a, b, is_decimal],
    );
    let res = builder.inst_results(inst)[0];
    ReturnValue::Num(res)
}

pub fn m_floor(
    compiler: &mut Compiler,
    n: &Input,
    builder: &mut FunctionBuilder<'_>,
) -> ReturnValue {
    let n = n.get_number(compiler, builder);
    let result = floor_call(n, compiler, builder);
    ReturnValue::Num(result)
}

pub fn str_letter(
    compiler: &mut Compiler,
    letter: &Input,
    string: &Input,
    builder: &mut FunctionBuilder<'_>,
) -> (Value, Value, Value, Value) {
    let (string, is_const) = string.get_string(compiler, builder);
    let letter = letter.get_number(compiler, builder);

    let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        3 * std::mem::size_of::<i64>() as u32,
        0,
    ));
    let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

    let is_const = compiler.constants.get_int(i64::from(is_const), builder);
    compiler.call_function(
        builder,
        callbacks::op_str_letter as usize,
        &[I64, I64, F64, I64],
        &[],
        &[string, is_const, letter, stack_ptr],
    );

    let id = compiler.constants.get_int(ID_STRING, builder);
    let i1 = builder.ins().stack_load(I64, stack_slot, 0);
    let i2 = builder.ins().stack_load(I64, stack_slot, 8);
    let i3 = builder.ins().stack_load(I64, stack_slot, 16);
    (id, i1, i2, i3)
}

pub fn str_contains(
    compiler: &mut Compiler,
    string: &Input,
    pattern: &Input,
    builder: &mut FunctionBuilder<'_>,
) -> Value {
    let (string, string_is_const) = string.get_string(compiler, builder);
    let (pattern, pattern_is_const) = pattern.get_string(compiler, builder);

    let string_is_const = compiler
        .constants
        .get_int(i64::from(string_is_const), builder);
    let pattern_is_const = compiler
        .constants
        .get_int(i64::from(pattern_is_const), builder);

    let ins = compiler.call_function(
        builder,
        callbacks::op_str_contains as usize,
        &[I64, I64, I64, I64],
        &[I64],
        &[string, string_is_const, pattern, pattern_is_const],
    );

    builder.inst_results(ins)[0]
}

pub fn round(compiler: &mut Compiler, num: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
    let num = num.get_number(compiler, builder);
    let inst = compiler.call_function(
        builder,
        callbacks::op_round as usize,
        &[F64],
        &[F64],
        &[num],
    );
    builder.inst_results(inst)[0]
}
