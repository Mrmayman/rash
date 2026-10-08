use std::fmt::Display;

use rash_vm::error::{ErrorConvert, RashError};
use zip::result::ZipError;

pub type Error = RashError<Sb3ErrorKind>;

pub(crate) trait ErrExt {
    fn field_not_found(field: &str) -> Self;
    fn field_not_typed(field: &str, ty: FieldType) -> Self;
    fn invalid_warp_kind(field: &str) -> Self;
    fn blockdef_not_found(trace: &str) -> Self;
}

impl ErrExt for Error {
    fn field_not_found(field: &str) -> Self {
        RashError {
            trace: vec![],
            kind: Sb3ErrorKind::FieldNotFound(field.to_owned()),
        }
    }

    fn field_not_typed(field: &str, ty: FieldType) -> Self {
        RashError {
            trace: vec![],
            kind: Sb3ErrorKind::FieldNotTyped(field.to_owned(), ty),
        }
    }

    fn invalid_warp_kind(field: &str) -> Self {
        RashError {
            trace: vec![],
            kind: Sb3ErrorKind::InvalidWarpKind(field.to_owned()),
        }
    }

    fn blockdef_not_found(trace: &str) -> Self {
        RashError {
            trace: vec![format!(
                "Block::compile.argument_reporter_string_number ({trace})"
            )],
            kind: Sb3ErrorKind::CurrentCustomBlockNotFound,
        }
    }
}

#[derive(Debug)]
pub enum Sb3ErrorKind {
    Zip(ZipError),
    Serde(serde_json::Error),
    FieldNotFound(String),
    FieldNotTyped(String, FieldType),
    InvalidWarpKind(String),
    Io(std::io::Error),
    CurrentCustomBlockNotFound,
}

#[derive(Debug)]
pub enum FieldType {
    String,
    Array,
}

impl Display for Sb3ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sb3ErrorKind::Zip(error) => {
                write!(f, "zip error: {error}")?;
            }
            Sb3ErrorKind::Serde(error) => {
                write!(f, "json error: {error}")?;
            }
            Sb3ErrorKind::FieldNotFound(field) => {
                write!(f, "field not found: {field}")?;
            }
            Sb3ErrorKind::Io(error) => {
                write!(f, "io error: {error}")?;
            }
            Sb3ErrorKind::FieldNotTyped(field, ty) => {
                write!(f, "field not correct datatype: {field}, expected {ty:?}")?;
            }
            Sb3ErrorKind::InvalidWarpKind(val) => {
                write!(f, "invalid value for self.mutation.warp: {val}")?;
            }
            Sb3ErrorKind::CurrentCustomBlockNotFound => {
                write!(f, "could not get info of current custom block!")?;
            }
        }
        Ok(())
    }
}

macro_rules! err_convert {
    ($ty:ident, $variant:path) => {
        impl<T> ErrorConvert<Sb3ErrorKind, T> for Result<T, $ty> {
            fn to(self, a: &str, b: &str) -> Result<T, Error> {
                self.map_err(|n| RashError {
                    trace: vec![a.to_owned(), b.to_owned()],
                    kind: $variant(n),
                })
            }
        }
    };
}

type IoErr = std::io::Error;
err_convert!(IoErr, Sb3ErrorKind::Io);
err_convert!(ZipError, Sb3ErrorKind::Zip);
type SerdeErr = serde_json::Error;
err_convert!(SerdeErr, Sb3ErrorKind::Serde);
