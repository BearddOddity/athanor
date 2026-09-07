//! Executable Format Parsers
//!
//! Parses Xbox XBE, PS2 ELF, and GameCube DOL executables.
//! Extracts import/export tables, sections, and entry points.

use crate::Result;

/// Xbox XBE header signature
const XBE_MAGIC: &[u8; 4] = b"XBEH";

/// PS2 ELF magic
const ELF_MAGIC: &[u8; 4] = b"\x7fELF";

/// GameCube DOL magic (actually no magic, check sections)

/// Result of executable parsing
#[derive(Debug, Clone)]
pub struct ExecutableInfo {
    pub exe_type: ExeType,
    pub entry_point: u64,
    pub sections: Vec<Section>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub metadata: ExecutableMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExeType {
    XboxXBE,
    PS2ELF,
    GameCubeDOL,
    WindowsPE,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub raw_address: u64,
    pub raw_size: u64,
    pub characteristics: u32,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub library: String,
    pub function: String,
    pub hint: u16,
    pub is_ordinal: bool,
}

#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub address: u64,
    pub ordinal: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ExecutableMetadata {
    pub timestamp: u64,
    pub version: String,
    pub subsystem: String,
    pub dll_characteristics: String,
}

/// Parse Xbox XBE executable
pub fn parse_xbe(data: &[u8]) -> Result<ExecutableInfo> {
    if data.len() < 0x100 {
        return Err(crate::AthanorError::InvalidData("XBE file too small".to_string()));
    }
    
    // Check XBE header magic
    if &data[0..4] != XBE_MAGIC {
        return Err(crate::AthanorError::InvalidData("Invalid XBE magic".to_string()));
    }
    
    // Parse XBE header (simplified)
    // XBE header is 0x100 bytes
    
    let mut sections = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    
    // Certificate table offset (at 0x48 in header)
    let cert_offset = u32::from_le_bytes([
        data[0x48], data[0x49], data[0x4a], data[0x4b]
    ]) as u64;
    
    // Section table offset (at 0x68 in header)
    let section_offset = u32::from_le_bytes([
        data[0x68], data[0x69], data[0x6a], data[0x6b]
    ]) as u64;
    
    // Number of sections (at 0x8c)
    let num_sections = u16::from_le_bytes([
        data[0x8c], data[0x8d]
    ]) as usize;
    
    // Parse sections
    for i in 0..num_sections {
        let sec_offset = section_offset + (i * 0x20) as u64;
        if sec_offset + 0x20 > data.len() as u64 { break; }
        
        let sec_start = sec_offset as usize;
        let sec_data = &data[sec_start..sec_start + 0x20];
        
        let name_start = sec_start;
        let name_end = name_start + 16;
        if name_end > data.len() { break; }
        
        let name = String::from_utf8_lossy(&data[name_start..name_end]);
        let name = name.trim_matches(char::from(0)).to_string();
        
        let virt_addr = u64::from_le_bytes([
            data[sec_start + 0x10], data[sec_start + 0x11], 
            data[sec_start + 0x12], data[sec_start + 0x13],
            data[sec_start + 0x14], data[sec_start + 0x15], 
            data[sec_start + 0x16], data[sec_start + 0x17]
        ]);
        
        let virt_size = u64::from_le_bytes([
            data[sec_start + 0x18], data[sec_start + 0x19], 
            data[sec_start + 0x1a], data[sec_start + 0x1b],
            data[sec_start + 0x1c], data[sec_start + 0x1d], 
            data[sec_start + 0x1e], data[sec_start + 0x1f]
        ]);
        
        sections.push(Section {
            name: if name.is_empty() { format!(".text{}", i) } else { name },
            virtual_address: virt_addr,
            virtual_size: virt_size,
            raw_address: 0, // Would need to parse fully
            raw_size: 0,
            characteristics: 0,
        });
    }
    
    // Entry point (at 0x140 in XBE header)
    let entry_point = u64::from_le_bytes([
        data[0x140], data[0x141], data[0x142], data[0x143],
        data[0x144], data[0x145], data[0x146], data[0x147]
    ]);
    
    // Timestamp (simplified)
    let timestamp = u64::from_le_bytes([
        data[0x60], data[0x61], data[0x62], data[0x63],
        data[0x64], data[0x65], data[0x66], data[0x67]
    ]);
    
    Ok(ExecutableInfo {
        exe_type: ExeType::XboxXBE,
        entry_point,
        sections,
        imports,
        exports,
        metadata: ExecutableMetadata {
            timestamp,
            version: "Xbox XBE".to_string(),
            subsystem: "Xbox Game".to_string(),
            dll_characteristics: String::new(),
        },
    })
}

/// Parse PS2 ELF executable
pub fn parse_elf(data: &[u8]) -> Result<ExecutableInfo> {
    if data.len() < 0x34 {
        return Err(crate::AthanorError::InvalidData("ELF file too small".to_string()));
    }
    
    // Check ELF magic
    if &data[0..4] != ELF_MAGIC {
        return Err(crate::AthanorError::InvalidData("Invalid ELF magic".to_string()));
    }
    
    // 32-bit or 64-bit?
    let is_64bit = data[0x04] == 2;
    
    let entry_point;
    let mut sections = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    
    if is_64bit {
        // ELF64
        if data.len() < 0x40 {
            return Err(crate::AthanorError::InvalidData("ELF64 header incomplete".to_string()));
        }
        entry_point = u64::from_le_bytes([
            data[0x18], data[0x19], data[0x1a], data[0x1b],
            data[0x1c], data[0x1d], data[0x1e], data[0x1f]
        ]);
        
        // Section header table offset
        let sht_offset = u64::from_le_bytes([
            data[0x28], data[0x29], data[0x2a], data[0x2b],
            data[0x2c], data[0x2d], data[0x2e], data[0x2f]
        ]);
        
        // Section header entry size and count
        let shentsize = u16::from_le_bytes([
            data[0x3a], data[0x3b]
        ]) as usize;
        let shnum = u16::from_le_bytes([
            data[0x3c], data[0x3d]
        ]) as usize;
        
        // Parse sections
        for i in 0..shnum {
            let sec_offset = sht_offset + (i * shentsize) as u64;
            if sec_offset + shentsize as u64 > data.len() as u64 { break; }
            
            let sec_start = sec_offset as usize;
            let sec_data = &data[sec_start..sec_start + shentsize];
            
            // Section name offset (in .shstrtab)
            let name_offset = u32::from_le_bytes([
                data[sec_start], data[sec_start + 1], 
                data[sec_start + 2], data[sec_start + 3]
            ]) as usize;
            
            // For now, just note we found a section
            sections.push(Section {
                name: format!(".section{}", i),
                virtual_address: 0,
                virtual_size: 0,
                raw_address: 0,
                raw_size: 0,
                characteristics: 0,
            });
        }
    } else {
        // ELF32
        if data.len() < 0x34 {
            return Err(crate::AthanorError::InvalidData("ELF32 header incomplete".to_string()));
        }
        entry_point = u32::from_le_bytes([
            data[0x18], data[0x19], data[0x1a], data[0x1b]
        ]) as u64;
        
        // Section header table offset
        let sht_offset = u32::from_le_bytes([
            data[0x20], data[0x21], data[0x22], data[0x23]
        ]) as u64;
        
        // Section header entry size and count
        let shentsize = u16::from_le_bytes([
            data[0x2e], data[0x2f]
        ]) as usize;
        let shnum = u16::from_le_bytes([
            data[0x30], data[0x31]
        ]) as usize;
        
        // Parse sections (simplified)
        for i in 0..shnum.min(10) { // Limit to avoid OOM
            let sec_offset = sht_offset + (i * shentsize) as u64;
            if sec_offset + shentsize as u64 > data.len() as u64 { break; }
            
            sections.push(Section {
                name: format!(".section{}", i),
                virtual_address: 0,
                virtual_size: 0,
                raw_address: 0,
                raw_size: 0,
                characteristics: 0,
            });
        }
    }
    
    // Note: Full ELF parsing would parse symbol tables for imports/exports
    // This is a simplified placeholder
    
    Ok(ExecutableInfo {
        exe_type: ExeType::PS2ELF,
        entry_point,
        sections,
        imports,
        exports,
        metadata: ExecutableMetadata {
            timestamp: 0,
            version: if is_64bit { "ELF64" } else { "ELF32" }.to_string(),
            subsystem: "PS2 Game".to_string(),
            dll_characteristics: String::new(),
        },
    })
}

/// Parse GameCube DOL executable
pub fn parse_dol(data: &[u8]) -> Result<ExecutableInfo> {
    if data.len() < 0x100 {
        return Err(crate::AthanorError::InvalidData("DOL file too small".to_string()));
    }
    
    // DOL has no magic header - identified by structure
    // First 0x100 bytes: 7 sections * (offset + size) + padding
    
    let mut sections = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    
    // Parse 7 sections (each has offset and size)
    for i in 0..7 {
        let offset = u32::from_le_bytes([
            data[i * 4], data[i * 4 + 1], 
            data[i * 4 + 2], data[i * 4 + 3]
        ]) as u64;
        
        let size = u32::from_le_bytes([
            data[0x40 + i * 4], data[0x40 + i * 4 + 1], 
            data[0x40 + i * 4 + 2], data[0x40 + i * 4 + 3]
        ]) as u64;
        
        if size > 0 {
            sections.push(Section {
                name: match i {
                    0 => ".text".to_string(),
                    1 => ".rodata".to_string(),
                    2 => ".data".to_string(),
                    3 => ".bss".to_string(),
                    _ => format!(".section{}", i),
                },
                virtual_address: offset,
                virtual_size: size,
                raw_address: 0x100 + (i * 0x20) as u64, // Approximate
                raw_size: size,
                characteristics: 0,
            });
        }
    }
    
    // Entry point is first instruction of .text section (usually)
    let entry_point = if sections.len() > 0 { sections[0].virtual_address } else { 0 };
    
    // Look for import table (simplified - would be in .init section)
    // GameCube DOL imports are resolved via OS links
    
    Ok(ExecutableInfo {
        exe_type: ExeType::GameCubeDOL,
        entry_point,
        sections,
        imports,
        exports,
        metadata: ExecutableMetadata {
            timestamp: 0,
            version: "GameCube DOL".to_string(),
            subsystem: "GameCube Game".to_string(),
            dll_characteristics: String::new(),
        },
    })
}

/// Parse Windows PE executable (for reference)
pub fn parse_pe(data: &[u8]) -> Result<ExecutableInfo> {
    if data.len() < 0x40 {
        return Err(crate::AthanorError::InvalidData("PE file too small".to_string()));
    }
    
    // Check PE signature
    if &data[0..2] != b"MZ" {
        return Err(crate::AthanorError::InvalidData("Not a PE file (no MZ header)".to_string()));
    }
    
    // Find PE header offset
    let pe_offset = u32::from_le_bytes([
        data[0x3c], data[0x3c + 1], data[0x3c + 2], data[0x3c + 3]
    ]) as usize;
    
    if pe_offset + 0x20 > data.len() {
        return Err(crate::AthanorError::InvalidData("PE header incomplete".to_string()));
    }
    
    // Check PE signature
    if &data[pe_offset..pe_offset + 4] != b"PE\x00\x00" {
        return Err(crate::AthanorError::InvalidData("Invalid PE signature".to_string()));
    }
    
    // Parse COFF header
    let machine = u16::from_le_bytes([
        data[pe_offset + 4], data[pe_offset + 5]
    ]);
    let num_sections = u16::from_le_bytes([
        data[pe_offset + 6], data[pe_offset + 7]
    ]) as usize;
    let timestamp = u32::from_le_bytes([
        data[pe_offset + 8], data[pe_offset + 9], 
        data[pe_offset + 10], data[pe_offset + 11]
    ]) as u64;
    
    // Optional header size
    let opt_header_size = u16::from_le_bytes([
        data[pe_offset + 16], data[pe_offset + 17]
    ]) as usize;
    
    // Parse sections
    let mut sections = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    
    let section_table = pe_offset + 0x18 + opt_header_size as usize;
    for i in 0..num_sections {
        let sec_offset = section_table + (i * 0x28);
        if sec_offset + 0x28 > data.len() { break; }
        
        let sec_start = sec_offset;
        let sec_data = &data[sec_start..sec_start + 0x28];
        
        // Section name (8 bytes)
        let name = String::from_utf8_lossy(&data[sec_start..sec_start + 8]);
        let name = name.trim_matches(char::from(0)).to_string();
        
        let virt_addr = u32::from_le_bytes([
            data[sec_start + 0x0c], data[sec_start + 0x0d], 
            data[sec_start + 0x0e], data[sec_start + 0x0f]
        ]) as u64;
        
        let virt_size = u32::from_le_bytes([
            data[sec_start + 0x10], data[sec_start + 0x11], 
            data[sec_start + 0x12], data[sec_start + 0x13]
        ]) as u64;
        
        let raw_addr = u32::from_le_bytes([
            data[sec_start + 0x14], data[sec_start + 0x15], 
            data[sec_start + 0x16], data[sec_start + 0x17]
        ]) as u64;
        
        let raw_size = u32::from_le_bytes([
            data[sec_start + 0x18], data[sec_start + 0x19], 
            data[sec_start + 0x1a], data[sec_start + 0x1b]
        ]) as u64;
        
        let characteristics = u32::from_le_bytes([
            data[sec_start + 0x1c], data[sec_start + 0x1d], 
            data[sec_start + 0x1e], data[sec_start + 0x1f]
        ]);
        
        sections.push(Section {
            name: if name.is_empty() { format!(".section{}", i) } else { name },
            virtual_address: virt_addr,
            virtual_size: virt_size,
            raw_address: raw_addr,
            raw_size: raw_size,
            characteristics,
        });
        
        // Note: Import/export parsing would require parsing the data directory
        // This is simplified
    }
    
    // Entry point from optional header
    let entry_point = u32::from_le_bytes([
        data[pe_offset + 0x18 + 0x10], data[pe_offset + 0x18 + 0x11], 
        data[pe_offset + 0x18 + 0x12], data[pe_offset + 0x18 + 0x13]
    ]) as u64;
    
    Ok(ExecutableInfo {
        exe_type: ExeType::WindowsPE,
        entry_point,
        sections,
        imports,
        exports,
        metadata: ExecutableMetadata {
            timestamp,
            version: "Windows PE".to_string(),
            subsystem: "Windows Executable".to_string(),
            dll_characteristics: String::new(),
        },
    })
}

/// Detect executable type and parse
pub fn parse_executable(data: &[u8]) -> Result<ExecutableInfo> {
    // Check magic bytes
    
    // XBE check
    if data.len() >= 4 && &data[0..4] == XBE_MAGIC {
        return parse_xbe(data);
    }
    
    // ELF check
    if data.len() >= 4 && &data[0..4] == ELF_MAGIC {
        return parse_elf(data);
    }
    
    // PE check (MZ header)
    if data.len() >= 2 && &data[0..2] == b"MZ" {
        // Could be XBE or PE - check further
        if data.len() >= 0x40 {
            let pe_offset = u32::from_le_bytes([
                data[0x3c], data[0x3c + 1], data[0x3c + 2], data[0x3c + 3]
            ]) as usize;
            if pe_offset + 4 < data.len() {
                if &data[pe_offset..pe_offset + 4] == b"PE\x00\x00" {
                    return parse_pe(data);
                }
            }
        }
    }
    
    // DOL doesn't have magic - try as last resort if reasonable size
    if data.len() >= 0x100 && data.len() < 0x20_0000 { // Reasonable DOL size
        // Additional heuristics: DOL often has section table at start
        let mut plausible = true;
        for i in 0..7 {
            let offset = u32::from_le_bytes([
                data[i * 4], data[i * 4 + 1], 
                data[i * 4 + 2], data[i * 4 + 3]
            ]) as usize;
            let size = u32::from_le_bytes([
                data[0x40 + i * 4], data[0x40 + i * 4 + 1], 
                data[0x40 + i * 4 + 2], data[0x40 + i * 4 + 3]
            ]) as usize;
            
            if offset > data.len() || size > data.len() {
                plausible = false;
                break;
            }
        }
        
        if plausible {
            return parse_dol(data);
        }
    }
    
    Err(crate::AthanorError::InvalidData("Unknown executable format".to_string()))
}