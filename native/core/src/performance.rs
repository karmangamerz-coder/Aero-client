// SPDX-License-Identifier: GPL-3.0-only
//! Auditable performance presets. Project compatibility is always resolved live
//! through Modrinth for the selected Minecraft version; no mod binary is bundled.
use crate::{CancellationToken, DownloadClient, Instance, Loader, Store, content};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Preset {
    Potato,
    MaximumFps,
    Balanced,
    VisualQuality,
}

impl Preset {
    pub fn projects(self) -> &'static [&'static str] {
        match self {
            Self::Potato => &[
                "sodium",
                "lithium",
                "ferrite-core",
                "immediatelyfast",
                "entityculling",
                "modernfix",
            ],
            Self::MaximumFps => &[
                "sodium",
                "lithium",
                "ferrite-core",
                "immediatelyfast",
                "entityculling",
            ],
            Self::Balanced => &["sodium", "lithium", "ferrite-core", "immediatelyfast"],
            Self::VisualQuality => &["sodium", "immediatelyfast"],
        }
    }
}

pub async fn apply(
    store: &Store,
    client: &DownloadClient,
    instance: &Instance,
    preset: Preset,
    cancel: &CancellationToken,
) -> Result<Vec<content::InstalledProject>> {
    ensure!(
        instance.loader == Loader::Fabric,
        "Performance presets currently require a Fabric instance"
    );
    let mut installed = vec![];
    for project in preset.projects() {
        installed.extend(content::install(store, client, &instance.id, project, cancel).await?);
    }
    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_are_deterministic_and_have_no_duplicates() {
        for preset in [
            Preset::Potato,
            Preset::MaximumFps,
            Preset::Balanced,
            Preset::VisualQuality,
        ] {
            let projects = preset.projects();
            let unique: std::collections::HashSet<_> = projects.iter().collect();
            assert_eq!(projects.len(), unique.len());
            assert!(
                projects
                    .iter()
                    .all(|id| id.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
            );
        }
    }
}
