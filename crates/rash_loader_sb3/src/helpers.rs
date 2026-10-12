use rash_vm::error::RashError;
use serde_json::Value;

use crate::{FieldType, Sb3Error, error::ErrExt};

pub fn get_idx_array<'a>(
    v: Option<&'a Vec<Value>>,
    idx: usize,
    path: &str,
) -> Result<&'a Value, Sb3Error> {
    v.ok_or(RashError::field_not_found(path))?
        .get(idx)
        .ok_or_else(|| RashError::field_not_found(&format!("{path}[{idx}]")))
}

pub fn idx_array<T: AsRef<str>>(
    v: &[Value],
    idx: usize,
    path: impl FnOnce() -> T,
) -> Result<&Value, Sb3Error> {
    v.get(idx)
        .ok_or_else(|| RashError::field_not_found(path().as_ref()))
}

pub fn expect_str<'a>(v: &'a Value, path: &str) -> Result<&'a str, Sb3Error> {
    v.as_str()
        .ok_or(RashError::field_not_typed(path, FieldType::String))
}

pub fn expect_array<T: AsRef<str>>(
    v: &Value,
    path: impl FnOnce() -> T,
) -> Result<&Vec<Value>, Sb3Error> {
    v.as_array()
        .ok_or_else(|| RashError::field_not_typed(path().as_ref(), FieldType::Array))
}

pub fn get_expect_str<T: AsRef<str>>(
    v: Option<&Value>,
    path: impl Fn() -> T,
) -> Result<&str, Sb3Error> {
    v.ok_or_else(|| RashError::field_not_found(path().as_ref()))?
        .as_str()
        .ok_or_else(|| RashError::field_not_typed(path().as_ref(), FieldType::String))
}

pub fn get_expect_array<T: AsRef<str>>(
    v: Option<&Value>,
    path: impl Fn() -> T,
) -> Result<&Vec<Value>, Sb3Error> {
    v.ok_or_else(|| RashError::field_not_found(path().as_ref()))?
        .as_array()
        .ok_or_else(|| RashError::field_not_typed(path().as_ref(), FieldType::Array))
}
