use std::fmt::Display;

use colored::Colorize;

use super::variable_allocator::{RegisterId, VariableIdentifier};

#[derive(Debug)]
pub enum RegisterAction {
    Freeing,
    Getting,
}

impl Display for RegisterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterAction::Freeing => write!(f, "freeing"),
            RegisterAction::Getting => write!(f, "getting"),
        }
    }
}

#[derive(Debug)]
pub enum CompilerError {
    RegisterNotFound(RegisterId, RegisterAction),
    VariableNotFoundWhenGetting(VariableIdentifier),
    FieldInvalid(String),
    UnsupportedBlock(String),
    UnsupportedHatBlock(String),
    ChildBlockDoesNotReturn,
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::RegisterNotFound(id, action) => write!(
                f,
                "Register allocator: Register {id} not found when {action} it"
            ),
            CompilerError::VariableNotFoundWhenGetting(var) => {
                write!(f, "Variable {var:?} not found")
            }
            CompilerError::FieldInvalid(field) => write!(f, "JSON: {field}"),
            CompilerError::UnsupportedBlock(block) => {
                write!(f, "Unsupported block: {}", block.as_str().blue())
            }
            CompilerError::UnsupportedHatBlock(hat_block) => {
                write!(f, "Unsupported hat block: {}", hat_block.as_str().blue())
            }
            CompilerError::ChildBlockDoesNotReturn => write!(f, "Child block does not return"),
        }
    }
}
