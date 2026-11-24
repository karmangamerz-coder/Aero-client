// SPDX-License-Identifier: GPL-3.0-only
//! First-launch setup state. Progress is written after every step, so closing the
//! launcher mid-setup resumes exactly where it stopped instead of starting over.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

pub const STEPS: usize = 7;
pub const LAST_STEP: usize = STEPS - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Next,
    Back,
    Skip,
}

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
        ensure!(
            ["Potato PC", "Balanced", "Maximum FPS", "Visual Quality"]
                .contains(&self.performance.as_str()),
            "Unknown performance preset"
        );
        ensure!(
            compatible(&self.version, &self.loader),
            "The selected loader is not compatible with this Minecraft version"
        );
        Ok(())
    }
}

pub fn compatible(version: &str, loader: &str) -> bool {
    !(loader == "Forge" && version != "1.8.9") && !(loader == "Fabric" && version == "1.8.9")
}

/// Persisted onboarding record: completion flag, current step and answers.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Progress {
    pub completed: bool,
    pub step: u8,
    pub answers: Onboarding,
}

impl Progress {
    pub fn needs_setup(&self) -> bool {
        !self.completed
    }

    pub fn current_step(&self) -> usize {
        (self.step as usize).min(LAST_STEP)
    }

    /// Apply one navigation event against the step the UI rendered. The expected-step
    /// check makes double clicks idempotent instead of advancing twice from queued events.
    pub fn navigate(
        &self,
        expected_step: usize,
        direction: Direction,
        answers: Onboarding,
    ) -> Result<Self> {
        ensure!(!self.completed, "Setup is already complete");
        ensure!(
            self.current_step() == expected_step,
            "Setup changed; review the current step and try again"
        );
        ensure!(expected_step <= LAST_STEP, "Unknown setup step");
        validate_step(expected_step, &answers)?;
        let next = match direction {
            Direction::Next => {
                ensure!(expected_step < LAST_STEP, "Setup is ready to finish");
                expected_step + 1
            }
            Direction::Back => expected_step.saturating_sub(1),
            Direction::Skip => {
                ensure!(
                    matches!(expected_step, 1 | 2),
                    "This setup step is required"
                );
                expected_step + 1
            }
        };
        Ok(Self {
            completed: false,
            step: next as u8,
            answers,
        })
    }

    pub fn finish(&self, expected_step: usize, answers: Onboarding) -> Result<Self> {
        ensure!(!self.completed, "Setup is already complete");
        ensure!(
            expected_step == LAST_STEP && self.current_step() == expected_step,
            "Complete every setup step before finishing"
        );
        answers.validate()?;
        Ok(Self {
            completed: true,
            step: LAST_STEP as u8,
            answers,
        })
    }

    /// Explicit recovery path. It opens the launcher without creating, installing or
    /// changing an instance; ownership is still required before game launch.
    pub fn defer(&self, expected_step: usize, answers: Onboarding) -> Result<Self> {
        ensure!(!self.completed, "Setup is already complete");
        ensure!(
            self.current_step() == expected_step,
            "Setup changed; try again"
        );
        answers.validate()?;
        Ok(Self {
            completed: true,
            step: expected_step as u8,
            answers,
        })
    }
}

pub fn validate_step(step: usize, answers: &Onboarding) -> Result<()> {
    ensure!(step <= LAST_STEP, "Unknown setup step");
    match step {
        // Welcome, optional account and hardware recommendation have no required input.
        0..=2 => Ok(()),
        3 => {
            ensure!(
                ["Aero Light", "Aero Dark", "Aero Lite"].contains(&answers.appearance.as_str()),
                "Choose an appearance preset"
            );
            Ok(())
        }
        4 => {
            ensure!(
                crate::TARGETS.contains(&answers.version.as_str()),
                "Choose a supported Minecraft version"
            );
            ensure!(
                ["Vanilla", "Fabric", "Forge"].contains(&answers.loader.as_str()),
                "Choose a supported loader"
            );
            ensure!(
                compatible(&answers.version, &answers.loader),
                "The selected loader is not compatible with this Minecraft version"
            );
            Ok(())
        }
        5 => {
            ensure!(
                (1024..=32768).contains(&answers.memory_mib),
                "Memory must be 1024–32768 MiB"
            );
            ensure!(
                ["Potato PC", "Balanced", "Maximum FPS", "Visual Quality"]
                    .contains(&answers.performance.as_str()),
                "Choose a performance preset"
            );
            Ok(())
        }
        _ => answers.validate(),
    }
}

pub fn file(root: &Path) -> std::path::PathBuf {
    root.join("onboarding.json")
}

pub async fn load(root: &Path) -> Progress {
    let path = file(root);
    match fs::read(&path).await {
        Ok(bytes) => serde_json::from_slice::<Progress>(&bytes)
            .ok()
            .filter(|progress| {
                progress.step as usize <= LAST_STEP && progress.answers.validate().is_ok()
            })
            .unwrap_or_default(),
        Err(_) => Progress::default(),
    }
}

pub async fn save(root: &Path, progress: &Progress) -> Result<()> {
    progress.answers.validate()?;
    ensure!(progress.step as usize <= LAST_STEP, "Unknown setup step");
    let bytes = serde_json::to_vec_pretty(progress)?;
    crate::storage::write_atomic(&file(root), &bytes).await
}
