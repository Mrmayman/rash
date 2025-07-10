use std::{collections::HashMap, path::Path};

use json::{Block, JsonBlock, JsonStruct};
use tempfile::TempDir;

use crate::{
    compiler::{ScratchBlock, MEMORY},
    data_types::ScratchObject,
    error::{ErrorConvert, ErrorConvertPath, RashError, RashErrorKind, Trace},
    input_primitives::{Input, Ptr},
    scheduler::{CustomBlockId, ProjectBuilder, Scheduler, Script, SpriteBuilder},
};
use rash_render::{CostumeId, IntermediateCostume, IntermediateState, SpriteId};

mod blocks;
mod get_utils;
pub mod json;

pub struct ProjectLoader {
    dir: TempDir,
    json: json::JsonStruct,
}

#[derive(Debug, Clone)]
pub struct CustomBlockDef {
    args: Vec<String>,
    args_name_to_id: Option<HashMap<String, String>>,
    is_screen_refresh: bool,
    id: CustomBlockId,
}

pub struct CompileContext<'a> {
    sprite_json: json::Target,
    variable_map: &'a mut HashMap<String, Ptr>,

    custom_block_defs: HashMap<String, CustomBlockDef>,
    custom_block_num: &'a mut usize,

    current_custom_block: Option<String>,
}

impl CompileContext<'_> {
    fn get_var(&mut self, variable: &str) -> Ptr {
        if let Some(ptr) = self.variable_map.get(variable) {
            *ptr
        } else {
            let ptr = Ptr(self.variable_map.len());
            self.variable_map.insert(variable.to_owned(), ptr);
            ptr
        }
    }

    fn get_block(&self, id: &str) -> Option<&JsonBlock> {
        self.sprite_json.blocks.get(id)
    }

    fn get_custom_block(&mut self, block: &Block) -> Result<CustomBlockDef, RashError> {
        const FN_N: &str = "CompileContext::get_custom_block";

        let block_mutation = block.mutation.as_ref().ok_or(RashError::field_not_found(
            "self(procedures_prototype).mutation",
        ))?;

        if let Some(def) = self.custom_block_defs.get_mut(&block_mutation.proccode) {
            if def.args_name_to_id.is_none() {
                let args: Vec<String> = serde_json::from_str(&block_mutation.argumentids)
                    .to("serde_json::from_str(self.mutation)", FN_N)?;
                if let Some(names) = &block_mutation.argumentnames {
                    def.args_name_to_id = Some(build_argument_names(&args, names)?);
                }
            }
            Ok(def.clone())
        } else {
            let args: Vec<String> = serde_json::from_str(&block_mutation.argumentids)
                .to("serde_json::from_str(self.mutation)", FN_N)?;
            let args_name_to_id = if let Some(names) = &block_mutation.argumentnames {
                Some(build_argument_names(&args, names)?)
            } else {
                None
            };

            let warp = if block_mutation.warp == "true" {
                true
            } else if block_mutation.warp == "false" {
                false
            } else {
                return Err(RashError::invalid_warp_kind(&block_mutation.warp));
            };
            let blockdef = CustomBlockDef {
                args,
                args_name_to_id,
                is_screen_refresh: !warp,
                id: CustomBlockId(*self.custom_block_num),
            };
            *self.custom_block_num += 1;
            self.custom_block_defs
                .insert(block_mutation.proccode.clone(), blockdef);
            Ok(self
                .custom_block_defs
                .get(&block_mutation.proccode)
                .unwrap()
                .clone())
        }
    }
}

fn build_argument_names(
    args: &[String],
    names: &String,
) -> Result<HashMap<String, String>, RashError> {
    const FN_N: &str = "build_argument_names";

    let arg_names: Vec<String> =
        serde_json::from_str(names).to("serde_json::from_str(self.mutation)", FN_N)?;
    let collect = args
        .iter()
        .zip(arg_names.into_iter())
        .map(|(a, b)| (b, a.clone()))
        .collect();
    Ok(collect)
}

impl ProjectLoader {
    pub fn new(file_path: &Path) -> Result<Self, RashError> {
        const FN_N: &str = "ProjectLoader::new";
        println!("[info] Loading file from {file_path:?}");
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

        let mut custom_block_num = 0;

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

            load_blocks(
                sprite_json,
                &mut variable_map,
                &mut custom_block_num,
                &mut sprite,
                &memory,
            )?;

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
    custom_block_num: &mut usize,
    sprite: &mut SpriteBuilder,
    memory: &[ScratchObject],
) -> Result<(), RashError> {
    const FN_N: &str = "sb3::load_blocks";

    let mut ctx = CompileContext {
        sprite_json: sprite_json.clone(),
        variable_map,
        custom_block_defs: HashMap::new(),
        custom_block_num,
        current_custom_block: None,
    };

    for (_, hat_block) in sprite_json.get_hat_blocks() {
        let JsonBlock::Block { block: hat_block } = hat_block else {
            println!("Array hat block encountered");
            continue;
        };

        let mut blocks: Vec<ScratchBlock> = Vec::new();

        let mut id = hat_block.next.clone();

        let custom_block = if hat_block.opcode == "procedures_definition" {
            let details = hat_block.get_custom_block_prototype()?;
            let details = sprite_json.blocks.get(details).unwrap();
            let JsonBlock::Block { block: details } = details else {
                eprintln!("[error] Array block encountered");
                break;
            };
            let custom_block = ctx.get_custom_block(details).trace(FN_N)?;

            let proccode = details
                .mutation
                .as_ref()
                .ok_or(RashError::field_not_found(
                    "self(procedures_prototype).mutation",
                ))?
                .proccode
                .clone();
            ctx.current_custom_block = Some(proccode);

            Some(custom_block)
        } else {
            ctx.current_custom_block = None;
            None
        };

        while let Some(block_id) = id {
            let block = sprite_json.blocks.get(&block_id).unwrap();
            let JsonBlock::Block { block } = block else {
                eprintln!("Array block encountered");
                break;
            };

            blocks.push(block.compile(&mut ctx).trace(&format!(
                "ProjectLoader::build (sprite: {})",
                sprite_json.name
            ))?);

            id = block.next.clone();
        }

        // for block in &blocks {
        //     println!("{}", block.format(0))
        // }

        match hat_block.opcode.as_str() {
            "event_whenflagclicked" => {
                let new_green_flag = Script::new_green_flag(blocks);
                sprite.add_script(&new_green_flag, memory);
            }
            "procedures_definition" => {
                let custom_block = custom_block.unwrap();

                println!("{custom_block:?}");
                sprite.add_script(
                    &Script::new_custom_block(
                        blocks,
                        custom_block.args.len(),
                        custom_block.id,
                        custom_block.is_screen_refresh,
                    ),
                    memory,
                );
            }
            _ => {
                println!("Unknown hat block opcode: {}", hat_block.opcode);
            }
        }
    }

    Ok(())
}

impl Block {
    pub fn compile(&self, ctx: &mut CompileContext) -> Result<ScratchBlock, RashError> {
        match self.opcode.as_str() {
            "data_setvariableto" => {
                // self.fields.VARIABLE[1]
                let variable_id = self.get_variable_field()?;

                let variable_ptr = ctx.get_var(variable_id);

                let value = self
                    .get_number_input(ctx, "VALUE")
                    .trace("Block::compile.data_setvariableto")?;

                Ok(ScratchBlock::VarSet(variable_ptr, value))
            }
            "operator_add" => self.c_op_add(ctx),
            "operator_subtract" => self.c_op_subtract(ctx),
            "operator_multiply" => self.c_op_multiply(ctx),
            "operator_divide" => self.c_op_divide(ctx),
            "operator_random" => self.c_op_random(ctx),
            "operator_join" => self.c_op_join(ctx),
            "operator_letter_of" => self.c_op_str_letter_of(ctx),
            "operator_contains" => self.c_op_str_contains(ctx),
            "operator_length" => self.c_op_str_length(ctx),
            "operator_mod" => self.c_op_mod(ctx),
            "operator_round" => self.c_op_round(ctx),
            "operator_gt" => self.c_op_greater(ctx),
            "operator_lt" => self.c_op_less(ctx),
            "operator_and" => self.c_op_and(ctx),
            "operator_not" => self.c_op_not(ctx),
            "data_changevariableby" => {
                let variable = self.get_variable_field()?;
                let value = self
                    .get_number_input(ctx, "VALUE")
                    .trace("Block::compile.data_changevariableby")?;
                Ok(ScratchBlock::VarChange(ctx.get_var(variable), value))
            }
            "motion_gotoxy" => {
                let x = self
                    .get_number_input(ctx, "X")
                    .trace("Block::compile.motion_gotoxy")?;
                let y = self
                    .get_number_input(ctx, "Y")
                    .trace("Block::compile.motion_gotoxy")?;

                Ok(ScratchBlock::MotionGoToXY(x, y))
            }
            "motion_setx" => {
                let n = self
                    .get_number_input(ctx, "X")
                    .trace("Block::compile.motion_setx")?;
                Ok(ScratchBlock::MotionSetX(n))
            }
            "motion_sety" => {
                let n = self
                    .get_number_input(ctx, "Y")
                    .trace("Block::compile.motion_sety")?;
                Ok(ScratchBlock::MotionSetY(n))
            }
            "motion_changexby" => {
                let val = self
                    .get_number_input(ctx, "DX")
                    .trace("Block::compile.motion_changexby")?;
                Ok(ScratchBlock::MotionChangeX(val))
            }
            "motion_changeyby" => {
                let val = self
                    .get_number_input(ctx, "DY")
                    .trace("Block::compile.motion_changeyby")?;
                Ok(ScratchBlock::MotionChangeY(val))
            }
            "control_if" => {
                let condition = self
                    .get_boolean_input(ctx, "CONDITION")
                    .trace("Block::compile.control_if")?;

                let compiled_blocks = self
                    .compile_substack(ctx, "SUBSTACK")
                    .trace("Block::compile.control_if")?;

                Ok(ScratchBlock::ControlIf(condition, compiled_blocks))
            }
            "control_if_else" => {
                let condition = self
                    .get_boolean_input(ctx, "CONDITION")
                    .trace("Block::compile.control_if_else")?;

                let compiled_blocks = self
                    .compile_substack(ctx, "SUBSTACK")
                    .trace("Block::compile.control_if_else")?;

                let compiled_blocks2 = self
                    .compile_substack(ctx, "SUBSTACK2")
                    .trace("Block::compile.control_if_else")?;

                Ok(ScratchBlock::ControlIfElse(
                    condition,
                    compiled_blocks,
                    compiled_blocks2,
                ))
            }
            "control_repeat" => {
                let times = self
                    .get_number_input(ctx, "TIMES")
                    .trace("Block::compile.control_repeat")?;

                let compiled_blocks = self
                    .compile_substack(ctx, "SUBSTACK")
                    .trace("Block::compile.control_repeat")?;

                Ok(ScratchBlock::ControlRepeat(times, compiled_blocks))
            }
            "control_repeat_until" => {
                let condition = self
                    .get_boolean_input(ctx, "CONDITION")
                    .trace("Block::compile.control_repeat_until")?;
                let compiled_blocks = self
                    .compile_substack(ctx, "SUBSTACK")
                    .trace("Block::compile.control_repeat")?;

                Ok(ScratchBlock::ControlRepeatUntil(condition, compiled_blocks))
            }
            "procedures_call" => {
                let block = ctx.get_custom_block(self)?;

                let args: Result<Vec<Input>, RashError> = block
                    .args
                    .iter()
                    .map(|n| {
                        self.get_number_input(ctx, n)
                            .trace("Block::compile.procedures_call")
                    })
                    .collect();
                let args = args?;

                Ok(if block.is_screen_refresh {
                    ScratchBlock::FunctionCallScreenRefresh(block.id, args)
                } else {
                    ScratchBlock::FunctionCallNoScreenRefresh(block.id, args)
                })
            }
            "argument_reporter_string_number" => self.c_argument_reporter(ctx),
            _ => {
                println!("Unknown opcode: {}\n{self:#?}\n", self.opcode);
                Ok(ScratchBlock::OpAdd(0.0.into(), 0.0.into()))
            }
        }
    }

    fn c_argument_reporter(&self, ctx: &mut CompileContext<'_>) -> Result<ScratchBlock, RashError> {
        let arg = self.fields.get("VALUE").ok_or(RashError::field_not_found(
            "self(argument_reporter_string_number).fields.VALUE",
        ))?;
        match arg {
            serde_json::Value::Array(values) => {
                let arg_name = values
                    .first()
                    .ok_or(RashError::field_not_found(
                        "self(argument_reporter_string_number).fields.VALUE[0]",
                    ))?
                    .as_str()
                    .ok_or(RashError::field_not_found(
                        "self(argument_reporter_string_number).fields.VALUE[0]: not string",
                    ))?;

                let current_custom_block = ctx
                    .current_custom_block
                    .as_ref()
                    .ok_or(get_blockdef_error("current_custom_block"))?;
                println!("{:?}", ctx.custom_block_defs);
                let blockdef = ctx
                    .custom_block_defs
                    .get(current_custom_block)
                    .ok_or(get_blockdef_error("blockdef"))?;
                let name_to_id = blockdef
                    .args_name_to_id
                    .as_ref()
                    .ok_or(get_blockdef_error("blockdef.name_to_id"))?;
                let arg_id = name_to_id
                    .get(arg_name)
                    .ok_or(get_blockdef_error("blockdef.name_to_id.get"))?;

                let position = blockdef
                    .args
                    .iter()
                    .position(|n| n == arg_id)
                    .ok_or(get_blockdef_error("blockdef.args"))?;

                Ok(ScratchBlock::FunctionGetArg(position))
            }
            _ => todo!(),
        }
    }
}

fn get_blockdef_error(n: &'static str) -> RashError {
    RashError {
        trace: vec![format!(
            "Block::compile.argument_reporter_string_number ({})",
            n
        )],
        kind: RashErrorKind::CurrentCustomBlockNotFound,
    }
}
