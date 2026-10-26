use cranelift::{
    codegen::ir::{MemFlags, condcodes::IntCC},
    prelude::{
        FloatCC, FunctionBuilder, InstBuilder, StackSlotData, StackSlotKind, Value,
        types::{F64, I64},
    },
};
use ecow::EcoString;

use crate::{
    callbacks,
    compiler::{Compiler, FunctionArg, ScratchBlock, VarTypeChecked},
    constant_set::ConstantMap,
    data_types::{ID_BOOL, ID_NUMBER, ID_STRING, ScratchObject},
    effects::VariableWrite,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ptr(pub usize);

impl std::fmt::Debug for Ptr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var{}", self.0)
    }
}

impl Ptr {
    pub fn constant(
        &self,
        map: &mut ConstantMap,
        builder: &mut FunctionBuilder<'_>,
        memory: &[ScratchObject],
    ) -> Value {
        map.get_int(unsafe { memory.as_ptr().add(self.0) } as i64, builder)
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
/// # use rash_vm::{Input, Ptr, ScratchBlock};
/// ScratchBlock::VarSet(Ptr(0), 5.0.into());
/// ScratchBlock::VarChange(
///     Ptr(1),
///     ScratchBlock::OpAdd(5.0.into(), 3.0.into()).into()
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum Input {
    Obj(ScratchObject),
    Block(Box<ScratchBlock>),
}

impl Input {
    #[must_use]
    pub fn format(&self, indent: usize) -> String {
        match self {
            Input::Obj(scratch_object) => format!("{scratch_object:?}"),
            Input::Block(scratch_block) => scratch_block.format(indent),
        }
    }
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

impl From<String> for Input {
    fn from(s: String) -> Self {
        Input::Obj(ScratchObject::String(s.into()))
    }
}

impl From<&str> for Input {
    fn from(s: &str) -> Self {
        Input::Obj(ScratchObject::String(s.into()))
    }
}

impl From<EcoString> for Input {
    fn from(s: EcoString) -> Self {
        Input::Obj(ScratchObject::String(s))
    }
}

impl From<ScratchBlock> for Input {
    fn from(block: ScratchBlock) -> Self {
        Input::Block(Box::new(block))
    }
}

impl From<Ptr> for Input {
    fn from(ptr: Ptr) -> Self {
        Input::Block(Box::new(ScratchBlock::VarRead(ptr)))
    }
}

impl Input {
    pub(crate) fn could_be_nan(
        &self,
        vartype: impl FnMut(Ptr) -> VariableWrite,
        localtype: impl Fn(usize) -> VariableWrite,
        args_list: &[FunctionArg],
    ) -> bool {
        match self {
            Input::Obj(obj) => obj.convert_to_number().is_nan(),
            Input::Block(block) => {
                if let ScratchBlock::FunctionGetArg(idx) = **block
                    && let Some(arg) = args_list.get(idx)
                {
                    return !arg.skip_nan;
                }
                block
                    .return_type(vartype, localtype)
                    .is_none_or(|n| !n.skip_nan)
            }
        }
    }

    pub(crate) fn get_number(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        let mut num = self.get_number_without_nan_check(compiler, builder);
        self.nan_check(compiler, builder, &mut num);
        num
    }

    pub(crate) fn get_number_negated(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        let mut num = match self {
            Input::Obj(scratch_object) => compiler
                .constants
                .get_float(-scratch_object.convert_to_number(), builder),
            Input::Block(scratch_block) => {
                let v = compiler
                    .compile_block(scratch_block, builder)
                    .unwrap()
                    .get_number(compiler, builder);
                builder.ins().fneg(v)
            }
        };
        self.nan_check(compiler, builder, &mut num);

        num
    }

    pub(crate) fn get_number_without_nan_check(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        match self {
            Input::Obj(scratch_object) => compiler
                .constants
                .get_float(scratch_object.convert_to_number(), builder),
            Input::Block(scratch_block) => compiler
                .compile_block(scratch_block, builder)
                .unwrap()
                .get_number(compiler, builder),
        }
    }

    pub(crate) fn get_number_int(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        match self {
            Input::Obj(scratch_object) => {
                let num = scratch_object.convert_to_number();
                compiler
                    .constants
                    .get_int(if num.is_nan() { 0 } else { num as i64 }, builder)
            }
            Input::Block(scratch_block) => {
                let num = compiler
                    .compile_block(scratch_block, builder)
                    .unwrap()
                    .get_number(compiler, builder);
                // NaN automatically becomes zero
                builder.ins().fcvt_to_sint_sat(I64, num)
            }
        }
    }

    fn nan_check(
        &self,
        compiler: &mut Compiler<'_>,
        builder: &mut FunctionBuilder<'_>,
        num: &mut Value,
    ) {
        if !crate::config::fast_nan()
            && self.could_be_nan(
                |ptr| compiler.vars.get_type(ptr),
                |n| compiler.get_local_type(n),
                &compiler.args_list,
            )
        {
            let is_not_nan = builder.ins().fcmp(FloatCC::Ordered, *num, *num);
            let zero_value = compiler.constants.get_float(0.0, builder);
            *num = builder.ins().select(is_not_nan, *num, zero_value);
        }
    }

    pub(crate) fn get_string(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        match self {
            Input::Obj(scratch_object) => {
                let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    3 * std::mem::size_of::<i64>() as u32,
                    0,
                ));
                let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

                let string = scratch_object.convert_to_string();
                let rtstr: RtStr = unsafe { std::mem::transmute(string) };
                compiler
                    .static_strings
                    .push(unsafe { std::mem::transmute::<RtStr, EcoString>(rtstr) });

                let compstr = rtstr.make_constant(builder, &mut compiler.constants);
                let [val1, val2] = compstr.0;

                compiler.call_function(
                    builder,
                    callbacks::types::CLONE_STR,
                    &[I64, I64, I64],
                    &[],
                    &[val1, val2, stack_ptr],
                );

                stack_ptr
            }
            Input::Block(scratch_block) => {
                let o = compiler.compile_block(scratch_block, builder).unwrap();
                o.get_string(compiler, builder)
            }
        }
    }

    pub(crate) fn get_bool(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        match self {
            Input::Obj(scratch_object) => {
                let b = i64::from(scratch_object.convert_to_bool());
                compiler.constants.get_int(b, builder)
            }
            Input::Block(scratch_block) => {
                let b = compiler.compile_block(scratch_block, builder).unwrap();
                b.get_bool(compiler, builder)
            }
        }
    }

    pub(crate) fn get_object(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> CompObj {
        match self {
            Input::Obj(scratch_object) => {
                // Transmute to ints
                let scratch_object = scratch_object.clone();
                let is_str = scratch_object.is_heap_allocated();

                let rtobj = unsafe { std::mem::transmute::<ScratchObject, RtObj>(scratch_object) };

                if is_str {
                    let rtstr = rtobj.get_str().unwrap();
                    compiler
                        .clone_str(builder, rtstr)
                        .to_object(builder, &mut compiler.constants)
                } else {
                    rtobj.make_constant(builder, &mut compiler.constants)
                }
            }
            Input::Block(scratch_block) => {
                let o = compiler.compile_block(scratch_block, builder).unwrap();
                match o {
                    ScratchValue::Object(arr) => arr,
                    ScratchValue::String(s) => s.to_object(builder, &mut compiler.constants),
                    ScratchValue::Num(value) => {
                        let id = compiler.constants.get_int(ID_NUMBER, builder);
                        let zero = compiler.constants.get_int(0, builder);
                        let value = builder.ins().bitcast(
                            I64,
                            cranelift::codegen::ir::MemFlags::new(),
                            value,
                        );
                        CompObj([id, value, zero])
                    }
                    ScratchValue::Bool(value) => {
                        let id = compiler.constants.get_int(ID_BOOL, builder);
                        let zero = compiler.constants.get_int(0, builder);
                        CompObj([id, value, zero])
                    }
                }
            }
        }
    }

    pub(crate) fn get_number_with_decimal_check(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> (Value, Value) {
        match self {
            Input::Obj(scratch_object) => {
                let (n, b) = scratch_object.convert_to_number_with_decimal_check();
                let n = compiler.constants.get_float(n, builder);
                let b = compiler.constants.get_int(i64::from(b), builder);
                (n, b)
            }
            Input::Block(scratch_block) => {
                let o = compiler.compile_block(scratch_block, builder).unwrap();
                match o {
                    ScratchValue::Num(value) => (value, compiler.constants.get_int(0, builder)),
                    ScratchValue::Object(obj) => {
                        compiler.call_to_num_with_decimal_check(builder, obj)
                    }
                    ScratchValue::String(string) => {
                        // TODO: fast path
                        let obj = string.to_object(builder, &mut compiler.constants);
                        compiler.call_to_num_with_decimal_check(builder, obj)
                    }
                    ScratchValue::Bool(value) => (
                        builder.ins().fcvt_from_sint(F64, value),
                        compiler.constants.get_int(0, builder),
                    ),
                }
            }
        }
    }

    pub fn expected_type(
        &self,
        vartype: impl FnMut(Ptr) -> VariableWrite,
        localtype: impl Fn(usize) -> VariableWrite,
    ) -> VariableWrite {
        match self {
            Input::Obj(o) => VariableWrite {
                ty: Some(o.get_type()).into(),
                skip_nan: !o.convert_to_number().is_nan(),
                direct: true,
            },
            Input::Block(b) => b
                .return_type(vartype, localtype)
                .expect("Shouldn't take return value of non-returning block"),
        }
    }

    pub(crate) fn get_value(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> ScratchValue {
        match self {
            Input::Obj(obj) => match obj {
                ScratchObject::Number(f) => {
                    let f = compiler.constants.get_float(*f, builder);
                    ScratchValue::Num(f)
                }
                ScratchObject::Bool(b) => {
                    let b = compiler.constants.get_int(*b as i64, builder);
                    ScratchValue::Bool(b)
                }
                ScratchObject::String(smol_str) => {
                    let rtstr =
                        unsafe { std::mem::transmute::<EcoString, RtStr>(smol_str.clone()) };

                    compiler
                        .static_strings
                        .push(unsafe { std::mem::transmute::<RtStr, EcoString>(rtstr) });

                    let cloned = compiler.clone_str(builder, rtstr);
                    ScratchValue::String(cloned)
                }
            },
            Input::Block(block) => compiler
                .compile_block(block, builder)
                .expect("Shouldn't take return value of non-returning block"),
        }
    }
}

impl From<ScratchValue> for VarTypeChecked {
    fn from(value: ScratchValue) -> Self {
        match value {
            ScratchValue::Num(_) => VarTypeChecked::Number,
            ScratchValue::Bool(_) => VarTypeChecked::Bool,
            ScratchValue::String(_) => VarTypeChecked::String,
            ScratchValue::Object(_) => VarTypeChecked::Object,
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct CompStr(pub [Value; 2]);
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct CompObj(pub [Value; 3]);

impl CompStr {
    pub fn to_object(
        self,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    ) -> CompObj {
        let id = constants.get_int(ID_STRING, builder);
        let [i1, i2] = self.0;
        CompObj([id, i1, i2])
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct RtStr(pub [i64; 2]);
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct RtObj([i64; 3]);

impl RtStr {
    pub fn make_constant(
        &self,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    ) -> CompStr {
        let c1 = constants.get_int(self.0[0], builder);
        let c2 = constants.get_int(self.0[1], builder);
        CompStr([c1, c2])
    }
}

impl RtObj {
    pub fn get_str(self) -> Option<RtStr> {
        if self.0[0] == ID_STRING {
            Some(RtStr([self.0[1], self.0[2]]))
        } else {
            None
        }
    }

    pub fn make_constant(
        &self,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    ) -> CompObj {
        let c1 = constants.get_int(self.0[0], builder);
        let c2 = constants.get_int(self.0[1], builder);
        let c3 = constants.get_int(self.0[2], builder);
        CompObj([c1, c2, c3])
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScratchValue {
    Num(Value),
    Bool(Value),
    String(CompStr),
    Object(CompObj),
}

impl ScratchValue {
    pub(crate) fn get_number(
        self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        match self {
            ScratchValue::Num(value) => value,
            ScratchValue::Object(arr) => {
                let num = compiler.call_function(
                    builder,
                    callbacks::types::TO_NUMBER,
                    &[I64, I64, I64],
                    &[F64],
                    &arr.0,
                );
                builder.inst_results(num)[0]
            }
            ScratchValue::String(arr) => {
                let num = compiler.call_function(
                    builder,
                    callbacks::types::TO_NUMBER_FROM_STRING,
                    &[I64, I64],
                    &[F64],
                    &arr.0,
                );
                builder.inst_results(num)[0]
            }
            ScratchValue::Bool(value) => builder.ins().fcvt_from_sint(F64, value),
        }
    }

    pub(crate) fn get_string(
        self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        match self {
            ScratchValue::Num(value) => {
                let stack_ptr = Compiler::create_string_stack_slot(builder);

                compiler.call_function(
                    builder,
                    callbacks::types::TO_STRING_FROM_NUM,
                    &[F64, I64],
                    &[],
                    &[value, stack_ptr],
                );
                stack_ptr
            }
            ScratchValue::String(s) => {
                // We don't clone it since it's already owned by self
                // TODO: slot pool
                let slot =
                    builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData::new(
                        cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                        std::mem::size_of::<EcoString>() as u32,
                        0,
                    ));

                let [i1, i2] = s.0;
                builder.ins().stack_store(i1, slot, 0);
                builder.ins().stack_store(i2, slot, 8);
                builder.ins().stack_addr(I64, slot, 0)
            }
            ScratchValue::Object(obj) => get_string_from_obj(builder, compiler, obj),
            ScratchValue::Bool(value) => {
                let stack_ptr = Compiler::create_string_stack_slot(builder);

                compiler.call_function(
                    builder,
                    callbacks::types::TO_STRING_FROM_BOOL,
                    &[I64, I64],
                    &[],
                    &[value, stack_ptr],
                );
                stack_ptr
            }
        }
    }

    fn get_bool(&self, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) -> Value {
        match self {
            ScratchValue::Num(value) => {
                // (*n != 0.0 && !n.is_nan()) as i64

                // In Scratch, -0.0 is truthy. But IEEE 754 says -0.0 == +0.0.
                // So we bitcast and check against raw bits of +0.0 (which is just 0)
                let bits = builder.ins().bitcast(I64, MemFlags::new(), *value);
                let is_not_zero = builder.ins().icmp_imm(IntCC::NotEqual, bits, 0);
                let is_not_nan = builder.ins().fcmp(FloatCC::Equal, *value, *value);

                let res = builder.ins().band(is_not_zero, is_not_nan);
                let one = compiler.constants.get_int(1, builder);
                let zero = compiler.constants.get_int(0, builder);

                builder.ins().select(res, one, zero)
            }
            ScratchValue::Bool(value) => *value,
            ScratchValue::String(s) => {
                // TODO: fast path
                let obj = s.to_object(builder, &mut compiler.constants);
                obj_to_bool(compiler, builder, obj)
            }
            ScratchValue::Object(obj) => obj_to_bool(compiler, builder, *obj),
        }
    }

    pub fn clone_in_code(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> ScratchValue {
        match self {
            ScratchValue::Num(value) => ScratchValue::Num(*value),
            ScratchValue::Bool(value) => ScratchValue::Bool(*value),
            ScratchValue::String(s) => {
                let [i1, i2] = s.0;
                compiler.call_function(
                    builder,
                    callbacks::types::CLONE_STR,
                    &[I64, I64, I64],
                    &[],
                    &[i1, i2, compiler.temp_slot4.0],
                );
                let i1 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 0);
                let i2 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 8);
                ScratchValue::String(CompStr([i1, i2]))
            }
            ScratchValue::Object(obj) => {
                let [i1, i2, i3] = obj.0;
                compiler.call_function(
                    builder,
                    callbacks::types::CLONE_OBJ,
                    &[I64, I64, I64, I64],
                    &[],
                    &[i1, i2, i3, compiler.temp_slot4.0],
                );
                let i1 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 0);
                let i2 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 8);
                let i3 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 16);
                ScratchValue::Object(CompObj([i1, i2, i3]))
            }
        }
    }

    pub fn drop_in_code(&self, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) {
        match self {
            ScratchValue::Num(_) | ScratchValue::Bool(_) => {}
            ScratchValue::String(s) => {
                compiler.call_function(builder, callbacks::types::DROP_STR, &[I64, I64], &[], &s.0);
            }
            ScratchValue::Object(obj) => {
                compiler.call_function(
                    builder,
                    callbacks::types::DROP_OBJ,
                    &[I64, I64, I64],
                    &[],
                    &obj.0,
                );
            }
        }
    }

    pub fn get_object(
        &self,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    ) -> CompObj {
        match self {
            ScratchValue::Num(n) => {
                let n = builder.ins().bitcast(I64, MemFlags::new(), *n);
                let id = constants.get_int(ID_NUMBER, builder);
                let zero = constants.get_int(0, builder);
                CompObj([id, n, zero])
            }
            ScratchValue::Bool(n) => {
                let id = constants.get_int(ID_BOOL, builder);
                let zero = constants.get_int(0, builder);
                CompObj([id, *n, zero])
            }
            ScratchValue::String(s) => s.to_object(builder, constants),
            ScratchValue::Object(n) => *n,
        }
    }
}

fn obj_to_bool(
    compiler: &mut Compiler<'_>,
    builder: &mut FunctionBuilder<'_>,
    obj: CompObj,
) -> Value {
    let ins = compiler.call_function(
        builder,
        callbacks::types::TO_BOOL,
        &[I64, I64, I64],
        &[I64],
        &obj.0,
    );
    builder.inst_results(ins)[0]
}

fn get_string_from_obj(
    builder: &mut FunctionBuilder<'_>,
    compiler: &mut Compiler,
    obj: CompObj,
) -> Value {
    let stack_ptr = Compiler::create_string_stack_slot(builder);
    let [i1, i2, i3] = obj.0;

    compiler.call_function(
        builder,
        callbacks::types::TO_STRING,
        &[I64, I64, I64, I64],
        &[],
        &[i1, i2, i3, stack_ptr],
    );

    stack_ptr
}
