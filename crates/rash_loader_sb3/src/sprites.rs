use std::{
    collections::HashMap,
    io::{Read, Seek},
};

use rash_core::{CostumeStore, RawCostumeData, SpriteId};
use rash_loader_sb3_json::{JsonBlock, Target};
use rash_vm::{
    Ptr, ScratchBlock, ScratchObject, SpriteBuilder,
    error::{ErrorConvert, RashError, Trace},
    runtime::Script,
};
use zip::ZipArchive;

use crate::{CompileContext, Res, Sb3ErrorKind, error::ErrExt, get, load_block};

pub fn load_costumes<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    sprite_json: &Target,
    costumes: &mut CostumeStore,
    id: SpriteId,
) -> Res<()> {
    const FN_N: &str = "ProjectLoader::load_costumes";

    for costume in &sprite_json.costumes {
        let bytes = get_costume_bytes(archive, costume).trace(FN_N)?;

        let is_svg = std::path::Path::new(&costume.md5ext)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));

        let data = RawCostumeData {
            bytes,
            name: costume.name.clone(),
            hash: costume.assetId.clone(),
            rotation_center_x: costume.rotationCenterX,
            rotation_center_y: costume.rotationCenterY,
            is_svg,
        };

        costumes.add_costume(data, costume.name.clone(), costume.assetId.clone(), id);
    }
    Ok(())
}

fn get_costume_bytes<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    costume: &rash_loader_sb3_json::TargetCostume,
) -> Result<Vec<u8>, RashError<Sb3ErrorKind>> {
    const F: &str = "ProjectLoader::get_costume_bytes";

    let mut file = archive
        .by_name(&costume.md5ext)
        .to("archive.by_name (costume)", F)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .to("read_to_end (costume)", F)?;
    Ok(bytes)
}

pub fn load_blocks(
    sprite_json: &Target,
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
