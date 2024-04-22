use std::collections::BTreeMap;

use rash_vm::{
    bytecode::instructions::{DataPointer, Instruction},
    data_types::ScratchObject,
};
use serde_json::Value;

use crate::json_struct::Block;

use super::{
    error::CompilerError,
    variable_manager::{RegisterId, VariableManager},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ThreadId {
    pub sprite_id: usize,
    pub thread_id: usize,
}

pub enum ThreadType {
    WhenFlagClicked,
    Invalid,
}

pub struct ThreadState {
    pub thread_type: ThreadType,
    pub instructions: Vec<Instruction>,
}

pub enum CompileResult {
    Register(RegisterId),
    None,
    Error(CompilerError),
}

pub struct Compiler {
    pub thread_id: ThreadId,
    pub allocator: VariableManager,
    pub thread_state: ThreadState,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            thread_id: ThreadId {
                sprite_id: 0,
                thread_id: 0,
            },
            allocator: VariableManager::new(),
            thread_state: ThreadState {
                thread_type: ThreadType::Invalid,
                instructions: Vec::new(),
            },
        }
    }

    pub fn compile_block(
        &mut self,
        block: &Block,
        blocks: &BTreeMap<String, Block>,
    ) -> CompileResult {
        match block {
            Block::Block {
                opcode,
                next,
                parent,
                inputs,
                fields,
                shadow,
                topLevel,
                x,
                y,
            } => match opcode.as_str() {
                "data_setvariableto" => {
                    println!("inputs: {inputs:?}\n\nfields: {fields:?}\n");

                    CompileResult::None
                }
                _ => {
                    eprintln!("[unsupported] Block: {opcode}");
                    CompileResult::Error(CompilerError::UnsupportedBlock(opcode.clone()))
                }
            },
            Block::Array(_) => todo!(),
        }
    }

    pub fn deal_with_input(
        &mut self,
        input: &Value,
        blocks: &BTreeMap<String, Block>,
    ) -> CompileResult {
        match input {
            Value::String(n) => {
                let block = blocks.get(n).unwrap();
                self.compile_block(&block, blocks)
            }
            Value::Array(parent_array) => match parent_array.get(1) {
                Some(Value::Array(array)) => match array.get(0) {
                    Some(Value::Number(type_id)) => match type_id.as_f64().unwrap() as i64 {
                        4 => {
                            let value = array.get(1).unwrap().as_f64().unwrap();
                            self.malloc_and_set_to_value(ScratchObject::Number(value))
                        }
                        5 => todo!(),
                        6 => todo!(),
                        7 => todo!(),
                        8 => todo!(),
                        9 => todo!(),
                        10 => todo!(),
                        11 => todo!(),
                        12 => todo!(),
                        13 => todo!(),
                        _ => CompileResult::Error(CompilerError::FieldInvalid(
                            "block.inputs.VALUE[1][0], expected number 4-13 (type id)".to_owned(),
                        )),
                    },
                    Some(_) => CompileResult::Error(CompilerError::FieldInvalid(
                        "block.inputs.VALUE[1][0], expected Number".to_owned(),
                    )),
                    None => todo!(),
                },
                Some(_) => CompileResult::Error(CompilerError::FieldInvalid(
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
                ptr: match self.allocator.register_get(register, self.thread_id) {
                    Ok(n) => DataPointer(n.0),
                    Err(err) => return CompileResult::Error(err),
                },
                value,
            });

        CompileResult::Register(register)
    }
}
