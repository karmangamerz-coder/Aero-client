// SPDX-License-Identifier: MIT
//! Installation services bridged to the UI thread.
//!
//! Exactly one command runs at a time on a two-thread Tokio runtime; the UI thread
//! only publishes snapshots through a watch channel, so rendering never blocks on
//! disk, network or hashing work.
use aero_core::{
    CancellationToken, Installer, Instance, Loader, Store, content, net::Progress,
    storage::Settings,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, watch};

const LOG_LIMIT: usize = 60;

#[derive(Clone, Default)]
pub struct Snapshot {
    pub instances: Vec<Instance>,
    pub versions: Vec<String>,
    pub news: Vec<content::NewsItem>,
    pub results: Vec<content::DiscoveryItem>,
    pub settings: Settings,
    pub plan: Option<serde_json::Value>,
    pub log: Vec<String>,
    pub storage: String,
    pub busy: bool,
    pub message: String,
    pub progress: Progress,
}

pub enum Command {
    RefreshVersions,
    RefreshNews,
    Create(String, String, Loader, bool),
    Rename(String, String),
    Duplicate(String),
    Remove(String),
    Install(String),
    Save(Settings),
    Configure(String, Option<u32>, String),
    LoadPlan(String),
    Discover(String, String, String, String),
}

#[derive(Clone)]
pub struct Backend {
    pub tx: mpsc::Sender<Command>,
    pub rx: watch::Receiver<Snapshot>,
    pub cancel: Arc<Mutex<CancellationToken>>,
}

impl Backend {
    pub fn send(&self, command: Command) {
        let _ = self.tx.try_send(command);
    }

    pub fn cancel(&self) {
        self.cancel.lock().unwrap().cancel();
    }

    pub fn start(settings: Settings) -> Self {
        let (tx, mut commands) = mpsc::channel(16);
        let (state, rx) = watch::channel(Snapshot {
            settings,
            message: "Ready. Choose an instance to begin.".into(),
            storage: Store::default_root().display().to_string(),
            ..Default::default()
        });
        let cancel = Arc::new(Mutex::new(CancellationToken::new()));
        let shared_cancel = cancel.clone();
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
            // News is one cached request per launch, never a timer.
            if state.borrow().settings.show_news {
                spawn_news(&installer, &state);
            }
            while let Some(command) = commands.recv().await {
                let token = CancellationToken::new();
                *shared_cancel.lock().unwrap() = token.clone();
                state.send_modify(|s| {
                    s.busy = true;
                    s.message = "Working…".into();
                    s.progress = Progress::default();
                });
                let result: Result<String, String> = async {
                    match command {
                        Command::RefreshVersions => {
                            let versions = installer
                                .versions(true, &token)
                                .await
                                .map_err(|e| format!("{e:#}"))?;
                            state.send_modify(|s| s.versions = versions.clone());
                            Ok(format!("Official metadata lists {}.", versions.join(", ")))
                        }
                        Command::RefreshNews => {
                            let news = content::news(&installer.store, &installer.client, &token)
                                .await
                                .map_err(|e| format!("{e:#}"))?;
                            let empty = news.is_empty();
                            state.send_modify(|s| s.news = news);
                            Ok(if empty {
                                "No announcements could be loaded.".into()
                            } else {
                                "News refreshed from official sources.".into()
                            })
                        }
                        Command::Create(name, version, loader, default) => {
                            installer
                                .store
                                .create(&name, &version, loader, default)
                                .await
                                .map_err(|e| format!("{e:#}"))?;
                            Ok("Instance created. Open it to install game files.".into())
                        }
                        Command::Rename(id, name) => {
                            installer
                                .store
                                .rename(&id, &name)
                                .await
                                .map_err(|e| format!("{e:#}"))?;
                            Ok("Instance name saved.".into())
                        }
                        Command::Duplicate(id) => {
                            installer
                                .store
                                .duplicate(&id, &token)
                                .await
                                .map_err(|e| format!("{e:#}"))?;
                            Ok("Independent copy created. Install to verify shared files.".into())
                        }
                        Command::Remove(id) => {
                            let path = installer
                                .store
                                .quarantine(&id)
                                .await
                                .map_err(|e| format!("{e:#}"))?;
                            state.send_modify(|s| s.plan = None);
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
                                Some(Path::new(instance.java_path.trim()).to_path_buf())
                            } else if !settings.custom_java.trim().is_empty() {
                                Some(Path::new(settings.custom_java.trim()).to_path_buf())
                            } else {
                                None
                            };
                            installer
                                .install(&instance, java.as_deref(), &token)
                                .await
                                .map_err(|e| format!("{e:#}"))?;
                            if let Ok(Some(plan)) = installer.plan(&instance).await {
                                state.send_modify(|s| s.plan = Some(plan));
                            }
                            Ok("Installation verified. Microsoft sign-in and launch come with M3.".into())
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
                            Ok("Installation details loaded.".into())
                        }
                        Command::Discover(query, kind, version, loader) => {
                            let results = content::discover(
                                &installer.store,
                                &installer.client,
                                &query,
                                &kind,
                                &version,
                                &loader,
                                &token,
                            )
                            .await
                            .map_err(|e| format!("{e:#}"))?;
                            let count = results.len();
                            state.send_modify(|s| s.results = results);
                            Ok(format!(
                                "{count} Modrinth projects shown; project installation is a later milestone."
                            ))
                        }
                    }
                }
                .await;
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
                        Err(error) => format!("Could not complete operation: {error}"),
                    };
                });
            }
        });
        Self { tx, rx, cancel }
    }
}

fn spawn_news(installer: &Installer, state: &watch::Sender<Snapshot>) {
    let installer = installer.clone();
    let state = state.clone();
    tokio::spawn(async move {
        let token = CancellationToken::new();
        if let Ok(news) = content::news(&installer.store, &installer.client, &token).await {
            state.send_modify(|s| s.news = news);
        }
    });
}
