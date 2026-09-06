//! Format detection and registration

use crate::{FileMetadata, Result};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    XMLB,
    PKGB,
    ENGB,
    CHRB,
    NAVB,
    BOYB,
    BNX,
    IGB,
    ZSM,
    ZSS,
    ZAM,
    ANIM,
    PHYS,
    AUD,
    COMP,
    PBR,
    PLGN,
    SAVE,
    PIPE,
    Unknown,
}

impl Format {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "xmlb" | "pkgb" | "engb" | "chrb" | "navb" | "boyb" => Format::XMLB,
            "bnx" => Format::BNX,
            "igb" => Format::IGB,
            "zsm" => Format::ZSM,
            "zss" => Format::ZSS,
            "zam" => Format::ZAM,
            "anim" | "anm" => Format::ANIM,
            "phys" => Format::PHYS,
            "bus" | "aud" => Format::AUD,
            "comp" => Format::COMP,
            "pbr" => Format::PBR,
            "plgn" => Format::PLGN,
            "save" => Format::SAVE,
            "pipe" => Format::PIPE,
            _ => Format::Unknown,
        }
    }

    pub fn from_magic(magic: u32) -> Self {
        match magic {
            0x000011B1 => Format::XMLB,
            0x0000414E => Format::ANIM,
            0x00005048 => Format::PHYS,
            0x00004155 => Format::AUD,
            0x0000434D => Format::COMP,
            0x00005042 => Format::PBR,
            0x00004C47 => Format::PLGN,
            0x00005347 => Format::SAVE,
            0x00005049 => Format::PIPE,
            _ => Format::Unknown,
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Format::XMLB => &["xmlb", "pkgb", "engb", "chrb", "navb", "boyb"],
            Format::PKGB => &["pkgb"],
            Format::ENGB => &["engb"],
            Format::CHRB => &["chrb"],
            Format::NAVB => &["navb"],
            Format::BOYB => &["boyb"],
            Format::BNX => &["bnx"],
            Format::IGB => &["igb"],
            Format::ZSM => &["zsm"],
            Format::ZSS => &["zss"],
            Format::ZAM => &["zam"],
            Format::ANIM => &["anim", "anm"],
            Format::PHYS => &["phys"],
            Format::AUD => &["bus", "aud"],
            Format::COMP => &["comp"],
            Format::PBR => &["pbr"],
            Format::PLGN => &["plgn"],
            Format::SAVE => &["save"],
            Format::PIPE => &["pipe"],
            Format::Unknown => &[],
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Format::XMLB => "XMLB",
            Format::PKGB => "PKGB",
            Format::ENGB => "ENGB",
            Format::CHRB => "CHRB",
            Format::NAVB => "NAVB",
            Format::BOYB => "BOYB",
            Format::BNX => "BNX",
            Format::IGB => "IGB",
            Format::ZSM => "ZSM",
            Format::ZSS => "ZSS",
            Format::ZAM => "ZAM",
            Format::ANIM => "ANIM",
            Format::PHYS => "PHYS",
            Format::AUD => "AUD",
            Format::COMP => "COMP",
            Format::PBR => "PBR",
            Format::PLGN => "PLGN",
            Format::SAVE => "SAVE",
            Format::PIPE => "PIPE",
            Format::Unknown => "UNKNOWN",
        }
    }
}

pub struct FormatDetector;

impl FormatDetector {
    pub fn detect(path: &Path) -> Result<Format> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let format = Format::from_extension(ext);
        if format != Format::Unknown {
            return Ok(format);
        }

        // Check magic bytes
        let data = std::fs::read(path)?;
        if data.len() >= 4 {
            let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            return Ok(Format::from_magic(magic));
        }

        // Check for BNX (text-based key=value)
        if let Ok(text) = std::str::from_utf8(&data[..data.len().min(200)]) {
            if text.contains('=') && text.lines().any(|l| l.trim().contains('=')) {
                return Ok(Format::BNX);
            }
        }

        // Check IGB signature
        if data.len() >= 2 && data[0] == 0xA4 && data[1] == 0x02 {
            return Ok(Format::IGB);
        }

        Ok(Format::Unknown)
    }

    pub fn detect_with_metadata(path: &Path) -> Result<FileMetadata> {
        let format = Self::detect(path)?;
        let metadata = std::fs::metadata(path)?;

        let mut file_metadata = FileMetadata {
            path: path.to_path_buf(),
            format: format.name().to_string(),
            size: metadata.len(),
            magic: None,
            version: None,
            node_count: None,
            string_count: None,
        };

        // Read magic if possible
        if let Ok(data) = std::fs::read(path) {
            if data.len() >= 4 {
                file_metadata.magic = Some(u32::from_le_bytes([data[0], data[1], data[2], data[3]]));
            }
        }

        Ok(file_metadata)
    }
}