use std::cmp::Ordering;

use crate::{
    Input, Ptr,
    effects::{VarTypeChecked, VariableWrite},
    input_primitives::ListId,
    runtime::CustomBlockId,
};

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
    ListClear(ListId),

    OpAdd(Input, Input),
    OpSub(Input, Input),
    OpMul(Input, Input),
    OpDiv(Input, Input),

    OpRound(Input),
    OpStrJoin(Input, Input),
    OpMod(Input, Input),
    OpCmp(Input, Input, Ordering),
    OpRandom(Input, Input),

    OpAnd(Input, Input),
    OpNot(Input),
    OpOr(Input, Input),

    OpFloor(Input),
    OpCeil(Input),
    OpAbs(Input),
    OpSqrt(Input),
    OpSin(Input),
    OpCos(Input),
    OpTan(Input),
    OpAsin(Input),
    OpAcos(Input),
    OpAtan(Input),
    OpLn(Input),
    OpLog(Input),
    OpExp(Input),
    OpExp10(Input),

    OpStrLen(Input),
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

    LooksSetShown(bool),
    SensingDaysSince2000,
    SensingMouseDown,

    LogPrint(Input),
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
            | ScratchBlock::OpSqrt(_)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpLn(_)
            | ScratchBlock::OpLog(_)
            | ScratchBlock::OpAsin(_)
            | ScratchBlock::OpAcos(_)
            | ScratchBlock::OpDiv(_, _) => Some(VariableWrite::normal(VarTypeChecked::Number)),
            ScratchBlock::FunctionGetArg(_) => Some(VariableWrite::normal(VarTypeChecked::Object)),

            ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpFloor(_)
            | ScratchBlock::OpCeil(_)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpAbs(_)
            | ScratchBlock::OpSin(_)
            | ScratchBlock::OpCos(_)
            | ScratchBlock::OpTan(_)
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::OpExp(_)
            | ScratchBlock::OpExp10(_)
            | ScratchBlock::OpAtan(_)
            | ScratchBlock::OpStrLen(_) => Some(VariableWrite::skip_nan(VarTypeChecked::Number)),
            ScratchBlock::OpStrLetterOf(_, _) | ScratchBlock::OpStrJoin(_, _) => {
                Some(VariableWrite::normal(VarTypeChecked::String))
            }
            ScratchBlock::OpAnd(_, _)
            | ScratchBlock::OpNot(_)
            | ScratchBlock::OpOr(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::SensingMouseDown
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
            | ScratchBlock::LooksSetShown(_)
            | ScratchBlock::LocalsWith(_, _)
            | ScratchBlock::ListClear(_)
            | ScratchBlock::LogPrint(_) => None,
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
            | ScratchBlock::OpAnd(_, _)
            | ScratchBlock::OpNot(_)
            | ScratchBlock::OpOr(_, _)
            | ScratchBlock::OpFloor(_)
            | ScratchBlock::OpCeil(_)
            | ScratchBlock::OpAbs(_)
            | ScratchBlock::OpSqrt(_)
            | ScratchBlock::OpSin(_)
            | ScratchBlock::OpCos(_)
            | ScratchBlock::OpTan(_)
            | ScratchBlock::OpLn(_)
            | ScratchBlock::OpLog(_)
            | ScratchBlock::OpExp(_)
            | ScratchBlock::OpExp10(_)
            | ScratchBlock::OpAsin(_)
            | ScratchBlock::OpAcos(_)
            | ScratchBlock::OpAtan(_)
            | ScratchBlock::OpCmp(_, _, _)
            | ScratchBlock::OpRandom(_, _)
            | ScratchBlock::OpStrLetterOf(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionGetArg(_)
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
            | ScratchBlock::LocalsGet(_)
            | ScratchBlock::SensingMouseDown
            | ScratchBlock::ListClear(_)
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
            ScratchBlock::LooksSetShown(b) => *b, // Triggers refresh on show, not hide

            // TODO: This actually depends on the function itself
            // Remove from here eventually
            ScratchBlock::FunctionCallScreenRefresh(_, _) => true,

            ScratchBlock::ScreenRefresh
            | ScratchBlock::LogPrint(_)
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
            | ScratchBlock::LogPrint(i)
            | ScratchBlock::MotionChangeX(i)
            | ScratchBlock::MotionChangeY(i)
            | ScratchBlock::MotionSetX(i)
            | ScratchBlock::MotionSetY(i)
            | ScratchBlock::OpRound(i)
            | ScratchBlock::OpAbs(i)
            | ScratchBlock::OpSqrt(i)
            | ScratchBlock::OpSin(i)
            | ScratchBlock::OpCos(i)
            | ScratchBlock::OpTan(i)
            | ScratchBlock::OpStrLen(i)
            | ScratchBlock::OpFloor(i)
            | ScratchBlock::OpCeil(i)
            | ScratchBlock::OpLn(i)
            | ScratchBlock::OpLog(i)
            | ScratchBlock::OpExp(i)
            | ScratchBlock::OpExp10(i)
            | ScratchBlock::OpAsin(i)
            | ScratchBlock::OpAcos(i)
            | ScratchBlock::OpAtan(i)
            | ScratchBlock::OpNot(i) => {
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
            | ScratchBlock::OpAnd(a, b)
            | ScratchBlock::OpOr(a, b)
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
            | ScratchBlock::LooksSetShown(_)
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::LocalsGet(_)
            | ScratchBlock::SensingMouseDown
            | ScratchBlock::ListClear(_)
            | ScratchBlock::VarRead(_) => {}
        }
    }

    pub fn walk_mut(&mut self, on_block: &mut impl FnMut(&mut ScratchBlock), walk_inputs: bool) {
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
            | ScratchBlock::LogPrint(i)
            | ScratchBlock::MotionChangeX(i)
            | ScratchBlock::MotionChangeY(i)
            | ScratchBlock::MotionSetX(i)
            | ScratchBlock::MotionSetY(i)
            | ScratchBlock::OpRound(i)
            | ScratchBlock::OpAbs(i)
            | ScratchBlock::OpSqrt(i)
            | ScratchBlock::OpSin(i)
            | ScratchBlock::OpCos(i)
            | ScratchBlock::OpTan(i)
            | ScratchBlock::OpStrLen(i)
            | ScratchBlock::OpFloor(i)
            | ScratchBlock::OpCeil(i)
            | ScratchBlock::OpLn(i)
            | ScratchBlock::OpLog(i)
            | ScratchBlock::OpExp(i)
            | ScratchBlock::OpExp10(i)
            | ScratchBlock::OpAsin(i)
            | ScratchBlock::OpAcos(i)
            | ScratchBlock::OpAtan(i)
            | ScratchBlock::OpNot(i) => {
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
            | ScratchBlock::OpAnd(a, b)
            | ScratchBlock::OpOr(a, b)
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
            | ScratchBlock::LooksSetShown(_)
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::LocalsGet(_)
            | ScratchBlock::SensingMouseDown
            | ScratchBlock::ListClear(_)
            | ScratchBlock::VarRead(_) => {}
        }
        on_block(self);
    }
}
