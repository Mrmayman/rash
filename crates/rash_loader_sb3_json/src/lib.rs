//! # JSON structures for Scratch projects.
//!
//! This module contains the structures that represent the
//! JSON data of a Scratch project.
//!
//! Scratch `.sb3` files are just ZIP files that
//! contain a JSON file called `project.json`,
//! as well as the costumes and sounds.
//!
//! # Why
//!
//! This would have been part of `rash_loader_sb3`
//! had it not been for compile time issues.

use std::{collections::BTreeMap, fmt::Debug};

use serde::Deserialize;
use serde_json::Value;

pub mod json_id {
    pub const NUMBER: i64 = 4;
    pub const POSITIVE_NUMBER: i64 = 5;
    pub const POSITIVE_INTEGER: i64 = 6;
    pub const INTEGER: i64 = 7;
    pub const ANGLE: i64 = 8;
    pub const COLOR: i64 = 9;
    pub const STRING: i64 = 10;
    pub const BROADCAST: i64 = 11;
    pub const VARIABLE: i64 = 12;
    pub const LIST: i64 = 13;
}

/// # The main JSON structure of a Scratch project.
///
/// This is the structure of the `project.json` file
/// located in the root of the `.sb3` file.
///
/// It contains the information and code of a project.
#[derive(Deserialize, Debug)]
pub struct JsonStruct {
    /// A list of targets (Scratch sprites).
    pub targets: Vec<Target>,
    /// A list of variable monitors (the boxes showing the values).
    pub monitors: Vec<Monitor>,
    // pub extensions: Vec<Value>,
    // pub meta: Value,
}

/// # A Scratch sprite.
#[derive(Deserialize, Debug, Clone)]

pub struct Target {
    #[serde(rename = "isStage")]
    pub is_stage: bool,
    pub name: String,
    pub variables: BTreeMap<String, Vec<Value>>,
    pub lists: Value,
    pub broadcasts: Value,
    pub blocks: BTreeMap<String, JsonBlock>,
    pub comments: Value,
    #[serde(rename = "currentCostume")]
    pub current_costume: i64,
    pub costumes: Vec<TargetCostume>,
    pub sounds: Vec<Value>,
    pub volume: f64,
    #[serde(rename = "layerOrder")]
    pub layer_order: i64,
    pub tempo: Option<f64>,
    pub visible: Option<bool>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub size: Option<f64>,
    pub direction: Option<f64>,
    pub draggable: Option<bool>,
    #[serde(rename = "rotationStyle")]
    pub rotation_style: Option<String>,
    #[serde(rename = "videoTransparency")]
    pub video_transparency: Option<f64>,
    #[serde(rename = "videoState")]
    pub video_state: Option<String>,
    #[serde(rename = "textToSpeechLanguage")]
    pub text_to_speech_language: Option<Value>,
}

impl Target {
    pub fn get_hat_blocks(&self) -> impl Iterator<Item = (&String, &JsonBlock)> {
        self.blocks.iter().filter(|(_, block)| {
            matches!(
                block,
                JsonBlock::Block {
                    block: Block {
                        next: Some(_),
                        parent: None,
                        ..
                    }
                }
            )
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Block {
    pub opcode: String,
    pub next: Option<String>,
    pub parent: Option<String>,
    pub inputs: BTreeMap<String, Value>,
    pub fields: BlockFields,
    pub shadow: bool,
    #[serde(rename = "topLevel")]
    pub top_level: bool,

    pub mutation: Option<BlockMutation>,

    // Only for hat blocks.
    pub x: Option<f64>,
    pub y: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockFields {
    #[serde(rename = "STOP_OPTION")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_option: Option<Vec<Value>>,
    #[serde(rename = "VARIABLE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable: Option<Vec<Value>>,
    #[serde(rename = "OPERATOR")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<Vec<Value>>,
    #[serde(rename = "VALUE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(flatten)]
    pub others: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockMutation {
    #[serde(rename = "tagName")]
    pub tag_name: String,
    pub children: Vec<Value>,
    pub proccode: Option<String>,
    pub argumentids: Option<String>,
    pub argumentnames: Option<String>,
    pub argumentdefaults: Option<String>,
    pub warp: Option<String>,
}

#[derive(Debug, Clone)]
pub enum JsonBlock {
    Block { block: Block },
    Array(Vec<Value>),
}

impl<'de> Deserialize<'de> for JsonBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Value = Value::deserialize(deserializer)?;

        if value.is_array() {
            let array = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(JsonBlock::Array(array))
        } else if value.is_object() {
            let block = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(JsonBlock::Block { block })
        } else {
            Err(serde::de::Error::custom(
                "JsonBlock: Could not determine type of Block, invalid JSON structure",
            ))
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct TargetCostume {
    pub name: String,
    #[serde(rename = "dataFormat")]
    pub data_format: String,
    #[serde(rename = "assetId")]
    pub asset_id: String,
    pub md5ext: String,
    #[serde(rename = "rotationCenterX")]
    pub rotation_center_x: f64,
    #[serde(rename = "rotationCenterY")]
    pub rotation_center_y: f64,
}

#[derive(Deserialize, Debug)]
pub struct Monitor {
    pub id: String,
    pub mode: String,
    pub opcode: String,
    pub params: Value,
    #[serde(rename = "spriteName")]
    pub sprite_name: Option<String>,
    pub value: Value,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub x: f64,
    pub y: f64,
    pub visible: bool,
    #[serde(rename = "sliderMin")]
    pub slider_min: Option<f64>,
    #[serde(rename = "sliderMax")]
    pub slider_max: Option<f64>,
    #[serde(rename = "isDiscrete")]
    pub is_discrete: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct MonitorParams {
    #[serde(rename = "VARIABLE")]
    pub variable: String,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub semver: String,
    pub vm: String,
    pub agent: String,
}
