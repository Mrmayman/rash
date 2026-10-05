use std::cmp::Ordering;

use crate::{compiler::ScratchBlock, input_primitives::Input};

fn pad(indent: usize) -> String {
    " ".repeat(indent * 4)
}

fn body(blocks: &[ScratchBlock], indent: usize) -> String {
    let mut out = String::new();
    for block in blocks {
        out.push_str(&block.format(indent + 1));
        out.push('\n');
    }
    out
}

impl ScratchBlock {
    #[must_use]
    pub fn format(&self, indent: usize) -> String {
        let out = match self {
            ScratchBlock::VarSet(ptr, input) => format!("{ptr:?} = {}", input.format(0)),
            ScratchBlock::VarChange(ptr, input) => format!("{ptr:?} += {}", input.format(0)),
            ScratchBlock::VarRead(ptr) => format!("{ptr:?}"),
            ScratchBlock::OpAdd(input, input1) => op_inner("+", input, input1),
            ScratchBlock::OpSub(input, input1) => op_inner("-", input, input1),
            ScratchBlock::OpMul(input, input1) => op_inner("*", input, input1),
            ScratchBlock::OpDiv(input, input1) => op_inner("/", input, input1),
            ScratchBlock::SensingDaysSince2000 => func_call_inner("days_since_2000", &[]),
            ScratchBlock::OpRound(input) => func_call_inner("round", &[input]),
            ScratchBlock::OpStrJoin(input, input1) => func_call_inner("str.join", &[input, input1]),
            ScratchBlock::OpMod(input, input1) => op_inner("mod", input, input1),
            ScratchBlock::OpStrLen(input) => func_call_inner("str.length", &[input]),
            ScratchBlock::OpBAnd(input, input1) => op_inner("and", input, input1),
            ScratchBlock::OpBNot(input) => func_call_inner("not!", &[input]),
            ScratchBlock::OpBOr(input, input1) => op_inner("or", input, input1),
            ScratchBlock::OpMFloor(input) => func_call_inner("floor", &[input]),
            ScratchBlock::OpMAbs(input) => func_call_inner("abs", &[input]),
            ScratchBlock::OpMSqrt(input) => func_call_inner("sqrt", &[input]),
            ScratchBlock::OpMSin(input) => func_call_inner("sin", &[input]),
            ScratchBlock::OpMCos(input) => func_call_inner("cos", &[input]),
            ScratchBlock::OpMTan(input) => func_call_inner("tan", &[input]),
            ScratchBlock::OpCmp(input, input1, Ordering::Greater) => op_inner(">", input, input1),
            ScratchBlock::OpCmp(input, input1, Ordering::Less) => op_inner("<", input, input1),
            ScratchBlock::OpCmp(input, input1, Ordering::Equal) => op_inner("==", input, input1),
            ScratchBlock::OpRandom(input, input1) => func_call_inner("random", &[input, input1]),
            ScratchBlock::OpStrLetterOf(input, input1) => {
                func_call_inner("str.letter_of", &[input, input1])
            }
            ScratchBlock::OpStrContains(input, input1) => {
                func_call_inner("str.contains", &[input, input1])
            }
            ScratchBlock::Log(input) => func_call_inner("log", &[input]),
            ScratchBlock::ControlIf(cond, blocks) => format!(
                "if {} {{\n{}{}}}",
                cond.format(0),
                body(blocks, indent),
                pad(indent),
            ),
            ScratchBlock::ControlIfElse(cond, then_, else_) => format!(
                "if {} {{\n{}{}}} else {{\n{}{}}}",
                cond.format(0),
                body(then_, indent),
                pad(indent),
                body(else_, indent),
                pad(indent),
            ),
            ScratchBlock::ControlRepeat(times, blocks) => format!(
                "repeat {} {{\n{}{}}}",
                times.format(0),
                body(blocks, indent),
                pad(indent),
            ),
            ScratchBlock::ControlForever(blocks) => {
                format!("forever {{\n{}{}}}", body(blocks, indent), pad(indent))
            }
            ScratchBlock::ControlRepeatUntil(cond, blocks) => format!(
                "repeat until {} {{\n{}{}}}",
                cond.format(0),
                body(blocks, indent),
                pad(indent),
            ),
            ScratchBlock::ControlStopThisScript => "return".to_owned(),
            ScratchBlock::FunctionCallNoScreenRefresh(id, args) => format_call(id.0, args, false),
            ScratchBlock::FunctionCallScreenRefresh(id, args) => format_call(id.0, args, true),
            ScratchBlock::FunctionGetArg(idx) => {
                format!("get_arg({idx})")
            }
            ScratchBlock::ScreenRefresh => "yield".to_owned(),
            ScratchBlock::MotionGoToXY(input, input1) => {
                func_call_inner("motion.go_to_xy", &[input, input1])
            }
            ScratchBlock::MotionChangeX(input) => format!("motion.x += {}", input.format(0)),
            ScratchBlock::MotionChangeY(input) => format!("motion.y += {}", input.format(0)),
            ScratchBlock::MotionSetX(input) => format!("motion.x = {}", input.format(0)),
            ScratchBlock::MotionSetY(input) => format!("motion.y = {}", input.format(0)),
            ScratchBlock::MotionGetX => "motion.x".to_owned(),
            ScratchBlock::MotionGetY => "motion.y".to_owned(),
            ScratchBlock::LooksShown(show) => if *show {
                "looks.show()"
            } else {
                "looks.hide()"
            }
            .to_owned(),
        };

        format!("{}{out}", " ".repeat(indent * 4))
    }
}

fn format_call(id: usize, args: &[Input], await_: bool) -> String {
    let mut out = format!("call ({id})(");
    let len = args.len();
    for (i, arg) in args.iter().enumerate() {
        out.push_str(&arg.format(0));
        if i < len - 1 {
            out.push_str(", ");
        }
    }
    out.push(')');
    if await_ {
        out.push_str(".await");
    }
    out
}

fn func_call_inner(name: &str, inputs: &[&Input]) -> String {
    let mut name = format!("{name}(");
    let len = inputs.len();
    for (i, input) in inputs.iter().enumerate() {
        name.push_str(&input.format(0));
        if i < len - 1 {
            name.push_str(", ");
        }
    }
    name.push(')');
    name
}

fn op_inner(name: &str, a: &Input, b: &Input) -> String {
    format!("({} {name} {})", a.format(0), b.format(0))
}
