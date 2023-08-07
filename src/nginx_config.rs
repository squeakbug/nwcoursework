use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub status: String,
    pub errors: Vec<Value>,
    pub config: Vec<Config>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub file: String,
    pub status: String,
    pub errors: Vec<Value>,
    pub parsed: Vec<Parsed>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parsed {
    pub directive: String,
    pub line: i64,
    pub args: Vec<Value>,
    pub block: Vec<Block>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub directive: String,
    pub line: i64,
    pub args: Vec<String>,
    pub block: Vec<Block2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block2 {
    pub directive: String,
    pub line: i64,
    pub args: Vec<String>,
    #[serde(default)]
    pub block: Vec<Block3>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block3 {
    pub directive: String,
    pub line: i64,
    pub args: Vec<String>,
}
