// SPDX-License-Identifier: MIT
use crate::{
    Artifact, CancellationToken, DownloadClient, Instance, Loader, Store, TARGETS,
    VERSION_MANIFEST, runtime,
    storage::{safe_path, validate_relative, write_atomic},
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
    io::Read,
    path::{Path, PathBuf},
};
use tokio::fs;

#[derive(Clone)]
pub struct Installer {
    pub store: Store,
    pub client: DownloadClient,
}
impl Installer {
    pub fn new(root: PathBuf, report: crate::net::Reporter) -> Result<Self> {
        Ok(Self {
            store: Store::new(root),
            client: DownloadClient::new(report)?,
        })
    }
    pub async fn versions(&self, refresh: bool, token: &CancellationToken) -> Result<Vec<String>> {
        let manifest = self
            .store
            .cached_json(&self.client, "versions", VERSION_MANIFEST, refresh, token)
            .await?;
        let entries = manifest["versions"]
            .as_array()
            .context("Invalid Minecraft version manifest")?;
        Ok(TARGETS
            .iter()
            .filter(|id| entries.iter().any(|entry| entry["id"].as_str() == Some(id)))
            .map(|id| (*id).to_owned())
            .collect())
    }
    pub async fn version(&self, id: &str, token: &CancellationToken) -> Result<Value> {
        ensure!(TARGETS.contains(&id), "Unsupported target version");
        let manifest = self
            .store
            .cached_json(&self.client, "versions", VERSION_MANIFEST, false, token)
            .await?;
        let entry = manifest["versions"]
            .as_array()
            .context("Invalid version manifest")?
            .iter()
            .find(|entry| entry["id"] == id)
            .context("Version is unavailable in official metadata; refresh versions")?;
        let artifact = Artifact::from_json(entry)?;
        let file = self
            .client
            .fetch(
                &artifact,
                &self.store.root.join("metadata"),
                &format!("versions/{id}.json"),
                token,
            )
            .await?;
        let value: Value = serde_json::from_slice(&fs::read(file).await?)?;
        ensure!(value["id"] == id, "Metadata version mismatch");
        Ok(value)
    }
    /// Read back the verified installation plan written by the last successful run.
    pub async fn plan(&self, instance: &Instance) -> Result<Option<Value>> {
        let path = safe_path(
            &self.store.directory(&instance.id).await?,
            "installation/plan.json",
        )
        .await?;
        if !fs::try_exists(&path).await? {
            return Ok(None);
        }
        Ok(Some(serde_json::from_slice(&fs::read(path).await?)?))
    }
    pub async fn loader(
        &self,
        instance: &Instance,
        token: &CancellationToken,
    ) -> Result<Option<Value>> {
        let directory = self.store.directory(&instance.id).await?;
        match instance.loader {
            Loader::Vanilla => Ok(None),
            Loader::Fabric => {
                ensure!(
                    instance.version != "1.8.9",
                    "Fabric is not available for 1.8.9"
                );
                let pin = safe_path(&directory, "fabric-version.txt").await?;
                let loader_version = if let Ok(value) = fs::read_to_string(&pin).await {
                    value.trim().to_owned()
                } else {
                    let list = self
                        .store
                        .cached_json(
                            &self.client,
                            &format!("fabric-{}", instance.version),
                            &format!(
                                "https://meta.fabricmc.net/v2/versions/loader/{}",
                                instance.version
                            ),
                            false,
                            token,
                        )
                        .await?;
                    list.as_array()
                        .context("Invalid Fabric compatibility metadata")?
                        .iter()
                        .find(|entry| entry["loader"]["stable"] == true)
                        .and_then(|entry| entry["loader"]["version"].as_str())
                        .context("No compatible stable Fabric loader; choose Vanilla")?
                        .to_owned()
                };
                ensure!(
                    loader_version.split('.').count() == 3
                        && loader_version
                            .chars()
                            .all(|c| c.is_ascii_digit() || c == '.'),
                    "Invalid loader pin"
                );
                let profile = self.store.cached_json(&self.client, &format!("fabric-profile-{}-{loader_version}", instance.version), &format!("https://meta.fabricmc.net/v2/versions/loader/{}/{loader_version}/profile/json", instance.version), false, token).await?;
                ensure!(
                    profile["inheritsFrom"] == instance.version,
                    "Fabric profile version mismatch"
                );
                write_atomic(&pin, loader_version.as_bytes()).await?;
                Ok(Some(profile))
            }
            Loader::Forge => {
                ensure!(
                    instance.version == "1.8.9",
                    "Legacy Forge is restricted to 1.8.9"
                );
                let version = "1.8.9-11.15.1.2318-1.8.9";
                let base = format!(
                    "https://maven.minecraftforge.net/net/minecraftforge/forge/{version}/forge-{version}"
                );
                let artifact = self
                    .sidecar(&format!("{base}-installer.jar"), token)
                    .await?;
                let file = self
                    .client
                    .fetch(
                        &artifact,
                        &self.store.root.join("metadata"),
                        &format!("forge/{version}-installer.jar"),
                        token,
                    )
                    .await?;
                let mut profile = tokio::task::spawn_blocking(move || -> Result<Value> {
                    let mut archive = zip::ZipArchive::new(std::fs::File::open(file)?)?;
                    let mut bytes = vec![];
                    archive
                        .by_name("install_profile.json")?
                        .take(4 * 1024 * 1024 + 1)
                        .read_to_end(&mut bytes)?;
                    ensure!(
                        bytes.len() <= 4 * 1024 * 1024,
                        "Forge profile exceeds limit"
                    );
                    let value: Value = serde_json::from_slice(&bytes)?;
                    Ok(value["versionInfo"].clone())
                })
                .await??;
                ensure!(
                    profile["inheritsFrom"] == "1.8.9",
                    "Unexpected Forge profile"
                );
                let universal = self
                    .sidecar(&format!("{base}-universal.jar"), token)
                    .await?;
                for library in profile["libraries"]
                    .as_array_mut()
                    .context("Missing Forge libraries")?
                {
                    if library["name"] == format!("net.minecraftforge:forge:{version}") {
                        library["downloads"] = json!({"artifact":{"url":universal.url,"sha1":universal.sha1,"path":maven_path(library["name"].as_str().unwrap())?}});
                    }
                }
                Ok(Some(profile))
            }
        }
    }
    async fn sidecar(&self, url: &str, token: &CancellationToken) -> Result<Artifact> {
        let bytes = self.client.metadata(&format!("{url}.sha1"), token).await?;
        let hash = std::str::from_utf8(&bytes)?
            .split_whitespace()
            .next()
            .context("Empty artifact checksum")?
            .into();
        let artifact = Artifact {
            url: url.into(),
            sha1: hash,
            size: None,
            alternatives: vec![],
        };
        artifact.validate()?;
        Ok(artifact)
    }
    async fn library_artifact(
        &self,
        library: &Value,
        token: &CancellationToken,
    ) -> Result<(Artifact, String)> {
        let path = maven_path(
            library["name"]
                .as_str()
                .context("Missing library coordinate")?,
        )?;
        let url = reqwest::Url::parse(
            library["url"]
                .as_str()
                .unwrap_or("https://libraries.minecraft.net/"),
        )?
        .join(&path)?
        .to_string();
        let artifact =
            if let Some(hashes) = library["checksums"].as_array().filter(|v| !v.is_empty()) {
                let hashes: Vec<String> = hashes
                    .iter()
                    .map(|value| value.as_str().unwrap_or("").into())
                    .collect();
                Artifact {
                    url,
                    sha1: hashes[0].clone(),
                    size: library["size"].as_u64(),
                    alternatives: hashes,
                }
            } else if let Some(hash) = library["sha1"].as_str() {
                Artifact {
                    url,
                    sha1: hash.into(),
                    size: library["size"].as_u64(),
                    alternatives: vec![],
                }
            } else {
                self.sidecar(&url, token)
                    .await
                    .with_context(|| format!("No trusted checksum for {path}"))?
            };
        artifact.validate()?;
        Ok((artifact, path))
    }
    async fn libraries(
        &self,
        list: &Value,
        natives: &Path,
        token: &CancellationToken,
    ) -> Result<Vec<String>> {
        let root = self.store.root.join("shared/libraries");
        let mut paths = vec![];
        for library in list.as_array().context("Invalid library list")? {
            if !library_allowed(library)? {
                continue;
            }
            if let Some(downloads) = library.get("downloads") {
                if let Some(spec) = downloads.get("artifact") {
                    let path = spec["path"]
                        .as_str()
                        .map(String::from)
                        .unwrap_or(maven_path(
                            library["name"].as_str().context("Missing library name")?,
                        )?);
                    self.client
                        .fetch(&Artifact::from_json(spec)?, &root, &path, token)
                        .await?;
                    paths.push(path);
                }
                if let Some(classifier) = library["natives"]["windows"].as_str() {
                    let classifier = classifier.replace("${arch}", "64");
                    let spec = &downloads["classifiers"][&classifier];
                    let file = self
                        .client
                        .fetch(
                            &Artifact::from_json(spec)?,
                            &root,
                            spec["path"].as_str().context("Missing native path")?,
                            token,
                        )
                        .await?;
                    let excludes = library["extract"]["exclude"]
                        .as_array()
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    extract_natives(file, natives.to_path_buf(), excludes, token.clone()).await?;
                }
            } else {
                let (artifact, path) = self.library_artifact(library, token).await?;
                self.client.fetch(&artifact, &root, &path, token).await?;
                paths.push(path);
            }
        }
        Ok(paths)
    }
    pub async fn install(
        &self,
        instance: &Instance,
        custom_java: Option<&Path>,
        token: &CancellationToken,
    ) -> Result<Instance> {
        ensure!(
            cfg!(all(target_os = "windows", target_arch = "x86_64")),
            "Game installation currently supports Windows x64"
        );
        let lock_path = safe_path(&self.store.root, "installation.lock").await?;
        let lock = tokio::task::spawn_blocking(move || -> Result<std::fs::File> {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(lock_path)?;
            fs2::FileExt::try_lock_exclusive(&file)
                .context("Another Aero installation is active; wait or cancel it first")?;
            Ok(file)
        })
        .await??;
        self.store
            .status(instance, "INSTALLING", "Preparing verified installation")
            .await?;
        let result = self.perform(instance, custom_java, token).await;
        let outcome = match result {
            Ok(()) => {
                self.store
                    .status(
                        instance,
                        "READY",
                        "Installation verified. Sign-in is required before playing.",
                    )
                    .await
            }
            Err(error) => {
                self.store
                    .status(
                        instance,
                        if token.is_cancelled() {
                            "CANCELLED"
                        } else {
                            "FAILED"
                        },
                        "Retry or Repair resumes verified progress",
                    )
                    .await?;
                Err(error)
            }
        };
        drop(lock);
        outcome
    }
    async fn perform(
        &self,
        instance: &Instance,
        custom_java: Option<&Path>,
        token: &CancellationToken,
    ) -> Result<()> {
        let version = self.version(&instance.version, token).await?;
        let directory = self.store.directory(&instance.id).await?;
        let runtime = runtime::ensure_runtime(
            &self.store,
            &self.client,
            &version["javaVersion"],
            custom_java,
            token,
        )
        .await?;
        let loader = self.loader(instance, token).await?;
        write_atomic(
            &safe_path(&directory, "installation/version.json").await?,
            &serde_json::to_vec(&version)?,
        )
        .await?;
        if let Some(profile) = &loader {
            write_atomic(
                &safe_path(&directory, "installation/loader.json").await?,
                &serde_json::to_vec(profile)?,
            )
            .await?;
        }
        let client = Artifact::from_json(&version["downloads"]["client"])?;
        self.client
            .fetch(
                &client,
                &self.store.root.join("shared"),
                &format!("clients/{}.jar", client.sha1),
                token,
            )
            .await?;
        let natives = safe_path(&directory, "installation/natives").await?;
        let mut libraries = self
            .libraries(&version["libraries"], &natives, token)
            .await?;
        if let Some(profile) = &loader {
            libraries.extend(
                self.libraries(&profile["libraries"], &natives, token)
                    .await?,
            );
        }
        let assets = Artifact::from_json(&version["assetIndex"])?;
        let asset_root = self.store.root.join("shared/assets");
        let index_file = self
            .client
            .fetch(
                &assets,
                &asset_root,
                &format!("indexes/{}.json", assets.sha1),
                token,
            )
            .await?;
        let index: Value = serde_json::from_slice(&fs::read(index_file).await?)?;
        let mut requests = BTreeMap::new();
        for (_, object) in index["objects"]
            .as_object()
            .context("Invalid asset index")?
        {
            let hash = object["hash"].as_str().context("Missing asset hash")?;
            ensure!(crate::net::valid_hash(hash), "Invalid asset hash");
            requests.entry(hash.to_owned()).or_insert((
                Artifact {
                    url: format!(
                        "https://resources.download.minecraft.net/{}/{hash}",
                        &hash[..2]
                    ),
                    sha1: hash.into(),
                    size: object["size"].as_u64(),
                    alternatives: vec![],
                },
                asset_root.clone(),
                format!("objects/{}/{hash}", &hash[..2]),
            ));
        }
        self.client
            .batch(requests.into_values().collect(), token)
            .await?;
        if index["virtual"] == true || index["map_to_resources"] == true {
            for (name, object) in index["objects"].as_object().unwrap() {
                let hash = object["hash"].as_str().unwrap();
                let source =
                    safe_path(&asset_root, &format!("objects/{}/{hash}", &hash[..2])).await?;
                let destination = safe_path(
                    &directory,
                    &format!(
                        "{}/{name}",
                        if index["map_to_resources"] == true {
                            "game/resources"
                        } else {
                            "installation/virtual-assets"
                        }
                    ),
                )
                .await?;
                if fs::try_exists(&destination).await? {
                    ensure!(
                        DownloadClient::hash(&destination, token).await? == hash,
                        "Existing legacy resource differs; user file preserved"
                    );
                } else {
                    fs::create_dir_all(destination.parent().unwrap()).await?;
                    fs::copy(source, destination).await?;
                }
            }
        }
        ensure!(!token.is_cancelled(), "Cancelled");
        let plan = json!({"schema":1,"minecraft":instance.version,"runtime":runtime,"clientSha1":client.sha1,"assetIndexSha1":assets.sha1,"assetIndexId":version["assetIndex"]["id"],"libraries":libraries,"authenticationRequired":true,"launchEnabled":false});
        write_atomic(
            &safe_path(&directory, "installation/plan.json").await?,
            &serde_json::to_vec_pretty(&plan)?,
        )
        .await
    }
}
pub fn maven_path(coordinate: &str) -> Result<String> {
    let parts: Vec<_> = coordinate.split(':').collect();
    ensure!(
        (3..=4).contains(&parts.len())
            && parts.iter().all(|p| !p.is_empty()
                && p.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_.+-".contains(c))),
        "Invalid Maven coordinate"
    );
    let classifier = if parts.len() == 4 {
        format!("-{}", parts[3])
    } else {
        String::new()
    };
    let path = format!(
        "{}/{}/{}/{}-{}{classifier}.jar",
        parts[0].replace('.', "/"),
        parts[1],
        parts[2],
        parts[1],
        parts[2]
    );
    validate_relative(&path)?;
    Ok(path)
}
pub fn library_allowed(library: &Value) -> Result<bool> {
    if library["clientreq"] == false {
        return Ok(false);
    }
    let Some(rules) = library["rules"].as_array() else {
        return Ok(true);
    };
    let mut allowed = false;
    for rule in rules {
        let mut matches = true;
        if let Some(name) = rule["os"]["name"].as_str() {
            matches &= name == "windows";
        }
        if let Some(arch) = rule["os"]["arch"].as_str() {
            matches &= regex::Regex::new(arch)?.is_match("amd64");
        }
        if let Some(version) = rule["os"]["version"].as_str() {
            matches &= regex::Regex::new(version)?.is_match("10.0");
        }
        if let Some(features) = rule["features"].as_object() {
            matches &= features.values().all(|value| value == false);
        }
        if matches {
            allowed = rule["action"] == "allow";
        }
    }
    Ok(allowed)
}
pub async fn extract_natives(
    file: PathBuf,
    destination: PathBuf,
    excludes: HashSet<String>,
    token: CancellationToken,
) -> Result<()> {
    let entries = tokio::task::spawn_blocking(move || -> Result<Vec<(String, Vec<u8>)>> {
        let mut zip = zip::ZipArchive::new(std::fs::File::open(file)?)?;
        ensure!(zip.len() <= 10000, "Native archive has too many entries");
        let mut output = vec![];
        let mut total = 0usize;
        for i in 0..zip.len() {
            ensure!(!token.is_cancelled(), "Cancelled");
            let entry = zip.by_index(i)?;
            if entry.is_dir()
                || entry.name().starts_with("META-INF/")
                || excludes
                    .iter()
                    .any(|prefix| entry.name().starts_with(prefix))
            {
                continue;
            }
            validate_relative(entry.name())?;
            ensure!(!entry.is_symlink(), "Native archive links are forbidden");
            let name = entry.name().to_owned();
            let mut bytes = vec![];
            entry.take(64 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
            total += bytes.len();
            ensure!(
                bytes.len() <= 64 * 1024 * 1024 && total <= 128 * 1024 * 1024,
                "Native archive exceeds extraction limit"
            );
            output.push((name, bytes));
        }
        Ok(output)
    })
    .await??;
    for (name, bytes) in entries {
        write_atomic(&safe_path(&destination, &name).await?, &bytes).await?;
    }
    Ok(())
}
