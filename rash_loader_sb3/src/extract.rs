use std::{io::Read, path::Path};

use tempfile::TempDir;

use crate::{error::LoadError, json_struct::JsonStruct};

pub struct ProjectFile {
    pub temp_dir: TempDir,
    pub json: JsonStruct,
}

impl ProjectFile {
    pub fn open(path: &Path) -> Result<Self, LoadError> {
        let loaded_project_dir = ProjectFile::extract_zip_file(path)?;

        let json = std::fs::read_to_string(loaded_project_dir.path().join("project.json"))?;
        let json: JsonStruct = serde_json::from_str(&json)?;

        Ok(Self {
            temp_dir: loaded_project_dir,
            json,
        })
    }
}

impl ProjectFile {
    fn extract_zip_file(file_path: &Path) -> Result<TempDir, LoadError> {
        let mut file = std::fs::File::open(file_path)?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        let temporary_extract_directory = tempfile::TempDir::new()?;

        zip_extract::extract(
            std::io::Cursor::new(file_bytes),
            temporary_extract_directory.path(),
            false,
        )?;

        Ok(temporary_extract_directory)
    }
}
