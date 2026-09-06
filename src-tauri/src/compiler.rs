//! Compilation pipeline - modify and repack game files

use crate::{
    CompileRequest, CompileResult, Modification, ModificationOp,
    ParsedFile, Result,
};
use std::path::PathBuf;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }
    
    /// Compile a game file with modifications
    pub fn compile(&self, request: &CompileRequest) -> Result<CompileResult> {
        let mut file = self.read_file(&request.input_path)?;
        
        // Apply modifications
        if let Some(modifications) = &request.modifications {
            for modification in modifications {
                self.apply_modification(&mut file, modification)?;
            }
        }
        
        // Recalculate headers
        self.recalculate_headers(&mut file);
        
        // Write output
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
    
    /// Read and parse a game file
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
        Ok(file.raw_data.clone())
    }
}