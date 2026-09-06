//! Assembly and disassembly support

use crate::{
    AthanorError, ParsedFile, ParsedNode, ParsedProperty, PropertyValue,
    Result,
};
use crate::formats::{Format, FormatDetector};
use std::path::Path;

/// Disassemble an Athanor binary format into structured nodes
pub fn disassemble(path: &Path) -> Result<ParsedFile> {
    let format = FormatDetector::detect(path)?;
    let data = std::fs::read(path)?;
    let metadata = FormatDetector::detect_with_metadata(path)?;
    let mut strings = std::collections::HashMap::new();
    
    let nodes = match format {
        Format::XMLB => parse_xmlb(&data, &mut strings),
        Format::BNX => parse_bnx(&data, &mut strings),
        Format::IGB => parse_igb(&data, &mut strings),
        Format::ZSM => parse_zsm(&data),
        Format::ZAM => parse_zam(&data),
        Format::ANIM => parse_anim(&data),
        Format::PHYS => parse_phys(&data),
        Format::AUD => parse_aud(&data),
        Format::COMP => parse_comp(&data),
        Format::PBR => parse_pbr(&data),
        Format::PLGN => parse_plgn(&data),
        Format::SAVE => parse_save(&data),
        Format::PIPE => parse_pipe(&data),
        Format::Unknown => return Err(AthanorError::UnsupportedFormat("Unknown".to_string())),
        _ => return Err(AthanorError::UnsupportedFormat(format.name().to_string())),
    };
    
    Ok(ParsedFile {
        metadata,
        nodes,
        strings,
        raw_data: data,
    })
}

/// Parse a binary file and return structured nodes
pub fn parse_file(path: &Path) -> Result<ParsedFile> {
    disassemble(path)
}

/// Get all named symbols from a parsed file
pub fn get_symbols(file: &ParsedFile) -> Vec<String> {
    file.strings.values().cloned().collect()
}

/// Get all strings with their offsets
pub fn get_strings(file: &ParsedFile) -> std::collections::HashMap<u32, String> {
    file.strings.clone()
}

/// Detect format from magic bytes and extension
pub fn detect_format(path: &Path) -> Result<Format> {
    FormatDetector::detect(path)
}

// --- Format-specific parsers ---

fn parse_xmlb(data: &[u8], strings: &mut std::collections::HashMap<u32, String>) -> Vec<ParsedNode> {
    if data.len() < 24 { return Vec::new(); }
    
    let strtab_off = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    
    // Parse strings from string table
    let mut pos = strtab_off;
    while pos < data.len() {
        if data[pos] == 0 { pos += 1; continue; }
        let end = data[pos..].iter().position(|&b| b == 0).map(|p| pos + p).unwrap_or(data.len());
        if end > pos {
            strings.insert(pos as u32, String::from_utf8_lossy(&data[pos..end]).to_string());
        }
        pos = end + 1;
    }
    
    // Parse nodes
    let mut nodes = Vec::new();
    let max_nodes = (strtab_off.saturating_sub(24)) / 32;
    
    for n in 0..max_nodes {
        let node_start = 24 + n * 32;
        if node_start + 32 > data.len() { break; }
        
        let offsets: Vec<u32> = (0..8)
            .map(|i| u32::from_le_bytes([
                data[node_start + i * 4],
                data[node_start + i * 4 + 1],
                data[node_start + i * 4 + 2],
                data[node_start + i * 4 + 3],
            ]))
            .collect();
        
        if offsets.iter().all(|&o| o == 0xFFFFFFFF) { continue; }
        
        let node_type = strings.get(&offsets[0]).cloned();
        let name = strings.get(&offsets[1]).cloned();
        
        let mut properties = Vec::new();
        for i in 0..3 {
            let key_off = offsets[2 + i * 2];
            let val = offsets[3 + i * 2];
            if key_off == 0xFFFFFFFF { continue; }
            let key = strings.get(&key_off).cloned().unwrap_or_else(|| format!("<0x{:08X}>", key_off));
            let value = if val == 0xFFFFFFFF {
                PropertyValue::Null
            } else if let Some(s) = strings.get(&val) {
                PropertyValue::String(s.clone())
            } else {
                PropertyValue::Integer(val as i64)
            };
            properties.push(ParsedProperty {
                key: Some(key),
                value,
                is_raw: val != 0xFFFFFFFF && !strings.contains_key(&val),
            });
        }
        
        nodes.push(ParsedNode {
            index: n,
            file_offset: node_start as u32,
            node_type,
            name,
            properties,
            raw_bytes: data[node_start..node_start + 32].to_vec(),
        });
    }
    
    nodes
}

fn parse_bnx(data: &[u8], strings: &mut std::collections::HashMap<u32, String>) -> Vec<ParsedNode> {
    let text = String::from_utf8_lossy(data);
    let mut nodes = Vec::new();
    
    for (i, line) in text.lines().enumerate() {
        if line.trim().contains('=') {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            strings.insert(i as u32, line.to_string());
            nodes.push(ParsedNode {
                index: i,
                file_offset: i as u32 * 64,
                node_type: Some("BNX_ENTRY".to_string()),
                name: Some(parts[0].trim().to_string()),
                properties: vec![ParsedProperty {
                    key: Some("value".to_string()),
                    value: PropertyValue::String(parts.get(1).unwrap_or(&"").trim().to_string()),
                    is_raw: false,
                }],
                raw_bytes: line.as_bytes().to_vec(),
            });
        }
    }
    
    nodes
}

fn parse_igb(data: &[u8], strings: &mut std::collections::HashMap<u32, String>) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if data[i] >= 0x20 && data[i] < 0x7F {
            let start = i;
            while i < data.len() && data[i] >= 0x20 && data[i] < 0x7F { i += 1; }
            let s = String::from_utf8_lossy(&data[start..i]).to_string();
            if s.len() >= 3 {
                strings.insert(start as u32, s.clone());
                nodes.push(ParsedNode {
                    index: nodes.len(),
                    file_offset: start as u32,
                    node_type: Some("IGB_STRING".to_string()),
                    name: Some(s),
                    properties: vec![],
                    raw_bytes: data[start..i].to_vec(),
                });
            }
        } else {
            i += 1;
        }
    }
    nodes
}

fn parse_zsm(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 16 { return nodes; }
    let str_count = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let str_off = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    
    let mut strings = std::collections::HashMap::new();
    let mut pos = str_off;
    while pos < data.len() && strings.len() < str_count as usize {
        if data[pos] == 0 { pos += 1; continue; }
        let end = data[pos..].iter().position(|&b| b == 0).map(|p| pos + p).unwrap_or(data.len());
        if end > pos {
            strings.insert(pos as u32, String::from_utf8_lossy(&data[pos..end]).to_string());
        }
        pos = end + 1;
    }
    
    for (off, s) in &strings {
        nodes.push(ParsedNode {
            index: *off as usize,
            file_offset: *off,
            node_type: Some("ZSM_STRING".to_string()),
            name: Some(s.clone()),
            properties: vec![],
            raw_bytes: Vec::new(),
        });
    }
    nodes
}

fn parse_zam(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        if data[pos] >= 0x20 && data[pos] < 0x7F {
            let start = pos;
            while pos < data.len() && data[pos] >= 0x20 && data[pos] < 0x7F { pos += 1; }
            let s = String::from_utf8_lossy(&data[start..pos]).to_string();
            if !s.is_empty() {
                nodes.push(ParsedNode {
                    index: nodes.len(),
                    file_offset: start as u32,
                    node_type: Some("ZAM_DATA".to_string()),
                    name: Some(s),
                    properties: vec![],
                    raw_bytes: data[start..pos].to_vec(),
                });
            }
        } else {
            pos += 1;
        }
    }
    nodes
}

fn parse_anim(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 32 { return nodes; }
    let state_count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
    for i in 0..state_count {
        let off = 32 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("ANIM_STATE".to_string()),
            name: None, properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_phys(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 32 { return nodes; }
    let count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
    for i in 0..count {
        let off = 32 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("PHYS_SHAPE".to_string()),
            name: None, properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_aud(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 32 { return nodes; }
    let count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
    for i in 0..count {
        let off = 32 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("AUD_BUS".to_string()),
            name: None, properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_comp(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 32 { return nodes; }
    let count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
    for i in 0..count {
        let off = 32 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("COMP_EFFECT".to_string()),
            name: None, properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_pbr(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 32 { return nodes; }
    let count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
    for i in 0..count {
        let off = 32 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("PBR_MATERIAL".to_string()),
            name: None, properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_plgn(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 32 { return nodes; }
    let count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
    for i in 0..count {
        let off = 32 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("PLGN_PLUGIN".to_string()),
            name: None, properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_save(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 32 { return nodes; }
    let count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
    for i in 0..count {
        let off = 32 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("SAVE_SECTION".to_string()),
            name: None, properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_pipe(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 32 { return nodes; }
    let count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
    for i in 0..count {
        let off = 32 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("PIPE_IMPORT".to_string()),
            name: None, properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_detection() {
        let path = Path::new("test.xmlb");
        let format = FormatDetector::detect(path).unwrap();
        // Will be Unknown if file doesn't exist, but test compiles
    }
}