use std::{cmp::Ordering, collections::HashMap, path::Path};

use json::{Block, JsonBlock, JsonStruct};
use tempfile::TempDir;

use rash_vm::{
    Input, MEMORY, Ptr, ScratchBlock,
    data_types::ScratchObject,
    error::{ErrorConvert, RashError, Trace},
    graphics::{CostumeData, CostumeHash, CostumeId, SpriteId, SpriteLoadData},
    runtime::{CustomBlockId, ProjectBuilder, Runtime, Script, SpriteBuilder},
};

use crate::{
    blocks::{control, op},
    error::{ErrExt, ErrorConvertPath},
};
use rash_loader_sb3_json as json;

mod blocks;
mod error;
mod get;

pub type Res<T> = Result<T, error::Error>;

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

    fn get_custom_block(&mut self, block: &Block) -> Res<CustomBlockDef> {
        const FN_N: &str = "CompileContext::get_custom_block";

        let block_mutation = block
            .mutation
            .as_ref()
            .ok_or(RashError::field_not_found("self.mutation"))
            .trace(FN_N)?;

        let proccode = block_mutation
            .proccode
            .as_ref()
            .ok_or(RashError::field_not_found("self.mutation.proccode"))
            .trace(FN_N)?;
        let argumentids = block_mutation
            .argumentids
            .as_ref()
            .ok_or(RashError::field_not_found("self.mutation.argumentids"))
            .trace(FN_N)?;

        if let Some(def) = self.custom_block_defs.get_mut(proccode) {
            if def.args_name_to_id.is_none() {
                let args: Vec<String> = serde_json::from_str(argumentids)
                    .to("serde_json::from_str(self.mutation)", FN_N)?;
                if let Some(names) = &block_mutation.argumentnames {
                    def.args_name_to_id = Some(build_argument_names(&args, names)?);
                }
            }
            Ok(def.clone())
        } else {
            let args: Vec<String> = serde_json::from_str(argumentids)
                .to("serde_json::from_str(self.mutation)", FN_N)?;
            let args_name_to_id = if let Some(names) = &block_mutation.argumentnames {
                Some(build_argument_names(&args, names)?)
            } else {
                None
            };

            let warp = block_mutation
                .warp
                .as_ref()
                .ok_or(RashError::field_not_found("self.mutation.warp"))
                .trace(FN_N)?;
            let warp = if warp == "true" {
                true
            } else if warp == "false" {
                false
            } else {
                return Err(RashError::invalid_warp_kind(warp));
            };
            let blockdef = CustomBlockDef {
                args,
                args_name_to_id,
                is_screen_refresh: !warp,
                id: CustomBlockId(*self.custom_block_num),
            };
            *self.custom_block_num += 1;
            self.custom_block_defs.insert(proccode.clone(), blockdef);
            Ok(self.custom_block_defs.get(proccode).unwrap().clone())
        }
    }
}

fn build_argument_names(args: &[String], names: &str) -> Res<HashMap<String, String>> {
    const FN_N: &str = "build_argument_names";

    let arg_names: Vec<String> =
        serde_json::from_str(names).to("serde_json::from_str(self.mutation)", FN_N)?;
    let collect = args
        .iter()
        .zip(arg_names)
        .map(|(a, b)| (b, a.clone()))
        .collect();
    Ok(collect)
}

impl ProjectLoader {
    pub fn new(file_path: &Path) -> Res<Self> {
        const FN_N: &str = "ProjectLoader::new";
        println!("[info] Loading file from {}", file_path.display());

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

    pub fn build(self) -> Res<Runtime> {
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
            let state = SpriteLoadData {
                x: sprite_json.x.unwrap_or_default(),
                y: sprite_json.y.unwrap_or_default(),
                size: sprite_json.size.unwrap_or(100.0),
                costume,
                shown: sprite_json.visible.unwrap_or(true),
            };

            state_map.insert(id, state);

            load_blocks(
                sprite_json,
                &mut variable_map,
                &mut custom_block_num,
                &mut sprite,
                &memory,
            )?;

            builder.add_sprite(sprite);
        }

        builder.set_costume(costume_names, costume_numbers, costume_hashes, costume_ids);
        builder.set_init_state(state_map);

        Ok(builder.build(&memory))
    }

    fn load_costumes(
        &self,
        sprite_json: &json::Target,
        costume_names: &mut HashMap<(SpriteId, String), CostumeHash>,
        costume_numbers: &mut HashMap<(SpriteId, usize), CostumeHash>,
        id: SpriteId,
        costume_hashes: &mut HashMap<CostumeHash, CostumeId>,
        costume_id: &mut CostumeId,
        costume_ids: &mut HashMap<CostumeId, CostumeData>,
    ) -> Res<()> {
        const FN_N: &str = "ProjectLoader::load_costumes";

        for (i, costume) in sprite_json.costumes.iter().enumerate() {
            let path = self.dir.path();
            let path = path.join(&costume.md5ext);
            let bytes = std::fs::read(&path).to_p(&path, "std::fs::read (costume)", FN_N)?;

            let is_svg = std::path::Path::new(&costume.md5ext)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));

            let intermediate = CostumeData {
                bytes,
                name: costume.name.clone(),
                hash: costume.assetId.clone(),
                rotation_center_x: costume.rotationCenterX,
                rotation_center_y: costume.rotationCenterY,
                is_svg,
            };

            let hash = CostumeHash::new(&costume.assetId);
            costume_names.insert((id, costume.name.clone()), hash.clone());
            costume_numbers.insert((id, i), hash.clone());

            if costume_hashes.contains_key(&hash) {
                continue;
            }
            costume_hashes.insert(hash, *costume_id);
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
) -> Res<()> {
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
            let details = get::custom_block_prototype(hat_block)?;
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
            ctx.current_custom_block = proccode;

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

            blocks.push(load_block(block, &mut ctx).trace(&format!(
                "ProjectLoader::build (sprite: {})",
                sprite_json.name
            ))?);

            id = block.next.clone();
        }

        match hat_block.opcode.as_str() {
            "event_whenflagclicked" => {
                let new_green_flag = Script::new_green_flag(blocks);
                sprite.add_script(new_green_flag, memory);
            }
            "procedures_definition" => {
                let custom_block = custom_block.unwrap();

                sprite.add_script(
                    Script::new_custom_block(
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

pub fn load_block(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
    match b.opcode.as_str() {
        "data_setvariableto" => {
            // b.fields.VARIABLE[1]
            let variable_id = get::variable_field(b)?;
            let variable_ptr = ctx.get_var(variable_id);

            let value = get::number(b, ctx, "VALUE").trace("Block::compile.data_setvariableto")?;

            Ok(ScratchBlock::VarSet(variable_ptr, value))
        }
        "operator_add" => op::add(b, ctx),
        "operator_subtract" => op::subtract(b, ctx),
        "operator_multiply" => op::multiply(b, ctx),
        "operator_divide" => op::divide(b, ctx),
        "operator_random" => op::random(b, ctx),
        "operator_join" => op::str_join(b, ctx),
        "operator_letter_of" => op::str_letter_of(b, ctx),
        "operator_contains" => op::str_contains(b, ctx),
        "operator_length" => op::str_length(b, ctx),
        "operator_mod" => op::modulo(b, ctx),
        "operator_round" => op::round(b, ctx),
        "operator_gt" => op::cmp(b, ctx, Ordering::Greater),
        "operator_lt" => op::cmp(b, ctx, Ordering::Less),
        "operator_equals" => op::cmp(b, ctx, Ordering::Equal),
        "operator_and" => Ok(op::and(b, ctx)),
        "operator_or" => Ok(op::or(b, ctx)),
        "operator_not" => op::not(b, ctx),
        "operator_mathop" => op::mathop(b, ctx),
        "data_changevariableby" => {
            let variable = get::variable_field(b)?;
            let value =
                get::number(b, ctx, "VALUE").trace("Block::compile.data_changevariableby")?;
            Ok(ScratchBlock::VarChange(ctx.get_var(variable), value))
        }
        "motion_gotoxy" => {
            let x = get::number(b, ctx, "X").trace("Block::compile.motion_gotoxy")?;
            let y = get::number(b, ctx, "Y").trace("Block::compile.motion_gotoxy")?;

            Ok(ScratchBlock::MotionGoToXY(x, y))
        }
        "motion_setx" => {
            let n = get::number(b, ctx, "X").trace("Block::compile.motion_setx")?;
            Ok(ScratchBlock::MotionSetX(n))
        }
        "motion_sety" => {
            let n = get::number(b, ctx, "Y").trace("Block::compile.motion_sety")?;
            Ok(ScratchBlock::MotionSetY(n))
        }
        "motion_changexby" => {
            let val = get::number(b, ctx, "DX").trace("Block::compile.motion_changexby")?;
            Ok(ScratchBlock::MotionChangeX(val))
        }
        "motion_changeyby" => {
            let val = get::number(b, ctx, "DY").trace("Block::compile.motion_changeyby")?;
            Ok(ScratchBlock::MotionChangeY(val))
        }
        "looks_show" => Ok(ScratchBlock::LooksShown(true)),
        "looks_hide" => Ok(ScratchBlock::LooksShown(false)),
        "control_if" => control::c_if(b, ctx),
        "control_if_else" => control::c_if_else(b, ctx),
        "control_repeat" => control::repeat(b, ctx),
        "control_repeat_until" => control::repeat_until(b, ctx),
        "control_forever" => control::forever(b, ctx),
        "looks_say" => {
            // TODO: implement this properly
            let message = get::string(b, ctx, "MESSAGE").trace("Block::compile.looks_say")?;
            Ok(ScratchBlock::Log(message))
        }
        "sensing_dayssince2000" => Ok(ScratchBlock::ControlDaysSince2000),
        "procedures_call" => {
            let block = ctx.get_custom_block(b)?;

            let args: Res<Vec<Input>> = block
                .args
                .iter()
                .map(|n| get::number(b, ctx, n).trace("Block::compile.procedures_call"))
                .collect();
            let args = args?;

            Ok(if block.is_screen_refresh {
                ScratchBlock::FunctionCallScreenRefresh(block.id, args)
            } else {
                ScratchBlock::FunctionCallNoScreenRefresh(block.id, args)
            })
        }
        "argument_reporter_string_number" => blocks::argument_reporter(b, ctx),
        _ => {
            println!("Unknown opcode: {}\n{b:#?}\n", b.opcode);
            Ok(ScratchBlock::OpAdd(0.0.into(), 0.0.into()))
        }
    }
}
