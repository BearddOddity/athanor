//! Format builders - compile structured data back to binary
use crate::{Result, AthanorError, PropertyValue};

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
    
    fn build_string_map(&self) -> std::collections::HashMap<String, u32> {
        let mut map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for (off, s) in &self.strings {
            // Only insert if not already present (prefer first occurrence)
            map.entry(s.clone()).or_insert(*off);
        }
        map
    }
}

impl BinaryBuilder for XMLBBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        
        // Header: 24 bytes
        output.extend_from_slice(&0x000011B1u32.to_le_bytes());
        output.extend_from_slice(&[0u8; 4]); // padding
        output.extend_from_slice(&[0u8; 4]); // string table offset placeholder
        output.extend_from_slice(&[0u8; 4]); // padding
        output.extend_from_slice(&[0u8; 2]); // reserved
        output.extend_from_slice(&[0u8; 6]); // reserved
        
        let string_table_offset = 24 + (self.nodes.len() * 32) as u32;
        
        // Build reverse string map: value -> offset
        let string_map = self.build_string_map();
        
        // Write nodes (32 bytes each)
        for (_i, node) in self.nodes.iter().enumerate() {
            let mut node_offsets = [0xFFFFFFFFu32; 8];
            
            if let Some(ref type_str) = node.node_type {
                if let Some(&off) = string_map.get(type_str) {
                    node_offsets[0] = off;
                }
            }
            
            if let Some(ref name) = node.name {
                if let Some(&off) = string_map.get(name) {
                    node_offsets[1] = off;
                }
            }
            
            for (j, prop) in node.properties.iter().enumerate().take(3) {
                let k = 2 + j * 2;
                let v = k + 1;
                if let Some(ref key) = prop.key {
                    if let Some(&off) = string_map.get(key) {
                        node_offsets[k] = off;
                    }
                }
                match &prop.value {
                    PropertyValue::String(ref s) => {
                        if let Some(&off) = string_map.get(s) {
                            node_offsets[v] = off;
                        }
                    }
                    PropertyValue::Number(val) => {
                        node_offsets[v] = *val as i64 as u32;
                    }
                    PropertyValue::Integer(val) => {
                        node_offsets[v] = *val as u32;
                    }
                    PropertyValue::Unsigned(val) => {
                        node_offsets[v] = *val as u32;
                    }
                    PropertyValue::Boolean(val) => {
                        node_offsets[v] = if *val { 1u32 } else { 0u32 };
                    }
                    PropertyValue::Bytes(_) => {}
                    PropertyValue::Null => {}
                }
            }
            
            for &offset in &node_offsets {
                output.extend_from_slice(&offset.to_le_bytes());
            }
        }
        
        // Write string table
        for (_off, s) in &self.strings {
            let bytes = s.as_bytes();
            output.extend_from_slice(bytes);
            output.push(0);
        }
        
        // Update string table offset in header
        output[8..12].copy_from_slice(&string_table_offset.to_le_bytes());
        
        Ok(output)
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

/// Builder for PKGB format (like XMLB, different magic)
pub struct PKGBBuilder {
    pub nodes: Vec<crate::ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
}

impl PKGBBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            strings: std::collections::HashMap::new(),
        }
    }
    
    fn build_string_map(&self) -> std::collections::HashMap<String, u32> {
        let mut map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for (off, s) in &self.strings {
            map.entry(s.clone()).or_insert(*off);
        }
        map
    }
}

impl BinaryBuilder for PKGBBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&0x00004B50u32.to_le_bytes());
        output.extend_from_slice(&[0u8; 16]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        let string_table_offset = 24 + (self.nodes.len() * 32) as u32;
        let string_map = self.build_string_map();
        for (_i, node) in self.nodes.iter().enumerate() {
            let mut node_offsets = [0xFFFFFFFFu32; 8];
            if let Some(ref type_str) = node.node_type {
                if let Some(&off) = string_map.get(type_str) {
                    node_offsets[0] = off;
                }
            }
            if let Some(ref name) = node.name {
                if let Some(&off) = string_map.get(name) {
                    node_offsets[1] = off;
                }
            }
            for (j, prop) in node.properties.iter().enumerate().take(3) {
                let k = 2 + j * 2;
                let v = k + 1;
                if let Some(ref key) = prop.key {
                    if let Some(&off) = string_map.get(key) {
                        node_offsets[k] = off;
                    }
                }
                match &prop.value {
                    PropertyValue::String(ref s) => {
                        if let Some(&off) = string_map.get(s) {
                            node_offsets[v] = off;
                        }
                    }
                    PropertyValue::Number(val) => {
                        node_offsets[v] = *val as i64 as u32;
                    }
                    PropertyValue::Integer(val) => {
                        node_offsets[v] = *val as u32;
                    }
                    PropertyValue::Unsigned(val) => {
                        node_offsets[v] = *val as u32;
                    }
                    PropertyValue::Boolean(val) => {
                        node_offsets[v] = if *val { 1u32 } else { 0u32 };
                    }
                    PropertyValue::Bytes(_) => {}
                    PropertyValue::Null => {}
                }
            }
            for &offset in &node_offsets {
                output.extend_from_slice(&offset.to_le_bytes());
            }
        }
        for (_off, s) in &self.strings {
            output.extend_from_slice(s.as_bytes());
            output.push(0);
        }
        output[8..12].copy_from_slice(&string_table_offset.to_le_bytes());
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 24 {
            return Err(AthanorError::InvalidData("PKGB file too small".to_string()));
        }
        Ok(())
    }
}

/// Builder for ENGB format (like XMLB, different magic)
pub struct ENGBBuilder {
    pub nodes: Vec<crate::ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
}

impl ENGBBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            strings: std::collections::HashMap::new(),
        }
    }
    
    fn build_string_map(&self) -> std::collections::HashMap<String, u32> {
        let mut map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for (off, s) in &self.strings {
            map.entry(s.clone()).or_insert(*off);
        }
        map
    }
}

impl BinaryBuilder for ENGBBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&0x00004745u32.to_le_bytes());
        output.extend_from_slice(&[0u8; 16]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        let string_table_offset = 24 + (self.nodes.len() * 32) as u32;
        let string_map = self.build_string_map();
        for (_i, node) in self.nodes.iter().enumerate() {
            let mut node_offsets = [0xFFFFFFFFu32; 8];
            if let Some(ref type_str) = node.node_type {
                if let Some(&off) = string_map.get(type_str) {
                    node_offsets[0] = off;
                }
            }
            if let Some(ref name) = node.name {
                if let Some(&off) = string_map.get(name) {
                    node_offsets[1] = off;
                }
            }
            for (j, prop) in node.properties.iter().enumerate().take(3) {
                let k = 2 + j * 2;
                let v = k + 1;
                if let Some(ref key) = prop.key {
                    if let Some(&off) = string_map.get(key) {
                        node_offsets[k] = off;
                    }
                }
                match &prop.value {
                    PropertyValue::String(ref s) => {
                        if let Some(&off) = string_map.get(s) {
                            node_offsets[v] = off;
                        }
                    }
                    PropertyValue::Number(val) => {
                        node_offsets[v] = *val as i64 as u32;
                    }
                    PropertyValue::Integer(val) => {
                        node_offsets[v] = *val as u32;
                    }
                    PropertyValue::Unsigned(val) => {
                        node_offsets[v] = *val as u32;
                    }
                    PropertyValue::Boolean(val) => {
                        node_offsets[v] = if *val { 1u32 } else { 0u32 };
                    }
                    PropertyValue::Bytes(_) => {}
                    PropertyValue::Null => {}
                }
            }
            for &offset in &node_offsets {
                output.extend_from_slice(&offset.to_le_bytes());
            }
        }
        for (_off, s) in &self.strings {
            output.extend_from_slice(s.as_bytes());
            output.push(0);
        }
        output[8..12].copy_from_slice(&string_table_offset.to_le_bytes());
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 24 {
            return Err(AthanorError::InvalidData("ENGB file too small".to_string()));
        }
        Ok(())
    }
}

/// Builder for BOYB format (like XMLB, different magic)
pub struct BOYBBuilder {
    pub nodes: Vec<crate::ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
}

impl BOYBBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            strings: std::collections::HashMap::new(),
        }
    }
    
    fn build_string_map(&self) -> std::collections::HashMap<String, u32> {
        let mut map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for (off, s) in &self.strings {
            map.entry(s.clone()).or_insert(*off);
        }
        map
    }
}

impl BinaryBuilder for BOYBBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&0x0000424Fu32.to_le_bytes());
        output.extend_from_slice(&[0u8; 16]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        let string_table_offset = 24 + (self.nodes.len() * 32) as u32;
        let string_map = self.build_string_map();
        for (_i, node) in self.nodes.iter().enumerate() {
            let mut node_offsets = [0xFFFFFFFFu32; 8];
            if let Some(ref type_str) = node.node_type {
                if let Some(&off) = string_map.get(type_str) {
                    node_offsets[0] = off;
                }
            }
            if let Some(ref name) = node.name {
                if let Some(&off) = string_map.get(name) {
                    node_offsets[1] = off;
                }
            }
            for (j, prop) in node.properties.iter().enumerate().take(3) {
                let k = 2 + j * 2;
                let v = k + 1;
                if let Some(ref key) = prop.key {
                    if let Some(&off) = string_map.get(key) {
                        node_offsets[k] = off;
                    }
                }
                match &prop.value {
                    PropertyValue::String(ref s) => {
                        if let Some(&off) = string_map.get(s) {
                            node_offsets[v] = off;
                        }
                    }
                    PropertyValue::Number(val) => {
                        node_offsets[v] = *val as i64 as u32;
                    }
                    PropertyValue::Integer(val) => {
                        node_offsets[v] = *val as u32;
                    }
                    PropertyValue::Unsigned(val) => {
                        node_offsets[v] = *val as u32;
                    }
                    PropertyValue::Boolean(val) => {
                        node_offsets[v] = if *val { 1u32 } else { 0u32 };
                    }
                    PropertyValue::Bytes(_) => {}
                    PropertyValue::Null => {}
                }
            }
            for &offset in &node_offsets {
                output.extend_from_slice(&offset.to_le_bytes());
            }
        }
        for (_off, s) in &self.strings {
            output.extend_from_slice(s.as_bytes());
            output.push(0);
        }
        output[8..12].copy_from_slice(&string_table_offset.to_le_bytes());
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 24 {
            return Err(AthanorError::InvalidData("BOYB file too small".to_string()));
        }
        Ok(())
    }
}

/// Builder for CHRB format (24-byte header, 64-byte nodes)
pub struct CHRBBuilder {
    pub nodes: Vec<crate::ParsedNode>,
}

impl CHRBBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl BinaryBuilder for CHRBBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&0x00004843u32.to_le_bytes());
        output.extend_from_slice(&[0u8; 16]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for (_i, node) in self.nodes.iter().enumerate() {
            output.extend_from_slice(&node.raw_bytes);
        }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 24 {
            return Err(AthanorError::InvalidData("CHRB file too small".to_string()));
        }
        Ok(())
    }
}

/// Builder for NAVB format (24-byte header, 48-byte nodes)
pub struct NAVBBuilder {
    pub nodes: Vec<crate::ParsedNode>,
}

impl NAVBBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl BinaryBuilder for NAVBBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&0x00004E41u32.to_le_bytes());
        output.extend_from_slice(&[0u8; 16]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for (_i, node) in self.nodes.iter().enumerate() {
            output.extend_from_slice(&node.raw_bytes);
        }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 24 {
            return Err(AthanorError::InvalidData("NAVB file too small".to_string()));
        }
        Ok(())
    }
}
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

/// Builder for IGB format (string table)
pub struct IGBBuilder {
    pub nodes: Vec<crate::ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
}
impl IGBBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new(), strings: std::collections::HashMap::new() } }
}
impl BinaryBuilder for IGBBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        for (_off, s) in &self.strings {
            output.extend_from_slice(s.as_bytes());
            output.push(0);
        }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(AthanorError::InvalidData("IGB file is empty".to_string()));
        }
        Ok(())
    }
}

/// Builder for ZSM format (string table)
pub struct ZSMBuilder {
    pub nodes: Vec<crate::ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
}
impl ZSMBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new(), strings: std::collections::HashMap::new() } }
}
impl BinaryBuilder for ZSMBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 16]);
        let str_off = 16;
        for (_off, s) in &self.strings {
            output.extend_from_slice(s.as_bytes());
            output.push(0);
        }
        output[8..12].copy_from_slice(&(str_off as u32).to_le_bytes());
        output[12..16].copy_from_slice(&(self.strings.len() as u32).to_le_bytes());
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 16 {
            return Err(AthanorError::InvalidData("ZSM file too small".to_string()));
        }
        Ok(())
    }
}

/// Builder for ZSS format (string table)
pub struct ZSSBuilder {
    pub nodes: Vec<crate::ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
}
impl ZSSBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new(), strings: std::collections::HashMap::new() } }
}
impl BinaryBuilder for ZSSBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 24]);
        let str_off = 24;
        for (_off, s) in &self.strings {
            output.extend_from_slice(s.as_bytes());
            output.push(0);
        }
        output[16..20].copy_from_slice(&(str_off as u32).to_le_bytes());
        output[20..24].copy_from_slice(&(self.strings.len() as u32).to_le_bytes());
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 24 {
            return Err(AthanorError::InvalidData("ZSS file too small".to_string()));
        }
        Ok(())
    }
}

/// Builder for ZAM format
pub struct ZAMBuilder {
    pub nodes: Vec<crate::ParsedNode>,
    pub strings: std::collections::HashMap<u32, String>,
}
impl ZAMBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new(), strings: std::collections::HashMap::new() } }
}
impl BinaryBuilder for ZAMBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        for (_off, s) in &self.strings {
            output.extend_from_slice(s.as_bytes());
            output.push(0);
        }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(AthanorError::InvalidData("ZAM file is empty".to_string()));
        }
        Ok(())
    }
}

/// Builder for ANIM format (32-byte nodes)
pub struct ANIMBuilder { pub nodes: Vec<crate::ParsedNode> }
impl ANIMBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
}
impl BinaryBuilder for ANIMBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 32]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for node in &self.nodes { output.extend_from_slice(&node.raw_bytes); }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 32 { return Err(AthanorError::InvalidData("ANIM file too small".to_string())); }
        Ok(())
    }
}

/// Builder for PHYS format (32-byte nodes)
pub struct PHYSBuilder { pub nodes: Vec<crate::ParsedNode> }
impl PHYSBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
}
impl BinaryBuilder for PHYSBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 32]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for node in &self.nodes { output.extend_from_slice(&node.raw_bytes); }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 32 { return Err(AthanorError::InvalidData("PHYS file too small".to_string())); }
        Ok(())
    }
}

/// Builder for AUD format (32-byte nodes)
pub struct AUDBuilder { pub nodes: Vec<crate::ParsedNode> }
impl AUDBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
}
impl BinaryBuilder for AUDBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 32]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for node in &self.nodes { output.extend_from_slice(&node.raw_bytes); }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 32 { return Err(AthanorError::InvalidData("AUD file too small".to_string())); }
        Ok(())
    }
}

/// Builder for COMP format (32-byte nodes)
pub struct COMPBuilder { pub nodes: Vec<crate::ParsedNode> }
impl COMPBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
}
impl BinaryBuilder for COMPBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 32]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for node in &self.nodes { output.extend_from_slice(&node.raw_bytes); }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 32 { return Err(AthanorError::InvalidData("COMP file too small".to_string())); }
        Ok(())
    }
}

/// Builder for PBR format (32-byte nodes)
pub struct PBRBuilder { pub nodes: Vec<crate::ParsedNode> }
impl PBRBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
}
impl BinaryBuilder for PBRBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 32]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for node in &self.nodes { output.extend_from_slice(&node.raw_bytes); }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 32 { return Err(AthanorError::InvalidData("PBR file too small".to_string())); }
        Ok(())
    }
}

/// Builder for PLGN format (32-byte nodes)
pub struct PLGNBuilder { pub nodes: Vec<crate::ParsedNode> }
impl PLGNBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
}
impl BinaryBuilder for PLGNBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 32]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for node in &self.nodes { output.extend_from_slice(&node.raw_bytes); }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 32 { return Err(AthanorError::InvalidData("PLGN file too small".to_string())); }
        Ok(())
    }
}

/// Builder for SAVE format (32-byte nodes)
pub struct SAVEBuilder { pub nodes: Vec<crate::ParsedNode> }
impl SAVEBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
}
impl BinaryBuilder for SAVEBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 32]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for node in &self.nodes { output.extend_from_slice(&node.raw_bytes); }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 32 { return Err(AthanorError::InvalidData("SAVE file too small".to_string())); }
        Ok(())
    }
}

/// Builder for PIPE format (32-byte nodes)
pub struct PIPEBuilder { pub nodes: Vec<crate::ParsedNode> }
impl PIPEBuilder {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
}
impl BinaryBuilder for PIPEBuilder {
    fn compile(&self, _data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&[0u8; 32]);
        output.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        for node in &self.nodes { output.extend_from_slice(&node.raw_bytes); }
        Ok(output)
    }
    fn validate(&self, data: &[u8]) -> Result<()> {
        if data.len() < 32 { return Err(AthanorError::InvalidData("PIPE file too small".to_string())); }
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
    
    pub fn pkgb() -> PKGBBuilder {
        PKGBBuilder::new()
    }
    
    pub fn engb() -> ENGBBuilder {
        ENGBBuilder::new()
    }
    
    pub fn chrb() -> CHRBBuilder {
        CHRBBuilder::new()
    }
    
    pub fn navb() -> NAVBBuilder {
        NAVBBuilder::new()
    }
    
    pub fn boyb() -> BOYBBuilder {
        BOYBBuilder::new()
    }
    
    pub fn igb() -> IGBBuilder {
        IGBBuilder::new()
    }
    
    pub fn zsm() -> ZSMBuilder {
        ZSMBuilder::new()
    }
    
    pub fn zss() -> ZSSBuilder {
        ZSSBuilder::new()
    }
    
    pub fn zam() -> ZAMBuilder {
        ZAMBuilder::new()
    }
    
    pub fn anim() -> ANIMBuilder {
        ANIMBuilder::new()
    }
    
    pub fn phys() -> PHYSBuilder {
        PHYSBuilder::new()
    }
    
    pub fn aud() -> AUDBuilder {
        AUDBuilder::new()
    }
    
    pub fn comp() -> COMPBuilder {
        COMPBuilder::new()
    }
    
    pub fn pbr() -> PBRBuilder {
        PBRBuilder::new()
    }
    
    pub fn plgn() -> PLGNBuilder {
        PLGNBuilder::new()
    }
    
    pub fn save() -> SAVEBuilder {
        SAVEBuilder::new()
    }
    
    pub fn pipe() -> PIPEBuilder {
        PIPEBuilder::new()
    }
}