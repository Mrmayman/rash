use std::cmp::Ordering;

use crate::{
    CompileContext, Sb3Error, get,
    helpers::{expect_str, get_idx_array},
    json::Block,
};
use rash_vm::{ScratchBlock, error::Trace};

pub fn mathop(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    const F: &str = "op::mathop";

    let num = get::number(b, ctx, "NUM").trace(F)?;

    let operator = get_idx_array(b.fields.operator.as_ref(), 0, "b.fields.OPERATOR").trace(F)?;
    let operator = expect_str(operator, "b.fields.OPERATOR").trace(F)?;

    match operator {
        "abs" => Ok(ScratchBlock::OpAbs(num)),
        "floor" => Ok(ScratchBlock::OpFloor(num)),
        "ceiling" => Ok(ScratchBlock::OpCeil(num)),
        "sqrt" => Ok(ScratchBlock::OpSqrt(num)),
        "sin" => Ok(ScratchBlock::OpSin(num)),
        "cos" => Ok(ScratchBlock::OpCos(num)),
        "tan" => Ok(ScratchBlock::OpTan(num)),
        "e ^" => Ok(ScratchBlock::OpExp(num)),
        "10 ^" => Ok(ScratchBlock::OpExp10(num)),
        "ln" => Ok(ScratchBlock::OpLn(num)),
        "log" => Ok(ScratchBlock::OpLog(num)),
        _ => {
            eprintln!("Unknown operator (mathop): {operator}\n");
            Ok(ScratchBlock::OpAdd(0.0.into(), 0.0.into()))
        }
    }
}

pub fn not(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let bool = get::boolean(b, ctx, "OPERAND").trace("op::not")?;
    Ok(ScratchBlock::OpNot(bool))
}

pub fn and(b: &Block, ctx: &mut CompileContext) -> ScratchBlock {
    let bool1 = get::number(b, ctx, "OPERAND1").unwrap_or(false.into());
    let bool2 = get::number(b, ctx, "OPERAND2").unwrap_or(false.into());
    ScratchBlock::OpAnd(bool1, bool2)
}

pub fn or(b: &Block, ctx: &mut CompileContext) -> ScratchBlock {
    let bool1 = get::number(b, ctx, "OPERAND1").unwrap_or(false.into());
    let bool2 = get::number(b, ctx, "OPERAND2").unwrap_or(false.into());
    ScratchBlock::OpOr(bool1, bool2)
}

pub fn cmp(b: &Block, ctx: &mut CompileContext, cmp: Ordering) -> Result<ScratchBlock, Sb3Error> {
    const FN_N: &str = "op::cmp";
    let num1 = get::number(b, ctx, "OPERAND1").trace(FN_N)?;
    let num2 = get::number(b, ctx, "OPERAND2").trace(FN_N)?;
    Ok(ScratchBlock::OpCmp(num1, num2, cmp))
}

pub fn round(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let num = get::number(b, ctx, "NUM").trace("op::round")?;
    Ok(ScratchBlock::OpRound(num))
}

pub fn modulo(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::modulo")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::modulo")?;
    Ok(ScratchBlock::OpMod(num1, num2))
}

pub fn str_length(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let string = get::string(b, ctx, "STRING").trace("op::str_length")?;
    Ok(ScratchBlock::OpStrLen(string))
}

pub fn str_contains(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let string1 = get::string(b, ctx, "STRING1").trace("op::str_contains")?;
    let string2 = get::string(b, ctx, "STRING2").trace("op::str_contains")?;
    Ok(ScratchBlock::OpStrContains(string1, string2))
}

pub fn str_letter_of(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let string = get::string(b, ctx, "STRING").trace("op::str_letter_of")?;
    let index = get::number(b, ctx, "LETTER").trace("op::str_letter_of")?;
    Ok(ScratchBlock::OpStrLetterOf(index, string))
}

pub fn str_join(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let string1 = get::string(b, ctx, "STRING1").trace("op::str_join")?;
    let string2 = get::string(b, ctx, "STRING2").trace("op::str_join")?;
    Ok(ScratchBlock::OpStrJoin(string1, string2))
}

pub fn random(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let from = get::string(b, ctx, "FROM").trace("op::random")?;
    let to = get::string(b, ctx, "TO").trace("op::random")?;
    Ok(ScratchBlock::OpRandom(from, to))
}

pub fn divide(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::divide")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::divide")?;
    Ok(ScratchBlock::OpDiv(num1, num2))
}

pub fn multiply(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::multiply")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::multiply")?;
    Ok(ScratchBlock::OpMul(num1, num2))
}

pub fn subtract(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::subtract")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::subtract")?;
    Ok(ScratchBlock::OpSub(num1, num2))
}

pub fn add(b: &Block, ctx: &mut CompileContext) -> Result<ScratchBlock, Sb3Error> {
    let num1 = get::number(b, ctx, "NUM1").trace("op::add")?;
    let num2 = get::number(b, ctx, "NUM2").trace("op::add")?;
    Ok(ScratchBlock::OpAdd(num1, num2))
}
