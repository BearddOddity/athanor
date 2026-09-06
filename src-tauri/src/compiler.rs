//! Compilation pipeline - modify and repack game files

use crate::{
    CompileRequest, CompileResult, Modification, ModificationOp,
    ParsedFile, Result, builder::BinaryBuilder,
};
use crate::formats::Format;
use std::path::PathBuf;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn compile(&self, request: &CompileRequest) -> Result<CompileResult> {
        // Resolve the actual file path from source
        let actual_input_path = self.resolve_game_path(&request.source, &request.input_path)?;
        
        // DUPLICATE THEN MODIFY: Always read from source, write to output
        // Output path should be different from source path (we're duplicating to modify)
        // The key protection: source files are NEVER written to, only READ
        
        // Read from source (creates in-memory copy)
        let mut file = self.read_file(&actual_input_path)?;
        file.metadata.path = request.input_path.clone(); // Track original relative path
        
        // Apply modifications to the in-memory copy
        if let Some(modifications) = &request.modifications {
            for modification in modifications {
                self.apply_modification(&mut file, modification)?;
            }
        }
        
        self.recalculate_headers(&mut file);
        
        // Write the modified copy to output_path (NEW file, source unchanged)
        let output_data = self.serialize(&file)?;
        if let Some(parent) = request.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&request.output_path, &output_data)?;
        
        let bytes_written = std::fs::metadata(&request.output_path).map(|m| m.len()).unwrap_or(0);
        
        Ok(CompileResult {
            success: true,
            output_path: request.output_path.clone(),
            bytes_written,
            errors: vec![],
            warnings: vec![],
        })
    }
    
    /// Resolve a game-relative path to an actual file path
    /// 
    /// This implements the "Bring Your Own Game" principle:
    /// - If source is None, input_path is used as-is
    /// - If source is provided, input_path is treated as relative to source
    /// - ISO/XBE support planned for future
    fn resolve_game_path(&self, source: &Option<PathBuf>, input_path: &PathBuf) -> Result<PathBuf> {
        match source {
            Some(source_path) if source_path.is_dir() => {
                // Directory source - resolve relative path
                let full_path = source_path.join(input_path);
                if full_path.exists() {
                    Ok(full_path)
                } else {
                    Err(crate::AthanorError::Parse(
                        format!("File not found in game source: {}", full_path.display())
                    ))
                }
            }
            Some(source_path) if source_path.extension().map(|e| e == "iso").unwrap_or(false) => {
                // ISO source - would need iso9660 parsing
                // For now, return error suggesting directory extraction
                Err(crate::AthanorError::Parse(
                    "ISO mounting not yet implemented. Please extract ISO to a directory.".to_string()
                ))
            }
            Some(source_path) if source_path.extension().map(|e| e == "xbe").unwrap_or(false) => {
                // XBE source - would need XBE parsing
                Err(crate::AthanorError::Parse(
                    "XBE mounting not yet implemented. Please extract XBE contents to a directory.".to_string()
                ))
            }
            _ => {
                // No source or direct path
                Ok(input_path.clone())
            }
        }
    }
    
    pub fn read_file(&self, path: &PathBuf) -> Result<ParsedFile> {
        crate::parser::parse_file(path)
    }
    
    fn apply_modification(&self, file: &mut ParsedFile, modification: &Modification) -> Result<()> {
        match modification.operation {
            ModificationOp::AddNode => {
                if let Some(node) = &modification.new_node {
                    file.nodes.push(node.clone());
                }
            }
            ModificationOp::RemoveNode => {
                file.nodes.retain(|n| n.index != modification.node_index);
            }
            ModificationOp::UpdateNode => {
                if let Some(node) = file.nodes.iter_mut().find(|n| n.index == modification.node_index) {
                    if let Some(new_node) = &modification.new_node {
                        *node = new_node.clone();
                    }
                }
            }
            ModificationOp::AddProperty => {
                if let Some(node) = file.nodes.iter_mut().find(|n| n.index == modification.node_index) {
                    if let Some(prop) = &modification.new_property {
                        node.properties.push(prop.clone());
                    }
                }
            }
            ModificationOp::UpdateProperty => {
                if let Some(node) = file.nodes.iter_mut().find(|n| n.index == modification.node_index) {
                    if let Some(prop) = node.properties.iter_mut().find(|p| p.key.as_deref() == modification.new_property.as_ref().and_then(|p| p.key.as_deref())) {
                        if let Some(new_prop) = &modification.new_property {
                            *prop = new_prop.clone();
                        }
                    }
                }
            }
            ModificationOp::RemoveProperty => {
                if let Some(node) = file.nodes.iter_mut().find(|n| n.index == modification.node_index) {
                    let key = modification.new_property.as_ref().and_then(|p| p.key.as_deref());
                    node.properties.retain(|p| p.key.as_deref() != key);
                }
            }
        }
        Ok(())
    }
    
    fn recalculate_headers(&self, file: &mut ParsedFile) {
        file.metadata.string_count = Some(file.strings.len() as u32);
        file.metadata.node_count = Some(file.nodes.len());
    }
    
    fn serialize(&self, file: &ParsedFile) -> Result<Vec<u8>> {
        let format = crate::formats::Format::from_extension(&file.metadata.format);
        match format {
            Format::XMLB => {
                let mut builder = crate::builder::XMLBBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.strings = file.strings.clone();
                builder.compile(&file.raw_data)
            }
            Format::PKGB => {
                let mut builder = crate::builder::PKGBBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.strings = file.strings.clone();
                builder.compile(&file.raw_data)
            }
            Format::ENGB => {
                let mut builder = crate::builder::ENGBBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.strings = file.strings.clone();
                builder.compile(&file.raw_data)
            }
            Format::BOYB => {
                let mut builder = crate::builder::BOYBBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.strings = file.strings.clone();
                builder.compile(&file.raw_data)
            }
            Format::CHRB => {
                let mut builder = crate::builder::CHRBBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::NAVB => {
                let mut builder = crate::builder::NAVBBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::BNX => {
                let mut builder = crate::builder::BNXBuilder::new();
                builder.pairs = file.nodes.iter().map(|n| {
                    (n.name.clone().unwrap_or_default(), 
                     n.properties.first().and_then(|p| match &p.value {
                         crate::PropertyValue::String(s) => Some(s.clone()),
                         _ => None,
                     }).unwrap_or_default())
                }).collect();
                builder.compile(&file.raw_data)
            }
            Format::IGB => {
                let mut builder = crate::builder::IGBBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.strings = file.strings.clone();
                builder.compile(&file.raw_data)
            }
            Format::ZSM => {
                let mut builder = crate::builder::ZSMBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.strings = file.strings.clone();
                builder.compile(&file.raw_data)
            }
            Format::ZSS => {
                let mut builder = crate::builder::ZSSBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.strings = file.strings.clone();
                builder.compile(&file.raw_data)
            }
            Format::ZAM => {
                let mut builder = crate::builder::ZAMBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.strings = file.strings.clone();
                builder.compile(&file.raw_data)
            }
            Format::ANIM => {
                let mut builder = crate::builder::ANIMBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::PHYS => {
                let mut builder = crate::builder::PHYSBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::AUD => {
                let mut builder = crate::builder::AUDBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::COMP => {
                let mut builder = crate::builder::COMPBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::PBR => {
                let mut builder = crate::builder::PBRBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::PLGN => {
                let mut builder = crate::builder::PLGNBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::SAVE => {
                let mut builder = crate::builder::SAVEBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            Format::PIPE => {
                let mut builder = crate::builder::PIPEBuilder::new();
                builder.nodes = file.nodes.clone();
                builder.compile(&file.raw_data)
            }
            _ => Ok(file.raw_data.clone()),
        }
    }
}