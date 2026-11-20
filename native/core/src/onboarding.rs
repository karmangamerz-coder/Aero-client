// SPDX-License-Identifier: MIT
//! First-launch setup state. Progress is written after every step, so closing the
//! launcher mid-setup resumes exactly where it stopped instead of starting over.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

pub const STEPS: usize = 7;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Onboarding {
    pub schema: u32,
    pub version: String,
    pub loader: String,
    pub memory_mib: u32,
    pub appearance: String,
    pub performance: String,
    pub installation_directory: String,
    pub auto_java: bool,
}

impl Default for Onboarding {
    fn default() -> Self {
        Self {
            schema: 1,
            version: "26.3".into(),
            loader: "Vanilla".into(),
            memory_mib: 4096,
            appearance: "Aero Light".into(),
            performance: "Balanced".into(),
            installation_directory: String::new(),
            auto_java: true,
        }
    }
}

impl Onboarding {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema == 1, "Unknown setup schema");
        ensure!(
            crate::TARGETS.contains(&self.version.as_str()),
            "Unsupported version"
        );
        ensure!(
            ["Vanilla", "Fabric", "Forge"].contains(&self.loader.as_str()),
            "Unsupported loader"
        );
        ensure!(
            (1024..=32768).contains(&self.memory_mib),
            "Memory must be 1024–32768 MiB"
        );
        ensure!(
            ["Aero Light", "Aero Dark", "Aero Lite"].contains(&self.appearance.as_str()),
            "Unknown appearance"
        );
        Ok(())
    }
}

/// Persisted onboarding record: completion flag, current step and answers.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Progress {
    pub completed: bool,
    pub step: u8,
    pub answers: Onboarding,
}

pub fn file(root: &Path) -> std::path::PathBuf {
    root.join("onboarding.json")
}

pub async fn load(root: &Path) -> Progress {
    let path = file(root);
    match fs::read(&path).await {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => Progress::default(),
    }
}

pub async fn save(root: &Path, progress: &Progress) -> Result<()> {
    progress.answers.validate()?;
    ensure!(progress.step as usize <= STEPS, "Unknown setup step");
    let bytes = serde_json::to_vec_pretty(progress)?;
    crate::storage::write_atomic(&file(root), &bytes).await
}
