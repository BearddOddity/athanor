//! Console disc extraction module

pub mod disc;

pub use disc::{extract_disc, extract_xbox_iso, extract_ps2_iso, extract_gcm_iso, extract_file, DiscExtractResult, DiscType, DiscEntry, DiscMetadata};