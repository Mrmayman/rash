use rash_vm::error::RashError;
use serde_json::Value;

use crate::{FieldType, Res, error::ErrExt};

pub fn get_idx_array<'a>(v: Option<&'a Vec<Value>>, idx: usize, path: &str) -> Res<&'a Value> {
    v.ok_or(RashError::field_not_found(path))?
        .get(idx)
        .ok_or_else(|| RashError::field_not_found(&format!("{path}[{idx}]")))
}

pub fn idx_array<'a, T: AsRef<str>>(
    v: &'a [Value],
    idx: usize,
    path: impl FnOnce() -> T,
) -> Res<&'a Value> {
    v.get(idx)
        .ok_or_else(|| RashError::field_not_found(path().as_ref()))
}

pub fn expect_str<'a>(v: &'a Value, path: &str) -> Res<&'a str> {
    v.as_str()
        .ok_or(RashError::field_not_typed(path, FieldType::String))
}

pub fn expect_array<'a, T: AsRef<str>>(
    v: &'a Value,
    path: impl FnOnce() -> T,
) -> Res<&'a Vec<Value>> {
    v.as_array()
        .ok_or_else(|| RashError::field_not_typed(path().as_ref(), FieldType::Array))
}

pub fn get_expect_str<'a, T: AsRef<str>>(
    v: Option<&'a Value>,
    path: impl Fn() -> T,
) -> Res<&'a str> {
    v.ok_or_else(|| RashError::field_not_found(path().as_ref()))?
        .as_str()
        .ok_or_else(|| RashError::field_not_typed(path().as_ref(), FieldType::String))
}

pub fn get_expect_array<'a, T: AsRef<str>>(
    v: Option<&'a Value>,
    path: impl Fn() -> T,
) -> Res<&'a Vec<Value>> {
    v.ok_or_else(|| RashError::field_not_found(path().as_ref()))?
        .as_array()
        .ok_or_else(|| RashError::field_not_typed(path().as_ref(), FieldType::Array))
}
