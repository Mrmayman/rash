use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler::Compiler,
    data_types::{ScratchObject, ID_STRING},
    input_primitives::{Input, ReturnValue},
};

pub fn str_join(
    compiler: &mut Compiler,
    a: &Input,
    b: &Input,
    builder: &mut FunctionBuilder<'_>,
    memory: &[ScratchObject],
) -> (Value, Value, Value, Value) {
    // Get strings
    let (a, a_is_const) = a.get_string(compiler, builder, memory);
    let (b, b_is_const) = b.get_string(compiler, builder, memory);

    // Create stack slot for result
    let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        3 * std::mem::size_of::<i64>() as u32,
        0,
    ));
    let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

    // Call join_string function
    let func = compiler
        .constants
        .get_int(callbacks::op_str_join as usize as i64, builder);
    let sig = builder.import_signature({
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig
    });
    let a_is_const = compiler.constants.get_int(i64::from(a_is_const), builder);
    let b_is_const = compiler.constants.get_int(i64::from(b_is_const), builder);
    builder
        .ins()
        .call_indirect(sig, func, &[a, b, stack_ptr, a_is_const, b_is_const]);

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
    memory: &[ScratchObject],
) -> Value {
    // let div = a / b;
    // let floor_div = div.floor();
    // let decimal_part = div - floor_div;
    // let modulo = decimal_part * b;

    let a = a.get_number(compiler, builder, memory);
    let b = b.get_number(compiler, builder, memory);
    let div = builder.ins().fdiv(a, b);

    // let floor_div = floor_branch(builder, div, compiler);

    let floor_div = floor_int_float_conversion(builder, div, compiler);

    // Calculate the decimal part and modulo as before
    let decimal_part = builder.ins().fsub(div, floor_div);
    let modulo = builder.ins().fmul(decimal_part, b);
    modulo
}

fn floor_int_float_conversion(
    builder: &mut FunctionBuilder<'_>,
    div: Value,
    compiler: &mut Compiler,
) -> Value {
    let trunc_div = builder.ins().fcvt_to_sint(I64, div);
    let trunc_div = builder.ins().fcvt_from_sint(F64, trunc_div);

    let needs_adjustment = builder.ins().fcmp(FloatCC::GreaterThan, trunc_div, div);
    let neg_one = compiler.constants.get_float(-1.0, builder);
    let adjustment = builder.ins().fadd(trunc_div, neg_one);
    let floor_div = builder
        .ins()
        .select(needs_adjustment, adjustment, trunc_div);
    floor_div
}

fn floor_bit_hack(builder: &mut FunctionBuilder<'_>, div: Value, compiler: &mut Compiler) -> Value {
    let div_bits = builder.ins().bitcast(I64, MemFlags::new(), div);

    // let exponent = (div_bits >> 52) & 0x7FF;
    let exponent = builder.ins().ushr_imm(div_bits, 52);
    let exponent = builder.ins().band_imm(exponent, 0x7FF);

    // exponent < 1023
    let exponent_lt_1023 = builder
        .ins()
        .icmp_imm(IntCC::UnsignedLessThan, exponent, 1023);

    // if exponent < 1023: (results in -1.0 or 0.0 based on sign)
    let minus_one = compiler.constants.get_float(-1.0, builder);
    let zero = compiler.constants.get_float(0.0, builder);
    let n_is_negative = builder.ins().fcmp(FloatCC::GreaterThan, zero, div);
    let neg_floor_result = builder.ins().select(n_is_negative, minus_one, zero);

    // if exponent >= 1023:

    // (exponent - 1023)
    // let exponent_offset = builder.ins().iadd_imm(exponent, -1023);
    // (52 - (exponent - 1023))
    let v_1075 = compiler.constants.get_int(52 + 1023, builder);
    let shift_amount = builder.ins().isub(v_1075, exponent);

    // (1 << (52 - (exponent - 1023))) - 1
    let one = compiler.constants.get_int(1, builder);
    let mask = builder.ins().ishl(one, shift_amount);
    let mask = builder.ins().iadd_imm(mask, -1);

    // let not_mask = builder.ins().bnot(mask);
    let truncated_bits = builder.ins().band_not(div_bits, mask);
    // Zero out fractional bits
    let trunc = builder.ins().bitcast(F64, MemFlags::new(), truncated_bits);

    // Step 6: Apply conditional adjustment: `if trunc > n { trunc - 1.0 } else { trunc }`
    let trunc_gt_n = builder.ins().fcmp(FloatCC::GreaterThan, trunc, div);
    let one = compiler.constants.get_float(1.0, builder);
    let zero = compiler.constants.get_float(0.0, builder);
    let num = builder.ins().select(trunc_gt_n, one, zero);
    let trunc_floor_result = builder.ins().fsub(trunc, num);

    // Step 7: Select between `neg_floor_result` and `trunc_floor_result` based on `exponent_lt_1023`
    builder
        .ins()
        .select(exponent_lt_1023, neg_floor_result, trunc_floor_result)
}

pub fn str_len(
    compiler: &mut Compiler,
    input: &Input,
    builder: &mut FunctionBuilder<'_>,
    memory: &[ScratchObject],
) -> ReturnValue {
    let (input, is_const) = input.get_string(compiler, builder, memory);
    let func = compiler
        .constants
        .get_int(callbacks::op_str_len as usize as i64, builder);
    let sig = builder.import_signature({
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.returns.push(AbiParam::new(I64));
        sig
    });
    let is_const = compiler.constants.get_int(i64::from(is_const), builder);
    let inst = builder.ins().call_indirect(sig, func, &[input, is_const]);
    let res = builder.inst_results(inst)[0];
    let res = builder.ins().fcvt_from_sint(F64, res);
    ReturnValue::Num(res)
}

pub fn random(
    compiler: &mut Compiler,
    a: &Input,
    b: &Input,
    builder: &mut FunctionBuilder<'_>,
    memory: &[ScratchObject],
) -> ReturnValue {
    let (a, a_is_decimal) = a.get_number_with_decimal_check(compiler, builder, memory);
    let (b, b_is_decimal) = b.get_number_with_decimal_check(compiler, builder, memory);

    let is_decimal = builder.ins().bor(a_is_decimal, b_is_decimal);
    let func = compiler
        .constants
        .get_int(callbacks::op_random as usize as i64, builder);
    let sig = builder.import_signature({
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(F64));
        sig.params.push(AbiParam::new(F64));
        sig.params.push(AbiParam::new(I64));
        sig.returns.push(AbiParam::new(F64));
        sig
    });
    let inst = builder.ins().call_indirect(sig, func, &[a, b, is_decimal]);
    let res = builder.inst_results(inst)[0];
    ReturnValue::Num(res)
}
