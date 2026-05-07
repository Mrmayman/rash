use crate::{
    compiler::ScratchBlock,
    data_types::{number_to_string, string_to_number},
    error::{RashError, Trace},
    input_primitives::Input,
};

use super::{
    json::{json_id, Block, JsonBlock},
    CompileContext,
};

impl Block {
    pub fn compile_substack(
        &self,
        ctx: &mut CompileContext,
        substack_name: &str,
    ) -> Result<Vec<ScratchBlock>, RashError> {
        let substack = self
            .inputs
            .get(substack_name)
            .ok_or(RashError::field_not_found(&format!(
                "self.inputs.{substack_name}"
            )))?
            .as_array()
            .ok_or(RashError::field_not_found(
                "self.inputs.{substack_name}: not array",
            ))?;
        let Some(child_block_id) = substack
            .get(1)
            .ok_or(RashError::field_not_found(&format!(
                "self.inputs.{substack_name}[1]"
            )))?
            .as_str()
        else {
            return Ok(Vec::new());
        };

        let mut compiled_blocks = Vec::new();
        let mut id = Some(child_block_id.to_owned());
        while let Some(block_id) = id {
            let block = ctx.get_block(&block_id).unwrap().clone();
            let JsonBlock::Block { block } = block else {
                eprintln!("Array block encountered");
                break;
            };
            compiled_blocks.push(block.compile(ctx).trace("Block::compile_substack()")?);
            id = block.next.clone();
        }
        Ok(compiled_blocks)
    }

    pub fn get_variable_field(&self) -> Result<&str, RashError> {
        Ok(self
            .fields
            .get("VARIABLE")
            .ok_or(RashError::field_not_found("self.fields.VARIABLE"))
            .trace("Block::get_variable_field")?
            .as_array()
            .unwrap()
            .get(1)
            .ok_or(RashError::field_not_found("self.fields.VARIABLE[1]"))
            .trace("Block::get_variable_field")?
            .as_str()
            .unwrap())
    }

    pub fn get_boolean_input(
        &self,
        ctx: &mut CompileContext,
        name: &str,
    ) -> Result<Input, RashError> {
        let Some(input) = self.inputs.get(name) else {
            return Ok(false.into());
        };
        let input = match input
            .as_array()
            .unwrap()
            .get(1)
            .ok_or(RashError::field_not_found(&format!(
                "self.inputs.{name}[1]"
            )))? {
            serde_json::Value::Null => false.into(),
            serde_json::Value::String(n) => match ctx.get_block(n).unwrap().clone() {
                JsonBlock::Block { block } => {
                    block.compile(ctx).trace("Block::get_boolean_input")?.into()
                }
                JsonBlock::Array(_) => todo!(),
            },
            serde_json::Value::Array(vec) => {
                let n = vec
                    .first()
                    .ok_or(RashError::field_not_found(&format!(
                        "self.inputs.{name}[1][0]"
                    )))?
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
                                "self.inputs.{name}[1][1]"
                            )))
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
                        panic!("Unknown array input: {:?}", vec)
                    }
                }
            }
            _ => {
                panic!("Unknown input: {:?}", self.inputs)
            }
        };

        Ok(input)
    }

    pub fn get_number_input(
        &self,
        ctx: &mut CompileContext,
        name: &str,
    ) -> Result<Input, RashError> {
        let input = match self
            .inputs
            .get(name)
            .ok_or(RashError::field_not_found(&format!("self.inputs.{name}")))?
            .as_array()
            .unwrap()
            .get(1)
            .ok_or(RashError::field_not_found(&format!(
                "self.inputs.{name}[1]"
            )))? {
            serde_json::Value::Null => false.into(),
            serde_json::Value::String(n) => match ctx.get_block(n).unwrap().clone() {
                JsonBlock::Block { block } => {
                    block.compile(ctx).trace("Block::get_number_input")?.into()
                }
                JsonBlock::Array(_) => todo!(),
            },
            serde_json::Value::Array(vec) => {
                let n = vec
                    .first()
                    .ok_or(RashError::field_not_found(&format!(
                        "self.inputs.{name}[1][0]"
                    )))?
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
                                "self.inputs.{name}[1][1]"
                            )))
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
                        panic!("Unknown array input: {:?}", vec)
                    }
                }
            }
            _ => {
                panic!("Unknown input: {:?}", self.inputs)
            }
        };

        Ok(input)
    }

    pub fn get_string_input(
        &self,
        ctx: &mut CompileContext,
        name: &str,
    ) -> Result<Input, RashError> {
        let input = match self
            .inputs
            .get(name)
            .ok_or(RashError::field_not_found(&format!("self.inputs.{name}")))?
            .as_array()
            .unwrap()
            .get(1)
            .ok_or(RashError::field_not_found(&format!(
                "self.inputs.{name}[1]"
            )))? {
            serde_json::Value::String(n) => match ctx.get_block(n).unwrap().clone() {
                JsonBlock::Block { block } => {
                    block.compile(ctx).trace("Block::get_string_input")?.into()
                }
                JsonBlock::Array(_) => todo!(),
            },
            serde_json::Value::Array(vec) => {
                let n = vec
                    .first()
                    .ok_or(RashError::field_not_found(&format!(
                        "self.inputs.{name}[1][0]"
                    )))?
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
                                "self.inputs.{name}[1][1]"
                            )))
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
                        panic!("Unknown input: {:?}", vec)
                    }
                }
            }
            _ => {
                panic!("Unknown input: {:?}", self.inputs)
            }
        };

        Ok(input)
    }

    pub fn get_custom_block_prototype(&self) -> Result<&str, RashError> {
        Ok(self
            .inputs
            .get("custom_block")
            .ok_or(RashError::field_not_found(
                "self(procedures_definition).inputs.custom_block",
            ))?
            .as_array()
            .ok_or(RashError::field_not_found(
                "self(procedures_definition).inputs.custom_block: not array",
            ))?
            .get(1)
            .ok_or(RashError::field_not_found(
                "self(procedures_definition).inputs.custom_block[1]",
            ))?
            .as_str()
            .ok_or(RashError::field_not_found(
                "self(procedures_definition).inputs.custom_block[1]: not string",
            ))?)
    }
}
