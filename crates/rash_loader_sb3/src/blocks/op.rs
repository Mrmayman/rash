use std::cmp::Ordering;

use crate::{
    CompileContext, Res, get,
    helpers::{expect_str, get_idx_array},
    json::Block,
};
use rash_vm::{ScratchBlock, error::Trace};

pub fn mathop(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    const F: &str = "op::mathop";

    let num = get::number(b, ctx, "NUM").trace(F)?;

    let operator = get_idx_array(b.fields.operator.as_ref(), 0, "b.fields.OPERATOR").trace(F)?;
    let operator = expect_str(operator, "b.fields.OPERATOR").trace(F)?;

    match operator {
        "abs" => Ok(ScratchBlock::OpMAbs(num)),
        "floor" => Ok(ScratchBlock::OpMFloor(num)),
        // "ceiling" => Ok(ScratchBlock::OpMCeiling(num)),
        "sqrt" => Ok(ScratchBlock::OpMSqrt(num)),
        "sin" => Ok(ScratchBlock::OpMSin(num)),
        "cos" => Ok(ScratchBlock::OpMCos(num)),
        "tan" => Ok(ScratchBlock::OpMTan(num)),
        _ => {
            println!("Unknown operator (mathop): {operator}\n");
            Ok(ScratchBlock::OpAdd(0.0.into(), 0.0.into()))
        }
    }
}

pub fn not(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let bool = get::boolean(b, ctx, "OPERAND").trace("op::not")?;
    Ok(ScratchBlock::OpBNot(bool))
}

pub fn and(b: &Block, ctx: &mut CompileContext) -> ScratchBlock {
    let bool1 = get::number(b, ctx, "OPERAND1").unwrap_or(false.into());
    let bool2 = get::number(b, ctx, "OPERAND2").unwrap_or(false.into());
    ScratchBlock::OpBAnd(bool1, bool2)
}

pub fn or(b: &Block, ctx: &mut CompileContext) -> ScratchBlock {
    let bool1 = get::number(b, ctx, "OPERAND1").unwrap_or(false.into());
    let bool2 = get::number(b, ctx, "OPERAND2").unwrap_or(false.into());
    ScratchBlock::OpBOr(bool1, bool2)
}

pub fn cmp(b: &Block, ctx: &mut CompileContext, cmp: Ordering) -> Res<ScratchBlock> {
    const FN_N: &str = "op::cmp";
    let num1 = get::number(b, ctx, "OPERAND1").trace(FN_N)?;
    let num2 = get::number(b, ctx, "OPERAND2").trace(FN_N)?;
    Ok(ScratchBlock::OpCmp(num1, num2, cmp))
}

pub fn round(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let num = get::number(b, ctx, "NUM").trace("op::round")?;
    Ok(ScratchBlock::OpRound(num))
}

pub fn modulo(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::modulo")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::modulo")?;
    Ok(ScratchBlock::OpMod(num1, num2))
}

pub fn str_length(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let string = get::string(b, ctx, "STRING").trace("op::str_length")?;
    Ok(ScratchBlock::OpStrLen(string))
}

pub fn str_contains(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let string1 = get::string(b, ctx, "STRING1").trace("op::str_contains")?;
    let string2 = get::string(b, ctx, "STRING2").trace("op::str_contains")?;
    Ok(ScratchBlock::OpStrContains(string1, string2))
}

pub fn str_letter_of(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let string = get::string(b, ctx, "STRING").trace("op::str_letter_of")?;
    let index = get::number(b, ctx, "LETTER").trace("op::str_letter_of")?;
    Ok(ScratchBlock::OpStrLetterOf(index, string))
}

pub fn str_join(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let string1 = get::string(b, ctx, "STRING1").trace("op::str_join")?;
    let string2 = get::string(b, ctx, "STRING2").trace("op::str_join")?;
    Ok(ScratchBlock::OpStrJoin(string1, string2))
}

pub fn random(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let from = get::string(b, ctx, "FROM").trace("op::random")?;
    let to = get::string(b, ctx, "TO").trace("op::random")?;
    Ok(ScratchBlock::OpRandom(from, to))
}

pub fn divide(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::divide")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::divide")?;
    Ok(ScratchBlock::OpDiv(num1, num2))
}

pub fn multiply(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::multiply")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::multiply")?;
    Ok(ScratchBlock::OpMul(num1, num2))
}

pub fn subtract(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::subtract")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::subtract")?;
    Ok(ScratchBlock::OpSub(num1, num2))
}

pub fn add(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::add")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::add")?;
    Ok(ScratchBlock::OpAdd(num1, num2))
}
