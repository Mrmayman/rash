use crate::{compiler::ScratchBlock, input_primitives::Input};

impl ScratchBlock {
    pub fn format(&self, indent: usize) -> String {
        let out = match self {
            ScratchBlock::VarSet(ptr, input) => format!("{ptr:?} = {}", input.format(0)),
            ScratchBlock::VarChange(ptr, input) => format!("{ptr:?} += {}", input.format(0)),
            ScratchBlock::VarRead(ptr) => format!("{ptr:?}"),
            ScratchBlock::OpAdd(input, input1) => op_inner("+", input, input1),
            ScratchBlock::OpSub(input, input1) => op_inner("-", input, input1),
            ScratchBlock::OpMul(input, input1) => op_inner("*", input, input1),
            ScratchBlock::OpDiv(input, input1) => op_inner("/", input, input1),
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
            ScratchBlock::OpCmpGreater(input, input1) => op_inner(">", input, input1),
            ScratchBlock::OpCmpLesser(input, input1) => op_inner("<", input, input1),
            ScratchBlock::OpRandom(input, input1) => func_call_inner("random", &[input, input1]),
            ScratchBlock::OpStrLetterOf(input, input1) => {
                func_call_inner("str.letter_of", &[input, input1])
            }
            ScratchBlock::OpStrContains(input, input1) => {
                func_call_inner("str.contains", &[input, input1])
            }
            ScratchBlock::ControlIf(input, vec) => {
                let mut out = format!("if {} {{\n", input.format(0));
                for block in vec {
                    out.push_str(&block.format(indent + 1));
                    out.push('\n');
                }
                out.push_str(&" ".repeat(indent * 4));
                out.push('}');

                out
            }
            ScratchBlock::ControlIfElse(input, vec, vec1) => {
                let mut out = format!("if {} {{\n", input.format(0));
                for block in vec {
                    out.push_str(&block.format(indent + 1));
                    out.push('\n');
                }
                out.push_str(&" ".repeat(indent * 4));
                out.push_str("} else {\n");
                for block in vec1 {
                    out.push_str(&block.format(indent + 1));
                    out.push('\n');
                }
                out.push_str(&" ".repeat(indent * 4));
                out.push('}');

                out
            }
            ScratchBlock::ControlRepeat(input, vec) => {
                let mut out = format!("repeat {} {{\n", input.format(0));
                for block in vec {
                    out.push_str(&block.format(indent + 1));
                    out.push('\n');
                }
                out.push_str(&" ".repeat(indent * 4));
                out.push('}');

                out
            }
            ScratchBlock::ControlRepeatScreenRefresh(input, vec) => {
                let mut out = format!("repeat (fast) {} {{\n", input.format(0));
                for block in vec {
                    out.push_str(&block.format(indent + 1));
                    out.push('\n');
                }
                out.push_str(&" ".repeat(indent * 4));
                out.push('}');

                out
            }
            ScratchBlock::ControlRepeatUntil(input, vec) => {
                let mut out = format!("repeat until {} {{\n", input.format(0));
                for block in vec {
                    out.push_str(&block.format(indent + 1));
                    out.push('\n');
                }
                out.push_str(&" ".repeat(indent * 4));
                out.push('}');

                out
            }
            ScratchBlock::ControlStopThisScript => "return".to_owned(),
            ScratchBlock::FunctionCallNoScreenRefresh(custom_block_id, vec) => {
                let mut out = format!("call ({})(", custom_block_id.0);
                let len = vec.len();
                for (i, arg) in vec.iter().enumerate() {
                    out.push_str(&arg.format(0));
                    if i < len - 1 {
                        out.push_str(", ");
                    }
                }
                out.push(')');

                out
            }
            ScratchBlock::FunctionGetArg(idx) => {
                format!("get_arg({idx})")
            }
            ScratchBlock::ScreenRefresh => "break".to_owned(),
            ScratchBlock::MotionGoToXY(input, input1) => {
                func_call_inner("motion.go_to_xy", &[input, input1])
            }
            ScratchBlock::MotionChangeX(input) => func_call_inner("motion.x += ", &[input]),
            ScratchBlock::MotionChangeY(input) => func_call_inner("motion.y += ", &[input]),
            ScratchBlock::MotionSetX(input) => func_call_inner("motion.x = ", &[input]),
            ScratchBlock::MotionSetY(input) => func_call_inner("motion.y = ", &[input]),
            ScratchBlock::MotionGetX => "motion.x".to_owned(),
            ScratchBlock::MotionGetY => "motion.y".to_owned(),
        };

        format!("{}{out}", " ".repeat(indent * 4))
    }
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
    format!("{} {name} {}", a.format(0), b.format(0))
}
