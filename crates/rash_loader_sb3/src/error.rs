use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use rash_vm::error::{ErrorConvert, RashError};
use zip_extract::ZipExtractError;

pub type Error = RashError<Sb3ErrorKind>;

pub(crate) trait ErrExt {
    fn field_not_found(field: &str) -> Self;
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

pub(crate) trait ErrorConvertPath<T, E> {
    fn to_p(self, path: &Path, a: &str, b: &str) -> Result<T, RashError<E>>;
}

impl<T> ErrorConvertPath<T, Sb3ErrorKind> for Result<T, std::io::Error> {
    fn to_p(self, path: &Path, a: &str, b: &str) -> Result<T, Error> {
        self.map_err(|n| RashError {
            trace: vec![a.to_owned(), b.to_owned()],
            kind: Sb3ErrorKind::IoError(n, Some(path.to_owned())),
        })
    }
}

#[derive(Debug)]
pub enum Sb3ErrorKind {
    ZipExtract(ZipExtractError),
    Serde(serde_json::Error),
    FieldNotFound(String),
    InvalidWarpKind(String),
    IoError(std::io::Error, Option<PathBuf>),
    CurrentCustomBlockNotFound,
}

impl Display for Sb3ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sb3ErrorKind::ZipExtract(zip_extract_error) => {
                write!(f, "zip extract error: {zip_extract_error}")?;
            }
            Sb3ErrorKind::Serde(error) => {
                write!(f, "json error: {error}")?;
            }
            Sb3ErrorKind::FieldNotFound(field) => {
                write!(f, "field not found: {field}")?;
            }
            Sb3ErrorKind::InvalidWarpKind(val) => {
                write!(f, "invalid value for self.mutation.warp: {val}")?;
            }
            Sb3ErrorKind::IoError(error, path_buf) => {
                if let Some(path) = path_buf {
                    write!(f, "io error: at {path:?}: {error}")?;
                } else {
                    write!(f, "io error: {error}")?;
                }
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
fn io_err_cvt(n: std::io::Error) -> Sb3ErrorKind {
    Sb3ErrorKind::IoError(n, None)
}
err_convert!(IoErr, io_err_cvt);
err_convert!(ZipExtractError, Sb3ErrorKind::ZipExtract);
type SerdeErr = serde_json::Error;
err_convert!(SerdeErr, Sb3ErrorKind::Serde);
