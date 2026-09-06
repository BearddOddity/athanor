//! Anchorpoint.app Integration
//! 
//! Integrates Athanor with Anchorpoint for Git-based version control
//! of binary game assets with file locking and asset management.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use crate::{AthanorError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorpointConfig {
    pub repo_path: std::path::PathBuf,
    pub anchorpoint_cli: String,
    pub api_endpoint: String,
    pub project_name: String,
}

impl Default for AnchorpointConfig {
    fn default() -> Self {
        Self {
            repo_path: std::path::PathBuf::from("./athanor-repo"),
            anchorpoint_cli: "anchorpoint".to_string(),
            api_endpoint: "http://127.0.0.1:9090".to_string(),
            project_name: "Athanor".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub path: std::path::PathBuf,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub locked_by: Option<String>,
    pub review_status: ReviewStatus,
    pub asset_type: AssetType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    InReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Xmlb,
    Bnx,
    Igb,
    Zsm,
    Zam,
    Anim,
    Phys,
    Audio,
    Texture,
    Model,
    Unknown,
}

pub struct AnchorpointClient {
    config: AnchorpointConfig,
}

impl AnchorpointClient {
    pub fn new(config: AnchorpointConfig) -> Self {
        Self { config }
    }
    
    /// Initialize Anchorpoint repository
    pub fn init_repo(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("init")
            .arg(&self.config.repo_path)
            .output()
            .map_err(|e| AthanorError::Parse(format!("Git init failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AthanorError::Parse("Failed to init repo".to_string()));
        }
        
        // Setup .gitattributes for LFS
        let gitattributes = r#"
# Anchorpoint Git LFS configuration
*.xmlb filter=lfs diff=lfs merge=lfs -text
*.bnx filter=lfs diff=lfs merge=lfs -text
*.igb filter=lfs diff=lfs merge=lfs -text
*.zsm filter=lfs diff=lfs merge=lfs -text
*.zss filter=lfs diff=lfs merge=lfs -text
*.zam filter=lfs diff=lfs merge=lfs -text
*.anim filter=lfs diff=lfs merge=lfs -text
*.phys filter=lfs diff=lfs merge=lfs -text
*.aud filter=lfs diff=lfs merge=lfs -text
*.comp filter=lfs diff=lfs merge=lfs -text
*.pbr filter=lfs diff=lfs merge=lfs -text
*.plgn filter=lfs diff=lfs merge=lfs -text
*.save filter=lfs diff=lfs merge=lfs -text
*.pipe filter=lfs diff=lfs merge=lfs -text
*.blend filter=lfs diff=lfs merge=lfs -text
*.fbx filter=lfs diff=lfs merge=lfs -text
*.png filter=lfs diff=lfs merge=lfs -text
*.dds filter=lfs diff=lfs merge=lfs -text
"#;
        
        std::fs::write(
            self.config.repo_path.join(".gitattributes"),
            gitattributes,
        )?;
        
        Ok(())
    }
    
    /// Add file to Anchorpoint with metadata
    pub fn add_asset(
        &self,
        file_path: &Path,
        metadata: AssetMetadata,
    ) -> Result<()> {
        // Stage file
        let output = Command::new("git")
            .arg("add")
            .arg(file_path)
            .current_dir(&self.config.repo_path)
            .output()
            .map_err(|e| AthanorError::Parse(format!("Git add failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AthanorError::Parse("Failed to stage file".to_string()));
        }
        
        // Write metadata sidecar file
        let meta_path = file_path.with_extension(
            format!("{}.meta.json", 
                file_path.extension().and_then(|e| e.to_str()).unwrap_or(""))
        );
        let meta_json = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(meta_path, meta_json)?;
        
        Ok(())
    }
    
    /// Lock file for editing
    pub fn lock_file(&self, file_path: &Path, user: &str) -> Result<()> {
        let lock_data = serde_json::json!({
            "file": file_path.to_string_lossy(),
            "user": user,
            "timestamp": chrono::Local::now().to_rfc3339(),
        });
        
        let lock_path = self.config.repo_path.join(".anchorpoint").join("locks");
        std::fs::create_dir_all(&lock_path)?;
        
        let lock_file = lock_path.join(format!(
            "{}.lock",
            file_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        std::fs::write(lock_file, serde_json::to_string_pretty(&lock_data)?)?;
        
        Ok(())
    }
    
    /// Unlock file
    pub fn unlock_file(&self, file_path: &Path) -> Result<()> {
        let lock_path = self.config.repo_path.join(".anchorpoint").join("locks");
        let lock_file = lock_path.join(format!(
            "{}.lock",
            file_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        
        if lock_file.exists() {
            std::fs::remove_file(lock_file)?;
        }
        
        Ok(())
    }
    
    /// Commit changes with message
    pub fn commit(&self, message: &str) -> Result<String> {
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(&self.config.repo_path)
            .output()
            .map_err(|e| AthanorError::Parse(format!("Git commit failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(AthanorError::Parse("Commit failed".to_string()));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Tag asset with metadata
    pub fn tag_asset(
        &self,
        file_path: &Path,
        tag: &str,
    ) -> Result<()> {
        let tags_dir = self.config.repo_path.join(".anchorpoint").join("tags");
        std::fs::create_dir_all(&tags_dir)?;
        
        let tag_file = tags_dir.join(format!(
            "{}.tags",
            file_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        
        let mut tags = if tag_file.exists() {
            std::fs::read_to_string(&tag_file)?
        } else {
            String::new()
        };
        
        if !tags.contains(tag) {
            tags.push_str(tag);
            tags.push('\n');
        }
        
        std::fs::write(tag_file, tags)?;
        Ok(())
    }
    
    /// Query assets by tag
    pub fn find_assets_by_tag(&self, tag: &str) -> Result<Vec<std::path::PathBuf>> {
        let tags_dir = self.config.repo_path.join(".anchorpoint").join("tags");
        let mut results = Vec::new();
        
        if !tags_dir.exists() {
            return Ok(results);
        }
        
        for entry in std::fs::read_dir(tags_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|e| e.to_str()) == Some("tags") {
                let content = std::fs::read_to_string(&path)?;
                if content.contains(tag) {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        let asset_name = name_str.trim_end_matches(".tags");
                        results.push(self.config.repo_path.join(asset_name));
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Setup review workflow for asset
    pub fn submit_for_review(
        &self,
        file_path: &Path,
        reviewer: &str,
    ) -> Result<()> {
        let review_data = serde_json::json!({
            "file": file_path.to_string_lossy(),
            "reviewer": reviewer,
            "status": "pending",
            "submitted": chrono::Local::now().to_rfc3339(),
        });
        
        let reviews_dir = self.config.repo_path.join(".anchorpoint").join("reviews");
        std::fs::create_dir_all(&reviews_dir)?;
        
        let review_file = reviews_dir.join(format!(
            "{}.review",
            file_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        
        std::fs::write(review_file, serde_json::to_string_pretty(&review_data)?)?;
        Ok(())
    }
}