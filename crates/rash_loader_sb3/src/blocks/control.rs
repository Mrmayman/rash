use rash_vm::{ScratchBlock, error::Trace};

use crate::{CompileContext, Res, get, json::Block};

pub fn c_if(b: &Block, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
    const F: &str = "Block::c_cont_if";
    let condition = get::boolean(b, ctx, "CONDITION").trace(F)?;
    let compiled_blocks = get::substack(b, ctx, "get::substack").trace(F)?;
    Ok(ScratchBlock::ControlIf(condition, compiled_blocks))
}

pub fn c_if_else(b: &Block, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
    const F: &str = "Block::c_cont_if_else";

    let condition = get::boolean(b, ctx, "CONDITION").trace(F)?;
    let blocks_then = get::substack(b, ctx, "get::substack").trace(F)?;
    let blocks_else = get::substack(b, ctx, "get::substack2").trace(F)?;

    Ok(ScratchBlock::ControlIfElse(
        condition,
        blocks_then,
        blocks_else,
    ))
}

pub fn repeat(b: &Block, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
    const F: &str = "Block::c_cont_repeat";
    let times = get::number(b, ctx, "TIMES").trace(F)?;
    let blocks = get::substack(b, ctx, "get::substack").trace(F)?;

    Ok(ScratchBlock::ControlRepeat(times, blocks))
}

pub fn repeat_until(b: &Block, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
    const F: &str = "Block::c_cont_repeat_until";
    let condition = get::boolean(b, ctx, "CONDITION").trace(F)?;
    let blocks = get::substack(b, ctx, "get::substack").trace(F)?;

    Ok(ScratchBlock::ControlRepeatUntil(condition, blocks))
}

pub fn forever(b: &Block, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
    let blocks = get::substack(b, ctx, "get::substack").trace("Block::c_cont_forever")?;
    Ok(ScratchBlock::ControlForever(blocks))
}
