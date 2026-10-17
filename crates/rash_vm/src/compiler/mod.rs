use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use bimap::BiHashMap;
use cranelift::{
    codegen::{
        ir::{
            AbiParam, ExtFuncData, ExternalName, FuncRef, SigRef, StackSlot, Type,
            UserExternalName, UserExternalNameRef,
        },
        isa::CallConv,
    },
    prelude::{
        Block, FunctionBuilder, InstBuilder, Signature, Value,
        types::{F64, I64},
    },
};
use smol_str::SmolStr;

use crate::{
    callbacks::{
        self,
        external::{looks, motion},
    },
    constant_set::ConstantMap,
    data_types::{ID_BOOL, ID_NUMBER, ID_STRING, ScratchObject},
    effects::{Effects, VariableWrite, analyze},
    input_primitives::{Input, Ptr, ScratchValue},
    jit_func::JitArgs,
    runtime::CustomBlockId,
    variable_storage::{GenericVarStore, SsaVarStore, VarStore},
};
use rash_core::SpriteId;

mod display;

#[derive(Debug, PartialEq, Clone)]
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
    OpCmp(Input, Input, Ordering),
    OpRandom(Input, Input),
    OpStrLetterOf(Input, Input),
    OpStrContains(Input, Input),
    ControlIf(Input, Vec<ScratchBlock>),
    ControlIfElse(Input, Vec<ScratchBlock>, Vec<ScratchBlock>),
    /// A repeat loop that *supports* screen refresh
    /// (pausing/resuming of code).
    ///
    /// This loop doesn't provide screen refresh by default
    /// but you can insert [`ScratchBlock::ScreenRefresh`]
    /// inside it.
    ControlRepeat(Input, Vec<ScratchBlock>),
    /// Repeats until a condition is true.
    ControlRepeatUntil(Input, Vec<ScratchBlock>),
    ControlForever(Vec<ScratchBlock>),
    ControlStopThisScript,
    FunctionCallNoScreenRefresh(CustomBlockId, Vec<Input>),
    FunctionCallScreenRefresh(CustomBlockId, Vec<Input>),
    FunctionGetArg(usize),
    #[doc = include_str!("../../../../docs/LOCALS_WITH.md")]
    LocalsWith(Vec<Input>, Vec<ScratchBlock>),
    /// Gets the nth local from the most recently entered [`ScratchBlock::LocalsWith`].
    ///
    /// ```txt
    /// with [a, b, c] {
    ///     with [d, e] {
    ///         LocalsGet(1) // returns e, not b
    ///     }
    /// }
    /// ```
    LocalsGet(usize),
    /// A block to trigger a screen refresh.
    ///
    /// Similar to coroutines in other languages,
    /// screen refresh allows a script to pause and
    /// resume.
    ///
    /// Note: don't insert this explicitly in a repeat
    /// loop as screen refreshes are auto-inserted in
    /// loops matching Scratch behaviour.
    ScreenRefresh,
    MotionGoToXY(Input, Input),
    MotionChangeX(Input),
    MotionChangeY(Input),
    MotionSetX(Input),
    MotionSetY(Input),
    MotionGetX,
    MotionGetY,
    LooksShown(bool),
    SensingDaysSince2000,

    Log(Input),
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy)]
pub enum VarTypeChecked {
    Number,
    Bool,
    String,
    #[default]
    Object,
}

impl VarTypeChecked {
    pub fn get_id(self) -> Option<i64> {
        match self {
            Self::Number => Some(ID_NUMBER),
            Self::Bool => Some(ID_BOOL),
            Self::String => Some(ID_STRING),
            Self::Object => None,
        }
    }
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

impl From<Option<VarType>> for VarTypeChecked {
    fn from(value: Option<VarType>) -> Self {
        match value {
            Some(n) => n.into(),
            None => Self::Object,
        }
    }
}

impl From<VarTypeChecked> for Option<VarType> {
    fn from(val: VarTypeChecked) -> Self {
        Some(match val {
            VarTypeChecked::Number => VarType::Number,
            VarTypeChecked::Bool => VarType::Bool,
            VarTypeChecked::String => VarType::String,
            VarTypeChecked::Object => return None,
        })
    }
}

impl ScratchBlock {
    #[must_use]
    pub fn return_type(
        &self,
        mut vartype: impl FnMut(Ptr) -> VariableWrite,
        localtype: impl Fn(usize) -> VariableWrite,
    ) -> Option<VariableWrite> {
        match self {
            ScratchBlock::VarRead(ptr) => Some(vartype(*ptr)),
            ScratchBlock::LocalsGet(n) => Some(localtype(*n)),
            ScratchBlock::OpRandom(_, _)
            | ScratchBlock::OpMSqrt(_)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpDiv(_, _) => Some(VariableWrite::normal(VarTypeChecked::Number)),
            ScratchBlock::FunctionGetArg(_) => Some(VariableWrite::normal(VarTypeChecked::Object)),

            ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpMFloor(_)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpMAbs(_)
            | ScratchBlock::OpMSin(_)
            | ScratchBlock::OpMCos(_)
            | ScratchBlock::OpMTan(_)
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::OpStrLen(_) => Some(VariableWrite::skip_nan(VarTypeChecked::Number)),
            ScratchBlock::OpStrLetterOf(_, _) | ScratchBlock::OpStrJoin(_, _) => {
                Some(VariableWrite::normal(VarTypeChecked::String))
            }
            ScratchBlock::OpBAnd(_, _)
            | ScratchBlock::OpBNot(_)
            | ScratchBlock::OpBOr(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::OpCmp(_, _, _) => Some(VariableWrite::normal(VarTypeChecked::Bool)),
            ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::ControlIf(_, _)
            | ScratchBlock::ControlIfElse(_, _, _)
            | ScratchBlock::ControlRepeat(_, _)
            | ScratchBlock::ControlForever(_)
            | ScratchBlock::ScreenRefresh
            | ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
            | ScratchBlock::FunctionCallScreenRefresh(_, _)
            | ScratchBlock::MotionGoToXY(_, _)
            | ScratchBlock::MotionChangeX(_)
            | ScratchBlock::MotionChangeY(_)
            | ScratchBlock::MotionSetX(_)
            | ScratchBlock::MotionSetY(_)
            | ScratchBlock::ControlRepeatUntil(_, _)
            | ScratchBlock::LooksShown(_)
            | ScratchBlock::LocalsWith(_, _)
            | ScratchBlock::Log(_) => None,
        }
    }

    #[must_use]
    pub fn could_trigger_refresh(&self) -> bool {
        match self {
            ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::VarRead(_)
            | ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpDiv(_, _)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpStrJoin(_, _)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpStrLen(_)
            | ScratchBlock::OpBAnd(_, _)
            | ScratchBlock::OpBNot(_)
            | ScratchBlock::OpBOr(_, _)
            | ScratchBlock::OpMFloor(_)
            | ScratchBlock::OpMAbs(_)
            | ScratchBlock::OpMSqrt(_)
            | ScratchBlock::OpMSin(_)
            | ScratchBlock::OpMCos(_)
            | ScratchBlock::OpMTan(_)
            | ScratchBlock::OpCmp(_, _, _)
            | ScratchBlock::OpRandom(_, _)
            | ScratchBlock::OpStrLetterOf(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionGetArg(_)
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
            | ScratchBlock::LocalsGet(_)
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY => false,

            ScratchBlock::ControlIf(_, blocks)
            | ScratchBlock::ControlRepeatUntil(_, blocks)
            | ScratchBlock::ControlForever(blocks)
            | ScratchBlock::LocalsWith(_, blocks)
            | ScratchBlock::ControlRepeat(_, blocks) => {
                blocks.iter().any(ScratchBlock::could_trigger_refresh)
            }
            ScratchBlock::ControlIfElse(_, blocks_then, blocks_else) => {
                blocks_then.iter().any(ScratchBlock::could_trigger_refresh)
                    || blocks_else.iter().any(ScratchBlock::could_trigger_refresh)
            }
            ScratchBlock::LooksShown(b) => *b, // Triggers refresh on show, not hide

            // TODO: This actually depends on the function itself
            // Remove from here eventually
            ScratchBlock::FunctionCallScreenRefresh(_, _) => true,

            ScratchBlock::ScreenRefresh
            | ScratchBlock::Log(_)
            | ScratchBlock::MotionGoToXY(_, _)
            | ScratchBlock::MotionChangeX(_)
            | ScratchBlock::MotionChangeY(_)
            | ScratchBlock::MotionSetX(_)
            | ScratchBlock::MotionSetY(_) => true,
        }
    }

    pub fn walk(&self, on_block: &mut impl FnMut(&ScratchBlock), walk_inputs: bool) {
        on_block(self);
        match self {
            ScratchBlock::ControlIf(i, blocks)
            | ScratchBlock::ControlRepeat(i, blocks)
            | ScratchBlock::ControlRepeatUntil(i, blocks) => {
                if let Input::Block(b) = i
                    && walk_inputs
                {
                    b.walk(on_block, walk_inputs);
                }
                for block in blocks {
                    block.walk(on_block, walk_inputs);
                }
            }
            ScratchBlock::LocalsWith(inputs, blocks) => {
                for i in inputs {
                    if let Input::Block(b) = i
                        && walk_inputs
                    {
                        b.walk(on_block, walk_inputs);
                    }
                }
                for block in blocks {
                    block.walk(on_block, walk_inputs);
                }
            }
            ScratchBlock::ControlForever(blocks) => {
                for block in blocks {
                    block.walk(on_block, walk_inputs);
                }
            }
            ScratchBlock::ControlIfElse(i, blocks1, blocks2) => {
                if let Input::Block(b) = i
                    && walk_inputs
                {
                    b.walk(on_block, walk_inputs);
                }
                for block in blocks1 {
                    block.walk(on_block, walk_inputs);
                }
                for block in blocks2 {
                    block.walk(on_block, walk_inputs);
                }
            }
            ScratchBlock::VarSet(_, i)
            | ScratchBlock::VarChange(_, i)
            | ScratchBlock::Log(i)
            | ScratchBlock::MotionChangeX(i)
            | ScratchBlock::MotionChangeY(i)
            | ScratchBlock::MotionSetX(i)
            | ScratchBlock::MotionSetY(i)
            | ScratchBlock::OpRound(i)
            | ScratchBlock::OpMAbs(i)
            | ScratchBlock::OpMSqrt(i)
            | ScratchBlock::OpMSin(i)
            | ScratchBlock::OpMCos(i)
            | ScratchBlock::OpMTan(i)
            | ScratchBlock::OpStrLen(i)
            | ScratchBlock::OpMFloor(i)
            | ScratchBlock::OpBNot(i) => {
                if let Input::Block(b) = i
                    && walk_inputs
                {
                    b.walk(on_block, walk_inputs);
                }
            }
            ScratchBlock::MotionGoToXY(a, b)
            | ScratchBlock::OpAdd(a, b)
            | ScratchBlock::OpSub(a, b)
            | ScratchBlock::OpMul(a, b)
            | ScratchBlock::OpDiv(a, b)
            | ScratchBlock::OpStrJoin(a, b)
            | ScratchBlock::OpMod(a, b)
            | ScratchBlock::OpBAnd(a, b)
            | ScratchBlock::OpBOr(a, b)
            | ScratchBlock::OpCmp(a, b, _)
            | ScratchBlock::OpRandom(a, b)
            | ScratchBlock::OpStrLetterOf(a, b)
            | ScratchBlock::OpStrContains(a, b) => {
                if walk_inputs {
                    if let Input::Block(b) = a {
                        b.walk(on_block, walk_inputs);
                    }
                    if let Input::Block(b) = b {
                        b.walk(on_block, walk_inputs);
                    }
                }
            }

            ScratchBlock::FunctionCallNoScreenRefresh(_, l)
            | ScratchBlock::FunctionCallScreenRefresh(_, l) => {
                if walk_inputs {
                    for i in l {
                        if let Input::Block(b) = i {
                            b.walk(on_block, walk_inputs);
                        }
                    }
                }
            }

            ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionGetArg(_)
            | ScratchBlock::ScreenRefresh
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
            | ScratchBlock::LooksShown(_)
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::LocalsGet(_)
            | ScratchBlock::VarRead(_) => {}
        }
    }

    pub fn walk_mut(&mut self, on_block: &mut impl FnMut(&mut ScratchBlock), walk_inputs: bool) {
        on_block(self);
        match self {
            ScratchBlock::ControlIf(i, blocks)
            | ScratchBlock::ControlRepeat(i, blocks)
            | ScratchBlock::ControlRepeatUntil(i, blocks) => {
                if let Input::Block(b) = i
                    && walk_inputs
                {
                    b.walk_mut(on_block, walk_inputs);
                }
                for block in blocks {
                    block.walk_mut(on_block, walk_inputs);
                }
            }
            ScratchBlock::ControlForever(blocks) => {
                for block in blocks {
                    block.walk_mut(on_block, walk_inputs);
                }
            }
            ScratchBlock::ControlIfElse(i, blocks1, blocks2) => {
                if let Input::Block(b) = i
                    && walk_inputs
                {
                    b.walk_mut(on_block, walk_inputs);
                }
                for block in blocks1 {
                    block.walk_mut(on_block, walk_inputs);
                }
                for block in blocks2 {
                    block.walk_mut(on_block, walk_inputs);
                }
            }
            ScratchBlock::LocalsWith(inputs, blocks) => {
                for i in inputs {
                    if let Input::Block(b) = i
                        && walk_inputs
                    {
                        b.walk_mut(on_block, walk_inputs);
                    }
                }
                for block in blocks {
                    block.walk_mut(on_block, walk_inputs);
                }
            }
            ScratchBlock::VarSet(_, i)
            | ScratchBlock::VarChange(_, i)
            | ScratchBlock::Log(i)
            | ScratchBlock::MotionChangeX(i)
            | ScratchBlock::MotionChangeY(i)
            | ScratchBlock::MotionSetX(i)
            | ScratchBlock::MotionSetY(i)
            | ScratchBlock::OpRound(i)
            | ScratchBlock::OpMAbs(i)
            | ScratchBlock::OpMSqrt(i)
            | ScratchBlock::OpMSin(i)
            | ScratchBlock::OpMCos(i)
            | ScratchBlock::OpMTan(i)
            | ScratchBlock::OpStrLen(i)
            | ScratchBlock::OpMFloor(i)
            | ScratchBlock::OpBNot(i) => {
                if let Input::Block(b) = i
                    && walk_inputs
                {
                    b.walk_mut(on_block, walk_inputs);
                }
            }
            ScratchBlock::MotionGoToXY(a, b)
            | ScratchBlock::OpAdd(a, b)
            | ScratchBlock::OpSub(a, b)
            | ScratchBlock::OpMul(a, b)
            | ScratchBlock::OpDiv(a, b)
            | ScratchBlock::OpStrJoin(a, b)
            | ScratchBlock::OpMod(a, b)
            | ScratchBlock::OpBAnd(a, b)
            | ScratchBlock::OpBOr(a, b)
            | ScratchBlock::OpCmp(a, b, _)
            | ScratchBlock::OpRandom(a, b)
            | ScratchBlock::OpStrLetterOf(a, b)
            | ScratchBlock::OpStrContains(a, b) => {
                if walk_inputs {
                    if let Input::Block(b) = a {
                        b.walk_mut(on_block, walk_inputs);
                    }
                    if let Input::Block(b) = b {
                        b.walk_mut(on_block, walk_inputs);
                    }
                }
            }

            ScratchBlock::FunctionCallNoScreenRefresh(_, l)
            | ScratchBlock::FunctionCallScreenRefresh(_, l) => {
                if walk_inputs {
                    for i in l {
                        if let Input::Block(b) = i {
                            b.walk_mut(on_block, walk_inputs);
                        }
                    }
                }
            }

            ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionGetArg(_)
            | ScratchBlock::ScreenRefresh
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
            | ScratchBlock::LooksShown(_)
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::LocalsGet(_)
            | ScratchBlock::VarRead(_) => {}
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Number,
    Bool,
    String,
}

#[derive(Debug, Clone, Copy)]
pub struct FunctionArg {
    pub val: ScratchValue,
    pub skip_nan: bool,
}

pub struct Compiler<'compiler> {
    pub args_list: Vec<FunctionArg>,
    pub break_counter: usize,
    pub break_points: Vec<Block>,
    pub memory: &'compiler [ScratchObject],
    pub effects: Effects,
    pub call_conv: CallConv,
    pub static_strings: Vec<SmolStr>,

    /// See [`Compiler::vars_to_avoid_clobbering`] for more info.
    pub clobber_stack: HashSet<Ptr>,
    pub locals_stack: Vec<Vec<(ScratchValue, bool)>>,

    pub constants: ConstantMap,
    pub func_store: FunctionStore,
    pub vars: Box<dyn VarStore>,
    pub jit_args: JitArgs,

    /// A function that is called when a custom block is encountered.
    ///
    /// It returns the effects of that custom block. Useful for
    /// type related optimizations, just put in a `|_| Effects::unknown()` to ignore.
    pub custom_block_effects: &'compiler dyn Fn(CustomBlockId) -> Effects,

    /// Temporary buffer to store some data.
    /// Be careful, don't let multiple things write to this at the same time.
    pub temp_slot4: (Value, StackSlot),

    /// Storing how many loops inside we are right now
    /// while compiling the current code.
    /// This is a **compile time value**
    ///
    /// # Example
    ///
    /// ```txt
    /// # fn repeat(_: usize) {}
    /// repeat(10) {
    ///     repeat(15) {
    ///         // repeat_stack: 2
    ///         // since we are 2 loops inside
    ///         your code
    ///     }
    ///     // repeat_stack: 1
    ///     // since we are 1 loop inside
    /// }
    /// ```
    ///
    /// This is used with [`ScratchBlock::ControlRepeat`] and [`ScratchBlock::ControlRepeatUntil`]
    pub repeat_stack: usize,

    pub sprite_id: SpriteId,
    pub is_inherently_pausable: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(
        jump_from_existing: bool,
        builder: &mut FunctionBuilder<'_>,
        code: &[ScratchBlock],
        memory: &'a [ScratchObject],
        jit_args: JitArgs,
        args_list: Vec<FunctionArg>,
        sprite_id: SpriteId,
        is_inherently_pausable: bool,
        temp_slot4: (Value, StackSlot),
        func_map: FuncMap,
        call_conv: CallConv,
        custom_block_effects: &'a dyn Fn(CustomBlockId) -> Effects,
        types_from_callsite: &'a dyn Fn(Ptr) -> VariableWrite,
    ) -> Self {
        let mut constants = ConstantMap::new();

        let code_block = builder.create_block();
        if jump_from_existing {
            builder.ins().jump(code_block, &[]);
        }
        builder.switch_to_block(code_block);

        let effects = analyze(
            &code,
            &custom_block_effects,
            types_from_callsite,
            &mut |_, _| {},
        );

        let vars: Box<dyn VarStore> = if effects.is_unknown {
            Box::new(GenericVarStore::new(memory))
        } else {
            Box::new(SsaVarStore::new(
                builder,
                &effects,
                &mut constants,
                memory,
                types_from_callsite,
            ))
        };

        Self {
            jit_args,
            temp_slot4,
            vars,
            effects,
            constants,
            call_conv,
            break_points: vec![code_block],
            func_store: FunctionStore::new(func_map),
            break_counter: 0,
            repeat_stack: 0,
            static_strings: Vec::new(),
            clobber_stack: HashSet::new(),
            locals_stack: Vec::new(),
            memory,
            args_list,
            sprite_id,
            is_inherently_pausable,
            custom_block_effects,
        }
    }

    pub fn compile_block(
        &mut self,
        block: &ScratchBlock,
        builder: &mut FunctionBuilder<'_>,
    ) -> Option<ScratchValue> {
        match block {
            ScratchBlock::VarSet(ptr, obj) => {
                self.var_set(obj, builder, *ptr);
            }
            ScratchBlock::VarRead(ptr) => {
                return Some(self.var_read(builder, *ptr));
            }
            ScratchBlock::OpAdd(a, b) => {
                return Some(ScratchValue::Num(self.op_add(a, b, builder)));
            }
            ScratchBlock::OpSub(a, b) => {
                return Some(ScratchValue::Num(self.op_sub(a, b, builder)));
            }
            ScratchBlock::OpMul(a, b) => {
                return Some(ScratchValue::Num(self.op_mul(a, b, builder)));
            }
            ScratchBlock::OpDiv(a, b) => {
                return Some(ScratchValue::Num(self.op_div(a, b, builder)));
            }
            ScratchBlock::OpMod(a, b) => {
                return Some(ScratchValue::Num(self.op_modulo(a, b, builder)));
            }
            ScratchBlock::OpStrJoin(a, b) => {
                return Some(ScratchValue::Object(self.op_str_join(a, b, builder)));
            }
            ScratchBlock::Log(msg) => self.dbg_log(msg, builder),
            ScratchBlock::ControlRepeat(input, vec) => {
                self.control_repeat(builder, input, vec);
            }
            ScratchBlock::ControlForever(vec) => {
                self.control_forever(builder, vec);
            }
            ScratchBlock::VarChange(ptr, input) => {
                self.var_change(input, builder, *ptr);
            }
            ScratchBlock::ControlIf(input, blocks) => {
                self.control_if(input, builder, blocks);
            }
            ScratchBlock::ControlIfElse(condition, then_block, else_block) => {
                self.control_if_else(condition, builder, then_block, else_block);
            }
            ScratchBlock::ControlRepeatUntil(input, blocks) => {
                self.control_repeat_until(builder, input, blocks);
            }
            ScratchBlock::LocalsWith(locals, blocks) => {
                self.control_locals_with(builder, locals, blocks);
            }
            ScratchBlock::LocalsGet(n) => {
                return self
                    .locals_stack
                    .last()
                    .and_then(|locals| locals.get(*n))
                    .map(|n| n.0)
                    .map(|n| n.clone_in_code(self, builder));
            }
            ScratchBlock::OpCmp(a, b, ordering) => {
                return Some(ScratchValue::Bool(self.op_cmp(a, b, builder, *ordering)));
            }
            ScratchBlock::OpStrLen(input) => {
                return Some(self.op_str_len(input, builder));
            }
            ScratchBlock::OpRandom(a, b) => return Some(self.op_random(a, b, builder)),
            ScratchBlock::OpBAnd(a, b) => {
                return Some(ScratchValue::Bool(self.op_b_and(a, b, builder)));
            }
            ScratchBlock::OpBNot(a) => {
                return Some(ScratchValue::Bool(self.op_b_not(a, builder)));
            }
            ScratchBlock::OpBOr(a, b) => {
                return Some(ScratchValue::Bool(self.op_b_or(a, b, builder)));
            }
            ScratchBlock::OpMFloor(n) => return Some(self.op_m_floor(n, builder)),
            ScratchBlock::OpStrLetterOf(letter, string) => {
                return Some(ScratchValue::Object(
                    self.op_str_letter(letter, string, builder),
                ));
            }
            ScratchBlock::OpStrContains(string, pattern) => {
                return Some(ScratchValue::Bool(
                    self.op_str_contains(string, pattern, builder),
                ));
            }
            ScratchBlock::OpRound(num) => {
                return Some(ScratchValue::Num(self.op_round(num, builder)));
            }
            ScratchBlock::OpMAbs(num) => {
                return Some(ScratchValue::Num(self.op_m_abs(num, builder)));
            }
            ScratchBlock::OpMSqrt(num) => {
                return Some(ScratchValue::Num(self.op_m_sqrt(num, builder)));
            }
            ScratchBlock::OpMSin(num) => {
                return Some(ScratchValue::Num(self.op_m_sin(num, builder)));
            }
            ScratchBlock::OpMCos(num) => {
                return Some(ScratchValue::Num(self.op_m_cos(num, builder)));
            }
            ScratchBlock::OpMTan(num) => {
                return Some(ScratchValue::Num(self.op_m_tan(num, builder)));
            }
            ScratchBlock::ScreenRefresh => {
                self.screen_refresh(builder);
            }
            ScratchBlock::ControlStopThisScript => {
                self.control_stop_this_script(builder);
            }
            ScratchBlock::FunctionCallNoScreenRefresh(custom_block_id, args) => {
                self.call_custom_block(*custom_block_id, builder, args, false);
            }
            ScratchBlock::FunctionCallScreenRefresh(custom_block_id, args) => {
                self.call_custom_block(*custom_block_id, builder, args, true);
            }
            ScratchBlock::FunctionGetArg(idx) => {
                let val = self.args_list[*idx];
                return Some(val.val.clone_in_code(self, builder));
            }
            ScratchBlock::MotionGoToXY(x, y) => {
                let x = x.get_number(self, builder);
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_GO_TO,
                    &[I64, I64, F64, F64],
                    &[],
                    &[self.jit_args.graphics_ptr, id, x, y],
                );
            }
            ScratchBlock::MotionChangeX(x) => {
                let x = x.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_CHANGE_X,
                    &[I64, I64, F64],
                    &[],
                    &[self.jit_args.graphics_ptr, id, x],
                );
            }
            ScratchBlock::MotionChangeY(y) => {
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_CHANGE_Y,
                    &[I64, I64, F64],
                    &[],
                    &[self.jit_args.graphics_ptr, id, y],
                );
            }
            ScratchBlock::MotionSetX(x) => {
                let x = x.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_SET_X,
                    &[I64, I64, F64],
                    &[],
                    &[self.jit_args.graphics_ptr, id, x],
                );
            }
            ScratchBlock::MotionSetY(y) => {
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_SET_Y,
                    &[I64, I64, F64],
                    &[],
                    &[self.jit_args.graphics_ptr, id, y],
                );
            }
            ScratchBlock::MotionGetX => {
                let id = self.constants.get_int(self.sprite_id.0, builder);

                let inst = self.call_function(
                    builder,
                    motion::C_GET_X,
                    &[I64, I64],
                    &[F64],
                    &[self.jit_args.graphics_ptr, id],
                );

                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
            ScratchBlock::MotionGetY => {
                let id = self.constants.get_int(self.sprite_id.0, builder);

                let inst = self.call_function(
                    builder,
                    motion::C_GET_Y,
                    &[I64, I64],
                    &[F64],
                    &[self.jit_args.graphics_ptr, id],
                );

                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
            ScratchBlock::LooksShown(shown) => {
                let id = self.constants.get_int(self.sprite_id.0, builder);
                let shown = self.constants.get_int(i64::from(*shown), builder);

                self.call_function(
                    builder,
                    looks::C_SHOWN,
                    &[I64, I64, I64],
                    &[],
                    &[self.jit_args.graphics_ptr, id, shown],
                );
            }
            ScratchBlock::SensingDaysSince2000 => {
                let inst =
                    self.call_function(builder, callbacks::env::DAYS_SINCE_2000, &[], &[F64], &[]);
                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
        }
        None
    }

    pub fn screen_refresh(&mut self, builder: &mut FunctionBuilder<'_>) {
        if !self.is_inherently_pausable {
            return;
        }

        for _ in 0..2 {
            // TODO: Hacky workaround for too-fast timing
            self.break_counter += 1;
            self.save_vars(builder);
            let break_counter = self.constants.get_int(self.break_counter as i64, builder);

            builder.ins().return_(&[break_counter]);
            self.constants.clear();

            let b = builder.create_block();
            self.break_points.push(b);
            builder.switch_to_block(b);

            self.vars.reinit(
                builder,
                &mut self.constants,
                self.memory,
                &Effects::unknown(),
            );
        }
    }

    pub fn save_vars(&mut self, builder: &mut FunctionBuilder<'_>) {
        self.vars
            .save(builder, &mut self.constants, self.memory, &self.effects);
    }

    pub fn clone_str(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        i2: i64,
        i3: i64,
        i4: i64,
    ) -> (Value, Value, Value) {
        let v2 = self.constants.get_int(i2, builder);
        let v3 = self.constants.get_int(i3, builder);
        let v4 = self.constants.get_int(i4, builder);

        self.static_strings
            .push(unsafe { std::mem::transmute::<[i64; 3], SmolStr>([i2, i3, i4]) });

        self.call_function(
            builder,
            callbacks::types::CLONE_STR,
            &[I64, I64, I64, I64],
            &[],
            &[v2, v3, v4, self.temp_slot4.0],
        );

        let o2 = builder.ins().stack_load(I64, self.temp_slot4.1, 0);
        let o3 = builder.ins().stack_load(I64, self.temp_slot4.1, 8);
        let o4 = builder.ins().stack_load(I64, self.temp_slot4.1, 16);
        (o2, o3, o4)
    }

    pub fn get_local_type(&self, n: usize) -> VariableWrite {
        self.locals_stack
            .last()
            .and_then(|locals| locals.get(n))
            .map(|(v, skip_nan)| VariableWrite {
                ty: (*v).into(),
                skip_nan: *skip_nan,
                direct: true,
            })
            .unwrap_or_default()
    }

    pub fn create_string_stack_slot(builder: &mut FunctionBuilder<'_>) -> Value {
        let stack_slot =
            builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData::new(
                cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                3 * std::mem::size_of::<i64>() as u32,
                0,
            ));
        builder.ins().stack_addr(I64, stack_slot, 0)
    }

    pub fn call_to_num_with_decimal_check(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        i1: Value,
        i2: Value,
        i3: Value,
        i4: Value,
    ) -> (Value, Value) {
        self.call_function(
            builder,
            callbacks::types::TO_NUMBER_WITH_DECIMAL_CHECK,
            &[I64, I64, I64, I64, I64],
            &[],
            &[i1, i2, i3, i4, self.temp_slot4.0],
        );
        let n = builder.ins().stack_load(F64, self.temp_slot4.1, 0);
        let b = builder.ins().stack_load(I64, self.temp_slot4.1, 8);
        (n, b)
    }
}

pub type FuncMap = BiHashMap<UserExternalNameRef, UserExternalName>;

pub struct FunctionStore {
    pub func_map: FuncMap,
    pub signatures: HashMap<Signature, SigRef>,
    definitions: HashMap<UserExternalName, FuncRef>,
}

impl FunctionStore {
    pub fn new(func_map: FuncMap) -> Self {
        Self {
            func_map,
            signatures: HashMap::new(),
            definitions: HashMap::new(),
        }
    }

    #[track_caller]
    pub fn get_function(
        &mut self,
        call_conv: CallConv,
        builder: &mut FunctionBuilder,
        name: UserExternalName,
        params: &[Type],
        returns: &[Type],
    ) -> FuncRef {
        if let Some(func_ref) = self.definitions.get(&name) {
            return *func_ref;
        }

        let mut sig = Signature::new(call_conv);
        for param in params {
            sig.params.push(AbiParam::new(*param));
        }
        for ret in returns {
            sig.returns.push(AbiParam::new(*ret));
        }

        let signature = if let Some(sigref) = self.signatures.get(&sig) {
            *sigref
        } else {
            let r = builder.import_signature(sig.clone());
            self.signatures.insert(sig.clone(), r);
            r
        };

        let Some(func_ref) = self.func_map.get_by_right(&name) else {
            crate::print_function_addresses();
            panic!("Function not found: {name}");
        };
        let func = ExtFuncData {
            name: ExternalName::User(*func_ref),
            signature,
            colocated: false,
            patchable: false,
        };
        let func_ref = builder.import_function(func);
        self.definitions.insert(name, func_ref);

        func_ref
    }
}
