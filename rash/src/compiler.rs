use std::{collections::HashMap, sync::Mutex};

use cranelift::prelude::*;
use lazy_static::lazy_static;
use types::F64;

use crate::{
    callbacks,
    constant_set::ConstantMap,
    data_types::ScratchObject,
    input_primitives::{Input, Ptr, ReturnValue},
    scheduler::CustomBlockId,
    stack_cache::StackCache,
};

lazy_static! {
    pub static ref MEMORY: Mutex<Box<[ScratchObject]>> =
        Mutex::new(vec![ScratchObject::Number(0.0); 1024].into_boxed_slice());
}

#[allow(unused)]
#[derive(Debug)]
pub enum ScratchBlock {
    /// Sets a variable to a value.
    VarSet(Ptr, Input),
    /// Sets a variable to variable + input.
    ///
    /// Basically like the `x += a` operation.
    VarChange(Ptr, Input),
    /// Reads a value from a value and returns it.
    /// Meant to be used as the input to other blocks.
    VarRead(Ptr),
    OpAdd(Input, Input),
    OpSub(Input, Input),
    OpMul(Input, Input),
    OpDiv(Input, Input),
    OpRound(Input),
    OpStrJoin(Input, Input),
    OpMod(Input, Input),
    OpStrLen(Input),
    OpBAnd(Input, Input),
    OpBNot(Input),
    OpBOr(Input, Input),
    OpMFloor(Input),
    OpMAbs(Input),
    OpMSqrt(Input),
    OpMSin(Input),
    OpMCos(Input),
    OpMTan(Input),
    OpCmpGreater(Input, Input),
    OpCmpLesser(Input, Input),
    OpRandom(Input, Input),
    OpStrLetterOf(Input, Input),
    OpStrContains(Input, Input),
    ControlIf(Input, Vec<ScratchBlock>),
    ControlIfElse(Input, Vec<ScratchBlock>, Vec<ScratchBlock>),
    /// A repeat loop that *doesn't support* screen refresh.
    /// MUCH faster than [`ScratchBlock::ControlRepeatScreenRefresh`]
    /// but you can't use [`ScratchBlock::ScreenRefresh`] inside.
    ControlRepeat(Input, Vec<ScratchBlock>),
    /// A repeat loop that *supports* screen refresh
    /// (pausing/resuming of code). It is slower than
    /// [`ScratchBlock::ControlRepeat`] which doesn't
    /// support screen refresh.
    ///
    /// This loop doesn't provide screen refresh by default
    /// but you can insert [`ScratchBlock::ScreenRefresh`]
    /// inside it.
    ControlRepeatScreenRefresh(Input, Vec<ScratchBlock>),
    /// Repeats until a condition is true.
    ControlRepeatUntil(Input, Vec<ScratchBlock>),
    ControlStopThisScript,
    FunctionCallNoScreenRefresh(CustomBlockId, Vec<Input>),
    FunctionGetArg(usize),
    /// A block to trigger a screen refresh.
    ///
    /// Similar to coroutines in other languages,
    /// screen refresh allows a script to pause and
    /// resume.
    ///
    /// Note: if you use this inside a repeat loop,
    /// make sure to use
    /// [`ScratchBlock::ControlRepeatScreenRefresh`]
    ScreenRefresh,
}

#[derive(PartialEq, Eq, Debug)]
pub enum VarTypeChecked {
    Number,
    Bool,
    String,
    Unknown,
}

impl From<VarType> for VarTypeChecked {
    fn from(value: VarType) -> Self {
        match value {
            VarType::Number => Self::Number,
            VarType::Bool => Self::Bool,
            VarType::String => Self::String,
        }
    }
}

impl ScratchBlock {
    pub fn return_type(
        &self,
        variable_type_data: &HashMap<Ptr, VarType>,
    ) -> Option<VarTypeChecked> {
        match self {
            ScratchBlock::VarRead(ptr) => match variable_type_data.get(ptr) {
                Some(vartype) => Some((*vartype).into()),
                None => Some(VarTypeChecked::Unknown),
            },
            ScratchBlock::FunctionGetArg(_) => Some(VarTypeChecked::Unknown),
            ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpDiv(_, _)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpRandom(_, _)
            | ScratchBlock::OpMFloor(_)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpMAbs(_)
            | ScratchBlock::OpMSqrt(_)
            | ScratchBlock::OpMSin(_)
            | ScratchBlock::OpMCos(_)
            | ScratchBlock::OpMTan(_)
            | ScratchBlock::OpStrLen(_) => Some(VarTypeChecked::Number),
            ScratchBlock::OpStrLetterOf(_, _) | ScratchBlock::OpStrJoin(_, _) => {
                Some(VarTypeChecked::String)
            }
            ScratchBlock::OpBAnd(_, _)
            | ScratchBlock::OpBNot(_)
            | ScratchBlock::OpBOr(_, _)
            | ScratchBlock::OpCmpGreater(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::OpCmpLesser(_, _) => Some(VarTypeChecked::Bool),
            ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::ControlIf(_, _)
            | ScratchBlock::ControlIfElse(_, _, _)
            | ScratchBlock::ControlRepeatScreenRefresh(_, _)
            | ScratchBlock::ControlRepeat(_, _)
            | ScratchBlock::ScreenRefresh
            | ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
            | ScratchBlock::ControlRepeatUntil(_, _) => None,
        }
    }

    pub fn affects_var(
        &self,
        var_ptr: Ptr,
        variable_type_data: &HashMap<Ptr, VarType>,
    ) -> Option<VarTypeChecked> {
        match self {
            ScratchBlock::VarSet(ptr, input) => {
                if var_ptr == *ptr {
                    match input {
                        Input::Obj(scratch_object) => Some(scratch_object.get_type().into()),
                        Input::Block(scratch_block) => {
                            scratch_block.return_type(variable_type_data)
                        }
                    }
                } else {
                    None
                }
            }
            ScratchBlock::VarChange(ptr, _) => {
                if var_ptr == *ptr {
                    Some(VarTypeChecked::Number)
                } else {
                    None
                }
            }
            ScratchBlock::ControlIf(_, vec)
            | ScratchBlock::ControlRepeat(_, vec)
            | ScratchBlock::ControlRepeatUntil(_, vec) => vec
                .iter()
                .filter_map(|n| n.affects_var(var_ptr, variable_type_data))
                .last(),
            ScratchBlock::ControlIfElse(_, then, else_block) => {
                let then = then
                    .iter()
                    .map(|n| n.affects_var(var_ptr, variable_type_data))
                    .last()
                    .flatten();
                let else_block = else_block
                    .iter()
                    .map(|n| n.affects_var(var_ptr, variable_type_data))
                    .last()
                    .flatten();

                match (then, else_block) {
                    (None, None) => None,
                    (None, Some(n)) | (Some(n), None) => Some(n),
                    (Some(a), Some(b)) => (a == b).then_some(a),
                }
            }
            _ => None,
        }
    }

    pub fn could_be_nan(&self) -> bool {
        match self {
            ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::ControlIf(_, _)
            | ScratchBlock::ControlIfElse(_, _, _)
            | ScratchBlock::ControlRepeat(_, _)
            | ScratchBlock::ControlRepeatUntil(_, _)
            | ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpStrJoin(_, _)
            | ScratchBlock::OpStrLen(_)
            | ScratchBlock::OpCmpGreater(_, _)
            | ScratchBlock::OpBAnd(_, _)
            | ScratchBlock::OpBNot(_)
            | ScratchBlock::OpBOr(_, _)
            | ScratchBlock::OpMFloor(_)
            | ScratchBlock::OpMAbs(_)
            | ScratchBlock::OpStrLetterOf(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpMSin(_)
            | ScratchBlock::OpMCos(_)
            | ScratchBlock::OpMTan(_)
            | ScratchBlock::ScreenRefresh
            | ScratchBlock::ControlRepeatScreenRefresh(_, _)
            | ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
            | ScratchBlock::OpCmpLesser(_, _) => false,
            ScratchBlock::VarRead(_)
            | ScratchBlock::OpDiv(_, _)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpMSqrt(_)
            | ScratchBlock::FunctionGetArg(_)
            | ScratchBlock::OpRandom(_, _) => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Number,
    Bool,
    String,
}

pub struct Compiler<'compiler> {
    pub variable_type_data: HashMap<Ptr, VarType>,
    pub args_list: Vec<[Value; 4]>,
    pub constants: ConstantMap,
    pub code_block: Block,
    pub cache: StackCache,
    pub break_counter: usize,
    pub break_points: Vec<Block>,
    pub memory: &'compiler [ScratchObject],
    pub vec_ptr: Value,
    pub scheduler_ptr: Value,
}

impl<'a> Compiler<'a> {
    pub fn new(
        block: Block,
        builder: &mut FunctionBuilder<'_>,
        code: &[ScratchBlock],
        memory: &'a [ScratchObject],
        vec_ptr: Value,
        scheduler_ptr: Value,
        args_list: Vec<[Value; 4]>,
    ) -> Self {
        Self {
            variable_type_data: HashMap::new(),
            constants: ConstantMap::new(),
            code_block: block,
            cache: StackCache::new(builder, code),
            break_points: Vec::new(),
            break_counter: 0,
            memory,
            scheduler_ptr,
            vec_ptr,
            args_list,
        }
    }

    pub fn compile_block(
        &mut self,
        block: &ScratchBlock,
        builder: &mut FunctionBuilder<'_>,
    ) -> Option<ReturnValue> {
        match block {
            ScratchBlock::VarSet(ptr, obj) => {
                self.var_set(obj, builder, *ptr);
            }
            ScratchBlock::OpAdd(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fadd(a, b);
                return Some(ReturnValue::Num(res));
            }
            ScratchBlock::OpSub(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fsub(a, b);
                return Some(ReturnValue::Num(res));
            }
            ScratchBlock::OpMul(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fmul(a, b);
                return Some(ReturnValue::Num(res));
            }
            ScratchBlock::OpDiv(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fdiv(a, b);
                return Some(ReturnValue::Num(res));
            }
            ScratchBlock::OpMod(a, b) => {
                return Some(ReturnValue::Num(self.op_modulo(a, b, builder)));
            }
            ScratchBlock::VarRead(ptr) => {
                return Some(self.var_read(builder, *ptr));
            }
            ScratchBlock::OpStrJoin(a, b) => {
                return Some(ReturnValue::Object(self.op_str_join(a, b, builder)));
            }
            ScratchBlock::ControlRepeat(input, vec) => {
                self.control_repeat(builder, input, vec, false);
            }
            ScratchBlock::ControlRepeatScreenRefresh(input, vec) => {
                self.control_repeat(builder, input, vec, true);
            }
            ScratchBlock::VarChange(ptr, input) => {
                self.var_change(input, builder, *ptr);
            }
            ScratchBlock::ControlIf(input, vec) => {
                self.control_if_statement(input, builder, vec);
            }
            ScratchBlock::ControlIfElse(condition, then_block, else_block) => {
                self.control_if_else(condition, builder, then_block, else_block);
            }
            ScratchBlock::ControlRepeatUntil(input, vec) => {
                self.control_repeat_until(builder, input, vec);
            }
            ScratchBlock::OpCmpGreater(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fcmp(FloatCC::GreaterThan, a, b);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpCmpLesser(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fcmp(FloatCC::LessThan, a, b);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpStrLen(input) => {
                return Some(self.op_str_len(input, builder));
            }
            ScratchBlock::OpRandom(a, b) => return Some(self.op_random(a, b, builder)),
            ScratchBlock::OpBAnd(a, b) => {
                let a = a.get_bool(self, builder);
                let b = b.get_bool(self, builder);
                let res = builder.ins().band(a, b);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpBNot(a) => {
                let a = a.get_bool(self, builder);
                let res = builder.ins().bnot(a);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpBOr(a, b) => {
                let a = a.get_bool(self, builder);
                let b = b.get_bool(self, builder);
                let res = builder.ins().bor(a, b);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpMFloor(n) => return Some(self.op_m_floor(n, builder)),
            ScratchBlock::OpStrLetterOf(letter, string) => {
                return Some(ReturnValue::Object(
                    self.op_str_letter(letter, string, builder),
                ))
            }
            ScratchBlock::OpStrContains(string, pattern) => {
                return Some(ReturnValue::Bool(
                    self.op_str_contains(string, pattern, builder),
                ))
            }
            ScratchBlock::OpRound(num) => {
                return Some(ReturnValue::Num(self.op_round(num, builder)))
            }
            ScratchBlock::OpMAbs(num) => {
                let num = num.get_number(self, builder);
                let abs = builder.ins().fabs(num);
                return Some(ReturnValue::Num(abs));
            }
            ScratchBlock::OpMSqrt(num) => {
                let num = num.get_number(self, builder);
                let sqrt = builder.ins().sqrt(num);
                return Some(ReturnValue::Num(sqrt));
            }
            ScratchBlock::OpMSin(num) => {
                let num = num.get_number(self, builder);
                let inst =
                    self.call_function(builder, callbacks::op_sin as usize, &[F64], &[F64], &[num]);
                let result = builder.inst_results(inst)[0];
                return Some(ReturnValue::Num(result));
            }
            ScratchBlock::OpMCos(num) => {
                let num = num.get_number(self, builder);
                let inst =
                    self.call_function(builder, callbacks::op_cos as usize, &[F64], &[F64], &[num]);
                let result = builder.inst_results(inst)[0];
                return Some(ReturnValue::Num(result));
            }
            ScratchBlock::OpMTan(num) => {
                let num = num.get_number(self, builder);
                let inst =
                    self.call_function(builder, callbacks::op_tan as usize, &[F64], &[F64], &[num]);
                let result = builder.inst_results(inst)[0];
                return Some(ReturnValue::Num(result));
            }
            ScratchBlock::ScreenRefresh => {
                self.screen_refresh(builder);
            }
            ScratchBlock::ControlStopThisScript => {
                self.control_stop_this_script(builder);
            }
            ScratchBlock::FunctionCallNoScreenRefresh(custom_block_id, args) => {
                self.call_no_screen_refresh(custom_block_id, builder, args);
            }
            ScratchBlock::FunctionGetArg(idx) => {
                return Some(ReturnValue::Object(self.args_list[*idx]))
            }
        }
        None
    }

    fn screen_refresh(&mut self, builder: &mut FunctionBuilder<'_>) {
        self.break_counter += 1;
        self.cache.save(builder, &mut self.constants, self.memory);
        let break_counter = self.constants.get_int(self.break_counter as i64, builder);

        builder.ins().return_(&[break_counter]);
        self.constants.clear();
        self.code_block = builder.create_block();
        self.break_points.push(self.code_block);
        builder.switch_to_block(self.code_block);

        self.cache.init(builder, self.memory, &mut self.constants);
    }
}

pub fn print_func_addresses() {
    println!("var_read: {:X}", callbacks::var_read as usize);
    println!("op_str_join: {:X}", callbacks::op_str_join as usize);
    println!("f64::floor: {:X}", f64::floor as usize);
    println!(
        "to_string_from_num: {:X}",
        callbacks::types::to_string_from_num as usize
    );
    println!("to_string: {:X}", callbacks::types::to_string as usize);
    println!(
        "to_string_from_bool: {:X}",
        callbacks::types::to_string_from_bool as usize
    );
    println!("to_number: {:X}", callbacks::types::to_number as usize);
    println!("drop_obj: {:X}", callbacks::types::drop_obj as usize);
}
