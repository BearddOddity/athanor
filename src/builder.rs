//! Format builders - compile structured data back to binary
use crate::{Result, AthanorError};

pub trait BinaryBuilder {
    /// Compile a node list back to binary
    fn compile(&self, data: &[u8]) -> Result<Vec<u8>>;
    /// Validate the binary structure
    fn validate(&self, data: &[u8]) -> Result<()>;
}

/// Builder for XMLB format
pub struct XMLBBuilder {
    pub nodes: Vec<crate::ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
}

impl XMLBBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            strings: std::collections::HashMap::new(),
        }
    }
}

impl BinaryBuilder for XMLBBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        Err(AthanorError::Build("XMLB compilation not yet implemented".to_string()))
    }
    
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 24 {
            return Err(AthanorError::InvalidData("XMLB file too small".to_string()));
        }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != 0x000011B1 {
            return Err(AthanorError::InvalidData("Invalid XMLB magic".to_string()));
        }
        Ok(())
    }
}

/// Builder for BNX format
pub struct BNXBuilder {
    pub pairs: Vec<(String, String)>,
}

impl BNXBuilder {
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
        }
    }
}

impl BinaryBuilder for BNXBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = String::new();
        for (key, value) in &self.pairs {
            output.push_str(&format!("{}={}\n", key, value));
        }
        Ok(output.into_bytes())
    }
    
    fn validate(&self, data: &[u8]) -> Result<()> {
        let text = std::str::from_utf8(data).map_err(|_| AthanorError::InvalidData("BNX file is not valid UTF-8".to_string()))?;
        for line in text.lines() {
            if !line.contains('=') && !line.trim().is_empty() {
                return Err(AthanorError::InvalidData(format!("BNX line missing '=' separator: {}", line)));
            }
        }
        Ok(())
    }
}

pub struct Builder;

impl Builder {
    pub fn xmlb() -> XMLBBuilder {
        XMLBBuilder::new()
    }
    
    pub fn bnx() -> BNXBuilder {
        BNXBuilder::new()
    }
}