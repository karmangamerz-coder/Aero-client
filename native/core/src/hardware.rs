// SPDX-License-Identifier: MIT
//! One-shot hardware detection for the setup wizard and performance presets.
//! Detection is read-only: Aero never changes system settings on its own, and the
//! recommendation is a suggestion the player can always override.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Hardware {
    pub cpu: String,
    pub cores: usize,
    pub memory_mib: u64,
    pub storage_free_gib: u64,
    pub os: String,
    pub gpu: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Performance {
    Potato,
    #[default]
    Balanced,
    MaxFps,
    VisualQuality,
}

impl Performance {
    pub fn label(self) -> &'static str {
        match self {
            Performance::Potato => "Potato PC",
            Performance::Balanced => "Balanced",
            Performance::MaxFps => "Maximum FPS",
            Performance::VisualQuality => "Visual Quality",
        }
    }
    pub fn hint(self) -> &'static str {
        match self {
            Performance::Potato => "Lowest memory use, static interface, smallest render distance.",
            Performance::Balanced => "Recommended defaults for most computers.",
            Performance::MaxFps => "Lean graphics settings that favour frame rate.",
            Performance::VisualQuality => {
                "Higher visual settings where the computer can afford them."
            }
        }
    }
    pub fn all() -> [Performance; 4] {
        [
            Performance::Potato,
            Performance::Balanced,
            Performance::MaxFps,
            Performance::VisualQuality,
        ]
    }
}

impl Hardware {
    /// Suggested game memory: leave the operating system room, never guess wildly.
    pub fn recommended_memory_mib(&self) -> u32 {
        let total = self.memory_mib;
        let value = if total < 7_500 {
            2048
        } else if total < 15_000 {
            3072
        } else {
            4096
        };
        value.min((total.saturating_sub(2048)).max(1024) as u32)
    }
    pub fn recommended_performance(&self) -> Performance {
        if self.memory_mib < 7_500 || self.cores <= 2 {
            Performance::Potato
        } else {
            Performance::Balanced
        }
    }
    pub fn summary(&self) -> String {
        format!(
            "{} · {} cores · {} GB RAM · {} GB free",
            self.cpu,
            self.cores,
            self.memory_mib / 1024,
            self.storage_free_gib
        )
    }
}

pub fn detect() -> Hardware {
    let mut system = sysinfo::System::new();
    system.refresh_cpu_all();
    system.refresh_memory();
    let cpu = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "Unknown CPU".into())
        });
    let cores = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);
    let memory_mib = system.total_memory() / (1024 * 1024);
    let storage_free_gib = sysinfo::Disks::new_with_refreshed_list()
        .list()
        .iter()
        .map(|disk| disk.available_space())
        .max()
        .unwrap_or(0)
        / (1024 * 1024 * 1024);
    let os = match sysinfo::System::long_os_version() {
        Some(version) => version,
        None => format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
    };
    Hardware {
        cpu,
        cores,
        memory_mib,
        storage_free_gib,
        os,
        // GPU names are not exposed by portable system APIs. Windows reads them
        // with one read-only WMI query; anywhere else the field is left honest.
        gpu: gpu().unwrap_or_else(|_| "Unavailable".into()),
    }
}

#[cfg(windows)]
pub fn gpu() -> Result<String> {
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name) -join ' / '",
        ])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(0x0800_0000)
        .output()
        .context("GPU query failed")?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok(if text.is_empty() {
        "Unavailable".into()
    } else {
        text
    })
}

#[cfg(not(windows))]
pub fn gpu() -> Result<String> {
    Ok("Unavailable on this platform".into())
}
