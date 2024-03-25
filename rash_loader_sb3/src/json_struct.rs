use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct JsonStruct {
    targets: Vec<Target>,
    monitors: Vec<Monitor>,
    extensions: Vec<Value>,
    meta: Value,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Target {
    isStage: bool,
    name: String,
    variables: BTreeMap<String, Vec<Value>>,
    lists: Value,
    broadcasts: Value,
    blocks: BTreeMap<String, Block>,
    comments: Value,
    currentCostume: i64,
    costumes: Vec<TargetCostume>,
    sounds: Vec<Value>,
    volume: f64,
    layerOrder: i64,
    tempo: Option<f64>,
    visible: Option<bool>,
    x: Option<f64>,
    y: Option<f64>,
    size: Option<f64>,
    direction: Option<f64>,
    draggable: Option<bool>,
    rotationStyle: Option<String>,
    videoTransparency: Option<f64>,
    videoState: Option<String>,
    textToSpeechLanguage: Option<Value>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
#[serde(untagged)]
pub enum Block {
    Block {
        opcode: String,
        next: Option<String>,
        parent: Option<String>,
        inputs: BTreeMap<String, Value>,
        fields: BTreeMap<String, Value>,
        shadow: bool,
        topLevel: bool,
        // Only for hat blocks.
        x: Option<f64>,
        y: Option<f64>,
    },
    Array(Vec<Value>),
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TargetCostume {
    name: String,
    dataFormat: String,
    assetId: String,
    md5ext: String,
    rotationCenterX: f64,
    rotationCenterY: f64,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Monitor {
    id: String,
    mode: String,
    opcode: String,
    params: Value,
    spriteName: Option<String>,
    value: Value,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    visible: bool,
    sliderMin: Option<f64>,
    sliderMax: Option<f64>,
    isDiscrete: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct MonitorParams {
    VARIABLE: String,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    semver: String,
    vm: String,
    agent: String,
}
