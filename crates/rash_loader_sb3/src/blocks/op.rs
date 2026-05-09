use std::cmp::Ordering;

use crate::{CompileContext, Res, error::ErrExt, json::Block};
use rash_vm::{
    ScratchBlock,
    error::{RashError, Trace},
};

impl Block {
    pub fn c_op_mathop(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        const F: &str = "Block::c_op_mathop";

        let num = self.get_number_input(ctx, "NUM").trace(F)?;

        let operator = self
            .fields
            .get("OPERATOR")
            .ok_or(RashError::field_not_found("self.fields.OPERATOR"))
            .trace(F)?
            .as_array()
            .ok_or(RashError::field_not_typed("self.fields.OPERATOR"))
            .trace(F)?
            .first()
            .ok_or(RashError::field_not_found("self.fields.OPERATOR[0]"))
            .trace(F)?
            .as_str()
            .ok_or(RashError::field_not_typed("self.fields.OPERATOR[0]"))
            .trace(F)?;

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

    pub fn c_op_not(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let bool = self
            .get_boolean_input(ctx, "OPERAND")
            .trace("Block::compile.operator_not.OPERAND")?;
        Ok(ScratchBlock::OpBNot(bool))
    }

    pub fn c_op_and(&self, ctx: &mut CompileContext) -> ScratchBlock {
        let bool1 = self
            .get_number_input(ctx, "OPERAND1")
            .unwrap_or(false.into());
        let bool2 = self
            .get_number_input(ctx, "OPERAND2")
            .unwrap_or(false.into());
        ScratchBlock::OpBAnd(bool1, bool2)
    }

    pub fn c_op_or(&self, ctx: &mut CompileContext) -> ScratchBlock {
        let bool1 = self
            .get_number_input(ctx, "OPERAND1")
            .unwrap_or(false.into());
        let bool2 = self
            .get_number_input(ctx, "OPERAND2")
            .unwrap_or(false.into());
        ScratchBlock::OpBOr(bool1, bool2)
    }

    pub fn c_op_cmp(&self, ctx: &mut CompileContext, cmp: Ordering) -> Res<ScratchBlock> {
        const FN_N: &str = "Block::compile.operator_(gt/lt/equals)";
        let num1 = self.get_number_input(ctx, "OPERAND1").trace(FN_N)?;
        let num2 = self.get_number_input(ctx, "OPERAND2").trace(FN_N)?;
        Ok(ScratchBlock::OpCmp(num1, num2, cmp))
    }

    pub fn c_op_round(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let num = self
            .get_number_input(ctx, "NUM")
            .trace("Block::compile.operator_round.NUM")?;
        Ok(ScratchBlock::OpRound(num))
    }

    pub fn c_op_mod(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_mod.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_mod.NUM2")?;
        Ok(ScratchBlock::OpMod(num1, num2))
    }

    pub fn c_op_str_length(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let string = self
            .get_string_input(ctx, "STRING")
            .trace("Block::compile.operator_length.STRING")?;
        Ok(ScratchBlock::OpStrLen(string))
    }

    pub fn c_op_str_contains(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let string1 = self
            .get_string_input(ctx, "STRING1")
            .trace("Block::compile.operator_contains.STRING1")?;
        let string2 = self
            .get_string_input(ctx, "STRING2")
            .trace("Block::compile.operator_contains.STRING2")?;
        Ok(ScratchBlock::OpStrContains(string1, string2))
    }

    pub fn c_op_str_letter_of(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let string = self
            .get_string_input(ctx, "STRING")
            .trace("Block::compile.operator_letter_of.STRING")?;
        let index = self
            .get_number_input(ctx, "LETTER")
            .trace("Block::compile.operator_letter_of.LETTER")?;
        Ok(ScratchBlock::OpStrLetterOf(index, string))
    }

    pub fn c_op_join(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let string1 = self
            .get_string_input(ctx, "STRING1")
            .trace("Block::compile.operator_join.STRING1")?;
        let string2 = self
            .get_string_input(ctx, "STRING2")
            .trace("Block::compile.operator_join.STRING2")?;
        Ok(ScratchBlock::OpStrJoin(string1, string2))
    }

    pub fn c_op_random(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let from = self
            .get_string_input(ctx, "FROM")
            .trace("Block::compile.operator_random.FROM")?;
        let to = self
            .get_string_input(ctx, "TO")
            .trace("Block::compile.operator_random.TO")?;
        Ok(ScratchBlock::OpRandom(from, to))
    }

    pub fn c_op_divide(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_divide.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_divide.NUM2")?;
        Ok(ScratchBlock::OpDiv(num1, num2))
    }

    pub fn c_op_multiply(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_multiply.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_multiply.NUM2")?;
        Ok(ScratchBlock::OpMul(num1, num2))
    }

    pub fn c_op_subtract(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_subtract.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_subtract.NUM2")?;
        Ok(ScratchBlock::OpSub(num1, num2))
    }

    pub fn c_op_add(&self, ctx: &mut CompileContext) -> Res<ScratchBlock> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_add.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_add.NUM2")?;
        Ok(ScratchBlock::OpAdd(num1, num2))
    }
}
