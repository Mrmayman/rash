use std::collections::{BTreeMap, HashMap};

use crate::{
    compiler::ScratchBlock,
    error::{RashError, Trace},
    input_primitives::Ptr,
    sb3::json::{Block, JsonBlock},
};

type Blocks = BTreeMap<String, JsonBlock>;
type VarMap = HashMap<String, Ptr>;

impl Block {
    pub fn c_op_not(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let bool = self
            .get_boolean_input(blocks, variable_map, "OPERAND")
            .trace("Block::compile.operator_not.OPERAND")?;
        Ok(ScratchBlock::OpBNot(bool))
    }

    pub fn c_op_and(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let bool1 = self
            .get_number_input(blocks, variable_map, "OPERAND1")
            .trace("Block::compile.operator_and.OPERAND1")?;
        let bool2 = self
            .get_number_input(blocks, variable_map, "OPERAND2")
            .trace("Block::compile.operator_and.OPERAND2")?;
        Ok(ScratchBlock::OpBAnd(bool1, bool2))
    }

    pub fn c_op_greater(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        const FN_N: &str = "Block::compile.operator_gt";
        let num1 = self
            .get_number_input(blocks, variable_map, "OPERAND1")
            .trace(FN_N)?;
        let num2 = self
            .get_number_input(blocks, variable_map, "OPERAND2")
            .trace(FN_N)?;
        Ok(ScratchBlock::OpCmpGreater(num1, num2))
    }

    pub fn c_op_less(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        const FN_N: &str = "Block::compile.operator_lt";
        let num1 = self
            .get_number_input(blocks, variable_map, "OPERAND1")
            .trace(FN_N)?;
        let num2 = self
            .get_number_input(blocks, variable_map, "OPERAND2")
            .trace(FN_N)?;
        Ok(ScratchBlock::OpCmpLesser(num1, num2))
    }

    pub fn c_op_round(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let num = self
            .get_number_input(blocks, variable_map, "NUM")
            .trace("Block::compile.operator_round.NUM")?;
        Ok(ScratchBlock::OpRound(num))
    }

    pub fn c_op_mod(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(blocks, variable_map, "NUM1")
            .trace("Block::compile.operator_mod.NUM1")?;
        let num2 = self
            .get_number_input(blocks, variable_map, "NUM2")
            .trace("Block::compile.operator_mod.NUM2")?;
        Ok(ScratchBlock::OpMod(num1, num2))
    }

    pub fn c_op_str_length(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let string = self
            .get_string_input(blocks, variable_map, "STRING")
            .trace("Block::compile.operator_length.STRING")?;
        Ok(ScratchBlock::OpStrLen(string))
    }

    pub fn c_op_str_contains(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let string1 = self
            .get_string_input(blocks, variable_map, "STRING1")
            .trace("Block::compile.operator_contains.STRING1")?;
        let string2 = self
            .get_string_input(blocks, variable_map, "STRING2")
            .trace("Block::compile.operator_contains.STRING2")?;
        Ok(ScratchBlock::OpStrContains(string1, string2))
    }

    pub fn c_op_str_letter_of(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let string = self
            .get_string_input(blocks, variable_map, "STRING")
            .trace("Block::compile.operator_letter_of.STRING")?;
        let index = self
            .get_number_input(blocks, variable_map, "LETTER")
            .trace("Block::compile.operator_letter_of.LETTER")?;
        Ok(ScratchBlock::OpStrLetterOf(index, string))
    }

    pub fn c_op_join(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let string1 = self
            .get_string_input(blocks, variable_map, "STRING1")
            .trace("Block::compile.operator_join.STRING1")?;
        let string2 = self
            .get_string_input(blocks, variable_map, "STRING2")
            .trace("Block::compile.operator_join.STRING2")?;
        Ok(ScratchBlock::OpStrJoin(string1, string2))
    }

    pub fn c_op_random(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let from = self
            .get_string_input(blocks, variable_map, "FROM")
            .trace("Block::compile.operator_random.FROM")?;
        let to = self
            .get_string_input(blocks, variable_map, "TO")
            .trace("Block::compile.operator_random.TO")?;
        Ok(ScratchBlock::OpRandom(from, to))
    }

    pub fn c_op_divide(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(blocks, variable_map, "NUM1")
            .trace("Block::compile.operator_divide.NUM1")?;
        let num2 = self
            .get_number_input(blocks, variable_map, "NUM2")
            .trace("Block::compile.operator_divide.NUM2")?;
        Ok(ScratchBlock::OpDiv(num1, num2))
    }

    pub fn c_op_multiply(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(blocks, variable_map, "NUM1")
            .trace("Block::compile.operator_multiply.NUM1")?;
        let num2 = self
            .get_number_input(blocks, variable_map, "NUM2")
            .trace("Block::compile.operator_multiply.NUM2")?;
        Ok(ScratchBlock::OpMul(num1, num2))
    }

    pub fn c_op_subtract(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(blocks, variable_map, "NUM1")
            .trace("Block::compile.operator_subtract.NUM1")?;
        let num2 = self
            .get_number_input(blocks, variable_map, "NUM2")
            .trace("Block::compile.operator_subtract.NUM2")?;
        Ok(ScratchBlock::OpSub(num1, num2))
    }

    pub fn c_op_add(
        &self,
        blocks: &Blocks,
        variable_map: &mut VarMap,
    ) -> Result<ScratchBlock, RashError> {
        let num1 = self
            .get_number_input(blocks, variable_map, "NUM1")
            .trace("Block::compile.operator_add.NUM1")?;
        let num2 = self
            .get_number_input(blocks, variable_map, "NUM2")
            .trace("Block::compile.operator_add.NUM2")?;
        Ok(ScratchBlock::OpAdd(num1, num2))
    }
}
