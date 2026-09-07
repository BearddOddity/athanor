//! Console extraction module

pub mod disc;
pub mod executable;

pub use disc::{extract_disc, extract_xbox_iso, extract_ps2_iso, extract_gcm_iso, extract_file, DiscExtractResult, DiscType, DiscEntry, DiscMetadata};
pub use executable::{parse_executable, parse_xbe, parse_elf, parse_dol, parse_pe, ExecutableInfo, ExeType, Section, Import, Export, ExecutableMetadata};