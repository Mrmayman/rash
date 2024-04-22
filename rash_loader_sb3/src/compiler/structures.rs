use rash_vm::bytecode::instructions::Instruction;

use super::{error::CompilerError, variable_manager::RegisterId};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
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

pub type CompileResult = Result<Option<RegisterId>, CompilerError>;
