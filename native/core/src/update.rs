// SPDX-License-Identifier: GPL-3.0-only
//! User-requested update discovery. Updates are never installed silently.
use crate::{CancellationToken, DownloadClient};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

pub const RELEASES: &str =
    "https://api.github.com/repos/karmangamerz-coder/Aero-client/releases/latest";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub page: String,
    pub portable_url: Option<String>,
    pub setup_url: Option<String>,
    pub checksums_url: Option<String>,
}

pub async fn latest(client: &DownloadClient, cancel: &CancellationToken) -> Result<Release> {
    let value: serde_json::Value =
        serde_json::from_slice(&client.metadata(RELEASES, cancel).await?)
            .context("Invalid GitHub release response")?;
    let tag = value["tag_name"].as_str().context("Release has no tag")?;
    let version = tag.strip_prefix('v').unwrap_or(tag);
    ensure!(
        valid_version(version),
        "Release tag is not semantic versioning"
    );
    let mut release = Release {
        version: version.into(),
        page: value["html_url"]
            .as_str()
            .context("Release has no page")?
            .into(),
        portable_url: None,
        setup_url: None,
        checksums_url: None,
    };
    for asset in value["assets"].as_array().into_iter().flatten() {
        let name = asset["name"].as_str().unwrap_or("");
        let url = asset["browser_download_url"].as_str().map(str::to_owned);
        match name {
            "AeroClient-Portable.zip" => release.portable_url = url,
            "AeroClient-Setup.exe" => release.setup_url = url,
            "SHA256SUMS.txt" => release.checksums_url = url,
            _ => {}
        }
    }
    Ok(release)
}

pub fn valid_version(value: &str) -> bool {
    let base = value.split_once('-').map_or(value, |(base, _)| base);
    let parts: Vec<_> = base.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semantic_versions_are_strict() {
        assert!(valid_version("1.0.0"));
        assert!(valid_version("1.0.0-beta.1"));
        assert!(!valid_version("1.0"));
        assert!(!valid_version("latest"));
    }
}
