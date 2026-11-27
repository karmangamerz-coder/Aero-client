// SPDX-License-Identifier: GPL-3.0-only
//! Authenticated game launching: command line resolution, Java selection, process
//! supervision, live logs and crash reporting.
//!
//! Aero only launches installations it verified itself, with the official access
//! token for an account whose ownership was checked. Nothing here is executed from
//! remote metadata: the command is built from the verified version manifest and the
//! plan written during installation.
use crate::{
    Account, CancellationToken, Installer, Instance, Store, runtime,
    storage::{safe_path, write_atomic},
};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{fs, io::AsyncBufReadExt, process::Child};

pub const LAUNCHER_BRAND: &str = "Aero Client";
pub const LAUNCHER_VERSION: &str = "2.1.0";

#[derive(Clone)]
pub struct LaunchSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub directory: PathBuf,
}

impl LaunchSpec {
    pub fn command_line(&self) -> String {
        format!("{} {}", self.program.display(), self.describe())
    }
    /// Safe for logs: no token, no session id.
    pub fn describe(&self) -> String {
        let mut filtered = vec![];
        let mut skip = false;
        for arg in &self.args {
            if skip {
                skip = false;
                filtered.push("<redacted>".to_owned());
                continue;
            }
            if matches!(
                arg.as_str(),
                "--accessToken" | "--clientId" | "--auth-xuid" | "--session"
            ) {
                filtered.push(arg.clone());
                skip = true;
                continue;
            }
            if arg.starts_with("--accessToken=") {
                filtered.push("--accessToken=<redacted>".into());
                continue;
            }
            filtered.push(arg.clone());
        }
        filtered.join(" ")
    }
}

#[cfg(test)]
mod session_log_tests {
    use super::*;
    #[test]
    fn legacy_and_modern_session_arguments_are_redacted_from_diagnostics() {
        let spec = LaunchSpec {
            program: "java".into(),
            directory: ".".into(),
            args: vec![
                "--accessToken".into(),
                "synthetic-token".into(),
                "--session".into(),
                "legacy-session".into(),
            ],
        };
        assert!(spec.args.contains(&"synthetic-token".to_owned()));
        for output in [spec.describe(), spec.command_line(), format!("{spec:?}")] {
            assert!(!output.contains("synthetic-token"));
            assert!(!output.contains("legacy-session"));
        }
        assert_eq!(
            redact_line("output synthetic-token", &["synthetic-token".into()]),
            "output <redacted>"
        );
    }
}

impl std::fmt::Debug for LaunchSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LaunchSpec")
            .field("program", &self.program)
            .field("arguments", &self.describe())
            .finish()
    }
}

/// Mojang metadata names macOS `osx` and only constrains 32-bit x86 installs.
fn platform_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "osx",
        other => other,
    }
}

fn rules_allow(rules: Option<&Value>) -> bool {
    let Some(rules) = rules.and_then(Value::as_array) else {
        return true;
    };
    let mut allowed = false;
    for rule in rules {
        let mut matches = true;
        if let Some(name) = rule["os"]["name"].as_str() {
            matches &= name == platform_name();
        }
        if let Some(arch) = rule["os"]["arch"].as_str() {
            // Aero ships 64-bit only; 32-bit-only rules never apply.
            matches &= arch != "x86";
        }
        if let Some(features) = rule["features"].as_object() {
            // Aero never enables demo mode or custom resolution placeholders here.
            matches &= features
                .keys()
                .all(|key| key == "has_custom_resolution" || key == "is_demo_user")
                && features.values().all(|value| value == false);
        }
        if matches {
            allowed = rule["action"] == "allow";
        }
    }
    allowed
}

fn collect_arguments(block: Option<&Value>, out: &mut Vec<String>) {
    let Some(items) = block.and_then(Value::as_array) else {
        return;
    };
    for item in items {
        match item {
            Value::String(value) => out.push(value.clone()),
            Value::Object(_) => {
                if rules_allow(item.get("rules")) {
                    if let Some(value) = item["value"].as_str() {
                        out.push(value.to_owned());
                    } else if let Some(values) = item["value"].as_array() {
                        out.extend(values.iter().filter_map(|v| v.as_str().map(str::to_owned)));
                    }
                }
            }
            _ => {}
        }
    }
}

fn substitute(value: &str, variables: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find("${") {
        result.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('}') else {
            break;
        };
        let key = &rest[start + 2..start + end];
        result.push_str(variables.get(key).map(String::as_str).unwrap_or(""));
        rest = &rest[start + end + 1..];
    }
    result.push_str(rest);
    result
}

/// Build the exact Java invocation for an installed instance.
pub async fn build(
    installer: &Installer,
    instance: &Instance,
    account: &Account,
    memory_mib: u32,
    custom_java: Option<&Path>,
    cancel: &CancellationToken,
) -> Result<LaunchSpec> {
    ensure!(
        (1024..=32768).contains(&memory_mib),
        "Memory must be 1024–32768 MiB"
    );
    let directory = installer.store.directory(&instance.id).await?;
    let version_file = safe_path(&directory, "installation/version.json").await?;
    ensure!(
        fs::try_exists(&version_file).await.unwrap_or(false),
        "Install the instance before launching it"
    );
    let version: Value = serde_json::from_slice(&fs::read(&version_file).await?)?;
    let plan: Value = serde_json::from_slice(
        &fs::read(safe_path(&directory, "installation/plan.json").await?).await?,
    )
    .context("Installation plan is missing; select Repair")?;
    let loader: Option<Value> =
        match fs::read(safe_path(&directory, "installation/loader.json").await?).await {
            Ok(bytes) => Some(serde_json::from_slice(&bytes)?),
            Err(_) => None,
        };
    let natives = safe_path(&directory, "installation/natives").await?;
    let game = safe_path(&directory, "game").await?;
    let libraries_root = installer.store.root.join("shared/libraries");
    let mut classpath: Vec<String> = plan["libraries"]
        .as_array()
        .context("Installation plan has no libraries")?
        .iter()
        .filter_map(|value| value.as_str())
        .map(|relative| libraries_root.join(relative))
        .map(|path| path.display().to_string())
        .collect();
    classpath.push(
        installer
            .store
            .root
            .join("shared/clients")
            .join(format!(
                "{}.jar",
                plan["clientSha1"]
                    .as_str()
                    .context("Plan has no client jar")?
            ))
            .display()
            .to_string(),
    );
    let separator = if cfg!(windows) { ";" } else { ":" };

    let mut variables = HashMap::new();
    variables.insert("auth_player_name".into(), account.name.clone());
    variables.insert("version_name".into(), instance.version.clone());
    variables.insert("game_directory".into(), game.display().to_string());
    variables.insert(
        "assets_root".into(),
        installer
            .store
            .root
            .join("shared/assets")
            .display()
            .to_string(),
    );
    variables.insert(
        "assets_index_name".into(),
        plan["assetIndexId"].as_str().unwrap_or("legacy").to_owned(),
    );
    variables.insert("auth_uuid".into(), account.uuid.replace('-', ""));
    variables.insert("auth_access_token".into(), account.access_token.clone());
    variables.insert(
        "auth_session".into(),
        format!("token:{}", account.access_token),
    );
    variables.insert("clientid".into(), "aero".into());
    variables.insert("auth_xuid".into(), "0".into());
    variables.insert("user_type".into(), "msa".into());
    variables.insert("version_type".into(), "release".into());
    variables.insert("natives_directory".into(), natives.display().to_string());
    variables.insert("launcher_name".into(), LAUNCHER_BRAND.into());
    variables.insert("launcher_version".into(), LAUNCHER_VERSION.into());
    variables.insert("classpath".into(), classpath.join(separator));
    variables.insert("classpath_separator".into(), separator.into());
    variables.insert(
        "library_directory".into(),
        libraries_root.display().to_string(),
    );
    variables.insert("user_properties".into(), "{}".into());
    variables.insert("resolution_width".into(), "854".into());
    variables.insert("resolution_height".into(), "480".into());

    let mut jvm = vec![
        format!("-Xmx{memory_mib}M"),
        "-Xms512M".to_owned(),
        format!("-Djava.library.path={}", natives.display()),
        format!("-Dminecraft.launcher.brand={LAUNCHER_BRAND}"),
        format!("-Dminecraft.launcher.version={LAUNCHER_VERSION}"),
    ];
    let mut game_args = vec![];
    if let Some(profile) = &loader {
        collect_arguments(profile["arguments"].get("jvm"), &mut jvm);
        collect_arguments(profile["arguments"].get("game"), &mut game_args);
    }
    let mut version_jvm = vec![];
    collect_arguments(version["arguments"].get("jvm"), &mut version_jvm);
    jvm.extend(version_jvm);
    if version["arguments"]["game"].is_array() {
        collect_arguments(version["arguments"].get("game"), &mut game_args);
    } else if let Some(legacy) = version["minecraftArguments"].as_str() {
        game_args.extend(
            legacy
                .split_whitespace()
                .filter(|part| !part.is_empty())
                .map(str::to_owned),
        );
    }
    ensure!(
        !game_args.is_empty(),
        "Version metadata has no launch arguments"
    );
    if !jvm.iter().any(|arg| arg == "-cp" || arg == "-classpath") {
        jvm.push("-cp".into());
        jvm.push("${classpath}".into());
    }
    let main_class = loader
        .as_ref()
        .and_then(|profile| profile["mainClass"].as_str())
        .or_else(|| version["mainClass"].as_str())
        .context("Version metadata has no main class")?
        .to_owned();
    ensure!(
        main_class
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '$'),
        "Invalid main class in metadata"
    );

    let java = runtime::ensure_runtime(
        &installer.store,
        &installer.client,
        &version["javaVersion"],
        custom_java,
        cancel,
    )
    .await?;
    let mut args: Vec<String> = jvm
        .iter()
        .map(|value| substitute(value, &variables))
        .collect();
    args.push(main_class);
    args.extend(game_args.iter().map(|value| substitute(value, &variables)));
    ensure!(
        !args.iter().any(|arg| arg.contains("${")),
        "Version metadata contains unknown launch variables"
    );
    Ok(LaunchSpec {
        program: java,
        args,
        directory: game,
    })
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub exit_code: Option<i32>,
    pub crashed: bool,
    pub crash_report: Option<PathBuf>,
    pub summary: String,
}

pub struct Sink {
    pub on_line: Arc<dyn Fn(String) + Send + Sync>,
    pub log_file: PathBuf,
    pub secrets: Vec<String>,
}

fn redact_line(line: &str, secrets: &[String]) -> String {
    let mut line = line.to_owned();
    for secret in secrets.iter().filter(|secret| !secret.is_empty()) {
        line = line.replace(secret, "<redacted>");
    }
    line
}

pub struct Running {
    pub pid: u32,
    pub child: Arc<tokio::sync::Mutex<Child>>,
    pub killed: Arc<AtomicBool>,
}

impl Running {
    pub async fn kill(&self) {
        self.killed.store(true, Ordering::SeqCst);
        let mut child = self.child.lock().await;
        if let Err(error) = child.start_kill() {
            tracing_kill_failure(&error);
        }
    }
}

fn tracing_kill_failure(error: &std::io::Error) {
    // Never panic on a kill race: the process may already be gone.
    let _ = error;
}

pub async fn spawn(spec: &LaunchSpec) -> Result<Running> {
    ensure!(
        fs::try_exists(&spec.program).await.unwrap_or(false),
        "The selected Java runtime is no longer available"
    );
    let mut command = tokio::process::Command::new(&spec.program);
    command
        .args(&spec.args)
        .current_dir(&spec.directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(false);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let child = command.spawn().context("Cannot start the game process")?;
    let pid = child.id().unwrap_or_default();
    Ok(Running {
        pid,
        child: Arc::new(tokio::sync::Mutex::new(child)),
        killed: Arc::new(AtomicBool::new(false)),
    })
}

/// Read both output streams until the game exits, forwarding every line and
/// writing a session log. Runs on the Tokio runtime, never on the UI thread.
pub async fn supervise(
    running: Running,
    sink: Sink,
    cancel: &CancellationToken,
) -> Result<Outcome> {
    if let Some(parent) = sink.log_file.parent() {
        fs::create_dir_all(parent).await.ok();
    }
    let file = Arc::new(tokio::sync::Mutex::new(
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&sink.log_file)
            .await
            .ok(),
    ));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(256);
    let (stdout, stderr) = {
        let mut child = running.child.lock().await;
        (child.stdout.take(), child.stderr.take())
    };
    let mut crashed = false;
    for stream in [stdout.map(StreamKind::Out), stderr.map(StreamKind::Err)] {
        let Some(stream) = stream else { continue };
        let tx = tx.clone();
        tokio::spawn(async move {
            let reader: Box<dyn tokio::io::AsyncBufRead + Unpin + Send> = match stream {
                StreamKind::Out(stdout) => Box::new(tokio::io::BufReader::new(stdout)),
                StreamKind::Err(stderr) => Box::new(tokio::io::BufReader::new(stderr)),
            };
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(line).await.is_err() {
                    break;
                }
            }
        });
    }
    drop(tx);
    let mut exit: Option<std::process::ExitStatus> = None;
    loop {
        if exit.is_none() {
            let mut child = running.child.lock().await;
            exit = child.try_wait()?;
        }
        tokio::select! {
            line = rx.recv() => {
                match line {
                    Some(line) => {
                        if line.len() > 4096 {
                            continue;
                        }
                        let line = redact_line(&line, &sink.secrets);
                        if line.contains("---- Minecraft Crash Report ----")
                            || line.contains("Minecraft has crashed")
                        {
                            crashed = true;
                        }
                        (sink.on_line)(line.clone());
                        if let Some(file) = file.lock().await.as_mut() {
                            use tokio::io::AsyncWriteExt;
                            let _ = file.write_all(format!("{line}\n").as_bytes()).await;
                        }
                    }
                    None => {}
                }
            }
            _ = cancel.cancelled() => {
                let mut child = running.child.lock().await;
                child.start_kill().ok();
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                exit = Some(child.wait().await?);
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(250)), if exit.is_none() => {}
        }
        if let Some(_status) = exit {
            // Drain any remaining buffered output before finishing.
            while let Ok(line) = rx.try_recv() {
                if line.len() <= 4096 {
                    (sink.on_line)(redact_line(&line, &sink.secrets));
                }
            }
            break;
        }
    }
    let code = exit.and_then(|status| status.code());
    let summary = if crashed {
        "Minecraft crashed. The crash report is saved with the instance.".to_owned()
    } else if code.unwrap_or(1) == 0 {
        "Minecraft closed normally.".to_owned()
    } else {
        format!("Minecraft exited with code {}.", code.unwrap_or(-1))
    };
    Ok(Outcome {
        exit_code: code,
        crashed: crashed || code.unwrap_or(0) != 0,
        crash_report: None,
        summary,
    })
}

enum StreamKind {
    Out(tokio::process::ChildStdout),
    Err(tokio::process::ChildStderr),
}

/// Newest crash report in the instance directory, if the game wrote one.
pub async fn crash_reports(directory: &Path) -> Result<Option<PathBuf>> {
    let reports = directory.join("game/crash-reports");
    let Ok(mut entries) = fs::read_dir(&reports).await else {
        return Ok(None);
    };
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("txt") {
            continue;
        }
        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
            newest = Some((modified, entry.path()));
        }
    }
    Ok(newest.map(|(_, path)| path))
}

impl Installer {
    /// Launch an installed instance with a signed-in account.
    pub async fn launch(
        &self,
        instance: &Instance,
        account: &Account,
        memory_mib: u32,
        custom_java: Option<&Path>,
        cancel: &CancellationToken,
        on_line: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<Outcome> {
        ensure!(
            cfg!(all(target_os = "windows", target_arch = "x86_64")),
            "Game launching currently supports Windows x64"
        );
        ensure!(
            account.owns_minecraft,
            "This account does not own Minecraft"
        );
        ensure!(account.is_active_token(), "Sign in again to continue");
        let instance = self.store.read(&instance.id).await?;
        ensure!(
            instance.status == "READY",
            "Install the instance before launching it"
        );
        let spec = build(self, &instance, account, memory_mib, custom_java, cancel).await?;
        let directory = self.store.directory(&instance.id).await?;
        let log_file = safe_path(&directory, "logs/latest.log").await?;
        write_atomic(
            &safe_path(&directory, "logs/last-command.txt").await?,
            spec.describe().as_bytes(),
        )
        .await
        .ok();
        let running = spawn(&spec).await?;
        let sink = Sink {
            on_line,
            log_file,
            secrets: vec![
                account.access_token.clone(),
                account.ms.access_token.clone(),
                account.ms.refresh_token.clone(),
            ],
        };
        let mut outcome = supervise(running, sink, cancel).await?;
        if let Some(report) = crash_reports(&directory).await? {
            outcome.crash_report = Some(report);
        }
        Ok(outcome)
    }
}

/// Marker so the launcher can show the login state without owning account logic.
pub fn account_label(account: Option<&Account>) -> String {
    account
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "No Account".into())
}

/// Read-only helper used by the UI to display the resolved memory value.
pub fn effective_memory(instance: &Instance, settings_mib: u32) -> u32 {
    instance.memory_mib.unwrap_or(settings_mib)
}

/// Small wrapper kept public for tests: the plan must describe an authenticated
/// launch, never an offline one.
pub async fn plan_requires_auth(store: &Store, instance: &Instance) -> Result<bool> {
    let plan = serde_json::from_slice::<Value>(
        &fs::read(
            safe_path(
                &store.directory(&instance.id).await?,
                "installation/plan.json",
            )
            .await?,
        )
        .await?,
    )?;
    Ok(plan["authenticationRequired"] == true)
}

#[cfg(test)]
pub fn substitute_for_test(value: &str, vars: &[(&str, &str)]) -> String {
    substitute(
        value,
        &vars
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect(),
    )
}

#[cfg(test)]
pub fn rules_for_test(rules: Option<&Value>) -> bool {
    rules_allow(rules)
}
