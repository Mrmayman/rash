use rash_vm::{ScratchBlock, error::Trace};

use crate::{CompileContext, Res, json::Block};

impl Block {
    pub fn c_cont_if(&self, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
        const F: &str = "Block::c_cont_if";
        let condition = self.get_boolean_input(ctx, "CONDITION").trace(F)?;
        let compiled_blocks = self.compile_substack(ctx, "SUBSTACK").trace(F)?;
        Ok(ScratchBlock::ControlIf(condition, compiled_blocks))
    }

    pub fn c_cont_if_else(&self, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
        const F: &str = "Block::c_cont_if_else";

        let condition = self.get_boolean_input(ctx, "CONDITION").trace(F)?;
        let blocks_then = self.compile_substack(ctx, "SUBSTACK").trace(F)?;
        let blocks_else = self.compile_substack(ctx, "SUBSTACK2").trace(F)?;

        Ok(ScratchBlock::ControlIfElse(
            condition,
            blocks_then,
            blocks_else,
        ))
    }

    pub fn c_cont_repeat(&self, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
        const F: &str = "Block::c_cont_repeat";
        let times = self.get_number_input(ctx, "TIMES").trace(F)?;
        let blocks = self.compile_substack(ctx, "SUBSTACK").trace(F)?;

        Ok(ScratchBlock::ControlRepeat(times, blocks))
    }

    pub fn c_cont_repeat_until(&self, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
        const F: &str = "Block::c_cont_repeat_until";
        let condition = self.get_boolean_input(ctx, "CONDITION").trace(F)?;
        let blocks = self.compile_substack(ctx, "SUBSTACK").trace(F)?;

        Ok(ScratchBlock::ControlRepeatUntil(condition, blocks))
    }

    pub fn c_cont_forever(&self, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
        let blocks = self
            .compile_substack(ctx, "SUBSTACK")
            .trace("Block::c_cont_forever")?;
        Ok(ScratchBlock::ControlForever(blocks))
    }
}
