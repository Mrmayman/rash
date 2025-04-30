use std::sync::Mutex;

use cranelift::{
    codegen::ir::StackSlot,
    prelude::{
        types::{F64, I64},
        FloatCC, FunctionBuilder, InstBuilder, StackSlotData, StackSlotKind, Value,
    },
};

use crate::{
    callbacks,
    compiler::{Compiler, ScratchBlock},
    constant_set::ConstantMap,
    data_types::{ScratchObject, ID_BOOL, ID_NUMBER},
    ARITHMETIC_NAN_CHECK,
};

pub static STRINGS_TO_DROP: Mutex<Vec<[i64; 3]>> = Mutex::new(Vec::new());

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

impl Input {
    pub fn format(&self, indent: usize) -> String {
        match self {
            Input::Obj(scratch_object) => format!("{scratch_object:?}"),
            Input::Block(scratch_block) => format!("({})", scratch_block.format(indent)),
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
        Input::Obj(ScratchObject::String(s))
    }
}

impl From<&str> for Input {
    fn from(s: &str) -> Self {
        Input::Obj(ScratchObject::String(s.to_owned()))
    }
}

impl From<ScratchBlock> for Input {
    fn from(block: ScratchBlock) -> Self {
        Input::Block(Box::new(block))
    }
}

impl Input {
    pub fn get_number(&self, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) -> Value {
        let (mut num, could_be_nan) = match self {
            Input::Obj(scratch_object) => {
                let o = scratch_object.convert_to_number();
                (compiler.constants.get_float(o, builder), o.is_nan())
            }
            Input::Block(scratch_block) => {
                let could_be_nan = scratch_block.could_be_nan();
                let o = compiler.compile_block(scratch_block, builder).unwrap();
                (o.get_number(compiler, builder), could_be_nan)
            }
        };
        if ARITHMETIC_NAN_CHECK && could_be_nan {
            let is_not_nan = builder.ins().fcmp(FloatCC::Ordered, num, num);
            let zero_value = compiler.constants.get_float(0.0, builder);
            num = builder.ins().select(is_not_nan, num, zero_value);
        }

        num
    }

    pub fn get_number_int(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
    ) -> Value {
        let (mut num, could_be_nan) = match self {
            Input::Obj(scratch_object) => {
                let o = scratch_object.convert_to_number();
                (compiler.constants.get_int(o as i64, builder), o.is_nan())
            }
            Input::Block(scratch_block) => {
                let could_be_nan = scratch_block.could_be_nan();

                let o = compiler.compile_block(scratch_block, builder).unwrap();
                let get_number = o.get_number(compiler, builder);
                let number = builder.ins().fcvt_to_sint(I64, get_number);

                (number, could_be_nan)
            }
        };
        if ARITHMETIC_NAN_CHECK && could_be_nan {
            let is_not_nan = builder.ins().fcmp(FloatCC::Ordered, num, num);
            let zero_value = compiler.constants.get_float(0.0, builder);
            num = builder.ins().select(is_not_nan, num, zero_value);
        }

        num
    }

    pub fn get_string(
        &self,
        compiler: &mut Compiler,
        builder: &mut FunctionBuilder<'_>,
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
                // println!("Getting string {scratch_object:?}");
                let string = scratch_object.convert_to_string();

                let bytes: [i64; 3] = unsafe { std::mem::transmute(string) };
                STRINGS_TO_DROP.lock().unwrap().push(bytes);

                let val1 = compiler.constants.get_int(bytes[0], builder);
                let val2 = compiler.constants.get_int(bytes[1], builder);
                let val3 = compiler.constants.get_int(bytes[2], builder);

                // Store the values in the stack slot
                builder.ins().stack_store(val1, stack_slot, 0);
                builder.ins().stack_store(val2, stack_slot, 8);
                builder.ins().stack_store(val3, stack_slot, 16);

                (stack_ptr, true)
            }
            Input::Block(scratch_block) => {
                let o = compiler.compile_block(scratch_block, builder).unwrap();
                (o.get_string(compiler, builder), false)
            }
        }
    }

    pub fn get_bool(&self, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) -> Value {
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

    pub fn get_object(
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
                    STRINGS_TO_DROP.lock().unwrap().push([i2, i3, i4]);
                }

                let i1 = compiler.constants.get_int(i1, builder);
                let i2 = compiler.constants.get_int(i2, builder);
                let i3 = compiler.constants.get_int(i3, builder);
                let i4 = compiler.constants.get_int(i4, builder);
                [i1, i2, i3, i4]
            }
            Input::Block(scratch_block) => {
                let o = compiler.compile_block(scratch_block, builder).unwrap();
                match o {
                    ReturnValue::Object(arr) => arr,
                    ReturnValue::ObjectPointer(_, slot) => {
                        let i1 = builder.ins().stack_load(I64, slot, 0);
                        let i2 = builder.ins().stack_load(I64, slot, 8);
                        let i3 = builder.ins().stack_load(I64, slot, 16);
                        let i4 = builder.ins().stack_load(I64, slot, 24);
                        [i1, i2, i3, i4]
                    }
                    ReturnValue::Num(value) => {
                        let id = builder.ins().iconst(I64, ID_NUMBER);
                        let zero = compiler.constants.get_int(0, builder);
                        [id, value, zero, zero]
                    }
                    ReturnValue::Bool(value) => {
                        let id = builder.ins().iconst(I64, ID_BOOL);
                        let zero = compiler.constants.get_int(0, builder);
                        [id, value, zero, zero]
                    }
                }
            }
        }
    }

    pub fn get_number_with_decimal_check(
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
                    ReturnValue::Num(value) => (value, compiler.constants.get_int(0, builder)),
                    ReturnValue::Object([i1, i2, i3, i4]) => {
                        compiler.ins_call_to_num_with_decimal_check(builder, i1, i2, i3, i4)
                    }
                    ReturnValue::Bool(value) => (
                        builder.ins().fcvt_from_sint(F64, value),
                        compiler.constants.get_int(0, builder),
                    ),
                    ReturnValue::ObjectPointer(_value, slot) => {
                        let i1 = builder.ins().stack_load(I64, slot, 0);
                        let i2 = builder.ins().stack_load(I64, slot, 8);
                        let i3 = builder.ins().stack_load(I64, slot, 16);
                        let i4 = builder.ins().stack_load(I64, slot, 24);

                        compiler.ins_call_to_num_with_decimal_check(builder, i1, i2, i3, i4)
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
    Object([Value; 4]),
    ObjectPointer(Value, StackSlot),
}

impl ReturnValue {
    pub fn get_number(self, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) -> Value {
        match self {
            ReturnValue::Num(value) => value,
            ReturnValue::Object(arr) => {
                let num = compiler.call_function(
                    builder,
                    callbacks::types::to_number as usize,
                    &[I64, I64, I64, I64],
                    &[F64],
                    &arr,
                );
                builder.inst_results(num)[0]
            }
            ReturnValue::Bool(value) => builder.ins().fcvt_from_sint(F64, value),
            ReturnValue::ObjectPointer(_value, slot) => {
                let i1 = builder.ins().stack_load(I64, slot, 0);
                let i2 = builder.ins().stack_load(I64, slot, 8);
                let i3 = builder.ins().stack_load(I64, slot, 16);
                let i4 = builder.ins().stack_load(I64, slot, 24);

                // Convert the object to number
                let num = compiler.call_function(
                    builder,
                    callbacks::types::to_number as usize,
                    &[I64, I64, I64, I64],
                    &[F64],
                    &[i1, i2, i3, i4],
                );
                builder.inst_results(num)[0]
            }
        }
    }

    pub fn get_string(self, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) -> Value {
        match self {
            ReturnValue::Num(value) => {
                let stack_ptr = Compiler::ins_create_string_stack_slot(builder);

                compiler.call_function(
                    builder,
                    callbacks::types::to_string_from_num as usize,
                    &[F64, I64],
                    &[],
                    &[value, stack_ptr],
                );
                stack_ptr
            }
            ReturnValue::Object([i1, i2, i3, i4]) => {
                get_string_from_obj(builder, compiler, i1, i2, i3, i4)
            }
            ReturnValue::Bool(value) => {
                let stack_ptr = Compiler::ins_create_string_stack_slot(builder);

                compiler.call_function(
                    builder,
                    callbacks::types::to_string_from_bool as usize,
                    &[I64, I64],
                    &[],
                    &[value, stack_ptr],
                );
                stack_ptr
            }
            ReturnValue::ObjectPointer(_value, slot) => {
                // read 4 i64 from pointer
                let i1 = builder.ins().stack_load(I64, slot, 0);
                let i2 = builder.ins().stack_load(I64, slot, 8);
                let i3 = builder.ins().stack_load(I64, slot, 16);
                let i4 = builder.ins().stack_load(I64, slot, 24);

                get_string_from_obj(builder, compiler, i1, i2, i3, i4)
            }
        }
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, ReturnValue::Bool(_))
    }

    fn get_bool(&self, compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>) -> Value {
        match self {
            ReturnValue::Num(value) => {
                // (*n != 0.0 && !n.is_nan()) as i64
                let zero = compiler.constants.get_float(0.0, builder);
                let is_not_zero = builder.ins().fcmp(FloatCC::NotEqual, *value, zero);
                let is_not_nan = builder.ins().fcmp(FloatCC::Equal, *value, *value);
                let res = builder.ins().band(is_not_zero, is_not_nan);
                let one = compiler.constants.get_int(1, builder);
                let zero = compiler.constants.get_int(0, builder);
                builder.ins().select(res, one, zero)
            }
            ReturnValue::Bool(value) => *value,
            ReturnValue::Object([i1, i2, i3, i4]) => {
                let ins = compiler.call_function(
                    builder,
                    callbacks::types::to_bool as usize,
                    &[I64, I64, I64, I64],
                    &[I64],
                    &[*i1, *i2, *i3, *i4],
                );
                builder.inst_results(ins)[0]
            }
            ReturnValue::ObjectPointer(_value, stack_slot) => {
                let i1 = builder.ins().stack_load(I64, *stack_slot, 0);
                let i2 = builder.ins().stack_load(I64, *stack_slot, 8);
                let i3 = builder.ins().stack_load(I64, *stack_slot, 16);
                let i4 = builder.ins().stack_load(I64, *stack_slot, 24);

                let ins = compiler.call_function(
                    builder,
                    callbacks::types::to_bool as usize,
                    &[I64, I64, I64, I64],
                    &[I64],
                    &[i1, i2, i3, i4],
                );
                builder.inst_results(ins)[0]
            }
        }
    }
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
        callbacks::types::to_string as usize,
        &[I64, I64, I64, I64, I64],
        &[],
        &[i1, i2, i3, i4, stack_ptr],
    );

    stack_ptr
}
