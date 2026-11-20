// SPDX-License-Identifier: MIT
use crate::{
    Artifact, CancellationToken, DownloadClient, RUNTIME_MANIFEST, Store, storage::safe_path,
};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{fs, io::AsyncReadExt, process::Command};

#[derive(Debug)]
pub struct JavaInfo {
    pub major: u32,
    pub x64: bool,
}
pub fn major_version(version: &str) -> Result<u32> {
    Ok(version
        .strip_prefix("1.")
        .unwrap_or(version)
        .split(['.', '_', '+', '-'])
        .next()
        .context("Missing Java version")?
        .parse()?)
}
pub async fn probe(path: &Path) -> Result<JavaInfo> {
    let mut command = Command::new(path);
    command
        .args(["-XshowSettings:properties", "-version"])
        .kill_on_drop(true)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let mut process = command.spawn().context("Cannot run Java version check")?;
    let output = tokio::time::timeout(Duration::from_secs(8), async {
        let mut bytes = vec![];
        process
            .stderr
            .take()
            .context("Java output unavailable")?
            .take(65537)
            .read_to_end(&mut bytes)
            .await?;
        ensure!(bytes.len() <= 65536, "Java version output exceeded limit");
        ensure!(process.wait().await?.success(), "Java version check failed");
        Ok::<_, anyhow::Error>(String::from_utf8(bytes)?)
    })
    .await
    .context("Java version check timed out")??;
    let property = |key: &str| {
        output.lines().find_map(|line| {
            line.trim()
                .split_once('=')
                .filter(|(name, _)| name.trim() == key)
                .map(|(_, value)| value.trim().to_owned())
        })
    };
    Ok(JavaInfo {
        major: major_version(&property("java.version").context("Cannot identify Java version")?)?,
        x64: property("os.arch").is_some_and(|arch| arch == "amd64" || arch == "x86_64"),
    })
}
pub async fn candidates() -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for variable in ["JAVA_HOME", "JDK_HOME"] {
        if let Some(path) = std::env::var_os(variable) {
            paths.insert(PathBuf::from(path).join("bin/java.exe"));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            paths.insert(directory.join("java.exe"));
        }
    }
    for root in [
        "C:/Program Files/Java",
        "C:/Program Files/Eclipse Adoptium",
        "C:/Program Files/Microsoft",
    ] {
        if let Ok(mut entries) = fs::read_dir(root).await {
            let mut count = 0;
            while let Ok(Some(entry)) = entries.next_entry().await {
                paths.insert(entry.path().join("bin/java.exe"));
                count += 1;
                if count >= 30 {
                    break;
                }
            }
        }
    }
    paths.into_iter().collect()
}
pub async fn ensure_runtime(
    store: &Store,
    client: &DownloadClient,
    requirement: &Value,
    custom: Option<&Path>,
    cancel: &CancellationToken,
) -> Result<PathBuf> {
    let major = requirement["majorVersion"]
        .as_u64()
        .context("Game metadata does not specify required Java")? as u32;
    let mut options = Vec::new();
    if let Some(path) = custom {
        options.push(path.to_path_buf());
    }
    options.extend(candidates().await);
    for path in options {
        ensure!(!cancel.is_cancelled(), "Cancelled");
        if fs::try_exists(&path).await.unwrap_or(false) {
            if let Ok(info) = probe(&path).await {
                if info.major == major && info.x64 {
                    return Ok(path);
                }
            }
        }
    }
    let component = requirement["component"]
        .as_str()
        .context("Missing Java component; refusing to guess")?;
    let index = store
        .cached_json(client, "runtimes", RUNTIME_MANIFEST, false, cancel)
        .await?;
    let selected = index["windows-x64"][component]
        .as_array()
        .and_then(|list| list.first())
        .context("No official Windows x64 runtime for this version")?;
    let manifest = Artifact::from_json(&selected["manifest"])?;
    let root = store.root.join("runtimes");
    let file = client
        .fetch(
            &manifest,
            &root,
            &format!("manifests/{}.json", manifest.sha1),
            cancel,
        )
        .await?;
    let description: Value = serde_json::from_slice(&fs::read(file).await?)?;
    let runtime = safe_path(&root, &format!("{component}/{}", manifest.sha1)).await?;
    let mut requests = vec![];
    for (relative, file) in description["files"]
        .as_object()
        .context("Invalid runtime manifest")?
    {
        ensure!(!cancel.is_cancelled(), "Cancelled");
        let target = safe_path(&runtime, relative).await?;
        match file["type"].as_str() {
            Some("directory") => fs::create_dir_all(target).await?,
            Some("file") => requests.push((
                Artifact::from_json(&file["downloads"]["raw"])?,
                runtime.clone(),
                relative.clone(),
            )),
            _ => bail!("Unsupported runtime link/type; no downloaded code was run"),
        }
    }
    client.batch(requests, cancel).await?;
    let executable = safe_path(&runtime, "bin/java.exe").await?;
    let info = probe(&executable).await?;
    ensure!(
        info.major == major && info.x64,
        "Downloaded Java does not satisfy the game metadata"
    );
    Ok(executable)
}
