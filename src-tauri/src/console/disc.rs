//! Console Disc Extraction
//!
//! Parses Xbox ISO/XISO, PS2 ISO, and GameCube GCM/ISO files.
//! Extracts filesystem metadata and files for the Bring Your Own Game architecture.

use crate::Result;

/// Xbox ISO Header signature
const XISO_MINI_HEADER_SIZE: u64 = 0x800;
const XISO_STD_HEADER_OFFSET: u64 = 0x200;

/// PS2 ISO Primary Volume Descriptor
const PVD_OFFSET: u64 = 0x8001;

/// GameCube ISO magic
const GCM_HEADER_SIZE: u64 = 0xC00;

/// Result of disc extraction
#[derive(Debug, Clone)]
pub struct DiscExtractResult {
    pub disc_type: DiscType,
    pub entries: Vec<DiscEntry>,
    pub metadata: DiscMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscType {
    XBox,
    XBox360,
    Playstation2,
    Gamecube,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DiscEntry {
    pub path: String,
    pub offset: u64,
    pub size: u64,
    pub is_directory: bool,
}

#[derive(Debug, Clone)]
pub struct DiscMetadata {
    pub boot_file: Option<String>,
    pub title: String,
    pub vendor: String,
    pub version: String,
    pub disc_id: String,
}

/// Extract files from an Xbox ISO/XISO file
pub fn extract_xbox_iso(data: &[u8]) -> Result<DiscExtractResult> {
    if data.len() < XISO_MINI_HEADER_SIZE as usize {
        return Err(crate::AthanorError::InvalidData("File too small for Xbox ISO".to_string()));
    }

    let magic = &data[0..4];
    
    // XISO mini format (direct filesystem extraction)
    if magic == b"X\x00\x00" || data.len() >= 0x8000 {
        return parse_xiso_mini(data);
    }
    
    // Standard XISO format with volume headers
    parse_xiso_std(data)
}

fn parse_xiso_mini(data: &[u8]) -> Result<DiscExtractResult> {
    let mut entries = Vec::new();
    
    // XISO mini has a simple table at offset 0x800
    // Each entry: 8 bytes offset, 8 bytes size, 256 bytes path
    
    let table_start = 0x800u64;
    let entry_size = 264u64; // 8 + 8 + 256
    
    if data.len() < table_start as usize {
        return Ok(DiscExtractResult {
            disc_type: DiscType::XBox,
            entries,
            metadata: DiscMetadata {
                boot_file: None,
                title: "Xbox Disc".to_string(),
                vendor: String::new(),
                version: String::new(),
                disc_id: String::new(),
            },
        });
    }
    
    let mut offset = table_start;
    while offset + entry_size <= data.len() as u64 {
        let entry_offset = offset as usize;
        let file_offset = u64::from_le_bytes([
            data[entry_offset], data[entry_offset + 1], data[entry_offset + 2], data[entry_offset + 3],
            data[entry_offset + 4], data[entry_offset + 5], data[entry_offset + 6], data[entry_offset + 7],
        ]);
        
        let file_size = u64::from_le_bytes([
            data[entry_offset + 8], data[entry_offset + 9], data[entry_offset + 10], data[entry_offset + 11],
            data[entry_offset + 12], data[entry_offset + 13], data[entry_offset + 14], data[entry_offset + 15],
        ]);
        
        // Path is next 256 bytes, null-terminated
        let path_start = entry_offset + 16;
        let path_end = path_start + 256;
        if path_end > data.len() { break; }
        
        let path_bytes = &data[path_start..path_end];
        let null_pos = path_bytes.iter().position(|&b| b == 0).unwrap_or(255);
        let path = String::from_utf8_lossy(&path_bytes[..null_pos]).to_string();
        
        if !path.is_empty() && file_size > 0 {
            entries.push(DiscEntry {
                path,
                offset: file_offset,
                size: file_size,
                is_directory: false,
            });
        }
        
        offset += entry_size;
    }
    
    Ok(DiscExtractResult {
        disc_type: DiscType::XBox,
        entries: entries.clone(),
        metadata: DiscMetadata {
            boot_file: entries.first().map(|e| e.path.clone()),
            title: "Xbox Disc".to_string(),
            vendor: String::new(),
            version: String::new(),
            disc_id: String::new(),
        },
    })
}

fn parse_xiso_std(data: &[u8]) -> Result<DiscExtractResult> {
    // Parse ISO 9660 Primary Volume Descriptor
    // Look for standard directory entries after the PVD
    
    let mut entries = Vec::new();
    let mut boot_file = None;
    
    // Search for PK00 directory structure (common in Xbox ISOs)
    if let Some(pk00_pos) = find_file_by_name(data, "PK00") {
        if let Some(pk00_entry) = parse_directory_entry(data, pk00_pos) {
            // PK00 contains files in subdirs
            let subdir_data = &data[pk00_entry.offset as usize..];
            entries.extend(parse_pk00_files(subdir_data, pk00_entry.offset));
        }
    }
    
    // Look for default.xbe
    if let Some(xbe_pos) = find_file_by_name(data, "default.xbe") {
        boot_file = Some("default.xbe".to_string());
        if let Some(entry) = parse_directory_entry(data, xbe_pos) {
            entries.push(DiscEntry {
                path: "default.xbe".to_string(),
                offset: entry.offset,
                size: entry.size,
                is_directory: false,
            });
        }
    }
    
    Ok(DiscExtractResult {
        disc_type: DiscType::XBox,
        entries,
        metadata: DiscMetadata {
            boot_file,
            title: "Xbox Game Disc".to_string(),
            vendor: "Microsoft".to_string(),
            version: "1.0".to_string(),
            disc_id: String::new(),
        },
    })
}

fn find_file_by_name(data: &[u8], name: &str) -> Option<usize> {
    let name_bytes = name.as_bytes();
    for window in data.windows(name.len() + 33) {
        if &window[..name.len()] == name_bytes && window[name.len()] != 0 {
            return Some(window.len() - 33);
        }
    }
    None
}

#[derive(Debug, Clone)]
struct DirectoryEntry {
    offset: u64,
    size: u64,
}

fn parse_directory_entry(data: &[u8], pos: usize) -> Option<DirectoryEntry> {
    if pos + 33 > data.len() { return None; }
    
    let length = data[pos] as usize;
    if length == 0 || pos + length > data.len() { return None; }
    
    let file_size = u32::from_le_bytes([
        data[pos + 28], data[pos + 29], data[pos + 30], data[pos + 31]
    ]) as u64;
    
    let offset = (data[pos + 2] as u64) << 2 | (data[pos + 3] as u64) << 24;
    
    Some(DirectoryEntry { offset, size: file_size })
}

fn parse_pk00_files(data: &[u8], base_offset: u64) -> Vec<DiscEntry> {
    // PK00 is a simple file list format used by Xbox
    let mut entries = Vec::new();
    
    // Look for directory listing pattern
    let mut pos = 0;
    while pos + 33 < data.len() {
        // Check for directory entry pattern
        if data[pos] != 0 && data[pos] <= 255 {
            let level = data[pos];
            let name_start = pos + 1;
            
            // Find null terminator for filename
            let name_end = data[name_start..].iter().position(|&b| b == 0).unwrap_or(32).min(32);
            if name_end == 0 { pos += 1; continue; }
            
            let name = String::from_utf8_lossy(&data[name_start..name_start + name_end]);
            
            if name.starts_with('/') || name.ends_with('/') || name == "." || name == ".." {
                pos += 1;
                continue;
            }
            
            // Calculate file offset (little-endian 32-bit, shifted left 2)
            let file_offset = u64::from_le_bytes([
                data[name_start + name_end], data[name_start + name_end + 1], 
                data[name_start + name_end + 2], data[name_start + name_end + 3], 0, 0, 0, 0
            ]) << 2;
            
            let file_size = u32::from_le_bytes([
                data[name_start + name_end + 4], data[name_start + name_end + 5],
                data[name_start + name_end + 6], data[name_start + name_end + 7]
            ]) as u64;
            
            if file_size > 0 && file_offset > 0 {
                entries.push(DiscEntry {
                    path: name.to_string(),
                    offset: base_offset + file_offset,
                    size: file_size,
                    is_directory: false,
                });
            }
            
            pos += name_end + 8;
        } else {
            pos += 1;
        }
    }
    
    entries
}

/// Extract files from a PS2 ISO (ISO 9660 filesystem)
pub fn extract_ps2_iso(data: &[u8]) -> Result<DiscExtractResult> {
    if data.len() < 0x8000 {
        return Err(crate::AthanorError::InvalidData("File too small for PS2 ISO".to_string()));
    }
    
    // Check for PS2 boot sector signature
    // PS2 ISOs have a special boot sector with executable info
    
    let mut entries = Vec::new();
    
    // Parse Primary Volume Descriptor at 0x8001
    let pvd_offset = PVD_OFFSET;
    if pvd_offset + 0x80 > data.len() as u64 {
        return Ok(DiscExtractResult {
            disc_type: DiscType::Playstation2,
            entries,
            metadata: DiscMetadata {
                boot_file: None,
                title: "PS2 Disc".to_string(),
                vendor: "Sony".to_string(),
                version: String::new(),
                disc_id: String::new(),
            },
        });
    }
    
    // Extract root directory entry from PVD
    let root_dir_offset = u32::from_le_bytes([
        data[pvd_offset as usize + 150], 
        data[pvd_offset as usize + 151],
        data[pvd_offset as usize + 152],
        data[pvd_offset as usize + 153]
    ]);
    
    let root_dir_block = root_dir_offset >> 15; // Convert sector to block
    let root_dir_pos = (root_dir_block as usize) * 2048;
    
    // Parse root directory files
    entries.extend(parse_iso9660_directory(data, root_dir_pos)?);
    
    // Look for main executable (usually SLUS_xx_xx or SLES_xx_xx)
    let boot_file = entries.iter()
        .find(|e| e.path.contains("SLUS") || e.path.contains("SLES") || e.path.ends_with(".elf"))
        .map(|e| e.path.clone());
    
    Ok(DiscExtractResult {
        disc_type: DiscType::Playstation2,
        entries,
        metadata: DiscMetadata {
            boot_file,
            title: "PS2 Game Disc".to_string(),
            vendor: "Sony Computer Entertainment".to_string(),
            version: "1.0".to_string(),
            disc_id: String::new(),
        },
    })
}

fn parse_iso9660_directory(data: &[u8], dir_pos: usize) -> Result<Vec<DiscEntry>> {
    let mut entries = Vec::new();
    let mut pos = dir_pos;
    
    while pos + 34 < data.len() {
        let length = data[pos] as usize;
        if length == 0 {
            pos += 1;
            continue;
        }
        
        if pos + length > data.len() { break; }
        
        let is_directory = (data[pos + 25] & 0x02) != 0;
        let file_size = u32::from_le_bytes([
            data[pos + 28], data[pos + 29], data[pos + 30], data[pos + 31]
        ]) as u64;
        
        let ext_attr_offset = u32::from_le_bytes([
            data[pos + 32], data[pos + 33], 0, 0
        ]);
        
        // Filename starts at pos + 34 + ext_attr_offset
        let filename_start = pos + 34;
        let filename_end = filename_start + 31;
        
        if filename_end > data.len() { break; }
        
        // ISO 9660 filenames are 31 bytes, may include extension
        let filename_bytes = &data[filename_start..filename_end];
        let name = String::from_utf8_lossy(filename_bytes);
        
        // Remove trailing spaces and handle ;1 extension terminator
        let clean_name = name.trim().replace(";", "").trim_end_matches('0').to_string();
        
        if !clean_name.is_empty() && clean_name != "." && clean_name != ".." {
            // Calculate data offset (LBA * 2048)
            let lba = u32::from_le_bytes([
                data[pos + 2], data[pos + 3], 0, 0
            ]);
            let data_offset = (lba as u64) * 2048 + ext_attr_offset as u64;
            
            entries.push(DiscEntry {
                path: clean_name,
                offset: data_offset,
                size: file_size,
                is_directory,
            });
        }
        
        pos += length;
    }
    
    Ok(entries)
}

/// Extract files from a GameCube ISO (GCM format)
pub fn extract_gcm_iso(data: &[u8]) -> Result<DiscExtractResult> {
    if data.len() < GCM_HEADER_SIZE as usize {
        return Err(crate::AthanorError::InvalidData("File too small for GameCube ISO".to_string()));
    }
    
    // GameCube header is 0xC00 bytes
    // Magic: " GameCube" at offset 0x200
    
    let mut entries = Vec::new();
    
    // Check magic
    if &data[0x200..0x207] != b" GameCube" {
        return Ok(DiscExtractResult {
            disc_type: DiscType::Gamecube,
            entries,
            metadata: DiscMetadata {
                boot_file: None,
                title: String::new(),
                vendor: String::new(),
                version: String::new(),
                disc_id: String::new(),
            },
        });
    }
    
    // Game title at offset 0x200
    let title = String::from_utf8_lossy(&data[0x200..0x210]).to_string();
    
    // Developer at offset 0x210
    let developer = String::from_utf8_lossy(&data[0x210..0x220]).to_string();
    let disc_id = String::from_utf8_lossy(&data[0x220..0x230]).to_string();
    
    // DOL executable is in the GCM
    // First 7 DOL sections, then 3 optional padding sections
    let _entries_start = 0x100000u64; // Typical location after DOL header
    
    // Look for .app files (GameCube executable partitions)
    if let Some(app_pos) = find_file_pattern(data, b".app") {
        entries.push(DiscEntry {
            path: "main.app".to_string(),
            offset: app_pos as u64,
            size: 0, // Would need to calculate from directory
            is_directory: false,
        });
    }
    
    // Look for .rel files (modules)
    let mut pos = 0;
    while pos + 8 < data.len() {
        if &data[pos..pos + 4] == b".rel" {
            // Found a module reference
            let module_offset = u32::from_le_bytes([
                data[pos + 1], data[pos + 2], data[pos + 3], 0
            ]);
            if module_offset > 0 {
                entries.push(DiscEntry {
                    path: format!("module_{}.rel", pos / 8),
                    offset: module_offset as u64,
                    size: 0,
                    is_directory: false,
                });
            }
        }
        pos += 1;
    }
    
    Ok(DiscExtractResult {
        disc_type: DiscType::Gamecube,
        entries,
        metadata: DiscMetadata {
            boot_file: Some("DOL".to_string()),
            title: title.trim().to_string(),
            vendor: developer.trim().to_string(),
            version: String::new(),
            disc_id: disc_id.trim().to_string(),
        },
    })
}

fn find_file_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len()).position(|w| w == pattern)
}

/// Detect disc type and extract files
pub fn extract_disc(data: &[u8]) -> Result<DiscExtractResult> {
    // Check magic bytes to determine disc type
    
    // Xbox/XISO check
    if data.len() > 4 {
        let magic = &data[0..4];
        if &magic[0..4] == b"XXXXXX" || &magic[..4] == b"X\x00\x00" {
            return extract_xbox_iso(data);
        }
    }
    
    // Check for PS2 magic at offset 32768 (0x8000)
    if data.len() > 0x8000 {
        if &data[0x8000..0x8004] == b"CD001" {
            return extract_ps2_iso(data);
        }
    }
    
    // Check for GameCube magic
    if data.len() > 0x207 {
        if &data[0x200..0x207] == b" GameCube" {
            return extract_gcm_iso(data);
        }
    }
    
    Err(crate::AthanorError::InvalidData("Unknown disc format".to_string()))
}

/// Extract single file from disc to local path
pub fn extract_file(data: &[u8], entry: &DiscEntry, output_path: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    
    if entry.is_directory {
        std::fs::create_dir_all(output_path)?;
        return Ok(());
    }
    
    let end = (entry.offset + entry.size) as usize;
    if end > data.len() {
        return Err(crate::AthanorError::InvalidData(format!(
            "File extends beyond disc data: {} + {} > {}",
            entry.offset, entry.size, data.len()
        )));
    }
    
    let file_data = &data[entry.offset as usize..end];
    
    // Create parent directories
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let mut file = File::create(output_path)?;
    file.write_all(file_data)?;
    
    Ok(())
}