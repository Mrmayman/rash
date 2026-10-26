use std::{cmp::Ordering, sync::LazyLock};

use cranelift::{
    codegen::ir::condcodes::{FloatCC, IntCC},
    prelude::{
        FunctionBuilder, InstBuilder, Value,
        types::{F64, I64},
    },
};

use crate::{
    ScratchBlock, callbacks,
    compiler::{Compiler, VarTypeChecked},
    input_primitives::{CompStr, Input, ScratchValue},
};

impl Compiler<'_> {
    pub fn op_cmp(
        &mut self,
        a: &Input,
        b: &Input,
        builder: &mut FunctionBuilder<'_>,
        comp: Ordering,
    ) -> Value {
        // Compile-time known value
        if let (Input::Obj(a), Input::Obj(b)) = (a, b) {
            let out = a.scratch_cmp(b); // Same logic run by the callback
            return self.constants.get_int(i64::from(out == comp), builder);
        }

        let var_checker = |ptr| self.vars.get_type(ptr);
        let local_checker = |n| self.get_local_type(n);

        let at = a.expected_type(var_checker, local_checker).ty;
        let bt = b.expected_type(var_checker, local_checker).ty;

        // Primitive checks involving numbers/bools,
        // based on our smart (conservative) type analysis
        match (at, bt) {
            (VarTypeChecked::Number | VarTypeChecked::Bool, VarTypeChecked::Number)
            | (VarTypeChecked::Number, VarTypeChecked::Bool) => {
                let na = a.get_number_without_nan_check(self, builder);
                let nb = b.get_number_without_nan_check(self, builder);

                let ieee_cc = match comp {
                    Ordering::Less => FloatCC::LessThan,
                    Ordering::Equal => FloatCC::Equal,
                    Ordering::Greater => FloatCC::GreaterThan,
                };
                let ieee = builder.ins().fcmp(ieee_cc, na, nb);

                // If neither operand can be NaN, the IEEE result is already correct.
                let can_a_nan = a.could_be_nan(
                    |ptr| self.vars.get_type(ptr),
                    |n| self.get_local_type(n),
                    &self.args_list,
                );
                let can_b_nan = b.could_be_nan(
                    |ptr| self.vars.get_type(ptr),
                    |n| self.get_local_type(n),
                    &self.args_list,
                );

                let res = match (can_a_nan, can_b_nan) {
                    // Either can be NaN
                    (true, true) => {
                        let a_is_nan = builder.ins().fcmp(FloatCC::Unordered, na, na);
                        let b_is_nan = builder.ins().fcmp(FloatCC::Unordered, nb, nb);

                        match comp {
                            Ordering::Equal => {
                                // a and b NaN -> a == b
                                let both_nan = builder.ins().band(a_is_nan, b_is_nan);
                                builder.ins().bor(ieee, both_nan)
                            }
                            Ordering::Less => {
                                // a not NaN and b Nan -> a < b
                                let a_not_nan = builder.ins().fcmp(FloatCC::Ordered, na, na);
                                let patch = builder.ins().band(b_is_nan, a_not_nan);
                                builder.ins().bor(ieee, patch)
                            }
                            Ordering::Greater => {
                                // a NaN and b not Nan -> a > b
                                let b_not_nan = builder.ins().fcmp(FloatCC::Ordered, nb, nb);
                                let patch = builder.ins().band(a_is_nan, b_not_nan);
                                builder.ins().bor(ieee, patch)
                            }
                        }
                    }
                    // B isn't NaN
                    (true, false) => {
                        let a_is_nan = builder.ins().fcmp(FloatCC::Unordered, na, na);

                        match comp {
                            Ordering::Equal => ieee,
                            Ordering::Less => ieee,
                            Ordering::Greater => builder.ins().bor(ieee, a_is_nan),
                        }
                    }
                    // A isn't NaN
                    (false, true) => {
                        let b_is_nan = builder.ins().fcmp(FloatCC::Unordered, nb, nb);

                        match comp {
                            Ordering::Equal => ieee,
                            Ordering::Less => builder.ins().bor(ieee, b_is_nan),
                            Ordering::Greater => ieee,
                        }
                    }
                    // Neither are NaN, just use regular comparison
                    (false, false) => ieee,
                };

                return builder.ins().uextend(I64, res);
            }
            (VarTypeChecked::Bool, VarTypeChecked::Bool) => {
                let ba = a.get_bool(self, builder);
                let bb = b.get_bool(self, builder);
                let res = builder.ins().icmp(
                    match comp {
                        Ordering::Equal => IntCC::Equal,
                        Ordering::Less => IntCC::UnsignedLessThan,
                        Ordering::Greater => IntCC::UnsignedGreaterThan,
                    },
                    ba,
                    bb,
                );
                return builder.ins().uextend(I64, res);
            }
            _ => {} // We'll deal with strings below
        }

        let obj_first = a.get_object(self, builder);
        let obj_second = b.get_object(self, builder);

        let [a1, a2, a3] = obj_first.0;
        let [b1, b2, b3] = obj_second.0;

        let inst = self.call_function(
            builder,
            callbacks::op::CMP,
            &[I64; 6],
            &[I64],
            &[a1, a2, a3, b1, b2, b3],
        );
        let r = builder.inst_results(inst)[0];
        let out = builder.ins().icmp_imm(IntCC::Equal, r, comp as i64);
        builder.ins().uextend(I64, out)
    }

    pub fn op_add(&mut self, a: &Input, b: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        if has_hardware_fma() {
            if let Input::Block(block) = a
                && let ScratchBlock::OpMul(a1, a2) = &**block
            {
                let a1 = a1.get_number(self, builder);
                let a2 = a2.get_number(self, builder);
                let b = b.get_number(self, builder);
                return builder.ins().fma(a1, a2, b);
            }
            if let Input::Block(block) = b
                && let ScratchBlock::OpMul(b1, b2) = &**block
            {
                let a = a.get_number(self, builder);
                let b1 = b1.get_number(self, builder);
                let b2 = b2.get_number(self, builder);
                return builder.ins().fma(b1, b2, a);
            }
        }

        let a = a.get_number(self, builder);
        let b = b.get_number(self, builder);
        builder.ins().fadd(a, b)
    }

    pub fn op_sub(&mut self, a: &Input, b: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        if has_hardware_fma()
            && let Input::Block(block) = a
            && let ScratchBlock::OpMul(a1, a2) = &**block
        {
            let a1 = a1.get_number(self, builder);
            let a2 = a2.get_number(self, builder);
            let b = b.get_number_negated(self, builder);
            return builder.ins().fma(a1, a2, b);
        }

        let a = a.get_number(self, builder);
        let b = b.get_number(self, builder);
        builder.ins().fsub(a, b)
    }

    pub fn op_div(&mut self, a: &Input, b: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let a = a.get_number(self, builder);
        if crate::config::fast_division()
            && let Input::Obj(obj) = b
        {
            let reciprocal = 1.0 / obj.convert_to_number();
            let r = self.constants.get_float(reciprocal, builder);
            return builder.ins().fmul(a, r);
        }
        let b = b.get_number(self, builder);
        builder.ins().fdiv(a, b)
    }

    pub fn op_str_join(
        &mut self,
        a: &Input,
        b: &Input,
        builder: &mut FunctionBuilder<'_>,
    ) -> CompStr {
        // Get strings
        let a = a.get_string(self, builder);
        let b = b.get_string(self, builder);

        self.call_function(
            builder,
            callbacks::string::JOIN,
            &[I64, I64, I64],
            &[],
            &[a, b, self.temp_slot4.0],
        );
        // Read resulting string
        let i1 = builder.ins().stack_load(I64, self.temp_slot4.1, 0);
        let i2 = builder.ins().stack_load(I64, self.temp_slot4.1, 8);
        CompStr([i1, i2])
    }

    pub fn dbg_log(&mut self, msg: &Input, builder: &mut FunctionBuilder<'_>) {
        let a = msg.get_string(self, builder);
        self.call_function(builder, callbacks::env::DBG_LOG, &[I64], &[], &[a]);
    }

    /// A Scratch-accurate modulo operation (complete with quirks).
    ///
    /// There's no fmod or frem instruction in cranelift, let alone
    /// a `rem_euclid` so we gotta make our own
    pub fn op_modulo(
        &mut self,
        a_in: &Input,
        b_in: &Input,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        let a = a_in.get_number(self, builder);
        let b = b_in.get_number(self, builder);

        // let div = a / b;
        // let modulo = (div - floor(div)) * b;

        let div = if let Input::Obj(objb) = b_in
            && crate::config::fast_division()
        {
            let reciprocal = 1.0 / objb.convert_to_number();
            let r = self.constants.get_float(reciprocal, builder);
            builder.ins().fmul(a, r)
        } else {
            builder.ins().fdiv(a, b)
        };
        let floor_div = builder.ins().floor(div);

        let decimal_part = builder.ins().fsub(div, floor_div);
        builder.ins().fmul(decimal_part, b)
    }

    pub fn op_str_len(&mut self, input: &Input, builder: &mut FunctionBuilder<'_>) -> ScratchValue {
        let input = input.get_string(self, builder);

        let inst = self.call_function(builder, callbacks::string::LEN, &[I64], &[I64], &[input]);
        let res = builder.inst_results(inst)[0];
        let res = builder.ins().fcvt_from_sint(F64, res);
        ScratchValue::Num(res)
    }

    pub fn op_random(
        &mut self,
        a: &Input,
        b: &Input,
        builder: &mut FunctionBuilder<'_>,
    ) -> ScratchValue {
        let (a, a_is_decimal) = a.get_number_with_decimal_check(self, builder);
        let (b, b_is_decimal) = b.get_number_with_decimal_check(self, builder);

        let is_decimal = builder.ins().bor(a_is_decimal, b_is_decimal);

        let inst = self.call_function(
            builder,
            callbacks::op::RANDOM,
            &[F64, F64, I64],
            &[F64],
            &[a, b, is_decimal],
        );
        let res = builder.inst_results(inst)[0];
        ScratchValue::Num(res)
    }

    pub fn op_str_letter(
        &mut self,
        letter: &Input,
        string: &Input,
        builder: &mut FunctionBuilder<'_>,
    ) -> CompStr {
        let string = string.get_string(self, builder);
        let letter = letter.get_number(self, builder);

        self.call_function(
            builder,
            callbacks::string::LETTER,
            &[I64, F64, I64],
            &[],
            &[string, letter, self.temp_slot4.0],
        );

        let i1 = builder.ins().stack_load(I64, self.temp_slot4.1, 0);
        let i2 = builder.ins().stack_load(I64, self.temp_slot4.1, 8);
        CompStr([i1, i2])
    }

    pub fn op_str_contains(
        &mut self,
        string: &Input,
        pattern: &Input,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        let string = string.get_string(self, builder);
        let pattern = pattern.get_string(self, builder);

        let ins = self.call_function(
            builder,
            callbacks::string::CONTAINS,
            &[I64, I64],
            &[I64],
            &[string, pattern],
        );

        builder.inst_results(ins)[0]
    }

    pub fn op_round(&mut self, num: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let num = num.get_number(self, builder);
        let floor = builder.ins().floor(num);
        let fract = builder.ins().fsub(num, floor);

        let half = self.constants.get_float(0.5, builder);
        let is_low = builder.ins().fcmp(FloatCC::LessThan, fract, half);

        let ceil = builder.ins().ceil(num);
        builder.ins().select(is_low, floor, ceil)
    }
}

pub fn has_hardware_fma() -> bool {
    fn check() -> bool {
        // Intel/AMD FMA3 instruction set extension.
        #[cfg(target_arch = "x86_64")]
        return std::is_x86_feature_detected!("fma");

        // FMA is built into the baseline ARMv8 Neon vector system.
        #[cfg(target_arch = "aarch64")]
        return true;

        // RISC-V mandates hardware FMA operations inside its floating point units.
        // We check for "d" (Double-precision FP)
        #[cfg(target_arch = "riscv64")]
        return std::is_riscv_feature_detected!("d");

        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64"
        )))]
        return false;
    }

    static VAL: LazyLock<bool> = LazyLock::new(check);

    *VAL
}
