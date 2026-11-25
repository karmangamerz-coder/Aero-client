// SPDX-License-Identifier: GPL-3.0-only
//! Explicit Modrinth discovery and installation.
//!
//! Nothing is downloaded in the background. Every artifact is selected for the
//! instance's exact game version/loader and verified by the SHA-1 published by
//! Modrinth before it becomes visible to Minecraft.
use crate::{Artifact, CancellationToken, DownloadClient, Instance, Loader, Store};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{fs, io::AsyncReadExt};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryItem {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub project_type: String,
    pub slug: String,
    pub icon_url: String,
    pub author: String,
    pub downloads: u64,
    pub categories: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ProjectDetail {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub project_type: String,
    pub downloads: u64,
    pub followers: u64,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InstalledProject {
    pub project_id: String,
    pub version_id: String,
    pub slug: String,
    pub title: String,
    pub project_type: String,
    pub file: String,
    pub sha1: String,
    pub enabled: bool,
    pub dependency: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ContentManifest {
    schema: u32,
    projects: Vec<InstalledProject>,
}

pub async fn discover(
    store: &Store,
    client: &DownloadClient,
    query: &str,
    project_type: &str,
    version: &str,
    loader: &str,
    category: &str,
    sort: &str,
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
    ensure!(
        category.is_empty()
            || (category.len() <= 48
                && category
                    .chars()
                    .all(|value| value.is_ascii_alphanumeric() || value == '-')),
        "Invalid Modrinth category"
    );
    if !category.is_empty() {
        facets.push(vec![format!("categories:{category}")]);
    }
    let sort = match sort {
        "downloads" | "follows" | "newest" | "updated" => sort,
        _ => "relevance",
    };
    let mut url = reqwest::Url::parse("https://api.modrinth.com/v2/search")?;
    url.query_pairs_mut()
        .append_pair("query", query)
        .append_pair("limit", "24")
        .append_pair("index", sort)
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
            project_id: hit["project_id"].as_str().unwrap_or("").into(),
            title: hit["title"].as_str().unwrap_or("").into(),
            description: hit["description"].as_str().unwrap_or("").into(),
            project_type: hit["project_type"].as_str().unwrap_or("").into(),
            slug: hit["slug"].as_str().unwrap_or("").into(),
            icon_url: hit["icon_url"].as_str().unwrap_or("").into(),
            author: hit["author"].as_str().unwrap_or("").into(),
            downloads: hit["downloads"].as_u64().unwrap_or_default(),
            categories: hit["categories"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
        })
        .collect())
}

pub async fn project_detail(
    client: &DownloadClient,
    project: &str,
    version: &str,
    loader: &str,
    cancel: &CancellationToken,
) -> Result<ProjectDetail> {
    validate_project_identity(project)?;
    let project_json = json(
        client,
        &format!("https://api.modrinth.com/v2/project/{project}"),
        cancel,
    )
    .await?;
    let mut versions_url = reqwest::Url::parse(&format!(
        "https://api.modrinth.com/v2/project/{project}/version"
    ))?;
    if !version.is_empty() {
        versions_url
            .query_pairs_mut()
            .append_pair("game_versions", &serde_json::to_string(&[version])?);
    }
    if !loader.is_empty() {
        versions_url
            .query_pairs_mut()
            .append_pair("loaders", &serde_json::to_string(&[loader])?);
    }
    let versions_json = json(client, versions_url.as_str(), cancel).await?;
    Ok(ProjectDetail {
        project_id: project_json["id"].as_str().unwrap_or("").into(),
        slug: project_json["slug"].as_str().unwrap_or(project).into(),
        title: project_json["title"].as_str().unwrap_or(project).into(),
        description: project_json["description"].as_str().unwrap_or("").into(),
        body: project_json["body"].as_str().unwrap_or("").into(),
        project_type: project_json["project_type"].as_str().unwrap_or("").into(),
        downloads: project_json["downloads"].as_u64().unwrap_or_default(),
        followers: project_json["followers"].as_u64().unwrap_or_default(),
        categories: project_json["categories"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        versions: versions_json
            .as_array()
            .into_iter()
            .flatten()
            .take(20)
            .filter_map(|item| item["name"].as_str())
            .map(str::to_owned)
            .collect(),
    })
}

fn validate_project_identity(project: &str) -> Result<()> {
    ensure!(
        !project.is_empty()
            && project.len() <= 128
            && project
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-'),
        "Invalid project identity"
    );
    Ok(())
}

fn loader_name(loader: Loader) -> &'static str {
    match loader {
        Loader::Vanilla => "",
        Loader::Fabric => "fabric",
        Loader::Forge => "forge",
    }
}

fn folder(project_type: &str) -> Result<&'static str> {
    match project_type {
        "mod" => Ok("mods"),
        "resourcepack" => Ok("resourcepacks"),
        "shader" => Ok("shaderpacks"),
        "modpack" => bail!("Modpack projects must be imported into a new instance"),
        _ => bail!("Unsupported Modrinth project type"),
    }
}

async fn json(client: &DownloadClient, url: &str, cancel: &CancellationToken) -> Result<Value> {
    serde_json::from_slice(&client.metadata(url, cancel).await?).context("Invalid Modrinth JSON")
}

async fn compatible_version(
    client: &DownloadClient,
    project: &str,
    instance: &Instance,
    cancel: &CancellationToken,
) -> Result<Value> {
    let mut url = reqwest::Url::parse(&format!(
        "https://api.modrinth.com/v2/project/{project}/version"
    ))?;
    url.query_pairs_mut().append_pair(
        "game_versions",
        &serde_json::to_string(&[&instance.version])?,
    );
    let loader = loader_name(instance.loader);
    if !loader.is_empty() {
        url.query_pairs_mut()
            .append_pair("loaders", &serde_json::to_string(&[loader])?);
    }
    let versions = json(client, url.as_str(), cancel).await?;
    let list = versions
        .as_array()
        .context("Invalid Modrinth version list")?;
    list.iter()
        .find(|v| {
            v["status"].as_str().is_none_or(|s| s == "listed")
                && v["version_type"].as_str().is_none_or(|s| s == "release")
        })
        .or_else(|| list.first())
        .cloned()
        .context("No compatible version is available for this instance")
}

async fn load_manifest(store: &Store, id: &str) -> Result<ContentManifest> {
    let file = store.directory(id).await?.join("content.json");
    if !fs::try_exists(&file).await? {
        return Ok(ContentManifest {
            schema: 1,
            projects: vec![],
        });
    }
    let manifest: ContentManifest = serde_json::from_slice(&fs::read(file).await?)?;
    ensure!(manifest.schema == 1, "Unsupported content manifest");
    Ok(manifest)
}

async fn save_manifest(store: &Store, id: &str, manifest: &ContentManifest) -> Result<()> {
    crate::storage::write_atomic(
        &store.directory(id).await?.join("content.json"),
        &serde_json::to_vec_pretty(manifest)?,
    )
    .await
}

pub async fn list(store: &Store, id: &str) -> Result<Vec<InstalledProject>> {
    Ok(load_manifest(store, id).await?.projects)
}

/// Installs a project and all required Modrinth dependencies after an explicit click.
pub async fn install(
    store: &Store,
    client: &DownloadClient,
    instance_id: &str,
    project: &str,
    cancel: &CancellationToken,
) -> Result<Vec<InstalledProject>> {
    validate_project_identity(project)?;
    let instance = store.read(instance_id).await?;
    let root = store.directory(instance_id).await?.join("game");
    let mut manifest = load_manifest(store, instance_id).await?;
    let mut queue = VecDeque::from([(project.to_owned(), false)]);
    let mut visited = HashSet::new();
    let mut installed = vec![];
    while let Some((identity, dependency)) = queue.pop_front() {
        ensure!(!cancel.is_cancelled(), "Installation cancelled");
        ensure!(visited.len() < 64, "Dependency graph is too large");
        if !visited.insert(identity.clone()) {
            continue;
        }
        let project_json = json(
            client,
            &format!("https://api.modrinth.com/v2/project/{identity}"),
            cancel,
        )
        .await?;
        let project_id = project_json["id"].as_str().context("Missing project id")?;
        if manifest.projects.iter().any(|p| p.project_id == project_id) {
            continue;
        }
        let project_type = project_json["project_type"]
            .as_str()
            .context("Missing project type")?;
        let destination = folder(project_type)?;
        let version = compatible_version(client, project_id, &instance, cancel).await?;
        for item in version["dependencies"].as_array().into_iter().flatten() {
            if item["dependency_type"] == "required" {
                if let Some(id) = item["project_id"].as_str() {
                    queue.push_back((id.to_owned(), true));
                }
            }
        }
        let files = version["files"]
            .as_array()
            .context("Version has no files")?;
        let file = files
            .iter()
            .find(|f| f["primary"] == true)
            .or_else(|| files.first())
            .context("Version has no downloadable file")?;
        let filename = file["filename"].as_str().context("Missing filename")?;
        crate::storage::validate_relative(filename)?;
        ensure!(
            !filename.contains('/'),
            "Nested content filenames are forbidden"
        );
        let sha1 = file["hashes"]["sha1"]
            .as_str()
            .context("Modrinth file has no SHA-1")?;
        let artifact = Artifact {
            url: file["url"].as_str().context("Missing file URL")?.into(),
            sha1: sha1.into(),
            size: file["size"].as_u64(),
            alternatives: vec![],
        };
        let downloaded = client
            .fetch(
                &artifact,
                &root,
                &format!("{destination}/{filename}"),
                cancel,
            )
            .await?;
        if let Some(expected_sha512) = file["hashes"]["sha512"].as_str() {
            ensure!(
                expected_sha512.len() == 128
                    && expected_sha512
                        .bytes()
                        .all(|value| value.is_ascii_hexdigit()),
                "Modrinth file has an invalid SHA-512"
            );
            ensure!(
                sha512(&downloaded, cancel)
                    .await?
                    .eq_ignore_ascii_case(expected_sha512),
                "Modrinth SHA-512 integrity mismatch"
            );
        }
        let record = InstalledProject {
            project_id: project_id.into(),
            version_id: version["id"].as_str().context("Missing version id")?.into(),
            slug: project_json["slug"].as_str().unwrap_or(project_id).into(),
            title: project_json["title"].as_str().unwrap_or(project_id).into(),
            project_type: project_type.into(),
            file: format!("{destination}/{filename}"),
            sha1: sha1.into(),
            enabled: true,
            dependency,
        };
        manifest.projects.push(record.clone());
        save_manifest(store, instance_id, &manifest).await?;
        installed.push(record);
    }
    Ok(installed)
}

async fn sha512(path: &std::path::Path, cancel: &CancellationToken) -> Result<String> {
    use sha2::{Digest, Sha512};
    let mut file = fs::File::open(path).await?;
    let mut buffer = vec![0; 65_536];
    let mut digest = Sha512::new();
    loop {
        ensure!(!cancel.is_cancelled(), "Installation cancelled");
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex::encode(digest.finalize()))
}

pub async fn set_enabled(
    store: &Store,
    instance_id: &str,
    project_id: &str,
    enabled: bool,
) -> Result<()> {
    let mut manifest = load_manifest(store, instance_id).await?;
    let record = manifest
        .projects
        .iter_mut()
        .find(|p| p.project_id == project_id)
        .context("Project is not installed")?;
    if record.enabled == enabled {
        return Ok(());
    }
    let root = store.directory(instance_id).await?.join("game");
    let from_relative = record.file.clone();
    let to_relative = if enabled {
        from_relative
            .strip_suffix(".disabled")
            .context("Invalid disabled filename")?
            .to_owned()
    } else {
        format!("{from_relative}.disabled")
    };
    let from = crate::storage::safe_path(&root, &from_relative).await?;
    let to = crate::storage::safe_path(&root, &to_relative).await?;
    fs::rename(from, to).await?;
    record.file = to_relative;
    record.enabled = enabled;
    save_manifest(store, instance_id, &manifest).await
}

/// Recoverable removal: content is moved under the instance's content-trash folder.
pub async fn remove(store: &Store, instance_id: &str, project_id: &str) -> Result<PathBuf> {
    let mut manifest = load_manifest(store, instance_id).await?;
    let index = manifest
        .projects
        .iter()
        .position(|p| p.project_id == project_id)
        .context("Project is not installed")?;
    let record = manifest.projects.remove(index);
    let directory = store.directory(instance_id).await?;
    let source = crate::storage::safe_path(&directory.join("game"), &record.file).await?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let record_path = PathBuf::from(&record.file);
    let filename = record_path
        .file_name()
        .context("Missing filename")?
        .to_string_lossy();
    let target =
        crate::storage::safe_path(&directory, &format!("content-trash/{stamp}-{filename}")).await?;
    fs::create_dir_all(target.parent().context("Missing trash directory")?).await?;
    fs::rename(source, &target).await?;
    save_manifest(store, instance_id, &manifest).await?;
    Ok(target)
}
