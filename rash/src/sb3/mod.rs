use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

use json::{Block, JsonBlock, JsonStruct};
use tempfile::TempDir;

use crate::{
    compiler::{ScratchBlock, MEMORY},
    data_types::ScratchObject,
    error::{ErrorConvert, ErrorConvertPath, RashError, RashErrorKind, Trace},
    input_primitives::Ptr,
    scheduler::{ProjectBuilder, Scheduler, Script, SpriteBuilder},
};
use rash_render::{CostumeId, IntermediateCostume, IntermediateState, SpriteId};

mod blocks;
mod get_utils;
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

    pub fn build(self) -> Result<Scheduler, RashError> {
        const FN_N: &str = "ProjectLoader::build";

        let memory = MEMORY.lock().unwrap();

        let mut builder = ProjectBuilder::new();

        let mut costume_names = HashMap::new();
        let mut costume_numbers = HashMap::new();
        let mut costume_hashes = HashMap::new();
        let mut costume_ids = HashMap::new();

        let mut costume_id = CostumeId(0);
        let mut variable_map = HashMap::new();

        let mut state_map = HashMap::new();

        for (sprite_i, sprite_json) in self.json.targets.iter().enumerate() {
            let id = SpriteId(sprite_i as i64);
            let mut sprite = SpriteBuilder::new(id);

            self.load_costumes(
                sprite_json,
                &mut costume_names,
                &mut costume_numbers,
                id,
                &mut costume_hashes,
                &mut costume_id,
                &mut costume_ids,
            )
            .trace(FN_N)?;

            let costume = costume_numbers
                .get(&(id, sprite_json.currentCostume as usize))
                .unwrap();
            let costume = *costume_hashes.get(costume).unwrap();
            let state = IntermediateState {
                x: sprite_json.x.unwrap_or_default(),
                y: sprite_json.y.unwrap_or_default(),
                size: sprite_json.size.unwrap_or(100.0),
                costume,
            };

            state_map.insert(id, state);

            load_blocks(sprite_json, &mut variable_map, &mut sprite, &memory)?;

            builder.finish_sprite(sprite);
        }

        builder.set_costume(costume_names, costume_numbers, costume_hashes, costume_ids);
        builder.set_init_state(state_map);

        Ok(builder.finish())
    }

    fn load_costumes(
        &self,
        sprite_json: &json::Target,
        costume_names: &mut HashMap<(SpriteId, String), String>,
        costume_numbers: &mut HashMap<(SpriteId, usize), String>,
        id: SpriteId,
        costume_hashes: &mut HashMap<String, CostumeId>,
        costume_id: &mut CostumeId,
        costume_ids: &mut HashMap<CostumeId, IntermediateCostume>,
    ) -> Result<(), RashError> {
        const FN_N: &str = "ProjectLoader::load_costumes";

        for (i, costume) in sprite_json.costumes.iter().enumerate() {
            let path = self.dir.path();
            let path = path.join(&costume.md5ext);
            let bytes = std::fs::read(&path).to_p(&path, "std::fs::read (costume)", FN_N)?;

            let is_svg = costume.md5ext.ends_with(".svg");

            let intermediate = IntermediateCostume {
                bytes,
                name: costume.name.clone(),
                hash: costume.assetId.clone(),
                rotation_center_x: costume.rotationCenterX,
                rotation_center_y: costume.rotationCenterY,
                is_svg,
            };

            costume_names.insert((id, costume.name.clone()), costume.assetId.clone());
            costume_numbers.insert((id, i), costume.assetId.clone());

            if costume_hashes.contains_key(&costume.assetId) {
                continue;
            }
            costume_hashes.insert(costume.assetId.clone(), *costume_id);
            costume_ids.insert(*costume_id, intermediate);
            costume_id.0 += 1;
        }
        Ok(())
    }
}

fn load_blocks(
    sprite_json: &json::Target,
    variable_map: &mut HashMap<String, Ptr>,
    sprite: &mut SpriteBuilder,
    memory: &[ScratchObject],
) -> Result<(), RashError> {
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
                eprintln!("Array block encountered");
                break;
            };

            blocks.push(
                block
                    .compile(variable_map, &sprite_json.blocks)
                    .trace(&format!(
                        "ProjectLoader::build (sprite: {})",
                        sprite_json.name
                    ))?,
            );

            id = block.next.clone();
        }

        for block in &blocks {
            println!("{}", block.format(0))
        }

        match hat_block.opcode.as_str() {
            "event_whenflagclicked" => {
                sprite.add_script(&Script::new_green_flag(blocks), memory);
            }
            _ => {
                println!("Unknown hat block opcode: {}", hat_block.opcode);
            }
        }
    }

    Ok(())
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
                let variable_id = self.get_variable_field()?;

                let variable_ptr = variable_map_get(variable_map, variable_id);

                let value = self
                    .get_number_input(blocks, variable_map, "VALUE")
                    .trace("Block::compile.data_setvariableto.VALUE")?;

                Ok(ScratchBlock::VarSet(variable_ptr, value))
            }
            "operator_add" => self.c_op_add(blocks, variable_map),
            "operator_subtract" => self.c_op_subtract(blocks, variable_map),
            "operator_multiply" => self.c_op_multiply(blocks, variable_map),
            "operator_divide" => self.c_op_divide(blocks, variable_map),
            "operator_random" => self.c_op_random(blocks, variable_map),
            "operator_join" => self.c_op_join(blocks, variable_map),
            "operator_letter_of" => self.c_op_str_letter_of(blocks, variable_map),
            "operator_contains" => self.c_op_str_contains(blocks, variable_map),
            "operator_length" => self.c_op_str_length(blocks, variable_map),
            "operator_mod" => self.c_op_mod(blocks, variable_map),
            "operator_round" => self.c_op_round(blocks, variable_map),
            "operator_gt" => self.c_op_greater(blocks, variable_map),
            "operator_lt" => self.c_op_less(blocks, variable_map),
            "operator_and" => self.c_op_and(blocks, variable_map),
            "operator_not" => self.c_op_not(blocks, variable_map),
            "data_changevariableby" => {
                let variable = self.get_variable_field()?;
                let value = self
                    .get_number_input(blocks, variable_map, "VALUE")
                    .trace("Block::compile.data_changevariableby.VALUE")?;
                Ok(ScratchBlock::VarChange(
                    variable_map_get(variable_map, variable),
                    value,
                ))
            }
            "motion_gotoxy" => {
                let x = self
                    .get_number_input(blocks, variable_map, "X")
                    .trace("Block::compile.motion_gotoxy.X")?;
                let y = self
                    .get_number_input(blocks, variable_map, "Y")
                    .trace("Block::compile.motion_gotoxy.X")?;

                Ok(ScratchBlock::MotionGoToXY(x, y))
            }
            "control_if" => {
                let condition = self
                    .get_boolean_input(blocks, variable_map, "CONDITION")
                    .trace("Block::compile.control_if.CONDITION")?;

                let compiled_blocks = self
                    .compile_substack(variable_map, blocks, "SUBSTACK")
                    .trace("Block::compile.control_if")?;

                Ok(ScratchBlock::ControlIf(condition, compiled_blocks))
            }
            "control_if_else" => {
                let condition = self
                    .get_boolean_input(blocks, variable_map, "CONDITION")
                    .trace("Block::compile.control_if_else.CONDITION")?;

                let compiled_blocks = self
                    .compile_substack(variable_map, blocks, "SUBSTACK")
                    .trace("Block::compile.control_if_else")?;

                let compiled_blocks2 = self
                    .compile_substack(variable_map, blocks, "SUBSTACK2")
                    .trace("Block::compile.control_if_else")?;

                Ok(ScratchBlock::ControlIfElse(
                    condition,
                    compiled_blocks,
                    compiled_blocks2,
                ))
            }
            "control_repeat" => {
                let times = self
                    .get_number_input(blocks, variable_map, "TIMES")
                    .trace("Block::compile.control_if.CONDITION")?;

                let mut compiled_blocks = self
                    .compile_substack(variable_map, blocks, "SUBSTACK")
                    .trace("Block::compile.control_if")?;
                compiled_blocks.push(ScratchBlock::ScreenRefresh);

                Ok(ScratchBlock::ControlRepeat(times, compiled_blocks))
            }
            _ => {
                println!("Unknown opcode: {}\n{self:#?}\n", self.opcode);
                Ok(ScratchBlock::OpAdd(0.0.into(), 0.0.into()))
            }
        }
    }
}

fn variable_map_get(variable_map: &mut HashMap<String, Ptr>, variable: &str) -> Ptr {
    if let Some(ptr) = variable_map.get(variable) {
        *ptr
    } else {
        let ptr = Ptr(variable_map.len());
        variable_map.insert(variable.to_owned(), ptr);
        ptr
    }
}
