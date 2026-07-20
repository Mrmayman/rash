use rash_vm::{
    Input, ScratchBlock,
    data_types::{number_to_string, string_to_number},
    error::{RashError, Trace},
};

use crate::{Res, error::ErrExt, load_block};

use super::{
    CompileContext,
    json::{Block, JsonBlock, json_id},
};

pub fn substack(
    b: &Block,
    ctx: &mut CompileContext,
    substack_name: &str,
) -> Res<Vec<ScratchBlock>> {
    const F: &str = "get::substack";
    let Some(substack) = b.inputs.get(substack_name) else {
        return Ok(Vec::new());
    };
    let substack = substack
        .as_array()
        .ok_or(RashError::field_not_found(
            "b.inputs.{substack_name}: not array",
        ))
        .trace(F)?;
    let Some(child_block_id) = substack
        .get(1)
        .ok_or(RashError::field_not_found(&format!(
            "b.inputs.{substack_name}[1]"
        )))
        .trace(F)?
        .as_str()
    else {
        return Ok(Vec::new());
    };

    let mut compiled_blocks = Vec::new();
    let mut id = Some(child_block_id.to_owned());
    while let Some(block_id) = &id {
        let block = ctx.get_block(block_id).unwrap().clone();
        let JsonBlock::Block { block } = block else {
            eprintln!("Array block encountered");
            break;
        };
        compiled_blocks.push(load_block(&block, ctx).trace(F)?);
        id.clone_from(&block.next);
    }
    Ok(compiled_blocks)
}

pub fn variable_field(b: &Block) -> Res<&str> {
    const F: &str = "get::variable_field";
    let variable_field = b
        .fields
        .get("VARIABLE")
        .ok_or(RashError::field_not_found("b.fields.VARIABLE"))
        .trace(F)?;
    let variable_array = variable_field
        .as_array()
        .ok_or(RashError::field_not_typed("b.fields.VARIABLE"))
        .trace(F)?;
    let first_elem = variable_array
        .get(1)
        .ok_or(RashError::field_not_found("b.fields.VARIABLE[1]"))
        .trace(F)?;
    first_elem
        .as_str()
        .ok_or(RashError::field_not_typed("b.fields.VARIABLE[1]"))
        .trace(F)
}

pub fn boolean(b: &Block, ctx: &mut CompileContext, name: &str) -> Res<Input> {
    const F: &str = "get::boolean";
    let Some(input) = b.inputs.get(name) else {
        return Ok(false.into());
    };
    let input = match input
        .as_array()
        .unwrap()
        .get(1)
        .ok_or(RashError::field_not_found(&format!("b.inputs.{name}[1]")))
        .trace(F)?
    {
        serde_json::Value::Null => false.into(),
        serde_json::Value::String(n) => match ctx.get_block(n).unwrap().clone() {
            JsonBlock::Block { block } => load_block(&block, ctx).trace(F)?.into(),
            JsonBlock::Array(_) => todo!(),
        },
        serde_json::Value::Array(vec) => {
            let n = vec
                .first()
                .ok_or(RashError::field_not_found(&format!(
                    "b.inputs.{name}[1][0]"
                )))
                .trace(F)?
                .as_i64()
                .unwrap();
            match n {
                json_id::NUMBER
                | json_id::ANGLE
                | json_id::INTEGER
                | json_id::POSITIVE_NUMBER
                | json_id::POSITIVE_INTEGER => match vec.get(1) {
                    Some(serde_json::Value::Number(number)) => number.as_f64().unwrap().into(),
                    Some(serde_json::Value::String(string)) => string_to_number(string).into(),
                    None => {
                        return Err(RashError::field_not_found(&format!(
                            "b.inputs.{name}[1][1]"
                        )))
                        .trace(F);
                    }
                    _ => panic!(),
                },
                json_id::STRING => vec.get(1).unwrap().as_str().unwrap().into(),
                json_id::VARIABLE => {
                    let id = vec.get(2).unwrap().as_str().unwrap();
                    let ptr = ctx.get_var(id);
                    ScratchBlock::VarRead(ptr).into()
                }
                _ => {
                    panic!("Unknown array input: {vec:?}")
                }
            }
        }
        _ => {
            panic!("Unknown input: {:?}", b.inputs)
        }
    };

    Ok(input)
}

pub fn number(b: &Block, ctx: &mut CompileContext, name: &str) -> Res<Input> {
    const F: &str = "get::number";

    let input = match b
        .inputs
        .get(name)
        .ok_or(RashError::field_not_found(&format!("b.inputs.{name}")))
        .trace(F)?
        .as_array()
        .ok_or(RashError::field_not_typed(&format!("b.inputs.{name}")))
        .trace(F)?
        .get(1)
        .ok_or(RashError::field_not_found(&format!("b.inputs.{name}[1]")))
        .trace(F)?
    {
        serde_json::Value::Null => false.into(),
        serde_json::Value::String(n) => match ctx.get_block(n).unwrap().clone() {
            JsonBlock::Block { block } => load_block(&block, ctx).trace(F)?.into(),
            JsonBlock::Array(_) => todo!(),
        },
        serde_json::Value::Array(vec) => {
            let n = vec
                .first()
                .ok_or(RashError::field_not_found(&format!(
                    "b.inputs.{name}[1][0]"
                )))
                .trace(F)?
                .as_i64()
                .unwrap();
            match n {
                json_id::NUMBER
                | json_id::ANGLE
                | json_id::INTEGER
                | json_id::POSITIVE_NUMBER
                | json_id::POSITIVE_INTEGER => match vec.get(1) {
                    Some(serde_json::Value::Number(number)) => number.as_f64().unwrap().into(),
                    Some(serde_json::Value::String(string)) => string_to_number(string).into(),
                    None => {
                        return Err(RashError::field_not_found(&format!(
                            "b.inputs.{name}[1][1]"
                        )))
                        .trace(F);
                    }
                    _ => panic!(),
                },
                json_id::STRING => {
                    let s = vec.get(1).unwrap().as_str().unwrap();
                    let num = string_to_number(s);
                    // Load-time optimization
                    if number_to_string(num) == s {
                        num.into()
                    } else {
                        s.into()
                    }
                }
                json_id::VARIABLE => {
                    let id = vec.get(2).unwrap().as_str().unwrap();
                    let ptr = ctx.get_var(id);
                    ScratchBlock::VarRead(ptr).into()
                }
                _ => {
                    panic!("Unknown array input: {vec:?}")
                }
            }
        }
        _ => {
            panic!("Unknown input: {:?}", b.inputs)
        }
    };

    Ok(input)
}

pub fn string(b: &Block, ctx: &mut CompileContext, name: &str) -> Res<Input> {
    const F: &str = "get::string";
    let input = match b
        .inputs
        .get(name)
        .ok_or(RashError::field_not_found(&format!("b.inputs.{name}")))
        .trace(F)?
        .as_array()
        .unwrap()
        .get(1)
        .ok_or(RashError::field_not_found(&format!("b.inputs.{name}[1]")))
        .trace(F)?
    {
        serde_json::Value::String(n) => match ctx.get_block(n).unwrap().clone() {
            JsonBlock::Block { block } => load_block(&block, ctx).trace(F)?.into(),
            JsonBlock::Array(_) => todo!(),
        },
        serde_json::Value::Array(vec) => {
            let n = vec
                .first()
                .ok_or(RashError::field_not_found(&format!(
                    "b.inputs.{name}[1][0]"
                )))
                .trace(F)?
                .as_i64()
                .unwrap();
            match n {
                json_id::NUMBER
                | json_id::ANGLE
                | json_id::INTEGER
                | json_id::POSITIVE_NUMBER
                | json_id::POSITIVE_INTEGER => match vec.get(1) {
                    Some(serde_json::Value::Number(number)) => {
                        number_to_string(number.as_f64().unwrap()).into()
                    }
                    Some(serde_json::Value::String(string)) => string.clone().into(),
                    None => {
                        return Err(RashError::field_not_found(&format!(
                            "b.inputs.{name}[1][1]"
                        )))
                        .trace(F);
                    }
                    _ => panic!(),
                },
                json_id::STRING => vec.get(1).unwrap().as_str().unwrap().into(),
                json_id::VARIABLE => {
                    let id = vec.get(2).unwrap().as_str().unwrap();
                    let ptr = ctx.get_var(id);
                    ScratchBlock::VarRead(ptr).into()
                }
                _ => {
                    panic!("Unknown input: {vec:?}")
                }
            }
        }
        _ => {
            panic!("Unknown input: {:?}", b.inputs)
        }
    };

    Ok(input)
}

pub fn custom_block_prototype(b: &Block) -> Res<&str> {
    b.inputs
        .get("custom_block")
        .ok_or(RashError::field_not_found(
            "b(procedures_definition).inputs.custom_block",
        ))?
        .as_array()
        .ok_or(RashError::field_not_found(
            "b(procedures_definition).inputs.custom_block: not array",
        ))?
        .get(1)
        .ok_or(RashError::field_not_found(
            "b(procedures_definition).inputs.custom_block[1]",
        ))?
        .as_str()
        .ok_or(RashError::field_not_found(
            "b(procedures_definition).inputs.custom_block[1]: not string",
        ))
}
