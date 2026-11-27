// SPDX-License-Identifier: GPL-3.0-only
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
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, watch};

const LOG_LIMIT: usize = 120;
const GAME_LOG_LIMIT: usize = 600;

#[derive(Clone)]
pub struct Snapshot {
    pub instances: Vec<Instance>,
    pub versions: Vec<String>,
    pub results: Vec<content::DiscoveryItem>,
    pub project_detail: Option<content::ProjectDetail>,
    pub installed_content: Vec<content::InstalledProject>,
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
    pub head: Arc<Vec<u8>>,
    pub hardware: Option<Hardware>,
    pub onboarding: onboarding::Progress,
    pub authenticating: bool,
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
            project_detail: None,
            installed_content: vec![],
            settings: Settings::default(),
            plan: None,
            log: vec![],
            storage: String::new(),
            busy: false,
            message: String::new(),
            progress: Progress::default(),
            accounts: Accounts::default(),
            skin: Arc::new(fallback_skin()),
            head: Arc::new(skin::head(&Skin::fallback())),
            hardware: None,
            onboarding: onboarding::Progress::default(),
            authenticating: false,
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
    Discover(String, String, String, String, String, String),
    LoadProject(String, String, String),
    LoadContent(String),
    InstallContent(String, String),
    ToggleContent(String, String, bool),
    RemoveContent(String, String),
    SignIn,
    CancelSignIn,
    SignInStage(u64),
    CompleteSignIn(u64, Result<Account, String>),
    SwitchAccount(String),
    SignOut,
    RemoveAccount(String),
    Launch(String),
    DetectHardware,
    NavigateSetup(onboarding::Direction, usize, onboarding::Onboarding),
    FinishSetup(usize, onboarding::Onboarding, bool),
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

    pub fn start(
        settings: Settings,
        accounts: Accounts,
        onboarding: onboarding::Progress,
        account_error: Option<String>,
    ) -> Self {
        let (tx, mut commands) = mpsc::channel(24);
        let initial = Snapshot {
            settings,
            accounts,
            onboarding,
            storage: Store::default_root().display().to_string(),
            message: account_error.clone().unwrap_or_else(|| "Ready.".into()),
            signin_state: account_error.unwrap_or_default(),
            ..Default::default()
        };
        let (state, rx) = watch::channel(initial);
        let cancel = Arc::new(Mutex::new(CancellationToken::new()));
        let launch_cancel = Arc::new(Mutex::new(CancellationToken::new()));
        let shared_cancel = cancel.clone();
        let shared_launch = launch_cancel.clone();
        let completion_tx = tx.clone();
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
                let selected = instances
                    .first()
                    .map(|item| item.id.clone())
                    .unwrap_or_default();
                let installed_content = if selected.is_empty() {
                    vec![]
                } else {
                    content::list(&installer.store, &selected)
                        .await
                        .unwrap_or_default()
                };
                state.send_modify(|s| {
                    s.instances = instances;
                    s.selected = selected;
                    s.installed_content = installed_content;
                });
            }
            // Character artwork is composed once from the saved account, if any.
            let saved_account = {
                let snapshot = state.borrow();
                snapshot.accounts.active_account().cloned()
            };
            if let Some(account) = saved_account {
                let composed = character_for_client(&account).await;
                let head = head_for_client(&account).await;
                state.send_modify(|s| {
                    s.skin = composed;
                    s.head = head;
                });
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
            let mut signin_generation = 0u64;
            let mut signin_cancel = CancellationToken::new();
            let account_gate = Arc::new(tokio::sync::Mutex::new(()));
            while let Some(command) = commands.recv().await {
                if matches!(
                    command,
                    Command::SignOut | Command::RemoveAccount(_) | Command::CancelSignIn
                ) {
                    signin_cancel.cancel();
                    signin_generation = signin_generation.wrapping_add(1);
                    state.send_modify(|s| {
                        s.authenticating = false;
                        s.signin_state.clear();
                    });
                }
                if matches!(command, Command::Launch(_)) {
                    let gate = account_gate.clone();
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
                        launch_instance(installer, state, id, token, gate).await;
                    });
                    continue;
                }
                match command {
                    Command::CancelSignIn => {
                        state.send_modify(|s| s.message = "Microsoft sign-in cancelled.".into());
                    }
                    Command::SignIn => {
                        if state.borrow().authenticating {
                            continue;
                        }
                        signin_cancel = CancellationToken::new();
                        signin_generation = signin_generation.wrapping_add(1);
                        let generation = signin_generation;
                        let configured = state.borrow().settings.microsoft_client_id.clone();
                        state.send_modify(|s| {
                            s.authenticating = true;
                            s.signin_state = "Opening Microsoft in your browser…".into();
                            s.message = "Waiting for Microsoft sign-in…".into();
                        });
                        let token = signin_cancel.clone();
                        let completed = completion_tx.clone();
                        tokio::spawn(async move {
                            let result: anyhow::Result<Account> = async {
                                let client = auth::client()?;
                                let login = auth::begin(&configured).await?;
                                anyhow::ensure!(open_browser(&login.authorization_url), "Could not open your default browser. Check the default-browser setting and retry.");
                                let tokens = login.complete(&client, &token).await?;
                                let _ = completed.send(Command::SignInStage(generation)).await;
                                tokio::select! {
                                    _ = token.cancelled() => anyhow::bail!("Microsoft sign-in cancelled."),
                                    result = auth::finish(&client, &tokens) => Ok(result?.0),
                                }
                            }.await;
                            let _ = completed
                                .send(Command::CompleteSignIn(
                                    generation,
                                    result.map_err(|e| e.to_string()),
                                ))
                                .await;
                        });
                    }
                    Command::SignInStage(generation) => {
                        if generation == signin_generation && state.borrow().authenticating {
                            state.send_modify(|s| s.signin_state = "Microsoft authorized. Checking Xbox access and Minecraft Java ownership…".into());
                        }
                    }
                    Command::CompleteSignIn(generation, result) => {
                        if generation != signin_generation {
                            continue;
                        }
                        let _guard = account_gate.lock().await;
                        let result: anyhow::Result<Account> = async {
                            let account = result.map_err(anyhow::Error::msg)?;
                            let mut accounts = state.borrow().accounts.clone();
                            accounts.upsert(account.clone());
                            auth::save_accounts(&Store::default_root(), &accounts).await?;
                            state.send_modify(|s| s.accounts = accounts);
                            Ok(account)
                        }
                        .await;
                        match result {
                            Ok(account) => {
                                let composed = character_for_client(&account).await;
                                let head = head_for_client(&account).await;
                                state.send_modify(|s| {
                                    s.head = head;
                                    s.skin = composed;
                                    s.authenticating = false;
                                    s.signin_state.clear();
                                    s.message = format!("Signed in as {}.", account.name);
                                });
                            }
                            Err(error) => {
                                let message = error.to_string();
                                append_launcher_log("Microsoft sign-in", &message).await;
                                state.send_modify(|s| {
                                    s.authenticating = false;
                                    s.signin_state = message.clone();
                                    s.message = message;
                                });
                            }
                        }
                    }
                    command => {
                        let token = CancellationToken::new();
                        *shared_cancel.lock().unwrap() = token.clone();
                        state.send_modify(|s| {
                            s.busy = true;
                            s.message = "Working…".into();
                            s.progress = Progress::default();
                        });
                        let _account_guard = account_gate.lock().await;
                        let result: Result<String, String> =
                            handle(installer.clone(), &state, command, &token).await;
                        if let Err(error) = &result {
                            append_launcher_log("Launcher command", error).await;
                        }
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

async fn append_launcher_log(context: &str, message: &str) {
    let root = Store::default_root();
    let path = match aero_core::storage::safe_path(&root, "logs/launcher.log").await {
        Ok(path) => path,
        Err(_) => return,
    };
    if let Some(parent) = path.parent() {
        if tokio::fs::create_dir_all(parent).await.is_err() {
            return;
        }
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or_default();
    let safe_message: String = message
        .replace(['\r', '\n'], " ")
        .chars()
        .take(2000)
        .collect();
    if let Ok(mut file) = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
    {
        let _ = file
            .write_all(format!("{stamp} [{context}] {safe_message}\n").as_bytes())
            .await;
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
            let installed_content = content::list(&installer.store, &id)
                .await
                .map_err(|e| format!("Cannot read installed content: {e:#}"))?;
            let name = instance.name.clone();
            state.send_modify(|s| {
                s.selected = id.clone();
                s.plan = plan;
                s.installed_content = installed_content;
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
        Command::Discover(query, kind, version, loader, category, sort) => {
            let results = content::discover(
                &installer.store,
                &installer.client,
                &query,
                &kind,
                &version,
                &loader,
                &category,
                &sort,
                token,
            )
            .await
            .map_err(|e| format!("Modrinth search failed: {e:#}"))?;
            let count = results.len();
            state.send_modify(|s| s.results = results);
            Ok(format!("{count} compatible Modrinth projects shown."))
        }
        Command::LoadProject(project, version, loader) => {
            let detail =
                content::project_detail(&installer.client, &project, &version, &loader, token)
                    .await
                    .map_err(|e| format!("Cannot load Modrinth project: {e:#}"))?;
            let title = detail.title.clone();
            state.send_modify(|s| s.project_detail = Some(detail));
            Ok(format!("Loaded {title}."))
        }
        Command::LoadContent(instance_id) => {
            let installed = content::list(&installer.store, &instance_id)
                .await
                .map_err(|e| format!("Cannot read installed content: {e:#}"))?;
            state.send_modify(|s| s.installed_content = installed);
            Ok(String::new())
        }
        Command::InstallContent(instance_id, project) => {
            let installed = content::install(
                &installer.store,
                &installer.client,
                &instance_id,
                &project,
                token,
            )
            .await
            .map_err(|e| format!("Content installation failed: {e:#}"))?;
            let current = content::list(&installer.store, &instance_id)
                .await
                .map_err(|e| format!("Cannot refresh installed content: {e:#}"))?;
            state.send_modify(|s| s.installed_content = current);
            Ok(format!(
                "Installed {} verified project file{}.",
                installed.len(),
                if installed.len() == 1 { "" } else { "s" }
            ))
        }
        Command::ToggleContent(instance_id, project_id, enabled) => {
            content::set_enabled(&installer.store, &instance_id, &project_id, enabled)
                .await
                .map_err(|e| format!("Cannot change installed content: {e:#}"))?;
            let current = content::list(&installer.store, &instance_id)
                .await
                .map_err(|e| format!("Cannot refresh installed content: {e:#}"))?;
            state.send_modify(|s| s.installed_content = current);
            Ok(if enabled {
                "Content enabled.".into()
            } else {
                "Content disabled.".into()
            })
        }
        Command::RemoveContent(instance_id, project_id) => {
            let path = content::remove(&installer.store, &instance_id, &project_id)
                .await
                .map_err(|e| format!("Cannot remove installed content: {e:#}"))?;
            let current = content::list(&installer.store, &instance_id)
                .await
                .map_err(|e| format!("Cannot refresh installed content: {e:#}"))?;
            state.send_modify(|s| s.installed_content = current);
            Ok(format!("Moved content to {}.", path.display()))
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
                let head = head_for_client(&account).await;
                state.send_modify(|s| {
                    s.skin = composed;
                    s.head = head;
                });
                Ok(format!("Playing as {name}."))
            } else {
                Ok("Account switched.".into())
            }
        }
        Command::SignOut => {
            let mut accounts = state.borrow().accounts.clone();
            if let Some(uuid) = accounts.active.clone() {
                accounts.remove(&uuid);
            }
            accounts.active = None;
            auth::save_accounts(&installer.store.root, &accounts)
                .await
                .map_err(|e| format!("{e:#}"))?;
            let fallback = full_character(&skin::Skin::fallback());
            state.send_modify(|s| {
                s.accounts = accounts;
                s.skin = fallback;
                s.head = Arc::new(skin::head(&Skin::fallback()));
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
            let still_active = accounts.active_account().cloned();
            let (body, head) = if let Some(account) = &still_active {
                (
                    character_for_client(account).await,
                    head_for_client(account).await,
                )
            } else {
                (
                    full_character(&Skin::fallback()),
                    Arc::new(skin::head(&Skin::fallback())),
                )
            };
            state.send_modify(|s| {
                s.accounts = accounts;
                s.skin = body;
                s.head = head;
            });
            Ok("Account removed from this device.".into())
        }
        Command::Launch(_)
        | Command::SignIn
        | Command::CancelSignIn
        | Command::SignInStage(_)
        | Command::CompleteSignIn(_, _) => unreachable!(),
        Command::DetectHardware => {
            let detected = tokio::task::spawn_blocking(hardware::detect)
                .await
                .map_err(|e| format!("{e}"))?;
            let summary = detected.summary();
            state.send_modify(|s| s.hardware = Some(detected));
            Ok(format!("Detected {summary}"))
        }
        Command::NavigateSetup(direction, expected_step, answers) => {
            let progress = state
                .borrow()
                .onboarding
                .navigate(expected_step, direction, answers)
                .map_err(|e| format!("Cannot change setup step: {e:#}"))?;
            onboarding::save(&installer.store.root, &progress)
                .await
                .map_err(|e| format!("Cannot save setup progress: {e:#}"))?;
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
        Command::FinishSetup(expected_step, answers, install) => {
            let progress = state
                .borrow()
                .onboarding
                .finish(expected_step, answers)
                .map_err(|e| format!("Cannot finish setup: {e:#}"))?;
            let answers = progress.answers.clone();
            let mut settings = state.borrow().settings.clone();
            settings.appearance = answers.appearance.clone();
            settings.memory_mib = answers.memory_mib;
            settings.performance = answers.performance.clone();
            settings.low_performance = answers.performance == "Potato PC";
            settings
                .save(&installer.store.root)
                .await
                .map_err(|e| format!("Cannot save launcher settings: {e:#}"))?;
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
            // Completion is committed only after all required durable state exists.
            onboarding::save(&installer.store.root, &progress)
                .await
                .map_err(|e| format!("Cannot save setup completion: {e:#}"))?;
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
    account_gate: Arc<tokio::sync::Mutex<()>>,
) {
    let result: anyhow::Result<()> = async {
        let instance = installer.store.read(&id).await?;
        let guard = account_gate.lock().await;
        let mut account = state
            .borrow()
            .accounts
            .active_account()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Sign in with a Microsoft account to play"))?;
        let settings = state.borrow().settings.clone();
        if !account.is_active_token() {
            state.send_modify(|s| s.message = "Refreshing the Microsoft session…".into());
            let client = auth::client()?;
            account = auth::renew(&client, &account, &settings.microsoft_client_id).await?;
            let mut accounts = state.borrow().accounts.clone();
            accounts.upsert(account.clone());
            auth::save_accounts(&installer.store.root, &accounts).await?;
            state.send_modify(|s| s.accounts = accounts);
        }
        ensure_account(&account)?;
        drop(guard);
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

async fn head_for_client(account: &Account) -> Arc<Vec<u8>> {
    let texture = match auth::client() {
        Ok(client) => auth::skin_bytes(&client, account, &Store::default_root())
            .await
            .ok()
            .and_then(|bytes| Skin::decode(&bytes).ok())
            .unwrap_or_else(Skin::fallback),
        Err(_) => Skin::fallback(),
    };
    Arc::new(skin::head(&texture))
}
fn open_browser(url: &str) -> bool {
    webbrowser::open(url).is_ok()
}
