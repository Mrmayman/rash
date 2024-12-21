use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

use json::{
    Block, JsonBlock, JsonStruct, JSON_ID_ANGLE, JSON_ID_INTEGER, JSON_ID_NUMBER,
    JSON_ID_POSITIVE_INTEGER, JSON_ID_POSITIVE_NUMBER, JSON_ID_STRING, JSON_ID_VARIABLE,
};
use tempfile::TempDir;

use crate::{
    compiler::ScratchBlock,
    data_types::string_to_number,
    error::{ErrorConvert, ErrorConvertPath, RashError, RashErrorKind, Trace},
    input_primitives::{Input, Ptr},
    scheduler::{ProjectBuilder, Scheduler, Script, SpriteBuilder, SpriteId},
};

pub mod json;

pub struct ProjectLoader {
    dir: TempDir,
    json: json::JsonStruct,
}

impl ProjectLoader {
    pub fn new(file_path: &Path) -> Result<Self, RashError> {
        const FN_N: &str = "ProjectLoader::new";
        if !file_path.is_file() {
            return Err(RashError {
                trace: vec![FN_N.to_owned()],
                kind: RashErrorKind::IoError(
                    std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
                    Some(file_path.to_owned()),
                ),
            });
        }

        let dir = TempDir::new().to("TempDir::new", FN_N)?;
        let dir_path = dir.path();

        let file_bytes = std::fs::read(file_path).to_p(file_path, "std::fs::read", FN_N)?;

        zip_extract::extract(std::io::Cursor::new(file_bytes), dir_path, true)
            .to("zip_extract::extract (sb3 file)", FN_N)?;

        let json_path = dir_path.join("project.json");
        let json = std::fs::read_to_string(&json_path).to_p(
            &json_path,
            "std::fs::read_to_string (project.json)",
            FN_N,
        )?;
        let json: JsonStruct =
            serde_json::from_str(&json).to("serde_json::from_str (project.json)", FN_N)?;

        Ok(Self { dir, json })
    }

    pub fn build(self) -> Scheduler {
        let mut builder = ProjectBuilder::new();

        for (sprite_i, sprite_json) in self.json.targets.iter().enumerate() {
            let mut sprite = SpriteBuilder::new(SpriteId(sprite_i));

            let mut variable_map = HashMap::new();

            for (_, hat_block) in sprite_json.get_hat_blocks() {
                let JsonBlock::Block { block: hat_block } = hat_block else {
                    println!("Array hat block encountered");
                    continue;
                };

                let mut blocks: Vec<ScratchBlock> = Vec::new();

                let mut id = hat_block.next.clone();

                while let Some(block_id) = id {
                    let block = sprite_json.blocks.get(&block_id).unwrap();
                    let JsonBlock::Block { block } = block else {
                        println!("Array block encountered");
                        break;
                    };

                    blocks.push(
                        block
                            .compile(&mut variable_map, &sprite_json.blocks)
                            .unwrap(),
                    );

                    id = block.next.clone();
                }

                println!("{blocks:#?}");

                match hat_block.opcode.as_str() {
                    "event_whenflagclicked" => {
                        sprite.add_script(&Script::new_green_flag(blocks));
                    }
                    _ => {
                        println!("Unknown hat block opcode: {}", hat_block.opcode);
                    }
                }
            }

            builder.finish_sprite(sprite);
        }
        builder.finish()
    }
}

impl Block {
    pub fn compile(
        &self,
        variable_map: &mut HashMap<String, Ptr>,
        blocks: &BTreeMap<String, JsonBlock>,
    ) -> Result<ScratchBlock, RashError> {
        match self.opcode.as_str() {
            "data_setvariableto" => {
                // self.fields.VARIABLE[1]
                let variable_id = self
                    .fields
                    .get("VARIABLE")
                    .ok_or(RashError::field_not_found("self.fields.VARIABLE"))?
                    .as_array()
                    .unwrap()
                    .get(1)
                    .ok_or(RashError::field_not_found("self.fields.VARIABLE[1]"))?
                    .as_str()
                    .unwrap();

                let variable_ptr = if let Some(n) = variable_map.get(variable_id) {
                    *n
                } else {
                    let ptr = Ptr(variable_map.len());
                    variable_map.insert(variable_id.to_owned(), ptr);
                    ptr
                };

                let value = self
                    .get_number_input(blocks, variable_map, "VALUE")
                    .trace("Block::compile.data_setvariableto.VALUE")?;

                Ok(ScratchBlock::VarSet(variable_ptr, value))
            }
            "operator_add" => {
                let num1 = self
                    .get_number_input(blocks, variable_map, "NUM1")
                    .trace("Block::compile.operator_add.NUM1")?;
                let num2 = self
                    .get_number_input(blocks, variable_map, "NUM2")
                    .trace("Block::compile.operator_add.NUM2")?;
                Ok(ScratchBlock::OpAdd(num1, num2))
            }
            _ => {
                println!("Unknown opcode: {}", self.opcode);
                Ok(ScratchBlock::VarSet(Ptr(0), 0.0.into()))
            }
        }
    }

    fn get_number_input(
        &self,
        blocks: &BTreeMap<String, JsonBlock>,
        variable_map: &mut HashMap<String, Ptr>,
        name: &str,
    ) -> Result<Input, RashError> {
        let input = match self
            .inputs
            .get(name)
            .ok_or(RashError::field_not_found(&format!("self.inputs.{name}")))?
            .as_array()
            .unwrap()
            .get(1)
            .ok_or(RashError::field_not_found(&format!(
                "self.inputs.{name}[1]"
            )))? {
            serde_json::Value::String(n) => match blocks.get(n).unwrap() {
                JsonBlock::Block { block } => block
                    .compile(variable_map, blocks)
                    .trace("Block::get_number_input")?
                    .into(),
                JsonBlock::Array(_) => todo!(),
            },
            serde_json::Value::Array(vec) => {
                let n = vec.get(0).unwrap().as_i64().unwrap();
                match n {
                    JSON_ID_NUMBER
                    | JSON_ID_ANGLE
                    | JSON_ID_INTEGER
                    | JSON_ID_POSITIVE_NUMBER
                    | JSON_ID_POSITIVE_INTEGER => match vec.get(1) {
                        Some(serde_json::Value::Number(number)) => number.as_f64().unwrap().into(),
                        Some(serde_json::Value::String(string)) => string_to_number(string).into(),
                        None => {
                            return Err(RashError::field_not_found(&format!(
                                "self.inputs.{name}[1][1]"
                            )))
                        }
                        _ => panic!(),
                    },
                    JSON_ID_STRING => vec.get(1).unwrap().as_str().unwrap().into(),
                    JSON_ID_VARIABLE => {
                        let id = vec.get(2).unwrap().as_str().unwrap();
                        let ptr = *variable_map.get(id).unwrap();
                        ScratchBlock::VarRead(ptr).into()
                    }
                    _ => {
                        panic!("Unknown input: {:?}", vec)
                    }
                }
            }
            _ => {
                panic!("Unknown input: {:?}", self.inputs)
            }
        };

        Ok(input)
    }
}
