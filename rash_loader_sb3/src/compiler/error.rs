use super::variable_manager::{RegisterId, VariableIdentifier};

pub enum RegisterAction {
    Freeing,
    Getting,
}

pub enum CompilerError {
    RegisterNotFound(RegisterId, RegisterAction),
    VariableNotFoundWhenGetting(VariableIdentifier),
    FieldInvalid(String),
    UnsupportedBlock(String),
}
