use std::{collections::HashMap, sync::Mutex};

use cranelift::prelude::{
    types::{F64, I64},
    Block, FunctionBuilder, InstBuilder, Value,
};
use lazy_static::lazy_static;
use rash_render::{RunState, SpriteId};

use crate::{
    callbacks,
    constant_set::ConstantMap,
    data_types::ScratchObject,
    input_primitives::{Input, Ptr, ReturnValue},
    scheduler::CustomBlockId,
    stack_cache::StackCache,
};

mod display;

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
    MotionGoToXY(Input, Input),
    MotionChangeX(Input),
    MotionChangeY(Input),
    MotionSetX(Input),
    MotionSetY(Input),
    MotionGetX,
    MotionGetY,
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
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
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
            | ScratchBlock::MotionGoToXY(_, _)
            | ScratchBlock::MotionChangeX(_)
            | ScratchBlock::MotionChangeY(_)
            | ScratchBlock::MotionSetX(_)
            | ScratchBlock::MotionSetY(_)
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
            | ScratchBlock::MotionGoToXY(_, _)
            | ScratchBlock::MotionChangeX(_)
            | ScratchBlock::MotionChangeY(_)
            | ScratchBlock::MotionSetX(_)
            | ScratchBlock::MotionSetY(_)
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
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
    pub graphics_ptr: Value,
    pub sprite_id: SpriteId,
}

impl<'a> Compiler<'a> {
    pub fn new(
        block: Block,
        builder: &mut FunctionBuilder<'_>,
        code: &[ScratchBlock],
        memory: &'a [ScratchObject],
        vec_ptr: Value,
        scheduler_ptr: Value,
        graphics_ptr: Value,
        args_list: Vec<[Value; 4]>,
        sprite_id: SpriteId,
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
            graphics_ptr,
            sprite_id,
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
                return Some(ReturnValue::Num(self.op_add(a, b, builder)));
            }
            ScratchBlock::OpSub(a, b) => {
                return Some(ReturnValue::Num(self.op_sub(a, b, builder)));
            }
            ScratchBlock::OpMul(a, b) => {
                return Some(ReturnValue::Num(self.op_mul(a, b, builder)));
            }
            ScratchBlock::OpDiv(a, b) => {
                return Some(ReturnValue::Num(self.op_div(a, b, builder)));
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
                return Some(ReturnValue::Bool(self.op_cmp_gt(a, b, builder)));
            }
            ScratchBlock::OpCmpLesser(a, b) => {
                return Some(ReturnValue::Bool(self.op_cmp_lt(a, b, builder)));
            }
            ScratchBlock::OpStrLen(input) => {
                return Some(self.op_str_len(input, builder));
            }
            ScratchBlock::OpRandom(a, b) => return Some(self.op_random(a, b, builder)),
            ScratchBlock::OpBAnd(a, b) => {
                return Some(ReturnValue::Bool(self.op_b_and(a, b, builder)));
            }
            ScratchBlock::OpBNot(a) => {
                return Some(ReturnValue::Bool(self.op_b_not(a, builder)));
            }
            ScratchBlock::OpBOr(a, b) => {
                return Some(ReturnValue::Bool(self.op_b_or(a, b, builder)));
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
                return Some(ReturnValue::Num(self.op_m_abs(num, builder)));
            }
            ScratchBlock::OpMSqrt(num) => {
                return Some(ReturnValue::Num(self.op_m_sqrt(num, builder)));
            }
            ScratchBlock::OpMSin(num) => {
                return Some(ReturnValue::Num(self.op_m_sin(num, builder)));
            }
            ScratchBlock::OpMCos(num) => {
                return Some(ReturnValue::Num(self.op_m_cos(num, builder)));
            }
            ScratchBlock::OpMTan(num) => {
                return Some(ReturnValue::Num(self.op_m_tan(num, builder)));
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
            ScratchBlock::MotionGoToXY(x, y) => {
                let x = x.get_number(self, builder);
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    RunState::c_go_to as usize,
                    &[I64, I64, F64, F64],
                    &[],
                    &[self.graphics_ptr, id, x, y],
                );
            }
            ScratchBlock::MotionChangeX(x) => {
                let x = x.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    RunState::c_change_x as usize,
                    &[I64, I64, F64],
                    &[],
                    &[self.graphics_ptr, id, x],
                );
            }
            ScratchBlock::MotionChangeY(y) => {
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    RunState::c_change_y as usize,
                    &[I64, I64, F64],
                    &[],
                    &[self.graphics_ptr, id, y],
                );
            }
            ScratchBlock::MotionSetX(x) => {
                let x = x.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    RunState::c_set_x as usize,
                    &[I64, I64, F64],
                    &[],
                    &[self.graphics_ptr, id, x],
                );
            }
            ScratchBlock::MotionSetY(y) => {
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    RunState::c_set_y as usize,
                    &[I64, I64, F64],
                    &[],
                    &[self.graphics_ptr, id, y],
                );
            }
            ScratchBlock::MotionGetX => {
                let id = self.constants.get_int(self.sprite_id.0, builder);

                let inst = self.call_function(
                    builder,
                    RunState::c_get_x as usize,
                    &[I64, I64],
                    &[F64],
                    &[self.graphics_ptr, id],
                );

                let val = builder.inst_results(inst)[0];
                return Some(ReturnValue::Num(val));
            }
            ScratchBlock::MotionGetY => {
                let id = self.constants.get_int(self.sprite_id.0, builder);

                let inst = self.call_function(
                    builder,
                    RunState::c_get_y as usize,
                    &[I64, I64],
                    &[F64],
                    &[self.graphics_ptr, id],
                );

                let val = builder.inst_results(inst)[0];
                return Some(ReturnValue::Num(val));
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

#[allow(unused)]
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
