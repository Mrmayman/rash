use crate::{
    compiler::ScratchBlock,
    error::{RashError, Trace},
    sb3::{json::Block, CompileContext},
};

impl Block {
    pub fn c_op_not(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let bool = self
            .get_boolean_input(ctx, "OPERAND")
            .trace("Block::compile.operator_not.OPERAND")?;
        Ok(ScratchBlock::OpBNot(bool))
    }

    pub fn c_op_and(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let bool1 = self
            .get_number_input(ctx, "OPERAND1")
            .trace("Block::compile.operator_and.OPERAND1")?;
        let bool2 = self
            .get_number_input(ctx, "OPERAND2")
            .trace("Block::compile.operator_and.OPERAND2")?;
        Ok(ScratchBlock::OpBAnd(bool1, bool2))
    }

    pub fn c_op_greater(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        const FN_N: &str = "Block::compile.operator_gt";
        let num1 = self.get_number_input(ctx, "OPERAND1").trace(FN_N)?;
        let num2 = self.get_number_input(ctx, "OPERAND2").trace(FN_N)?;
        Ok(ScratchBlock::OpCmpGreater(num1, num2))
    }

    pub fn c_op_less(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        const FN_N: &str = "Block::compile.operator_lt";
        let num1 = self.get_number_input(ctx, "OPERAND1").trace(FN_N)?;
        let num2 = self.get_number_input(ctx, "OPERAND2").trace(FN_N)?;
        Ok(ScratchBlock::OpCmpLesser(num1, num2))
    }

    pub fn c_op_round(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let num = self
            .get_number_input(ctx, "NUM")
            .trace("Block::compile.operator_round.NUM")?;
        Ok(ScratchBlock::OpRound(num))
    }

    pub fn c_op_mod(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_mod.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_mod.NUM2")?;
        Ok(ScratchBlock::OpMod(num1, num2))
    }

    pub fn c_op_str_length(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let string = self
            .get_string_input(ctx, "STRING")
            .trace("Block::compile.operator_length.STRING")?;
        Ok(ScratchBlock::OpStrLen(string))
    }

    pub fn c_op_str_contains(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let string1 = self
            .get_string_input(ctx, "STRING1")
            .trace("Block::compile.operator_contains.STRING1")?;
        let string2 = self
            .get_string_input(ctx, "STRING2")
            .trace("Block::compile.operator_contains.STRING2")?;
        Ok(ScratchBlock::OpStrContains(string1, string2))
    }

    pub fn c_op_str_letter_of(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let string = self
            .get_string_input(ctx, "STRING")
            .trace("Block::compile.operator_letter_of.STRING")?;
        let index = self
            .get_number_input(ctx, "LETTER")
            .trace("Block::compile.operator_letter_of.LETTER")?;
        Ok(ScratchBlock::OpStrLetterOf(index, string))
    }

    pub fn c_op_join(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let string1 = self
            .get_string_input(ctx, "STRING1")
            .trace("Block::compile.operator_join.STRING1")?;
        let string2 = self
            .get_string_input(ctx, "STRING2")
            .trace("Block::compile.operator_join.STRING2")?;
        Ok(ScratchBlock::OpStrJoin(string1, string2))
    }

    pub fn c_op_random(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let from = self
            .get_string_input(ctx, "FROM")
            .trace("Block::compile.operator_random.FROM")?;
        let to = self
            .get_string_input(ctx, "TO")
            .trace("Block::compile.operator_random.TO")?;
        Ok(ScratchBlock::OpRandom(from, to))
    }

    pub fn c_op_divide(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_divide.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_divide.NUM2")?;
        Ok(ScratchBlock::OpDiv(num1, num2))
    }

    pub fn c_op_multiply(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_multiply.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_multiply.NUM2")?;
        Ok(ScratchBlock::OpMul(num1, num2))
    }

    pub fn c_op_subtract(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_subtract.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_subtract.NUM2")?;
        Ok(ScratchBlock::OpSub(num1, num2))
    }

    pub fn c_op_add(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(ctx, "NUM1")
            .trace("Block::compile.operator_add.NUM1")?;
        let num2 = self
            .get_number_input(ctx, "NUM2")
            .trace("Block::compile.operator_add.NUM2")?;
        Ok(ScratchBlock::OpAdd(num1, num2))
    }
}
