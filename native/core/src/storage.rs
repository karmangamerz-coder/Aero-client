// SPDX-License-Identifier: MIT
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};
use tokio::fs;
use uuid::Uuid;

pub fn validate_relative(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && !value.contains(['\\', ':']) && !value.starts_with('/'),
        "Unsafe installation path: {value}"
    );
    for part in value.split('/') {
        let base = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&base.as_str())
            || (base.len() == 4
                && (base.starts_with("COM") || base.starts_with("LPT"))
                && matches!(base.as_bytes()[3], b'1'..=b'9'));
        ensure!(
            !part.is_empty()
                && part != "."
                && part != ".."
                && !part.ends_with(['.', ' '])
                && !reserved
                && !part
                    .chars()
                    .any(|c| c.is_control() || "<>\"|?*".contains(c)),
            "Unsafe installation path: {value}"
        );
    }
    Ok(())
}
fn linked(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.is_symlink()
    }
}
pub async fn safe_path(root: &Path, relative: &str) -> Result<PathBuf> {
    validate_relative(relative)?;
    fs::create_dir_all(root).await?;
    ensure!(
        !linked(&fs::symlink_metadata(root).await?),
        "Linked storage roots are not supported"
    );
    let mut path = root.to_path_buf();
    for part in relative.split('/') {
        path.push(part);
        if let Ok(metadata) = fs::symlink_metadata(&path).await {
            ensure!(
                !linked(&metadata),
                "Linked paths are forbidden: {}",
                path.display()
            );
        }
    }
    Ok(path)
}
pub async fn atomic_replace(from: &Path, to: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
        let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
        tokio::task::spawn_blocking(move || {
            // Both strings are owned, NUL-terminated UTF-16 and remain alive for the call.
            if unsafe {
                MoveFileExW(
                    from.as_ptr(),
                    to.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            } == 0
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        })
        .await??;
    }
    #[cfg(not(windows))]
    fs::rename(from, to).await?;
    Ok(())
}
pub async fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    fs::create_dir_all(path.parent().context("Missing parent directory")?).await?;
    let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    fs::write(&temporary, data).await?;
    let result = atomic_replace(&temporary, path).await;
    if result.is_err() {
        fs::remove_file(&temporary).await.ok();
    }
    result
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Loader {
    #[default]
    Vanilla,
    Fabric,
    Forge,
}
impl std::fmt::Display for Loader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Instance {
    pub schema: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub loader: Loader,
    pub status: String,
    pub detail: String,
}
#[derive(Clone)]
pub struct Store {
    pub root: PathBuf,
}
impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn default_root() -> PathBuf {
        std::env::var_os("AERO_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("USERPROFILE")
                    .or_else(|| std::env::var_os("HOME"))
                    .unwrap_or_else(|| ".".into());
                PathBuf::from(home).join(".aero-client/data")
            })
    }
    pub async fn directory(&self, id: &str) -> Result<PathBuf> {
        ensure!(
            Uuid::parse_str(id)?.to_string() == id,
            "Invalid instance identity"
        );
        safe_path(&self.root.join("instances"), id).await
    }
    pub async fn save(&self, instance: &Instance) -> Result<()> {
        let file = safe_path(&self.directory(&instance.id).await?, "instance.json").await?;
        write_atomic(&file, &serde_json::to_vec_pretty(instance)?).await
    }
    pub async fn read(&self, id: &str) -> Result<Instance> {
        let path = safe_path(&self.directory(id).await?, "instance.json").await?;
        let mut instance: Instance = serde_json::from_slice(&fs::read(path).await?)?;
        ensure!(
            instance.schema == 1
                && instance.id == id
                && crate::TARGETS.contains(&instance.version.as_str())
                && !instance.name.is_empty()
                && instance.name.len() <= 160,
            "Invalid instance manifest"
        );
        if instance.status == "INSTALLING" {
            instance.status = "INTERRUPTED".into();
            instance.detail = "Select Repair to resume".into();
        }
        Ok(instance)
    }
    pub async fn list(&self) -> Result<Vec<Instance>> {
        let root = self.root.join("instances");
        if !fs::try_exists(&root).await? {
            return Ok(vec![]);
        }
        let mut directory = fs::read_dir(&root).await?;
        let mut result = vec![];
        while let Some(entry) = directory.next_entry().await? {
            if let Some(id) = entry.file_name().to_str() {
                if Uuid::parse_str(id).is_ok() {
                    result.push(self.read(id).await?);
                }
            }
        }
        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }
    pub async fn create(
        &self,
        name: &str,
        version: &str,
        loader: Loader,
        default_instance: bool,
    ) -> Result<Instance> {
        ensure!(
            !name.trim().is_empty() && name.len() <= 160 && !name.chars().any(char::is_control),
            "Use a printable instance name (up to 160 bytes)"
        );
        ensure!(
            crate::TARGETS.contains(&version),
            "Version is outside supported targets"
        );
        ensure!(
            loader != Loader::Forge || version == "1.8.9",
            "Legacy Forge requires 1.8.9"
        );
        ensure!(
            loader != Loader::Fabric || version != "1.8.9",
            "Use Vanilla or Forge for 1.8.9"
        );
        if default_instance {
            if let Some(existing) = self.list().await?.into_iter().find(|i| {
                i.version == version && i.loader == loader && i.name.starts_with("Aero Default")
            }) {
                return Ok(existing);
            }
        }
        let id = if default_instance {
            Uuid::new_v5(
                &Uuid::NAMESPACE_DNS,
                format!("aero-default:{version}:{loader}").as_bytes(),
            )
        } else {
            Uuid::new_v4()
        }
        .to_string();
        let directory = self.directory(&id).await?;
        if fs::try_exists(directory.join("instance.json")).await? {
            return self.read(&id).await;
        }
        for folder in ["saves", "config", "mods", "resourcepacks", "shaderpacks"] {
            fs::create_dir_all(safe_path(&directory, &format!("game/{folder}")).await?).await?;
        }
        let instance = Instance {
            schema: 1,
            id,
            name: name.trim().into(),
            version: version.into(),
            loader,
            status: "NEW".into(),
            detail: "Not installed".into(),
        };
        self.save(&instance).await?;
        Ok(instance)
    }
    pub async fn status(
        &self,
        instance: &Instance,
        status: &str,
        detail: &str,
    ) -> Result<Instance> {
        let mut updated = instance.clone();
        updated.status = status.into();
        updated.detail = detail.into();
        self.save(&updated).await?;
        Ok(updated)
    }
    /// Copies user content to a new identity. Never follows links or shares writable files.
    pub async fn duplicate(&self, id: &str, cancel: &crate::CancellationToken) -> Result<Instance> {
        let original = self.read(id).await?;
        let copy = self
            .create(
                &format!("{} (copy)", original.name),
                &original.version,
                original.loader,
                false,
            )
            .await?;
        let source = self.directory(id).await?;
        let destination = self.directory(&copy.id).await?;
        let result: Result<()> = async {
            let mut pending = vec!["game".to_string()];
            let mut count = 0usize;
            while let Some(relative) = pending.pop() {
                ensure!(!cancel.is_cancelled(), "Copy cancelled");
                let from = safe_path(&source, &relative).await?;
                let to = safe_path(&destination, &relative).await?;
                let metadata = fs::symlink_metadata(&from).await?;
                if metadata.is_dir() {
                    fs::create_dir_all(&to).await?;
                    let mut entries = fs::read_dir(from).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        count += 1;
                        ensure!(
                            count <= 100_000,
                            "Instance has too many files to duplicate safely"
                        );
                        let name = entry
                            .file_name()
                            .into_string()
                            .map_err(|_| anyhow::anyhow!("Non-Unicode filename"))?;
                        pending.push(format!("{relative}/{name}"));
                    }
                } else {
                    ensure!(metadata.is_file(), "Unsupported instance file");
                    if let Some(parent) = to.parent() {
                        fs::create_dir_all(parent).await?;
                    }
                    tokio::select! {
                        _ = cancel.cancelled() => anyhow::bail!("Copy cancelled"),
                        copied = fs::copy(&from, &to) => { copied?; }
                    }
                }
            }
            Ok(())
        }
        .await;
        if let Err(error) = result {
            let _ = self.quarantine(&copy.id).await;
            return Err(error);
        }
        Ok(copy)
    }
    pub async fn rename(&self, id: &str, name: &str) -> Result<()> {
        ensure!(
            !name.trim().is_empty() && name.len() <= 160 && !name.chars().any(char::is_control),
            "Invalid name"
        );
        let mut instance = self.read(id).await?;
        instance.name = name.trim().into();
        self.save(&instance).await
    }
    /// Explicitly confirmed removal is reversible: never delete saves in M2-R.
    pub async fn quarantine(&self, id: &str) -> Result<PathBuf> {
        let source = self.directory(id).await?;
        self.read(id).await?;
        let destination = safe_path(
            &self.root,
            &format!(
                "trash/{id}-{}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_millis()
            ),
        )
        .await?;
        fs::create_dir_all(destination.parent().unwrap()).await?;
        fs::rename(source, &destination).await?;
        Ok(destination)
    }
    pub async fn cached_json(
        &self,
        client: &crate::DownloadClient,
        key: &str,
        url: &str,
        refresh: bool,
        cancel: &crate::CancellationToken,
    ) -> Result<serde_json::Value> {
        let file = safe_path(&self.root.join("metadata"), &format!("{key}.json")).await?;
        if !refresh {
            if let Ok(metadata) = fs::metadata(&file).await {
                if metadata.modified()?.elapsed().unwrap_or_default().as_secs() < 21600 {
                    if let Ok(value) = serde_json::from_slice(&fs::read(&file).await?) {
                        return Ok(value);
                    }
                }
            }
        }
        let bytes = client.metadata(url, cancel).await?;
        let value = serde_json::from_slice(&bytes).context("Invalid remote JSON")?;
        write_atomic(&file, &bytes).await?;
        Ok(value)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub appearance: String,
    pub memory_mib: u32,
    pub custom_java: String,
    pub low_performance: bool,
    pub reduced_motion: bool,
    pub show_news: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            appearance: "Aero Light".into(),
            memory_mib: 4096,
            custom_java: String::new(),
            low_performance: false,
            reduced_motion: true,
            show_news: true,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (1024..=32768).contains(&self.memory_mib),
            "Memory must be 1024–32768 MiB"
        );
        ensure!(
            ["Aero Light", "Aero Dark", "Aero Lite"].contains(&self.appearance.as_str()),
            "Unknown appearance preset"
        );
        Ok(())
    }
    pub async fn load(root: &Path) -> Result<Self> {
        let file = root.join("launcher-settings.json");
        if fs::try_exists(&file).await? {
            let value: Self = serde_json::from_slice(&fs::read(file).await?)?;
            value.validate()?;
            return Ok(value);
        }
        let mut result = Self::default();
        if let Ok(text) = fs::read_to_string(root.join("settings.properties")).await {
            for line in text.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    match key {
                        "memoryMiB" => {
                            result.memory_mib =
                                value.parse().context("Invalid legacy memory setting")?
                        }
                        "theme" if value == "DARK" => result.appearance = "Aero Dark".into(),
                        _ => {}
                    }
                }
            }
        }
        result.custom_java = fs::read_to_string(root.join("custom-java.txt"))
            .await
            .unwrap_or_default()
            .trim()
            .into();
        result.validate()?;
        Ok(result)
    }
    pub async fn save(&self, root: &Path) -> Result<()> {
        self.validate()?;
        write_atomic(
            &root.join("launcher-settings.json"),
            &serde_json::to_vec_pretty(self)?,
        )
        .await
    }
}
