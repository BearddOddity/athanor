//! Athanor Core - Binary format assembly, disassembly, and compilation engine
//! 
//! This is a console recompilation engine. Athanor never stores game assets -
//! it scans the game at the time of assembly. Users provide their own game.iso,
//! game.xbe, or game directory.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AthanorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Build error: {0}")]
    Build(String),
    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AthanorError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub format: String,
    pub size: u64,
    pub magic: Option<u32>,
    pub version: Option<u32>,
    pub node_count: Option<usize>,
    pub string_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub metadata: FileMetadata,
    pub nodes: Vec<ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
    pub raw_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedNode {
    pub index: usize,
    pub file_offset: u32,
    pub node_type: Option<String>,
    pub name: Option<String>,
    pub properties: Vec<ParsedProperty>,
    pub raw_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedProperty {
    pub key: Option<String>,
    pub value: PropertyValue,
    pub is_raw: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Integer(i64),
    Unsigned(u64),
    Boolean(bool),
    Bytes(Vec<u8>),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileRequest {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub format: String,
    /// Path to game.iso, game.xbe, or game directory - scanned dynamically at assembly time
    pub source: Option<PathBuf>,
    pub modifications: Option<Vec<Modification>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modification {
    pub node_index: usize,
    pub operation: ModificationOp,
    pub new_node: Option<ParsedNode>,
    pub new_property: Option<ParsedProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModificationOp {
    AddNode,
    RemoveNode,
    UpdateNode,
    AddProperty,
    UpdateProperty,
    RemoveProperty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub success: bool,
    pub output_path: PathBuf,
    pub bytes_written: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// Modules
pub mod formats;
pub mod parser;
pub mod builder;
pub mod compiler;
pub mod dxt;
pub mod anchorpoint_integration;
pub mod console;
pub mod ghidra;

// Re-exports
pub use compiler::Compiler;