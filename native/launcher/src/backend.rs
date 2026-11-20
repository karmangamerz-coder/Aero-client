// SPDX-License-Identifier: MIT
//! Services bridged to the UI thread.
//!
//! Exactly one interactive command runs at a time on a two-thread Tokio runtime;
//! sign-in polling and the game process run as supervised tasks so the queue stays
//! responsive (and can be cancelled). The UI thread only publishes snapshots through
//! a watch channel, so rendering never blocks on disk, network or process work.
use aero_core::{
    Account, Accounts, CancellationToken, Hardware, Installer, Instance, Loader, Progress,
    Settings, Store, auth, content, hardware, launch, onboarding,
    skin::{self, Skin},
};
use std::{
    process::Stdio,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, watch};

const LOG_LIMIT: usize = 120;
const GAME_LOG_LIMIT: usize = 600;

#[derive(Clone)]
pub struct Snapshot {
    pub instances: Vec<Instance>,
    pub versions: Vec<String>,
    pub results: Vec<content::DiscoveryItem>,
    pub settings: Settings,
    pub plan: Option<serde_json::Value>,
    pub log: Vec<String>,
    pub storage: String,
    pub busy: bool,
    pub message: String,
    pub progress: Progress,
    pub accounts: Accounts,
    /// Composed character PNG: the signed-in skin, or the original Aero fallback.
    pub skin: Arc<Vec<u8>>,
    pub hardware: Option<Hardware>,
    pub onboarding: onboarding::Progress,
    pub prompt: Option<auth::DevicePrompt>,
    pub game_logs: Vec<String>,
    pub launching: bool,
    pub signin_state: String,
    /// Currently selected instance (empty means the first one).
    pub selected: String,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            instances: vec![],
            versions: vec![],
            results: vec![],
            settings: Settings::default(),
            plan: None,
            log: vec![],
            storage: String::new(),
            busy: false,
            message: String::new(),
            progress: Progress::default(),
            accounts: Accounts::default(),
            skin: Arc::new(fallback_skin()),
            hardware: None,
            onboarding: onboarding::Progress::default(),
            prompt: None,
            game_logs: vec![],
            launching: false,
            signin_state: String::new(),
            selected: String::new(),
        }
    }
}

pub enum Command {
    RefreshVersions,
    Create(String, String, Loader),
    Rename(String, String),
    Duplicate(String),
    Remove(String),
    Install(String),
    Save(Settings),
    Configure(String, Option<u32>, String),
    LoadPlan(String),
    Discover(String, String, String, String),
    SignIn,
    SwitchAccount(String),
    SignOut,
    RemoveAccount(String),
    Launch(String),
    DetectHardware,
    SaveSetup(onboarding::Progress),
    FinishSetup(onboarding::Progress, bool),
    ReopenSetup,
    Select(String),
    Target(String, Loader),
}

#[derive(Clone)]
pub struct Backend {
    pub tx: mpsc::Sender<Command>,
    pub rx: watch::Receiver<Snapshot>,
    pub cancel: Arc<Mutex<CancellationToken>>,
    pub launch_cancel: Arc<Mutex<CancellationToken>>,
}

impl Backend {
    pub fn send(&self, command: Command) {
        let _ = self.tx.try_send(command);
    }
    pub fn cancel(&self) {
        self.cancel.lock().unwrap().cancel();
    }
    pub fn cancel_launch(&self) {
        self.launch_cancel.lock().unwrap().cancel();
    }

    pub fn start(settings: Settings, accounts: Accounts, onboarding: onboarding::Progress) -> Self {
        let (tx, mut commands) = mpsc::channel(24);
        let initial = Snapshot {
            settings,
            accounts,
            onboarding,
            storage: Store::default_root().display().to_string(),
            message: "Ready.".into(),
            ..Default::default()
        };
        let (state, rx) = watch::channel(initial);
        let cancel = Arc::new(Mutex::new(CancellationToken::new()));
        let launch_cancel = Arc::new(Mutex::new(CancellationToken::new()));
        let shared_cancel = cancel.clone();
        let shared_launch = launch_cancel.clone();
        tokio::spawn(async move {
            let report = state.clone();
            let installer = match Installer::new(
                Store::default_root(),
                Arc::new(move |progress: Progress| {
                    report.send_modify(|s| {
                        if progress.item.is_empty() || progress.item == s.progress.item {
                            s.progress = progress;
                            return;
                        }
                        if !s.progress.item.is_empty() {
                            s.log.push(s.progress.item.clone());
                            if s.log.len() > LOG_LIMIT {
                                s.log.remove(0);
                            }
                        }
                        s.progress = progress;
                    });
                }),
            ) {
                Ok(value) => value,
                Err(error) => {
                    state.send_modify(|s| s.message = format!("Startup failed: {error:#}"));
                    return;
                }
            };
            if let Ok(instances) = installer.store.list().await {
                state.send_modify(|s| s.instances = instances);
            }
            // Character artwork is composed once from the saved account, if any.
            let saved_account = {
                let snapshot = state.borrow();
                snapshot.accounts.active_account().cloned()
            };
            if let Some(account) = saved_account {
                let composed = character_for_client(&account).await;
                state.send_modify(|s| s.skin = composed);
            }
            // Hardware detection runs once, in the background, never on the UI thread.
            {
                let state = state.clone();
                tokio::spawn(async move {
                    let detected = tokio::task::spawn_blocking(hardware::detect).await.ok();
                    state.send_modify(|s| {
                        if let Some(detected) = detected {
                            if s.onboarding.answers.installation_directory.is_empty() {
                                s.hardware = Some(detected);
                            } else {
                                s.hardware = Some(detected);
                            }
                        }
                    });
                });
            }
            while let Some(command) = commands.recv().await {
                if matches!(command, Command::Launch(_)) {
                    let token = CancellationToken::new();
                    *shared_launch.lock().unwrap() = token.clone();
                    let installer = installer.clone();
                    let state = state.clone();
                    let id = match command {
                        Command::Launch(id) => id,
                        _ => unreachable!(),
                    };
                    state.send_modify(|s| {
                        s.launching = true;
                        s.game_logs.clear();
                        s.message = "Starting Minecraft…".into();
                    });
                    tokio::spawn(async move {
                        launch_instance(installer, state, id, token).await;
                    });
                    continue;
                }
                match command {
                    Command::SignIn => {
                        let client = match auth::client() {
                            Ok(client) => client,
                            Err(error) => {
                                state.send_modify(|s| s.message = format!("{error:#}"));
                                continue;
                            }
                        };
                        let state = state.clone();
                        let opener_state = state.clone();
                        let token = CancellationToken::new();
                        *shared_cancel.lock().unwrap() = token.clone();
                        tokio::spawn(async move {
                            let prompt = match auth::begin(&client).await {
                                Ok(prompt) => prompt,
                                Err(error) => {
                                    state.send_modify(|s| {
                                        s.signin_state = format!("Sign-in unavailable: {error:#}");
                                    });
                                    return;
                                }
                            };
                            open_browser(&prompt.verification_uri);
                            state.send_modify(|s| {
                                s.signin_state = format!(
                                    "Enter code {} at {} — waiting for approval…",
                                    prompt.user_code, prompt.verification_uri
                                );
                                s.prompt = Some(prompt.clone());
                            });
                            let tokens = match auth::poll(&client, &prompt, &token).await {
                                Ok(tokens) => tokens,
                                Err(error) => {
                                    state.send_modify(|s| {
                                        s.signin_state = format!("{error:#}");
                                        s.prompt = None;
                                    });
                                    return;
                                }
                            };
                            state.send_modify(|s| {
                                s.signin_state = "Verifying Xbox and Minecraft ownership…".into()
                            });
                            let (account, _profile) = match auth::finish(&client, &tokens).await {
                                Ok(value) => value,
                                Err(error) => {
                                    state.send_modify(|s| {
                                        s.signin_state = format!("{error:#}");
                                        s.prompt = None;
                                    });
                                    return;
                                }
                            };
                            let mut accounts = state.borrow().accounts.clone();
                            accounts.upsert(account.clone());
                            if let Err(error) =
                                auth::save_accounts(&Store::default_root(), &accounts).await
                            {
                                state.send_modify(|s| {
                                    s.signin_state = format!("Cannot store the account: {error:#}")
                                });
                                return;
                            }
                            state.send_modify(|s| {
                                s.accounts = accounts;
                                s.signin_state = String::new();
                                s.prompt = None;
                                s.message = format!("Signed in as {}.", account.name);
                            });
                            let composed = character_for(&client, &account).await;
                            state.send_modify(|s| s.skin = composed);
                        });
                        let _ = opener_state;
                    }
                    command => {
                        let token = CancellationToken::new();
                        *shared_cancel.lock().unwrap() = token.clone();
                        if !matches!(command, Command::SaveSetup(_)) {
                            state.send_modify(|s| {
                                s.busy = true;
                                s.message = "Working…".into();
                                s.progress = Progress::default();
                            });
                        }
                        let result: Result<String, String> =
                            handle(installer.clone(), &state, command, &token).await;
                        let instances = installer.store.list().await;
                        state.send_modify(|s| {
                            if let Ok(instances) = instances {
                                s.instances = instances;
                            }
                            s.busy = false;
                            s.message = match result {
                                Ok(message) => message,
                                Err(error) if token.is_cancelled() => {
                                    format!("Cancelled. Retry resumes verified downloads. {error}")
                                }
                                Err(error) => error,
                            };
                        });
                    }
                }
            }
        });
        Self {
            tx,
            rx,
            cancel,
            launch_cancel,
        }
    }
}

async fn handle(
    installer: Installer,
    state: &watch::Sender<Snapshot>,
    command: Command,
    token: &CancellationToken,
) -> Result<String, String> {
    match command {
        Command::RefreshVersions => {
            let versions = installer
                .versions(true, token)
                .await
                .map_err(|e| format!("Metadata refresh failed: {e:#}"))?;
            state.send_modify(|s| s.versions = versions.clone());
            Ok(format!("Official metadata lists {}.", versions.join(", ")))
        }
        Command::Create(name, version, loader) => {
            let instance = installer
                .store
                .create(&name, &version, loader, false)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let id = instance.id.clone();
            state.send_modify(|s| {
                s.plan = None;
                s.selected = id;
                s.message = format!("{} created.", instance.name);
            });
            Ok(format!("{} created. Install it to play.", instance.name))
        }
        Command::Select(id) => {
            let instance = installer
                .store
                .read(&id)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let plan = installer.plan(&instance).await.ok().flatten();
            let name = instance.name.clone();
            state.send_modify(|s| {
                s.selected = id.clone();
                s.plan = plan;
                s.message = format!("Selected {name}.");
            });
            Ok(String::new())
        }
        Command::Target(version, loader) => {
            let found = state
                .borrow()
                .instances
                .iter()
                .find(|i| i.version == version && i.loader == loader)
                .map(|i| i.id.clone());
            let id = match found {
                Some(id) => id,
                None => {
                    installer
                        .store
                        .create(&format!("Aero Default {version}"), &version, loader, true)
                        .await
                        .map_err(|e| format!("{e:#}"))?
                        .id
                }
            };
            state.send_modify(|s| {
                s.selected = id.clone();
                s.plan = None;
            });
            Ok(String::new())
        }
        Command::Rename(id, name) => {
            installer
                .store
                .rename(&id, &name)
                .await
                .map_err(|e| format!("{e:#}"))?;
            Ok("Instance renamed.".into())
        }
        Command::Duplicate(id) => {
            installer
                .store
                .duplicate(&id, token)
                .await
                .map_err(|e| format!("{e:#}"))?;
            Ok("Independent copy created.".into())
        }
        Command::Remove(id) => {
            let path = installer
                .store
                .quarantine(&id)
                .await
                .map_err(|e| format!("{e:#}"))?;
            state.send_modify(|s| {
                s.plan = None;
                if s.selected == id {
                    s.selected.clear();
                }
            });
            Ok(format!(
                "Moved to {}. Saves stay recoverable there.",
                path.display()
            ))
        }
        Command::Install(id) => {
            let instance = installer
                .store
                .read(&id)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let settings = state.borrow().settings.clone();
            let java = if !instance.java_path.trim().is_empty() {
                Some(std::path::PathBuf::from(instance.java_path.trim()))
            } else if !settings.custom_java.trim().is_empty() {
                Some(std::path::PathBuf::from(settings.custom_java.trim()))
            } else {
                None
            };
            installer
                .install(&instance, java.as_deref(), token)
                .await
                .map_err(|e| format!("Installation failed: {e:#}"))?;
            if let Ok(Some(plan)) = installer.plan(&instance).await {
                state.send_modify(|s| s.plan = Some(plan));
            }
            Ok("Installation verified.".into())
        }
        Command::Save(settings) => {
            settings
                .save(&installer.store.root)
                .await
                .map_err(|e| format!("{e:#}"))?;
            state.send_modify(|s| s.settings = settings);
            Ok("Preferences saved on this device.".into())
        }
        Command::Configure(id, memory, java) => {
            installer
                .store
                .configure(&id, memory, &java)
                .await
                .map_err(|e| format!("{e:#}"))?;
            Ok("Instance settings saved.".into())
        }
        Command::LoadPlan(id) => {
            let instance = installer
                .store
                .read(&id)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let plan = installer
                .plan(&instance)
                .await
                .map_err(|e| format!("{e:#}"))?;
            state.send_modify(|s| s.plan = plan);
            Ok(String::new())
        }
        Command::Discover(query, kind, version, loader) => {
            let results = content::discover(
                &installer.store,
                &installer.client,
                &query,
                &kind,
                &version,
                &loader,
                token,
            )
            .await
            .map_err(|e| format!("Modrinth search failed: {e:#}"))?;
            let count = results.len();
            state.send_modify(|s| s.results = results);
            Ok(format!(
                "{count} Modrinth projects shown. Installation arrives with M4."
            ))
        }
        Command::SwitchAccount(uuid) => {
            let mut accounts = state.borrow().accounts.clone();
            accounts.select(&uuid).map_err(|e| format!("{e:#}"))?;
            auth::save_accounts(&installer.store.root, &accounts)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let account = accounts.active_account().cloned();
            state.send_modify(|s| s.accounts = accounts);
            if let Some(account) = account {
                let composed = character_for_client(&account).await;
                let name = account.name.clone();
                state.send_modify(|s| s.skin = composed);
                Ok(format!("Playing as {name}."))
            } else {
                Ok("Account switched.".into())
            }
        }
        Command::SignOut => {
            let mut accounts = state.borrow().accounts.clone();
            accounts.active = None;
            auth::save_accounts(&installer.store.root, &accounts)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let fallback = full_character(&skin::Skin::fallback());
            state.send_modify(|s| {
                s.accounts = accounts;
                s.skin = fallback;
                s.game_logs.clear();
            });
            Ok("Signed out of Minecraft.".into())
        }
        Command::RemoveAccount(uuid) => {
            let mut accounts = state.borrow().accounts.clone();
            accounts.remove(&uuid);
            auth::save_accounts(&installer.store.root, &accounts)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let fallback = full_character(&skin::Skin::fallback());
            let still_active = accounts.active_account().cloned();
            state.send_modify(|s| {
                s.accounts = accounts;
                if still_active.is_none() {
                    s.skin = fallback;
                }
            });
            Ok("Account removed from this device.".into())
        }
        Command::Launch(_) | Command::SignIn => unreachable!(),
        Command::DetectHardware => {
            let detected = tokio::task::spawn_blocking(hardware::detect)
                .await
                .map_err(|e| format!("{e}"))?;
            let summary = detected.summary();
            state.send_modify(|s| s.hardware = Some(detected));
            Ok(format!("Detected {summary}"))
        }
        Command::SaveSetup(progress) => {
            onboarding::save(&installer.store.root, &progress)
                .await
                .map_err(|e| format!("{e:#}"))?;
            state.send_modify(|s| s.onboarding = progress);
            Ok(String::new())
        }
        Command::ReopenSetup => {
            let mut progress = state.borrow().onboarding.clone();
            progress.completed = false;
            progress.step = 0;
            onboarding::save(&installer.store.root, &progress)
                .await
                .map_err(|e| format!("{e:#}"))?;
            state.send_modify(|s| s.onboarding = progress);
            Ok("Setup restarted.".into())
        }
        Command::FinishSetup(progress, install) => {
            onboarding::save(&installer.store.root, &progress)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let answers = progress.answers.clone();
            let mut settings = state.borrow().settings.clone();
            settings.appearance = answers.appearance.clone();
            settings.memory_mib = answers.memory_mib;
            settings.performance = answers.performance.clone();
            settings.low_performance = answers.performance == "Potato PC";
            settings.save(&installer.store.root).await.ok();
            let loader = match answers.loader.as_str() {
                "Fabric" => Loader::Fabric,
                "Forge" => Loader::Forge,
                _ => Loader::Vanilla,
            };
            let existing = state
                .borrow()
                .instances
                .iter()
                .find(|i| i.version == answers.version && i.loader == loader)
                .cloned();
            let instance = match existing {
                Some(instance) => instance,
                None => installer
                    .store
                    .create("Aero Default", &answers.version, loader, true)
                    .await
                    .map_err(|e| format!("{e:#}"))?,
            };
            let instance_id = instance.id.clone();
            state.send_modify(|s| {
                s.settings = settings;
                s.onboarding = progress;
                s.selected = instance_id.clone();
                s.message = "Setup complete.".into();
            });
            // Installation only starts when the player asked for it in the wizard.
            if install {
                let installer = installer.clone();
                let state = state.clone();
                state.send_modify(|s| s.busy = true);
                tokio::spawn(async move {
                    let token = CancellationToken::new();
                    let result: anyhow::Result<()> = async {
                        let instance = installer.store.read(&instance_id).await?;
                        let settings = state.borrow().settings.clone();
                        let java = if !instance.java_path.trim().is_empty() {
                            Some(std::path::PathBuf::from(instance.java_path.trim()))
                        } else if !settings.custom_java.trim().is_empty() {
                            Some(std::path::PathBuf::from(settings.custom_java.trim()))
                        } else {
                            None
                        };
                        installer
                            .install(&instance, java.as_deref(), &token)
                            .await?;
                        Ok(())
                    }
                    .await;
                    let message = match result {
                        Ok(()) => "Minecraft is ready to play.".to_owned(),
                        Err(error) => format!("Installation did not finish: {error:#}"),
                    };
                    let instances = installer.store.list().await.ok();
                    state.send_modify(|s| {
                        s.busy = false;
                        s.message = message;
                        if let Some(instances) = instances {
                            s.instances = instances;
                        }
                    });
                });
            }
            Ok("Setup complete.".into())
        }
    }
}

async fn launch_instance(
    installer: Installer,
    state: watch::Sender<Snapshot>,
    id: String,
    token: CancellationToken,
) {
    let result: anyhow::Result<()> = async {
        let instance = installer.store.read(&id).await?;
        let account = state
            .borrow()
            .accounts
            .active_account()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Sign in with a Microsoft account to play"))?;
        ensure_account(&account)?;
        let settings = state.borrow().settings.clone();
        let memory = launch::effective_memory(&instance, settings.memory_mib);
        let java = if !instance.java_path.trim().is_empty() {
            Some(std::path::PathBuf::from(instance.java_path.trim()))
        } else if !settings.custom_java.trim().is_empty() {
            Some(std::path::PathBuf::from(settings.custom_java.trim()))
        } else {
            None
        };
        let state_lines = state.clone();
        let on_line: Arc<dyn Fn(String) + Send + Sync> = Arc::new(move |line: String| {
            state_lines.send_modify(|s| {
                s.game_logs.push(line);
                if s.game_logs.len() > GAME_LOG_LIMIT {
                    s.game_logs.remove(0);
                }
            });
        });
        state.send_modify(|s| s.message = "Preparing Java and classpath…".into());
        let outcome = installer
            .launch(
                &instance,
                &account,
                memory,
                java.as_deref(),
                &token,
                on_line,
            )
            .await?;
        state.send_modify(|s| {
            s.message = match &outcome.crash_report {
                Some(report) => format!("{} Crash report: {}", outcome.summary, report.display()),
                None => outcome.summary.clone(),
            };
        });
        Ok(())
    }
    .await;
    if let Err(error) = result {
        let message = if token.is_cancelled() {
            "Launch cancelled.".to_owned()
        } else {
            format!("Could not launch: {error:#}")
        };
        state.send_modify(|s| {
            s.message = message;
            s.game_logs.push(format!("[Aero] {error:#}"));
        });
    }
    state.send_modify(|s| s.launching = false);
}

fn ensure_account(account: &Account) -> anyhow::Result<()> {
    anyhow::ensure!(
        account.owns_minecraft,
        "This Microsoft account does not own Minecraft"
    );
    anyhow::ensure!(
        account.is_active_token(),
        "The session expired. Sign in again to continue"
    );
    Ok(())
}

fn fallback_skin() -> Vec<u8> {
    full_character(&Skin::fallback()).as_ref().clone()
}

fn full_character(texture: &Skin) -> Arc<Vec<u8>> {
    Arc::new(skin::model(texture))
}

async fn character_for_client(account: &Account) -> Arc<Vec<u8>> {
    match auth::client() {
        Ok(client) => character_for(&client, account).await,
        Err(_) => full_character(&Skin::fallback()),
    }
}

async fn character_for(client: &reqwest::Client, account: &Account) -> Arc<Vec<u8>> {
    match auth::skin_bytes(client, account, &Store::default_root()).await {
        Ok(bytes) => match Skin::decode(&bytes) {
            Ok(texture) => full_character(&texture),
            Err(_) => full_character(&Skin::fallback()),
        },
        Err(_) => full_character(&Skin::fallback()),
    }
}

fn open_browser(url: &str) {
    if !url.starts_with("https://") {
        return;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .creation_flags(0x0800_0000)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
}
