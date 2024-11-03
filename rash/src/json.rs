use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonStruct {
    pub targets: Vec<Target>,
    pub monitors: Vec<Monitor>,
    pub extensions: Vec<Value>,
    pub meta: Value,
}

#[derive(Serialize, Deserialize, Debug)]
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

/*impl Target {
    pub fn get_hat_blocks(&self) -> Vec<(&String, &JsonBlock)> {
        self.blocks
            .iter()
            .filter(|(_, block)| !block.has_parent())
            .collect()
    }
}*/

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Block {
    pub opcode: String,
    pub next: Option<String>,
    pub parent: Option<String>,
    pub inputs: BTreeMap<String, Value>,
    pub fields: BTreeMap<String, Value>,
    pub shadow: bool,
    pub topLevel: bool,
    // Only for hat blocks.
    pub x: Option<f64>,
    pub y: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
#[serde(untagged)]
pub enum JsonBlock {
    Block {
        #[serde(flatten)]
        block: Block,
    },
    Array(Vec<Value>),
}

/*impl JsonBlock {
    pub fn has_parent(&self) -> bool {
        matches!(
            self,
            JsonBlock::Block {
                block: Block {
                    parent: Some(_),
                    ..
                },
                ..
            }
        )
    }
}*/

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct TargetCostume {
    pub name: String,
    pub dataFormat: String,
    pub assetId: String,
    pub md5ext: String,
    pub rotationCenterX: f64,
    pub rotationCenterY: f64,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct MonitorParams {
    VARIABLE: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    pub semver: String,
    pub vm: String,
    pub agent: String,
}
