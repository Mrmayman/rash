use rash_loader_sb3_json::Block;
use rash_vm::{ScratchBlock, error::RashError};

use crate::{CompileContext, Res, error::ErrExt};

pub mod control;
pub mod op;

pub fn argument_reporter(b: &Block, ctx: &mut CompileContext<'_>) -> Res<ScratchBlock> {
    let arg = b.fields.get("VALUE").ok_or(RashError::field_not_found(
        "b(argument_reporter_string_number).fields.VALUE",
    ))?;
    match arg {
        serde_json::Value::Array(values) => {
            let arg_name = values
                .first()
                .ok_or(RashError::field_not_found(
                    "b(argument_reporter_string_number).fields.VALUE[0]",
                ))?
                .as_str()
                .ok_or(RashError::field_not_found(
                    "b(argument_reporter_string_number).fields.VALUE[0]: not string",
                ))?;

            let current_custom_block = ctx
                .current_custom_block
                .as_ref()
                .ok_or(RashError::blockdef_not_found("current_custom_block"))?;
            // println!("{:?}", ctx.custom_block_defs);
            let blockdef = ctx
                .custom_block_defs
                .get(current_custom_block)
                .ok_or(RashError::blockdef_not_found("blockdef"))?;
            let name_to_id = blockdef
                .args_name_to_id
                .as_ref()
                .ok_or(RashError::blockdef_not_found("blockdef.name_to_id"))?;
            let arg_id = name_to_id
                .get(arg_name)
                .ok_or(RashError::blockdef_not_found("blockdef.name_to_id.get"))?;

            let position = blockdef
                .args
                .iter()
                .position(|n| n == arg_id)
                .ok_or(RashError::blockdef_not_found("blockdef.args"))?;

            Ok(ScratchBlock::FunctionGetArg(position))
        }
        _ => todo!(),
    }
}
