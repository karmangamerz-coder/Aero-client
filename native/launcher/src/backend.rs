// SPDX-License-Identifier: MIT
use aero_core::{
    CancellationToken, Installer, Instance, Loader, Store, content, net::Progress,
    storage::Settings,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, watch};

#[derive(Clone, Default)]
pub struct Snapshot {
    pub instances: Vec<Instance>,
    pub versions: Vec<String>,
    pub news: Vec<content::NewsItem>,
    pub results: Vec<content::DiscoveryItem>,
    pub settings: Settings,
    pub busy: bool,
    pub message: String,
    pub progress: Progress,
}
pub enum Command {
    Refresh,
    Create(String, String, Loader, bool),
    Rename(String, String),
    Duplicate(String),
    Remove(String),
    Install(String),
    Save(Settings),
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
            message: "Choose an instance to get started.".into(),
            ..Default::default()
        });
        let cancel = Arc::new(Mutex::new(CancellationToken::new()));
        let shared_cancel = cancel.clone();
        tokio::spawn(async move {
            let report = state.clone();
            let installer = match Installer::new(
                Store::default_root(),
                Arc::new(move |progress| {
                    report.send_modify(|s| s.progress = progress);
                }),
            ) {
                Ok(v) => v,
                Err(e) => {
                    state.send_modify(|s| s.message = format!("Startup failed: {e:#}"));
                    return;
                }
            };
            if let Ok(instances) = installer.store.list().await {
                state.send_modify(|s| s.instances = instances);
            }
            // News is independent of the installation queue and has no periodic timer.
            let news_installer = installer.clone();
            let news_state = state.clone();
            if state.borrow().settings.show_news {
                tokio::spawn(async move {
                    if let Ok(news) = content::news(
                        &news_installer.store,
                        &news_installer.client,
                        &CancellationToken::new(),
                    )
                    .await
                    {
                        news_state.send_modify(|s| s.news = news);
                    }
                });
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
                        Command::Refresh => {
                            let versions = installer.versions(true, &token).await.map_err(|e| format!("{e:#}"))?;
                            state.send_modify(|s| s.versions = versions);
                            Ok("Available versions checked with Minecraft.".into())
                        }
                        Command::Create(name, version, loader, default) => {
                            installer.store.create(&name, &version, loader, default).await.map_err(|e| format!("{e:#}"))?;
                            Ok("Instance created. Select Install to download game files.".into())
                        }
                        Command::Rename(id, name) => {
                            installer.store.rename(&id, &name).await.map_err(|e| format!("{e:#}"))?;
                            Ok("Instance name saved.".into())
                        }
                        Command::Duplicate(id) => {
                            installer.store.duplicate(&id, &token).await.map_err(|e| format!("{e:#}"))?;
                            Ok("Independent copy created. Select Install to verify shared game files.".into())
                        }
                        Command::Remove(id) => {
                            let path = installer.store.quarantine(&id).await.map_err(|e| format!("{e:#}"))?;
                            Ok(format!("Moved to {}. Saves can be recovered from this folder.", path.display()))
                        }
                        Command::Install(id) => {
                            let instance = installer.store.read(&id).await.map_err(|e| format!("{e:#}"))?;
                            let custom = state.borrow().settings.custom_java.clone();
                            installer.install(&instance, (!custom.trim().is_empty()).then(|| Path::new(custom.trim())), &token).await.map_err(|e| format!("{e:#}"))?;
                            Ok("Installation verified. Microsoft sign-in and game launch are coming in a future release.".into())
                        }
                        Command::Save(settings) => {
                            settings.save(&installer.store.root).await.map_err(|e| format!("{e:#}"))?;
                            state.send_modify(|s| s.settings = settings);
                            Ok("Settings saved.".into())
                        }
                        Command::Discover(query, kind, version, loader) => {
                            let results = content::discover(&installer.store, &installer.client, &query, &kind, &version, &loader, &token).await.map_err(|e| format!("{e:#}"))?;
                            state.send_modify(|s| s.results = results);
                            Ok("Modrinth results loaded. Project installation is planned for a later release.".into())
                        }
                    }
                }.await;
                let instances = installer.store.list().await;
                state.send_modify(|s| {
                    if let Ok(instances) = instances {
                        s.instances = instances;
                    }
                    s.busy = false;
                    s.message = match result {
                        Ok(m) => m,
                        Err(e) if token.is_cancelled() => {
                            format!("Cancelled. Retry resumes verified downloads. {e}")
                        }
                        Err(e) => format!("Could not complete operation: {e}"),
                    };
                });
            }
        });
        Self { tx, rx, cancel }
    }
}
