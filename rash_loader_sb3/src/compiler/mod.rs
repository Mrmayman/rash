mod blocks;
use std::collections::BTreeMap;

use rash_vm::{
    bytecode::instructions::{DataPointer, Instruction},
    data_types::ScratchObject,
};
use serde_json::Value;

use crate::{
    compiler::{error::CompilerError, variable_manager::VariableIdentifier},
    json_struct::JsonBlock,
};

use self::{
    structures::{CompileResult, ThreadId, ThreadState, ThreadType},
    variable_manager::VariableAllocator,
};

mod data_ids;
pub mod error;
pub mod structures;
pub mod variable_manager;

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
            JsonBlock::Block { block: data } => match data.opcode.as_str() {
                "data_setvariableto" => {
                    let register_id = self
                        .deal_with_input(data.inputs.get("VALUE").unwrap(), blocks)?
                        .ok_or(CompilerError::ChildBlockDoesNotReturn)?;

                    let register_vm_id =
                        self.allocator.register_get(register_id, self.thread_id)?;

                    const VARIABLE_HASH_INDEX: usize = 1;

                    let variable_id = data
                        .fields
                        .get("VARIABLE")
                        .ok_or(CompilerError::FieldInvalid(
                            "block.fields.VARIABLE does not exist".to_owned(),
                        ))?
                        .as_array()
                        .ok_or(CompilerError::FieldInvalid(
                            "block.fields.VARIABLE, expected Array".to_owned(),
                        ))?
                        .get(VARIABLE_HASH_INDEX)
                        .ok_or(CompilerError::FieldInvalid(
                            "blocks.fields.VARIABLE[1] does not exist".to_owned(),
                        ))?
                        .as_str()
                        .ok_or(CompilerError::FieldInvalid(
                            "blocks.fields.VARIABLE[1], expected String".to_owned(),
                        ))?
                        .to_owned();

                    self.thread_state
                        .instructions
                        .push(Instruction::MemSetToValue {
                            ptr: DataPointer(
                                self.allocator
                                    .variable_get(&VariableIdentifier::Hash(variable_id))?
                                    .0,
                            ),
                            value: ScratchObject::Pointer(register_vm_id.0),
                        });

                    self.allocator.register_free(register_id)?;

                    Ok(None)
                }
                _ => {
                    eprintln!("[unsupported] Block: {}", data.opcode);
                    Err(CompilerError::UnsupportedBlock(data.opcode.clone()))
                }
            },
            JsonBlock::Array(_) => todo!(),
        }
    }

    pub fn deal_with_input(
        &mut self,
        input: &Value,
        blocks: &BTreeMap<String, JsonBlock>,
    ) -> CompileResult {
        match input {
            Value::Array(parent_array) => match parent_array.get(1) {
                // If a block takes in a variable or value as input,
                // it is represented as a JSON array.

                // The first element of the array represents the type of the value as a number.
                // I have defined constants for the different types.
                Some(Value::Array(array)) => match array.first() {
                    Some(Value::Number(type_id)) => match type_id.as_f64().unwrap() as i64 {
                        data_ids::NUMBER
                        | data_ids::INTEGER
                        | data_ids::POSITIVE_INTEGER
                        | data_ids::POSITIVE_NUMBER
                        | data_ids::ANGLE => {
                            let value = array.get(1).unwrap().as_f64().unwrap();
                            self.malloc_and_set_to_value(ScratchObject::Number(value))
                        }
                        data_ids::STRING => {
                            let value = array.get(1).unwrap().as_str().unwrap().to_owned();
                            self.malloc_and_set_to_value(ScratchObject::String(value))
                        }
                        data_ids::COLOR => todo!("Implement data id: Color"),
                        data_ids::BROADCAST => todo!("Implement data id: Broadcast"),
                        data_ids::VARIABLE => todo!(),
                        data_ids::LIST => todo!("Implement data id: List"),
                        _ => Err(CompilerError::FieldInvalid(
                            "block.inputs.VALUE[1][0], expected number 4-13 (type id)".to_owned(),
                        )),
                    },
                    Some(_) => Err(CompilerError::FieldInvalid(
                        "block.inputs.VALUE[1][0], expected Number".to_owned(),
                    )),
                    None => todo!(),
                },
                // If a block takes in another block as input,
                // Example: say (2 + 2)
                // then it has a String containing the child block id.
                Some(Value::String(n)) => {
                    let block = blocks.get(n).unwrap();
                    println!("Compiling block");
                    self.compile_block(block, blocks)
                }
                Some(_) => Err(CompilerError::FieldInvalid(
                    "block.inputs.VALUE, expected Array, got something else.".to_owned(),
                )),
                None => todo!(),
            },
            _ => panic!("Block input is invalid data type (not string or array)"),
        }
    }

    fn malloc_and_set_to_value(&mut self, value: ScratchObject) -> CompileResult {
        let register = self.allocator.register_malloc(self.thread_id);
        self.thread_state
            .instructions
            .push(Instruction::MemSetToValue {
                ptr: DataPointer(self.allocator.register_get(register, self.thread_id)?.0),
                value,
            });

        Ok(Some(register))
    }
}
