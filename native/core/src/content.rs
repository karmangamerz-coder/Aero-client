// SPDX-License-Identifier: MIT
use crate::{CancellationToken, DownloadClient, Store, VERSION_MANIFEST};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct NewsItem {
    pub title: String,
    pub source: String,
    pub date: String,
    pub url: String,
}
pub async fn news(
    store: &Store,
    client: &DownloadClient,
    cancel: &CancellationToken,
) -> Result<Vec<NewsItem>> {
    let manifest = store
        .cached_json(client, "versions", VERSION_MANIFEST, false, cancel)
        .await?;
    let mut result = vec![];
    for version in manifest["versions"]
        .as_array()
        .context("Invalid Minecraft manifest")?
        .iter()
        .filter(|v| v["type"] == "release")
        .take(3)
    {
        result.push(NewsItem {
            title: format!("Minecraft {}", version["id"].as_str().unwrap_or("")),
            source: "Official release metadata".into(),
            date: version["releaseTime"]
                .as_str()
                .unwrap_or("")
                .chars()
                .take(10)
                .collect(),
            url: "https://www.minecraft.net/en-us/articles".into(),
        });
    }
    let releases = store
        .cached_json(
            client,
            "aero-releases",
            "https://api.github.com/repos/karmangamerz-coder/Aero-client/releases?per_page=3",
            false,
            cancel,
        )
        .await?;
    for release in releases
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| v["draft"] != true)
        .take(2)
    {
        result.push(NewsItem {
            title: release["name"]
                .as_str()
                .or_else(|| release["tag_name"].as_str())
                .unwrap_or("Aero release")
                .into(),
            source: "Aero Client release notes".into(),
            date: release["published_at"]
                .as_str()
                .unwrap_or("")
                .chars()
                .take(10)
                .collect(),
            url: release["html_url"].as_str().unwrap_or("").into(),
        });
    }
    Ok(result)
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryItem {
    pub title: String,
    pub description: String,
    pub project_type: String,
    pub slug: String,
    pub icon_url: String,
}
pub async fn discover(
    store: &Store,
    client: &DownloadClient,
    query: &str,
    project_type: &str,
    version: &str,
    loader: &str,
    cancel: &CancellationToken,
) -> Result<Vec<DiscoveryItem>> {
    ensure!(
        ["mod", "modpack", "resourcepack", "shader"].contains(&project_type),
        "Unknown project category"
    );
    let mut facets = vec![vec![format!("project_type:{project_type}")]];
    if !version.is_empty() {
        facets.push(vec![format!("versions:{version}")]);
    }
    if ["fabric", "forge"].contains(&loader) {
        facets.push(vec![format!("categories:{loader}")]);
    }
    let mut url = reqwest::Url::parse("https://api.modrinth.com/v2/search")?;
    url.query_pairs_mut()
        .append_pair("query", query)
        .append_pair("limit", "12")
        .append_pair("facets", &serde_json::to_string(&facets)?);
    use sha1::{Digest, Sha1};
    let key = format!(
        "modrinth-{}",
        hex::encode(Sha1::digest(url.as_str().as_bytes()))
    );
    let value = store
        .cached_json(client, &key, url.as_str(), false, cancel)
        .await?;
    Ok(value["hits"]
        .as_array()
        .context("Invalid Modrinth response")?
        .iter()
        .map(|hit| DiscoveryItem {
            title: hit["title"].as_str().unwrap_or("").into(),
            description: hit["description"].as_str().unwrap_or("").into(),
            project_type: hit["project_type"].as_str().unwrap_or("").into(),
            slug: hit["slug"].as_str().unwrap_or("").into(),
            icon_url: hit["icon_url"].as_str().unwrap_or("").into(),
        })
        .collect())
}
