use std::{cmp::Ordering, collections::HashMap, path::Path};

use json::{Block, JsonBlock, JsonStruct};

use rash_vm::{
    Costumes, Input, MEMORY, Ptr, ScratchBlock,
    error::{ErrorConvert, RashError, Trace},
    graphics::{SpriteId, SpriteLoadData},
    runtime::{CustomBlockId, ProjectBuilder, Runtime, SpriteBuilder},
};
use zip::ZipArchive;

use crate::{
    blocks::{control, op},
    error::{ErrExt, ErrorConvertPath},
    sprites::{load_blocks, load_costumes},
};
use rash_loader_sb3_json as json;

mod blocks;
mod error;
mod get;
mod helpers;
mod sprites;

pub use error::{FieldType, Sb3ErrorKind};

pub type Res<T> = Result<T, error::Error>;

pub struct ProjectLoader {
    json: json::JsonStruct,
    archive: ZipArchive<std::fs::File>,
}

impl ProjectLoader {
    pub fn new(file_path: &Path) -> Res<Self> {
        const FN_N: &str = "ProjectLoader::new";
        println!("[info] Loading file from {}", file_path.display());

        let file = std::fs::File::open(file_path).to_p(file_path, "std::fs::File::open", FN_N)?;
        let mut archive = zip::ZipArchive::new(file).to("zip::ZipArchive::new", FN_N)?;

        let json = archive
            .by_name("project.json")
            .to("archive.by_name (project.json)", FN_N)?;
        let json: JsonStruct =
            serde_json::from_reader(json).to("serde_json::from_reader (project.json)", FN_N)?;

        Ok(Self { archive, json })
    }

    pub fn build(mut self) -> Res<Runtime> {
        const FN_N: &str = "ProjectLoader::build";

        let memory = MEMORY.lock().unwrap();

        let mut builder = ProjectBuilder::new();
        let mut costumes = Costumes::new();
        let mut variable_map = HashMap::new();
        let mut state_map = HashMap::new();

        let mut custom_block_num = 0;

        for (sprite_i, sprite_json) in self.json.targets.iter().enumerate() {
            let id = SpriteId(sprite_i as i64);
            let mut sprite = SpriteBuilder::new(id);

            load_costumes(&mut self.archive, sprite_json, &mut costumes, id).trace(FN_N)?;

            let costume = costumes
                .get_by_number(id, sprite_json.currentCostume as usize)
                .unwrap();
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

        builder.set_costumes(costumes);
        builder.set_init_state(state_map);

        Ok(builder.build(&memory))
    }
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

fn load_block(b: &Block, ctx: &mut CompileContext) -> Res<ScratchBlock> {
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
        "motion_xposition" => Ok(ScratchBlock::MotionGetX),
        "motion_yposition" => Ok(ScratchBlock::MotionGetY),
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
        "control_stop" => {
            const T: &str = "Block::compile.data_setvariableto";
            let stop_option = b
                .fields
                .stop_option
                .as_deref()
                .ok_or(RashError::field_not_found("b.fields.STOP_OPTION"))
                .trace(T)?;
            let option = stop_option
                .first()
                .ok_or(RashError::field_not_found("b.fields.STOP_OPTION[0]"))
                .trace(T)?;
            let s = option.as_str();

            if s == Some("this script") {
                Ok(ScratchBlock::ControlStopThisScript)
            } else if s == Some("all") {
                todo!("Stop all")
            } else if s == Some("other scripts in sprite") {
                todo!("Stop other scripts in sprite")
            } else {
                unreachable!();
            }
        }
        "sensing_dayssince2000" => Ok(ScratchBlock::SensingDaysSince2000),
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
