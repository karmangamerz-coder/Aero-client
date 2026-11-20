// SPDX-License-Identifier: MIT
use crate::{CancellationToken, DownloadClient, Store};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
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
