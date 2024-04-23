mod blocks;
use std::collections::BTreeMap;

use rash_vm::{
    bytecode::instructions::{DataPointer, Instruction},
    data_types::ScratchObject,
};
use serde_json::Value;

use crate::{
    compiler::{error::CompilerError, variable_allocator::VariableIdentifier},
    json_struct::{Block, JsonBlock},
};

use self::{
    structures::{CompileResult, ThreadId, ThreadState, ThreadType},
    variable_allocator::{RegisterId, VariableAllocator},
};

mod data_ids;
pub mod error;
pub mod structures;
pub mod variable_allocator;

pub struct Compiler {
    pub thread_id: ThreadId,
    pub allocator: VariableAllocator,
    pub thread_state: ThreadState,
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            thread_id: ThreadId {
                sprite_id: 0,
                thread_id: 0,
            },
            allocator: VariableAllocator::new(),
            thread_state: ThreadState {
                thread_type: ThreadType::Invalid,
                instructions: Vec::new(),
            },
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn compile_block(
        &mut self,
        block: &JsonBlock,
        blocks: &BTreeMap<String, JsonBlock>,
    ) -> CompileResult {
        match block {
            JsonBlock::Block { block } => match block.opcode.as_str() {
                "data_setvariableto" => self.c_variable_set(block, blocks),
                "operator_add" => self.c_operators_add(block, blocks),
                _ => Err(CompilerError::UnsupportedBlock(block.opcode.clone())),
            },
            JsonBlock::Array(_) => todo!(),
        }
    }

    pub fn deal_with_input(
        &mut self,
        block: &Block,
        input: &str,
        blocks: &BTreeMap<String, JsonBlock>,
    ) -> Result<RegisterId, CompilerError> {
        let input_value = block.inputs.get(input).ok_or(CompilerError::FieldInvalid(
            "block.inputs.VALUE does not exist".to_owned(),
        ))?;
        match input_value {
            Value::Array(parent_array) => {
                match parent_array.get(1) {
                    // If a block takes in a variable or value as input,
                    // it is represented as a JSON array.
                    Some(Value::Array(array)) => match array.first() {
                        // The first element of the array represents the type of the value as a number.
                        // I have defined constants for the different types.
                        Some(Value::Number(type_id)) => {
                            self.input_parse_type_id(type_id, input, array)
                        }

                        Some(_) => Err(CompilerError::FieldInvalid(format!(
                            "block.inputs.{input}[1][0], expected Number"
                        ))),
                        None => Err(CompilerError::FieldInvalid(format!(
                            "block.inputs.{input}[1][0] not found"
                        ))),
                    },
                    // If a block takes in another block as input,
                    // Example: say (2 + 2)
                    // then it has a String containing the child block id.
                    Some(Value::String(n)) => {
                        let block = blocks.get(n).unwrap();
                        match self.compile_block(block, blocks) {
                            Ok(Some(register)) => Ok(register),
                            Ok(None) => Err(CompilerError::ChildBlockDoesNotReturn),
                            Err(err) => Err(err),
                        }
                    }
                    Some(_) => Err(CompilerError::FieldInvalid(format!(
                        "block.inputs.{input}, expected Array"
                    ))),
                    None => todo!(),
                }
            }
            _ => Err(CompilerError::FieldInvalid(format!(
                "block.inputs.{input}, expected Array"
            ))),
        }
    }

    fn input_parse_type_id(
        &mut self,
        type_id: &serde_json::Number,
        input: &str,
        array: &[Value],
    ) -> Result<RegisterId, CompilerError> {
        match type_id.as_f64().unwrap() as i64 {
            data_ids::NUMBER
            | data_ids::INTEGER
            | data_ids::POSITIVE_INTEGER
            | data_ids::POSITIVE_NUMBER
            | data_ids::ANGLE => match array.get(1).unwrap() {
                Value::Number(n) => {
                    self.malloc_and_set_to_value(ScratchObject::Number(n.as_f64().unwrap()))
                }
                Value::String(s) => self.malloc_and_set_to_value(ScratchObject::String(s.clone())),
                _ => Err(CompilerError::FieldInvalid(format!(
                    "block.inputs.{input}[1][0], expected Number or String"
                ))),
            },
            data_ids::STRING => {
                let value = array.get(1).unwrap().as_str().unwrap().to_owned();
                self.malloc_and_set_to_value(ScratchObject::String(value))
            }
            data_ids::COLOR => todo!("Implement data id: Color"),
            data_ids::BROADCAST => todo!("Implement data id: Broadcast"),
            data_ids::VARIABLE => {
                let reg = self.allocator.register_malloc(self.thread_id);

                let var_hash = array
                    .get(2)
                    .ok_or(CompilerError::FieldInvalid(format!(
                        "block.inputs.{input}[1][2] not found"
                    )))?
                    .as_str()
                    .ok_or(CompilerError::FieldInvalid(format!(
                        "block.inputs.{input}[1][2], expected String"
                    )))?
                    .to_owned();

                let var = *self
                    .allocator
                    .variable_get(&VariableIdentifier::Hash(var_hash))?;

                self.thread_state
                    .instructions
                    .push(Instruction::MemSetToValue {
                        ptr: self.allocator.register_get(reg, self.thread_id)?.into(),
                        value: var.into(),
                    });

                Ok(reg)
            }
            data_ids::LIST => todo!("Implement data id: List"),
            _ => Err(CompilerError::FieldInvalid(format!(
                "block.inputs.{input}[1][0], expected number 4-13 (type id)"
            ))),
        }
    }

    fn malloc_and_set_to_value(
        &mut self,
        value: ScratchObject,
    ) -> Result<RegisterId, CompilerError> {
        let register = self.allocator.register_malloc(self.thread_id);
        self.thread_state
            .instructions
            .push(Instruction::MemSetToValue {
                ptr: DataPointer(self.allocator.register_get(register, self.thread_id)?.0),
                value,
            });

        Ok(register)
    }
}
