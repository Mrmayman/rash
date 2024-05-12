use cranelift::{codegen::CodegenError, prelude::*};

use crate::{bytecode::instructions::Instruction, data_types::ScratchObject, vm_thread::Thread};

use self::opblocks::OpBuffer;

mod opblocks;

impl Thread {
    pub fn jit(&mut self, memory: *const ScratchObject) -> Result<(), JitError> {
        let mut buffer = OpBuffer::new();

        for instruction in self.code.iter() {
            match instruction {
                Instruction::JumpDefinePoint { .. } => {
                    buffer.flush();
                    buffer.push(instruction.clone());
                }
                Instruction::JumpToPointIfTrue { .. } => {
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

        println!("{buffer:?}");

        let blocks = buffer.finish();

        Ok(())
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
