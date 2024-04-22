use std::{collections::BTreeMap, io::Read, path::Path};

use rash_vm::data_types::ScratchObject;
use serde_json::Value;
use tempfile::TempDir;

use crate::{
    compiler::{
        instance::{Compiler, ThreadType},
        variable_manager::VariableIdentifier,
    },
    error::LoadError,
    json_struct::{Block, JsonStruct},
};

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

    pub fn load(&mut self) {
        let mut compiler = Compiler::new();

        for (sprite_number, sprite) in self.json.targets.iter().enumerate() {
            compiler.thread_id.sprite_id = sprite_number;

            for (variable_hash, variable_data) in sprite.variables.iter() {
                let name = variable_data.get(0).unwrap();

                let data = match variable_data.get(1) {
                    Some(Value::Number(n)) => Some(ScratchObject::Number(n.as_f64().unwrap())),
                    Some(Value::Bool(b)) => Some(ScratchObject::Bool(*b)),
                    Some(Value::String(s)) => Some(ScratchObject::String(s.clone())),
                    _ => None,
                };

                if data.is_none() {
                    eprintln!(
                        "[warning] Variable {name} does not have default value: {variable_data:?}"
                    );
                }

                compiler
                    .allocator
                    .variable_add(VariableIdentifier::Hash(variable_hash.clone()), data);
            }

            let hat_blocks = sprite.get_hat_blocks();

            for (thread_id, (_, hat_block)) in hat_blocks.iter().enumerate() {
                compiler.thread_id.thread_id = thread_id;
                compiler.thread_state.instructions = Vec::new();

                ProjectFile::compile_hat_block(hat_block, &mut compiler, &sprite.blocks);
            }
        }
    }

    fn compile_hat_block(
        hat_block: &Block,
        compiler: &mut Compiler,
        blocks: &BTreeMap<String, Block>,
    ) {
        // Assuming we are compiling the following script:

        // WhenFlagClicked
        // GoTo(X)(Y)
        // Hide
        if let Block::Block { opcode, next, .. } = hat_block {
            // 1) When flag clicked...
            match opcode.as_str() {
                "event_whenflagclicked" => {
                    compiler.thread_state.thread_type = ThreadType::WhenFlagClicked
                }
                _ => return,
            }

            // 2) Some(id to GoTo(X)(Y) block).
            let mut next_block: Option<String> = next.clone();

            'process_blocks: loop {
                // 3) id to GoTo(X)(Y) block.
                let next_block_unwrapped: &mut String = if let Some(ref mut next) = next_block {
                    next
                } else {
                    return;
                };

                // 4) GoTo(X)(Y) block.
                let block: &Block = if let Some(block) = blocks.get(next_block_unwrapped) {
                    block
                } else {
                    return;
                };

                // 5) Compile the GoTo block.
                compiler.compile_block(block, &blocks);

                // 6) id to Hide block.
                if let Block::Block { next, .. } = block {
                    // 7) Set the next block's id to Hide's id.
                    //    Go back to step 3.
                    next_block = next.clone();
                } else {
                    // How can an a maths operator or an input block
                    // have a block attached to it? Just stop the compilation.
                    break 'process_blocks;
                }
            }
        }
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
