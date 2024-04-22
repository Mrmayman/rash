use super::variable_allocator::{RegisterId, VariableIdentifier};

#[derive(Debug)]
pub enum RegisterAction {
    Freeing,
    Getting,
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
