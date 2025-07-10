//! # JSON structures for Scratch projects.
//!
//! This module contains the structures that represent the
//! JSON data of a Scratch project.
//!
//! Scratch `.sb3` files are just ZIP files that
//! contain a JSON file called `project.json`,
//! as well as the costumes and sounds.

#![allow(unused)]

use std::{collections::BTreeMap, fmt::Debug};

use serde::Deserialize;
use serde_json::Value;

#[allow(unused)]
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
#[allow(non_snake_case)]
pub struct Target {
    pub isStage: bool,
    pub name: String,
    pub variables: BTreeMap<String, Vec<Value>>,
    pub lists: Value,
    pub broadcasts: Value,
    pub blocks: BTreeMap<String, JsonBlock>,
    pub comments: Value,
    pub currentCostume: i64,
    pub costumes: Vec<TargetCostume>,
    pub sounds: Vec<Value>,
    pub volume: f64,
    pub layerOrder: i64,
    pub tempo: Option<f64>,
    pub visible: Option<bool>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub size: Option<f64>,
    pub direction: Option<f64>,
    pub draggable: Option<bool>,
    pub rotationStyle: Option<String>,
    pub videoTransparency: Option<f64>,
    pub videoState: Option<String>,
    pub textToSpeechLanguage: Option<Value>,
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
#[allow(non_snake_case)]
pub struct Block {
    pub opcode: String,
    pub next: Option<String>,
    pub parent: Option<String>,
    pub inputs: BTreeMap<String, Value>,
    pub fields: BTreeMap<String, Value>,
    pub shadow: bool,
    pub topLevel: bool,

    pub mutation: Option<BlockMutation>,

    // Only for hat blocks.
    pub x: Option<f64>,
    pub y: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct BlockMutation {
    pub tagName: String,
    pub children: Vec<Value>,
    pub proccode: String,
    pub argumentids: String,
    pub argumentnames: Option<String>,
    pub argumentdefaults: Option<String>,
    pub warp: String,
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
#[allow(non_snake_case)]
pub struct TargetCostume {
    pub name: String,
    pub dataFormat: String,
    pub assetId: String,
    pub md5ext: String,
    pub rotationCenterX: f64,
    pub rotationCenterY: f64,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Monitor {
    pub id: String,
    pub mode: String,
    pub opcode: String,
    pub params: Value,
    pub spriteName: Option<String>,
    pub value: Value,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub x: f64,
    pub y: f64,
    pub visible: bool,
    pub sliderMin: Option<f64>,
    pub sliderMax: Option<f64>,
    pub isDiscrete: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct MonitorParams {
    VARIABLE: String,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub semver: String,
    pub vm: String,
    pub agent: String,
}
