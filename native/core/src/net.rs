// SPDX-License-Identifier: GPL-3.0-only
use crate::storage::{atomic_replace, safe_path};
use anyhow::{Context, Result, bail, ensure};
use futures_util::{StreamExt, stream};
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Progress {
    pub item: String,
    pub bytes: u64,
    pub total: u64,
    pub bytes_per_second: u64,
    pub files_done: usize,
    pub files_total: usize,
}
pub type Reporter = Arc<dyn Fn(Progress) + Send + Sync>;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub url: String,
    pub sha1: String,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub alternatives: Vec<String>,
}
impl Artifact {
    pub fn from_json(v: &serde_json::Value) -> Result<Self> {
        let artifact = Self {
            url: v["url"].as_str().context("Missing download URL")?.into(),
            sha1: v["sha1"].as_str().context("Missing checksum")?.into(),
            size: v["size"].as_u64(),
            alternatives: vec![],
        };
        artifact.validate()?;
        Ok(artifact)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            valid_hash(&self.sha1) && self.alternatives.iter().all(|h| valid_hash(h)),
            "Invalid artifact checksum"
        );
        Ok(())
    }
}
pub fn valid_hash(h: &str) -> bool {
    h.len() == 40 && h.bytes().all(|b| b.is_ascii_hexdigit())
}

#[derive(Clone)]
pub struct DownloadClient {
    client: Client,
    test_origin: Option<String>,
    report: Reporter,
    throttle: Arc<Mutex<Instant>>,
}
impl DownloadClient {
    pub fn new(report: Reporter) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(20))
                .read_timeout(Duration::from_secs(30))
                .user_agent(
                    "AeroClient/1.0.0-beta.1 (+https://github.com/karmangamerz-coder/Aero-client)",
                )
                .build()?,
            test_origin: None,
            report,
            throttle: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1))),
        })
    }
    #[cfg(test)]
    pub fn local(origin: String) -> Self {
        let mut result = Self::new(Arc::new(|_| {})).unwrap();
        result.test_origin = Some(origin);
        result
    }
    pub fn validate_url(&self, value: &str) -> Result<Url> {
        let url = Url::parse(value)?;
        let hosts = [
            "piston-meta.mojang.com",
            "piston-data.mojang.com",
            "launchermeta.mojang.com",
            "launcher.mojang.com",
            "libraries.minecraft.net",
            "resources.download.minecraft.net",
            "meta.fabricmc.net",
            "maven.fabricmc.net",
            "maven.minecraftforge.net",
            "files.minecraftforge.net",
            "repo.maven.apache.org",
            "api.modrinth.com",
            "cdn.modrinth.com",
            "api.github.com",
        ];
        let local = self
            .test_origin
            .as_ref()
            .is_some_and(|origin| url.origin().ascii_serialization() == *origin);
        ensure!(
            local
                || (url.scheme() == "https"
                    && hosts.contains(&url.host_str().unwrap_or(""))
                    && url.port_or_known_default() == Some(443)),
            "Untrusted distribution endpoint: {value}"
        );
        ensure!(
            url.username().is_empty() && url.password().is_none() && url.fragment().is_none(),
            "Credentials/fragments are forbidden in download URLs"
        );
        Ok(url)
    }
    async fn response(
        &self,
        value: &str,
        offset: u64,
        cancel: &CancellationToken,
    ) -> Result<reqwest::Response> {
        let mut url = self.validate_url(value)?;
        for _ in 0..5 {
            let mut request = self.client.get(url.clone());
            if offset > 0 {
                request = request.header("Range", format!("bytes={offset}-"));
            }
            let response = tokio::select! { _ = cancel.cancelled() => bail!("Cancelled; Retry resumes partial files"), value = request.send() => value? };
            if response.status().is_redirection() {
                url = self.validate_url(
                    url.join(
                        response
                            .headers()
                            .get("Location")
                            .context("Missing redirect destination")?
                            .to_str()?,
                    )?
                    .as_str(),
                )?;
            } else {
                return Ok(response);
            }
        }
        bail!("Too many redirects")
    }
    pub async fn metadata(&self, url: &str, cancel: &CancellationToken) -> Result<Vec<u8>> {
        let mut failure = None;
        for attempt in 0..3 {
            let result = async {
                let response = self.response(url, 0, cancel).await?;
                if response.status() == StatusCode::TOO_MANY_REQUESTS {
                    let retry_after = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or(2)
                        .clamp(1, 30);
                    tokio::select! {
                        _ = cancel.cancelled() => bail!("Cancelled"),
                        _ = tokio::time::sleep(Duration::from_secs(retry_after)) => {}
                    }
                    bail!("Service rate limit reached; waited {retry_after}s before retrying")
                }
                ensure!(response.status() == StatusCode::OK, "Metadata HTTP {} from {url}", response.status());
                let mut stream = response.bytes_stream(); let mut output = Vec::new();
                loop {
                    let chunk = tokio::select! { _ = cancel.cancelled() => bail!("Cancelled"), next = stream.next() => next };
                    match chunk { Some(chunk) => { let chunk = chunk?; ensure!(output.len() + chunk.len() <= 8 * 1024 * 1024, "Metadata exceeds size limit"); output.extend_from_slice(&chunk); }, None => break }
                }
                Ok::<_, anyhow::Error>(output)
            }.await;
            match result {
                Ok(bytes) => return Ok(bytes),
                Err(error) => failure = Some(error),
            }
            self.retry_delay(attempt, cancel).await?;
        }
        Err(failure.unwrap()).context("Metadata unavailable; check your connection and retry")
    }
    async fn retry_delay(&self, attempt: u32, cancel: &CancellationToken) -> Result<()> {
        ensure!(
            !cancel.is_cancelled(),
            "Cancelled; Retry resumes partial files"
        );
        if attempt < 2 {
            tokio::select! { _ = cancel.cancelled() => bail!("Cancelled"), _ = tokio::time::sleep(Duration::from_millis(300 * (attempt + 1) as u64)) => {} }
        }
        Ok(())
    }
    pub async fn hash(path: &Path, cancel: &CancellationToken) -> Result<String> {
        let mut file = fs::File::open(path).await?;
        let mut buffer = vec![0; 65536];
        let mut digest = Sha1::new();
        loop {
            ensure!(!cancel.is_cancelled(), "Cancelled");
            let size = file.read(&mut buffer).await?;
            if size == 0 {
                break;
            }
            digest.update(&buffer[..size]);
        }
        Ok(hex::encode(digest.finalize()))
    }
    pub async fn valid(
        path: &Path,
        artifact: &Artifact,
        cancel: &CancellationToken,
    ) -> Result<bool> {
        let metadata = match fs::symlink_metadata(path).await {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.into()),
        };
        if !metadata.is_file()
            || metadata.is_symlink()
            || artifact.size.is_some_and(|size| size != metadata.len())
        {
            return Ok(false);
        }
        let hash = Self::hash(path, cancel).await?;
        Ok(hash.eq_ignore_ascii_case(&artifact.sha1)
            || artifact
                .alternatives
                .iter()
                .any(|h| h.eq_ignore_ascii_case(&hash)))
    }
    fn progress(&self, progress: Progress) {
        let mut last = self.throttle.lock().unwrap();
        if last.elapsed() >= Duration::from_millis(120) {
            *last = Instant::now();
            (self.report)(progress);
        }
    }
    pub async fn fetch(
        &self,
        artifact: &Artifact,
        root: &Path,
        relative: &str,
        cancel: &CancellationToken,
    ) -> Result<PathBuf> {
        artifact.validate()?;
        self.validate_url(&artifact.url)?;
        let target = safe_path(root, relative).await?;
        if Self::valid(&target, artifact, cancel).await? {
            return Ok(target);
        }
        fs::create_dir_all(target.parent().unwrap()).await?;
        let part = safe_path(root, &format!("{relative}.part")).await?;
        let mut failure = None;
        for attempt in 0..3 {
            let result = async {
                ensure!(!cancel.is_cancelled(), "Cancelled");
                let mut offset = fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
                if artifact.size.is_some_and(|size| offset >= size) { fs::remove_file(&part).await.ok(); offset = 0; }
                let response = self.response(&artifact.url, offset, cancel).await?;
                if response.status() == StatusCode::RANGE_NOT_SATISFIABLE { fs::remove_file(&part).await.ok(); bail!("Restarting stale partial file"); }
                ensure!(response.status() == StatusCode::OK || response.status() == StatusCode::PARTIAL_CONTENT, "Download HTTP {}: {relative}", response.status());
                if response.status() == StatusCode::PARTIAL_CONTENT {
                    ensure!(response.headers().get("Content-Range").and_then(|h| h.to_str().ok()).unwrap_or("").starts_with(&format!("bytes {offset}-")), "Invalid resume range");
                } else { offset = 0; }
                let mut output = fs::OpenOptions::new().create(true).write(true).append(offset > 0).truncate(offset == 0).open(&part).await?;
                let start = Instant::now(); let initial = offset; let mut stream = response.bytes_stream();
                loop {
                    let chunk = tokio::select! { _ = cancel.cancelled() => bail!("Cancelled"), next = stream.next() => next };
                    match chunk {
                        Some(chunk) => { let chunk = chunk?; offset += chunk.len() as u64;
                            ensure!(offset <= artifact.size.unwrap_or(1024 * 1024 * 1024), "Download exceeds expected size");
                            output.write_all(&chunk).await?;
                            self.progress(Progress { item: relative.into(), bytes: offset, total: artifact.size.unwrap_or(0), bytes_per_second: ((offset - initial) as f64 / start.elapsed().as_secs_f64().max(0.01)) as u64, ..Default::default() });
                        }, None => break
                    }
                }
                output.flush().await?; output.sync_all().await?; drop(output);
                if !Self::valid(&part, artifact, cancel).await? { fs::remove_file(&part).await?; bail!("Integrity mismatch: {relative}"); }
                safe_path(root, relative).await?; atomic_replace(&part, &target).await?; Ok::<_, anyhow::Error>(target.clone())
            }.await;
            match result {
                Ok(path) => return Ok(path),
                Err(error) => failure = Some(error),
            }
            self.retry_delay(attempt, cancel).await?;
        }
        Err(failure.unwrap()).with_context(|| {
            format!("Failed {relative}; check disk space/connection and select Repair")
        })
    }
    pub async fn batch(
        &self,
        requests: Vec<(Artifact, PathBuf, String)>,
        cancel: &CancellationToken,
    ) -> Result<()> {
        let total = requests.len();
        let mut done = 0;
        let mut pending = stream::iter(requests.into_iter().map(|(artifact, root, path)| {
            let client = self.clone();
            let cancel = cancel.clone();
            async move { client.fetch(&artifact, &root, &path, &cancel).await }
        }))
        .buffer_unordered(3);
        while let Some(result) = pending.next().await {
            result?;
            done += 1;
            self.progress(Progress {
                item: format!("Verified {done} of {total} files"),
                files_done: done,
                files_total: total,
                ..Default::default()
            });
        }
        Ok(())
    }
}
