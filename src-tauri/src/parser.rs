//! Assembly and disassembly support

use serde::{Deserialize, Serialize};
use crate::{
    AthanorError, ParsedFile, ParsedNode, ParsedProperty, PropertyValue,
    Result,
};
use crate::dxt::{decompress, DxtFormat};
use crate::formats::{Format, FormatDetector};
use std::path::Path;
use std::collections::HashMap;

/// IGB Texture metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgbTexture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub format: String, // DXT1 or DXT5
    pub data_offset: u32,
    pub data_size: u32,
    pub raw_data: Vec<u8>,
}

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
        Format::ZSS => parse_zss(&data),
        Format::ZAM => parse_zam(&data),
        Format::ANIM => parse_anim(&data),
        Format::PHYS => parse_phys(&data),
        Format::AUD => parse_aud(&data),
        Format::COMP => parse_comp(&data),
        Format::PBR => parse_pbr(&data),
        Format::PLGN => parse_plgn(&data),
        Format::SAVE => parse_save(&data),
        Format::PIPE => parse_pipe(&data),
        Format::CHRB => parse_chrb(&data),
        Format::NAVB => parse_navb(&data),
        Format::PKGB => parse_pkgb(&data),
        Format::ENGB => parse_engb(&data),
        Format::BOYB => parse_boyb(&data, &mut strings),
        Format::Unknown => return Err(AthanorError::UnsupportedFormat("Unknown".to_string())),
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
    
    // Extract strings as before (for texture names, etc)
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
    
    // Parse IGB texture header if present
    if data.len() >= 16 && data[0] == 0xA4 && data[1] == 0x02 {
        let texture_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let width = u16::from_le_bytes([data[8], data[9]]) as u32;
        let height = u16::from_le_bytes([data[10], data[11]]) as u32;
        let format_tag = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        
        // This would be for extracting actual texture data
        // For now, just add a metadata node
        nodes.push(ParsedNode {
            index: nodes.len(),
            file_offset: 0,
            node_type: Some("IGB_HEADER".to_string()),
            name: Some("IGB Texture Container".to_string()),
            properties: vec![
                ParsedProperty {
                    key: Some("width".to_string()),
                    value: PropertyValue::Integer(width as i64),
                    is_raw: true,
                },
                ParsedProperty {
                    key: Some("height".to_string()),
                    value: PropertyValue::Integer(height as i64),
                    is_raw: true,
                },
                ParsedProperty {
                    key: Some("texture_count".to_string()),
                    value: PropertyValue::Integer(texture_count as i64),
                    is_raw: true,
                },
                ParsedProperty {
                    key: Some("format".to_string()),
                    value: PropertyValue::String(format_tag.to_string()),
                    is_raw: false,
                },
            ],
            raw_bytes: data[0..16.min(data.len())].to_vec(),
        });
    }
    
    nodes
}

/// Extract IGB textures and convert to PNG

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

fn parse_zss(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 24 { return nodes; }
    let str_count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
    let str_off = u32::from_le_bytes([data[16], data[17], data[18], data[19]]) as usize;

    let mut pos = str_off;
    for _ in 0..str_count {
        if pos >= data.len() { break; }
        if data[pos] == 0 { pos += 1; continue; }
        let end = data[pos..].iter().position(|&b| b == 0).map(|p| pos + p).unwrap_or(data.len());
        if end > pos {
            let s = String::from_utf8_lossy(&data[pos..end]).to_string();
            nodes.push(ParsedNode {
                index: nodes.len(),
                file_offset: pos as u32,
                node_type: Some("ZSS_NAME".to_string()),
                name: Some(s),
                properties: vec![],
                raw_bytes: data[pos..end].to_vec(),
            });
        }
        pos = end + 1;
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

fn parse_chrb(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 24 { return nodes; }
    let count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
    for i in 0..count {
        let off = 24 + i * 64;
        if off + 64 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("CHRB_CHARACTER".to_string()),
            name: None,
            properties: vec![],
            raw_bytes: data[off..off + 64].to_vec(),
        });
    }
    nodes
}

fn parse_navb(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 24 { return nodes; }
    let count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
    for i in 0..count {
        let off = 24 + i * 48;
        if off + 48 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("NAVB_NODE".to_string()),
            name: None,
            properties: vec![],
            raw_bytes: data[off..off + 48].to_vec(),
        });
    }
    nodes
}

fn parse_pkgb(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 24 { return nodes; }
    let count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
    for i in 0..count {
        let off = 24 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("PKGGB_ENTRY".to_string()),
            name: None,
            properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_engb(data: &[u8]) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    if data.len() < 24 { return nodes; }
    let count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
    for i in 0..count {
        let off = 24 + i * 32;
        if off + 32 > data.len() { break; }
        nodes.push(ParsedNode {
            index: i, file_offset: off as u32,
            node_type: Some("ENGB_CONFIG".to_string()),
            name: None,
            properties: vec![],
            raw_bytes: data[off..off + 32].to_vec(),
        });
    }
    nodes
}

fn parse_boyb(data: &[u8], strings: &mut std::collections::HashMap<u32, String>) -> Vec<ParsedNode> {
    if data.len() < 24 { return Vec::new(); }
    
    let str_count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
    let str_off = u32::from_le_bytes([data[16], data[17], data[18], data[19]]) as usize;
    
    let mut nodes = Vec::new();
    let mut pos = str_off;
    
    for _ in 0..str_count {
        if pos >= data.len() { break; }
        if data[pos] == 0 { pos += 1; continue; }
        let end = data[pos..].iter().position(|&b| b == 0).map(|p| pos + p).unwrap_or(data.len());
        if end > pos {
            strings.insert(pos as u32, String::from_utf8_lossy(&data[pos..end]).to_string());
        }
        pos = end + 1;
    }
    
    let max_nodes = str_off.saturating_sub(24) / 32;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::BinaryBuilder;
    
    #[test]
    fn test_format_detection() {
        let path = Path::new("test.xmlb");
        let _format = FormatDetector::detect(path).unwrap();
    }
    
    #[test]
    fn test_xmlb_builder_round_trip() {
        let mut builder = crate::builder::XMLBBuilder::new();
        let mut strings = std::collections::HashMap::new();
        strings.insert(24u32, "type_test".to_string());
        strings.insert(40u32, "name_test".to_string());
        
        let node = crate::ParsedNode {
            index: 0,
            file_offset: 24,
            node_type: Some("type_test".to_string()),
            name: Some("name_test".to_string()),
            properties: vec![],
            raw_bytes: vec![0u8; 32],
        };
        
        builder.nodes = vec![node];
        builder.strings = strings;
        
        let output = builder.compile(&[]).unwrap();
        assert!(output.len() > 56);
        assert_eq!(&output[0..4], &0x000011B1u32.to_le_bytes());
    }
    
    #[test]
    fn test_parse_xl2_bnx() {
        let path = std::path::Path::new("D:/My Games/X-Men Legends II Rise of Apocalypse/igct.bnx");
        if path.exists() {
            let result = parse_file(path);
            assert!(result.is_ok(), "Failed to parse igct.bnx: {:?}", result.err());
            let parsed = result.unwrap();
            assert!(!parsed.nodes.is_empty(), "Should have parsed nodes from igct.bnx");
            println!("Parsed {} nodes from igct.bnx", parsed.nodes.len());
        }
    }
    
    #[test]
    fn test_parse_xl2_bnx_round_trip() {
        use crate::builder::BinaryBuilder;
        let path = std::path::Path::new("D:/My Games/X-Men Legends II Rise of Apocalypse/igct.bnx");
        if path.exists() {
            let parsed = parse_file(path).unwrap();
            let mut builder = crate::builder::BNXBuilder::new();
            builder.pairs = parsed.nodes.iter().map(|n| {
                (n.name.clone().unwrap_or_default(),
                 n.properties.first().and_then(|p| match &p.value {
                     crate::PropertyValue::String(s) => Some(s.clone()),
                     _ => None,
                 }).unwrap_or_default())
            }).collect();
            let output = builder.compile(&parsed.raw_data).unwrap();
            assert!(!output.is_empty(), "Should generate output");
        }
    }
    
    #[test]
    fn test_parse_multiple_xl2_formats() {
        let game_dir = "D:/My Games/X-Men Legends II Rise of Apocalypse";
        let bnx_files = vec!["igct.bnx", "igctfre.bnx", "igctger.bnx"];
        for file in bnx_files {
            let full_path = format!("{}/{}", game_dir, file);
            let path = std::path::Path::new(&full_path);
            if path.exists() {
                let result = parse_file(path);
                if result.is_ok() {
                    let parsed = result.unwrap();
                    println!("{}: {} nodes", file, parsed.nodes.len());
                } else {
                    println!("{}: parse error {:?}", file, result.err());
                }
            }
        }
    }
}