// SPDX-License-Identifier: GPL-3.0-only
pub mod auth;
pub mod content;
pub mod hardware;
pub mod install;
pub mod launch;
pub mod net;
pub mod onboarding;
pub mod performance;
pub mod runtime;
pub mod skin;
pub mod storage;
pub mod update;
#[cfg(test)]
mod tests;
pub use auth::{Account, Accounts};
pub use hardware::{Hardware, Performance};
pub use install::Installer;
pub use launch::{LaunchSpec, Outcome};
pub use net::{Artifact, DownloadClient, Progress};
pub use storage::{Instance, Loader, Settings, Store};
pub use tokio_util::sync::CancellationToken;

pub const TARGETS: [&str; 4] = ["1.8.9", "1.21.11", "26.2", "26.3"];
pub const VERSION_MANIFEST: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub const RUNTIME_MANIFEST: &str = "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";
