#[derive(Debug)]
pub enum LoadError {
    IoError(std::io::Error),
    ZipExtractError(zip_extract::ZipExtractError),
    SerdeJsonError(serde_json::Error),
}

impl From<std::io::Error> for LoadError {
    fn from(value: std::io::Error) -> Self {
        LoadError::IoError(value)
    }
}

impl From<zip_extract::ZipExtractError> for LoadError {
    fn from(value: zip_extract::ZipExtractError) -> Self {
        LoadError::ZipExtractError(value)
    }
}

impl From<serde_json::Error> for LoadError {
    fn from(value: serde_json::Error) -> Self {
        LoadError::SerdeJsonError(value)
    }
}
