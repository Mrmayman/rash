//! The Just-In-Time Compiler for Rash.
//!
//! Normally, Scratch code is interpreted by
//! looping over all the instructions and doing the
//! corresponding action. Something like the pseudocode
//!
//! `for instruction in instructions { match instruction { ... } }`
//!
//! However, this is quite slow. A faster way is to directly
//! compile Scratch to machine code like C or C++.
//!
//! That's what this module does!
//!
//! # Enabling
//!
//! To enable JIT, enable the `"jit"` crate feature

use std::collections::HashMap;

use cranelift::{codegen::CodegenError, prelude::*};

use crate::{
    bytecode::{Instruction, JumpPoint},
    data_types::ScratchObject,
    vm_thread::Thread,
};

use self::opblocks::{OpBlock, OpBuffer};

mod opblocks;

impl Thread {
    /// Just-In-Time compiles the Scratch code.
    pub fn jit(&mut self, memory: *const ScratchObject) -> Result<(), JitError> {
        let blocks = self.split_into_blocks()?;

        Ok(())
    }

    fn split_into_blocks(&self) -> Result<Vec<OpBlock>, JitError> {
        let mut buffer = OpBuffer::new();
        let mut block_lookup: HashMap<JumpPoint, Option<usize>> = HashMap::new();

        for instruction in self.code.iter() {
            match instruction {
                Instruction::JumpDefinePoint { place } => {
                    buffer.flush();
                    let current_block_id = buffer.len();
                    if block_lookup.contains_key(place) {
                        *block_lookup.get_mut(place).unwrap() = Some(current_block_id);
                    } else {
                        block_lookup.insert(place.clone(), Some(current_block_id));
                    }
                    buffer.push(Instruction::JumpDefinePoint {
                        place: JumpPoint(current_block_id),
                    });
                }
                Instruction::JumpToPointIfTrue { place, condition } => {
                    if !block_lookup.contains_key(place) {
                        block_lookup.insert(place.clone(), None);
                    }
                    buffer.push(instruction.clone());
                    buffer.flush();
                }
                Instruction::JumpToRawLocationIfTrue { .. } => {
                    return Err(JitError::CannotJitRawLocation)
                }
                Instruction::ThreadPause => {
                    buffer.flush();
                    buffer.push_flow_stop(false);
                }
                Instruction::ThreadKill => {
                    buffer.flush();
                    buffer.push_flow_stop(true);
                }
                _ => buffer.push(instruction.clone()),
            }
        }

        buffer.fix_jumps(&block_lookup);

        println!("{buffer:?}");
        println!("{block_lookup:?}");
        Ok(buffer.finish())
    }
}

#[derive(Debug)]
pub enum JitError {
    ArchitectureNotSupported,
    CannotJitRawLocation,
    CodegenError(CodegenError),
}

impl From<CodegenError> for JitError {
    fn from(value: CodegenError) -> Self {
        JitError::CodegenError(value)
    }
}
