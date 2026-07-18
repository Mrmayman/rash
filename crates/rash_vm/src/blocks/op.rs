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
    config::IMPRECISE_DIVISION,
    data_types::ID_STRING,
    input_primitives::{Input, ScratchValue},
};

impl Compiler<'_> {
    pub fn op_m_tan(&mut self, num: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let num = num.get_number(self, builder);
        let inst = self.call_function(
            builder,
            callbacks::op::tan as *const (),
            &[F64],
            &[F64],
            &[num],
        );
        builder.inst_results(inst)[0]
    }

    pub fn op_m_cos(&mut self, num: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let num = num.get_number(self, builder);
        let inst = self.call_function(
            builder,
            callbacks::op::cos as *const (),
            &[F64],
            &[F64],
            &[num],
        );
        builder.inst_results(inst)[0]
    }

    pub fn op_m_sin(&mut self, num: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let num = num.get_number(self, builder);
        let inst = self.call_function(
            builder,
            callbacks::op::sin as *const (),
            &[F64],
            &[F64],
            &[num],
        );
        builder.inst_results(inst)[0]
    }

    pub fn op_m_sqrt(&mut self, num: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let num = num.get_number(self, builder);
        builder.ins().sqrt(num)
    }

    pub fn op_m_abs(&mut self, num: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let num = num.get_number(self, builder);
        builder.ins().fabs(num)
    }

    pub fn op_b_or(&mut self, a: &Input, b: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let a = a.get_bool(self, builder);
        let b = b.get_bool(self, builder);
        builder.ins().bor(a, b)
    }

    pub fn op_b_not(&mut self, a: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let a = a.get_bool(self, builder);
        let one = self.constants.get_int(1, builder);
        builder.ins().isub(one, a)
    }

    pub fn op_b_and(&mut self, a: &Input, b: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let a = a.get_bool(self, builder);
        let b = b.get_bool(self, builder);
        builder.ins().band(a, b)
    }

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
            return self.constants.get_int((out == comp) as i64, builder);
        }

        let var_checker = |ptr| self.cache.get_type(ptr);

        let at = a.expected_type(var_checker).ty;
        let bt = b.expected_type(var_checker).ty;

        // Primitive checks involving numbers/bools,
        // based on our smart (conservative) type analysis
        match (at, bt) {
            (VarTypeChecked::Number | VarTypeChecked::Bool, VarTypeChecked::Number)
            | (VarTypeChecked::Number, VarTypeChecked::Bool) => {
                let na = a.get_number(self, builder);
                let nb = b.get_number(self, builder);
                let res = builder.ins().fcmp(
                    match comp {
                        Ordering::Less => FloatCC::LessThan,
                        Ordering::Equal => FloatCC::Equal,
                        Ordering::Greater => FloatCC::GreaterThan,
                    },
                    na,
                    nb,
                );
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

        let obja = a.get_object(self, builder);
        let objb = b.get_object(self, builder);

        let inst = self.call_function(
            builder,
            callbacks::op::cmp as *const (),
            &[I64; 8],
            &[I64],
            &[
                obja[0], obja[1], obja[2], obja[3], objb[0], objb[1], objb[2], objb[3],
            ],
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

    pub fn op_mul(&mut self, a: &Input, b: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let a = a.get_number(self, builder);
        let b = b.get_number(self, builder);
        builder.ins().fmul(a, b)
    }

    pub fn op_div(&mut self, a: &Input, b: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let a = a.get_number(self, builder);
        if IMPRECISE_DIVISION && let Input::Obj(obj) = b {
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
    ) -> [Value; 4] {
        // Get strings
        let (a, a_is_const) = a.get_string(self, builder);
        let (b, b_is_const) = b.get_string(self, builder);

        // Call join_string function
        let a_is_const = self.constants.get_int(i64::from(a_is_const), builder);
        let b_is_const = self.constants.get_int(i64::from(b_is_const), builder);

        self.call_function(
            builder,
            callbacks::op::str_join as *const (),
            &[I64, I64, I64, I64, I64],
            &[],
            &[a, b, self.temp_slot4.0, a_is_const, b_is_const],
        );
        // Read resulting string
        let id = self.constants.get_int(ID_STRING, builder);
        let i1 = builder.ins().stack_load(I64, self.temp_slot4.1, 0);
        let i2 = builder.ins().stack_load(I64, self.temp_slot4.1, 8);
        let i3 = builder.ins().stack_load(I64, self.temp_slot4.1, 16);
        [id, i1, i2, i3]
    }

    pub fn dbg_log(&mut self, msg: &Input, builder: &mut FunctionBuilder<'_>) {
        // Get strings
        let (a, a_is_const) = msg.get_string(self, builder);

        let a_is_const = self.constants.get_int(i64::from(a_is_const), builder);

        self.call_function(
            builder,
            callbacks::dbg_log as *const (),
            &[I64, I64],
            &[],
            &[a, a_is_const],
        );
    }

    /// A Scratch-accurate modulo operation (complete with quirks).
    pub fn op_modulo(
        &mut self,
        a_in: &Input,
        b_in: &Input,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        // There is no fmod or frem instruction in cranelift bruh
        let a = a_in.get_number(self, builder);
        let b = b_in.get_number(self, builder);

        // let div = a / b;
        // let modulo = (div - floor(div)) * b;

        let div = if let (true, Input::Obj(objb)) = (IMPRECISE_DIVISION, b_in) {
            let reciprocal = 1.0 / objb.convert_to_number();
            let r = self.constants.get_float(reciprocal, builder);
            builder.ins().fmul(a, r)
        } else {
            builder.ins().fdiv(a, b)
        };
        let floor_div = self.floor_call(div, builder);

        let decimal_part = builder.ins().fsub(div, floor_div);
        builder.ins().fmul(decimal_part, b)
    }

    fn floor_call(&mut self, n: Value, builder: &mut FunctionBuilder<'_>) -> Value {
        builder.ins().floor(n)

        // let ins = self.call_function(builder, f64::floor as *const (), &[F64], &[F64], &[n]);
        // builder.inst_results(ins)[0]
    }

    pub fn op_str_len(&mut self, input: &Input, builder: &mut FunctionBuilder<'_>) -> ScratchValue {
        let (input, is_const) = input.get_string(self, builder);
        let is_const = self.constants.get_int(i64::from(is_const), builder);

        let inst = self.call_function(
            builder,
            callbacks::op::str_len as *const (),
            &[I64, I64],
            &[I64],
            &[input, is_const],
        );
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
            callbacks::op::random as *const (),
            &[F64, F64, I64],
            &[F64],
            &[a, b, is_decimal],
        );
        let res = builder.inst_results(inst)[0];
        ScratchValue::Num(res)
    }

    pub fn op_m_floor(&mut self, n: &Input, builder: &mut FunctionBuilder<'_>) -> ScratchValue {
        let n = n.get_number(self, builder);
        let result = self.floor_call(n, builder);
        ScratchValue::Num(result)
    }

    pub fn op_str_letter(
        &mut self,
        letter: &Input,
        string: &Input,
        builder: &mut FunctionBuilder<'_>,
    ) -> [Value; 4] {
        let (string, is_const) = string.get_string(self, builder);
        let letter = letter.get_number(self, builder);

        let is_const = self.constants.get_int(i64::from(is_const), builder);
        self.call_function(
            builder,
            callbacks::op::str_letter as *const (),
            &[I64, I64, F64, I64],
            &[],
            &[string, is_const, letter, self.temp_slot4.0],
        );

        let id = self.constants.get_int(ID_STRING, builder);
        let i1 = builder.ins().stack_load(I64, self.temp_slot4.1, 0);
        let i2 = builder.ins().stack_load(I64, self.temp_slot4.1, 8);
        let i3 = builder.ins().stack_load(I64, self.temp_slot4.1, 16);
        [id, i1, i2, i3]
    }

    pub fn op_str_contains(
        &mut self,
        string: &Input,
        pattern: &Input,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        let (string, string_is_const) = string.get_string(self, builder);
        let (pattern, pattern_is_const) = pattern.get_string(self, builder);

        let string_is_const = self.constants.get_int(i64::from(string_is_const), builder);
        let pattern_is_const = self.constants.get_int(i64::from(pattern_is_const), builder);

        let ins = self.call_function(
            builder,
            callbacks::op::str_contains as *const (),
            &[I64, I64, I64, I64],
            &[I64],
            &[string, string_is_const, pattern, pattern_is_const],
        );

        builder.inst_results(ins)[0]
    }

    pub fn op_round(&mut self, num: &Input, builder: &mut FunctionBuilder<'_>) -> Value {
        let num = num.get_number(self, builder);
        let inst = self.call_function(
            builder,
            callbacks::op::round as *const (),
            &[F64],
            &[F64],
            &[num],
        );
        builder.inst_results(inst)[0]
    }
}

pub fn has_hardware_fma() -> bool {
    fn check() -> bool {
        // Intel/AMD FMA3 instruction set extension.
        #[cfg(target_arch = "x86_64")]
        return std::is_x86_feature_detected!("fma");

        // FMA is built into the baseline ARMv8 Neon vector system.
        #[cfg(target_arch = "aarch64")]
        return std::is_aarch64_feature_detected!("neon");

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
