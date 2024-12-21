use std::path::{Path, PathBuf};

use zip_extract::ZipExtractError;

#[derive(Debug)]
#[allow(dead_code)]
pub struct RashError {
    pub trace: Vec<String>,
    pub kind: RashErrorKind,
}

impl RashError {
    pub fn field_not_found(field: &str) -> Self {
        RashError {
            trace: vec![],
            kind: RashErrorKind::FieldNotFound(field.to_owned()),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RashErrorKind {
    IoError(std::io::Error, Option<PathBuf>),
    ZipExtract(ZipExtractError),
    Serde(serde_json::Error),
    FieldNotFound(String),
}

pub trait Trace {
    fn trace(self, t: &str) -> Self;
}

impl<T> Trace for Result<T, RashError> {
    fn trace(mut self, t: &str) -> Self {
        if let Err(err) = &mut self {
            err.trace.push(t.to_owned());
        }
        self
    }
}

pub trait ErrorConvert<T> {
    fn to(self, a: &str, b: &str) -> Result<T, RashError>;
}

#[macro_export]
macro_rules! err_convert {
    ($ty:ident, $variant:path) => {
        impl<T> ErrorConvert<T> for Result<T, $ty> {
            fn to(self, a: &str, b: &str) -> Result<T, RashError> {
                self.map_err(|n| RashError {
                    trace: vec![a.to_owned(), b.to_owned()],
                    kind: $variant(n),
                })
            }
        }
    };
}

type IoErr = std::io::Error;
fn io_err_cvt(n: std::io::Error) -> RashErrorKind {
    RashErrorKind::IoError(n, None)
}
err_convert!(IoErr, io_err_cvt);
err_convert!(ZipExtractError, RashErrorKind::ZipExtract);
type SerdeErr = serde_json::Error;
err_convert!(SerdeErr, RashErrorKind::Serde);

pub trait ErrorConvertPath<T> {
    fn to_p(self, path: &Path, a: &str, b: &str) -> Result<T, RashError>;
}

impl<T> ErrorConvertPath<T> for Result<T, std::io::Error> {
    fn to_p(self, path: &Path, a: &str, b: &str) -> Result<T, RashError> {
        self.map_err(|n| RashError {
            trace: vec![a.to_owned(), b.to_owned()],
            kind: RashErrorKind::IoError(n, Some(path.to_owned())),
        })
    }
}
