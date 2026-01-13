use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphOutput {
    pub nodes: Vec<NodeWrapper>,
    pub edges: Vec<EdgeWrapper>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeWrapper {
    Module(ModuleNode),
    Function(FunctionNode),
    Struct(StructNode),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleNode {
    pub id: String,      // e.g., "0x1::coin"
    pub address: String, // "0x1"
    pub name: String,    // "coin"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionNode {
    pub id: String, // e.g., "0x1::coin::mint"
    pub module_id: String,
    pub name: String,
    pub visibility: String,
    pub is_native: bool,
    pub arg_count: usize,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructNode {
    pub id: String, // e.g., "0x1::coin::Coin"
    pub module_id: String,
    pub name: String,
    pub abilities: Vec<String>,
    pub is_resource: bool, // true if has 'key' ability
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EdgeWrapper {
    Defines { from: String, to: String }, // Module defines Function/Struct
    Calls { from: String, to: String },   // Function calls Function
}
