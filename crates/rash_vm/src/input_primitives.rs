use cranelift::{
    codegen::ir::MemFlags,
    prelude::{
        FloatCC, FunctionBuilder, InstBuilder, StackSlotData, StackSlotKind, Value,
        types::{F64, I64},
    },
};
use smol_str::SmolStr;

use crate::{
    callbacks,
    compiler::{Compiler, ScratchBlock, VarTypeChecked},
    config::ARITHMETIC_NAN_CHECK,
    constant_set::ConstantMap,
    data_types::{ID_BOOL, ID_NUMBER, ID_STRING, ScratchObject},
    effects::VariableWrite,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ptr(pub usize);

impl std::fmt::Debug for Ptr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*({})", self.0)
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
#[derive(Debug, PartialEq)]
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

impl From<SmolStr> for Input {
    fn from(s: SmolStr) -> Self {
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
    pub(crate) fn could_be_nan(&self, vartype: impl FnMut(Ptr) -> VariableWrite) -> bool {
        match self {
            Input::Obj(obj) => obj.convert_to_number().is_nan(),
            Input::Block(block) => block.return_type(vartype).is_none_or(|n| !n.skip_nan),
        }
    }

    pub(crate) fn get_number(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        let mut num = match self {
            Input::Obj(scratch_object) => compiler
                .constants
                .get_float(scratch_object.convert_to_number(), builder),
            Input::Block(scratch_block) => compiler
                .compile_block(scratch_block, builder)
                .unwrap()
                .get_number(compiler, builder),
        };
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
                let mut num = compiler
                    .compile_block(scratch_block, builder)
                    .unwrap()
                    .get_number(compiler, builder);
                self.nan_check(compiler, builder, &mut num);
                builder.ins().fcvt_to_sint(I64, num)
            }
        }
    }

    fn nan_check(
        &self,
        compiler: &mut Compiler<'_>,
        builder: &mut FunctionBuilder<'_>,
        num: &mut Value,
    ) {
        if ARITHMETIC_NAN_CHECK && self.could_be_nan(|ptr| compiler.vars.get_type(ptr)) {
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

                let bytes: [i64; 3] = unsafe { std::mem::transmute(string) };
                compiler
                    .static_strings
                    .push(unsafe { std::mem::transmute::<[i64; 3], smol_str::SmolStr>(bytes) });

                let val1 = compiler.constants.get_int(bytes[0], builder);
                let val2 = compiler.constants.get_int(bytes[1], builder);
                let val3 = compiler.constants.get_int(bytes[2], builder);

                compiler.call_function(
                    builder,
                    callbacks::types::CLONE_STR,
                    &[I64, I64, I64, I64],
                    &[],
                    &[val1, val2, val3, stack_ptr],
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
    ) -> [Value; 4] {
        match self {
            Input::Obj(scratch_object) => {
                // Transmute to [i64; 4]
                let scratch_object = scratch_object.clone();
                let is_string = matches!(scratch_object, ScratchObject::String(_));
                let [i1, i2, i3, i4] =
                    unsafe { std::mem::transmute::<ScratchObject, [i64; 4]>(scratch_object) };
                if is_string {
                    compiler
                        .static_strings
                        .push(unsafe { std::mem::transmute::<[i64; 3], SmolStr>([i2, i3, i4]) });
                }

                let i1 = compiler.constants.get_int(i1, builder);
                let i2 = compiler.constants.get_int(i2, builder);
                let i3 = compiler.constants.get_int(i3, builder);
                let i4 = compiler.constants.get_int(i4, builder);

                compiler.call_function(
                    builder,
                    callbacks::types::CLONE_OBJ,
                    &[I64, I64, I64, I64, I64],
                    &[],
                    &[i1, i2, i3, i4, compiler.temp_slot4.0],
                );

                let o1 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 0);
                let o2 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 8);
                let o3 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 16);
                let o4 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 24);

                [o1, o2, o3, o4]
            }
            Input::Block(scratch_block) => {
                let o = compiler.compile_block(scratch_block, builder).unwrap();
                match o {
                    ScratchValue::Object(arr) => arr,
                    ScratchValue::String([i2, i3, i4]) => {
                        let i1 = compiler.constants.get_int(ID_STRING, builder);
                        [i1, i2, i3, i4]
                    }
                    ScratchValue::Num(value) => {
                        let id = builder.ins().iconst(I64, ID_NUMBER);
                        let zero = compiler.constants.get_int(0, builder);
                        let value = builder.ins().bitcast(
                            I64,
                            cranelift::codegen::ir::MemFlags::new(),
                            value,
                        );
                        [id, value, zero, zero]
                    }
                    ScratchValue::Bool(value) => {
                        let id = builder.ins().iconst(I64, ID_BOOL);
                        let zero = compiler.constants.get_int(0, builder);
                        [id, value, zero, zero]
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
                    ScratchValue::Object([i1, i2, i3, i4]) => {
                        compiler.ins_call_to_num_with_decimal_check(builder, i1, i2, i3, i4)
                    }
                    ScratchValue::String([i2, i3, i4]) => {
                        let i1 = compiler.constants.get_int(ID_STRING, builder);
                        compiler.ins_call_to_num_with_decimal_check(builder, i1, i2, i3, i4)
                    }
                    ScratchValue::Bool(value) => (
                        builder.ins().fcvt_from_sint(F64, value),
                        compiler.constants.get_int(1, builder),
                    ),
                }
            }
        }
    }

    pub fn expected_type(&self, vartype: impl FnMut(Ptr) -> VariableWrite) -> VariableWrite {
        match self {
            Input::Obj(o) => VariableWrite {
                ty: Some(o.get_type()).into(),
                skip_nan: !o.convert_to_number().is_nan(),
            },
            Input::Block(b) => b
                .return_type(vartype)
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScratchValue {
    Num(Value),
    Bool(Value),
    String([Value; 3]),
    Object([Value; 4]),
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
                    &[I64, I64, I64, I64],
                    &[F64],
                    &arr,
                );
                builder.inst_results(num)[0]
            }
            ScratchValue::String(arr) => {
                let num = compiler.call_function(
                    builder,
                    callbacks::types::TO_NUMBER_FROM_STRING,
                    &[I64, I64, I64],
                    &[F64],
                    &arr,
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
                let stack_ptr = Compiler::ins_create_string_stack_slot(builder);

                compiler.call_function(
                    builder,
                    callbacks::types::TO_STRING_FROM_NUM,
                    &[F64, I64],
                    &[],
                    &[value, stack_ptr],
                );
                stack_ptr
            }
            ScratchValue::String([i2, i3, i4]) => {
                let i1 = compiler.constants.get_int(ID_STRING, builder);
                get_string_from_obj(builder, compiler, i1, i2, i3, i4)
            }
            ScratchValue::Object([i1, i2, i3, i4]) => {
                get_string_from_obj(builder, compiler, i1, i2, i3, i4)
            }
            ScratchValue::Bool(value) => {
                let stack_ptr = Compiler::ins_create_string_stack_slot(builder);

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

    pub fn is_bool(&self) -> bool {
        matches!(self, ScratchValue::Bool(_))
    }

    fn get_bool(&self, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) -> Value {
        match self {
            ScratchValue::Num(value) => {
                // (*n != 0.0 && !n.is_nan()) as i64
                let zero = compiler.constants.get_float(0.0, builder);
                let is_not_zero = builder.ins().fcmp(FloatCC::NotEqual, *value, zero);
                let is_not_nan = builder.ins().fcmp(FloatCC::Equal, *value, *value);
                let res = builder.ins().band(is_not_zero, is_not_nan);
                let one = compiler.constants.get_int(1, builder);
                let zero = compiler.constants.get_int(0, builder);
                builder.ins().select(res, one, zero)
            }
            ScratchValue::Bool(value) => *value,
            ScratchValue::String([i2, i3, i4]) => {
                let i1 = compiler.constants.get_int(ID_STRING, builder);
                obj_to_bool(compiler, builder, i1, *i2, *i3, *i4)
            }
            ScratchValue::Object([i1, i2, i3, i4]) => {
                obj_to_bool(compiler, builder, *i1, *i2, *i3, *i4)
            }
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
            ScratchValue::String([i2, i3, i4]) => {
                let i1 = compiler.constants.get_int(ID_STRING, builder);
                obj_clone(compiler, builder, i1, *i2, *i3, *i4)
            }
            ScratchValue::Object([i1, i2, i3, i4]) => {
                obj_clone(compiler, builder, *i1, *i2, *i3, *i4)
            }
        }
    }

    pub fn get_object(
        &self,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    ) -> [Value; 4] {
        match self {
            ScratchValue::Num(n) => {
                let n = builder.ins().bitcast(I64, MemFlags::new(), *n);
                let id = constants.get_int(ID_NUMBER, builder);
                let zero = constants.get_int(0, builder);
                [id, n, zero, zero]
            }
            ScratchValue::Bool(n) => {
                let id = constants.get_int(ID_BOOL, builder);
                let zero = constants.get_int(0, builder);
                [id, *n, zero, zero]
            }
            ScratchValue::String([i2, i3, i4]) => {
                let i1 = constants.get_int(ID_STRING, builder);
                [i1, *i2, *i3, *i4]
            }
            ScratchValue::Object(n) => *n,
        }
    }
}

fn obj_clone(
    compiler: &mut Compiler<'_>,
    builder: &mut FunctionBuilder<'_>,
    i1: Value,
    i2: Value,
    i3: Value,
    i4: Value,
) -> ScratchValue {
    compiler.call_function(
        builder,
        callbacks::types::CLONE_OBJ,
        &[I64, I64, I64, I64, I64],
        &[],
        &[i1, i2, i3, i4, compiler.temp_slot4.0],
    );
    let i1 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 0);
    let i2 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 8);
    let i3 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 16);
    let i4 = builder.ins().stack_load(I64, compiler.temp_slot4.1, 24);
    ScratchValue::Object([i1, i2, i3, i4])
}

fn obj_to_bool(
    compiler: &mut Compiler<'_>,
    builder: &mut FunctionBuilder<'_>,
    i1: Value,
    i2: Value,
    i3: Value,
    i4: Value,
) -> Value {
    let ins = compiler.call_function(
        builder,
        callbacks::types::TO_BOOL,
        &[I64, I64, I64, I64],
        &[I64],
        &[i1, i2, i3, i4],
    );
    builder.inst_results(ins)[0]
}

fn get_string_from_obj(
    builder: &mut FunctionBuilder<'_>,
    compiler: &mut Compiler,
    i1: Value,
    i2: Value,
    i3: Value,
    i4: Value,
) -> Value {
    let stack_ptr = Compiler::ins_create_string_stack_slot(builder);

    compiler.call_function(
        builder,
        callbacks::types::TO_STRING,
        &[I64, I64, I64, I64, I64],
        &[],
        &[i1, i2, i3, i4, stack_ptr],
    );

    stack_ptr
}
