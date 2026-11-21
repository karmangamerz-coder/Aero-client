// SPDX-License-Identifier: MIT
use crate::{install::*, storage::*, *};
use serde_json::json;
use sha1::{Digest, Sha1};
use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

async fn server(
    payload: &'static [u8],
    fail_first: bool,
    corrupt: bool,
) -> (
    DownloadClient,
    Artifact,
    Arc<AtomicUsize>,
    tokio::task::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let seen = count.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buffer = [0; 4096];
            let size = socket.read(&mut buffer).await.unwrap_or(0);
            let headers = String::from_utf8_lossy(&buffer[..size]).to_lowercase();
            let request = seen.fetch_add(1, Ordering::SeqCst);
            if fail_first && request == 0 {
                socket.write_all(b"HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await.ok();
                continue;
            }
            let offset = headers
                .lines()
                .find_map(|line| {
                    line.strip_prefix("range: bytes=")
                        .and_then(|s| s.trim_end_matches('-').parse::<usize>().ok())
                })
                .unwrap_or(0);
            let body = if corrupt {
                vec![0; payload.len()]
            } else {
                payload.to_vec()
            };
            let reply = if offset > 0 {
                format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {offset}-{}/{}\r\n",
                    body.len() - 1,
                    body.len()
                )
            } else {
                "HTTP/1.1 200 OK\r\n".into()
            };
            let response = format!(
                "{reply}Content-Length: {}\r\nConnection: close\r\n\r\n",
                body.len() - offset
            );
            socket.write_all(response.as_bytes()).await.ok();
            socket.write_all(&body[offset..]).await.ok();
        }
    });
    (
        DownloadClient::local(origin.clone()),
        Artifact {
            url: format!("{origin}/file"),
            sha1: hex::encode(Sha1::digest(payload)),
            size: Some(payload.len() as u64),
            alternatives: vec![],
        },
        count,
        task,
    )
}
#[tokio::test]
async fn verified_download_reuse_and_repair() {
    let (client, artifact, count, server) = server(b"fixture not game code", false, false).await;
    let dir = tempfile::tempdir().unwrap();
    let token = CancellationToken::new();
    let path = client
        .fetch(&artifact, dir.path(), "file", &token)
        .await
        .unwrap();
    client
        .fetch(&artifact, dir.path(), "file", &token)
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);
    fs::write(&path, "corrupt").await.unwrap();
    client
        .fetch(&artifact, dir.path(), "file", &token)
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 2);
    server.abort();
}
#[tokio::test]
async fn range_resume_preserves_integrity() {
    let (client, artifact, _, server) = server(b"resume fixture", false, false).await;
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("file.part"), b"res")
        .await
        .unwrap();
    let path = client
        .fetch(&artifact, dir.path(), "file", &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(fs::read(path).await.unwrap(), b"resume fixture");
    server.abort();
}
#[tokio::test]
async fn transient_failure_retries() {
    let (client, artifact, count, server) = server(b"retry", true, false).await;
    let dir = tempfile::tempdir().unwrap();
    client
        .fetch(&artifact, dir.path(), "file", &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 2);
    server.abort();
}
#[tokio::test]
async fn corrupt_data_never_published() {
    let (client, artifact, count, server) = server(b"good", false, true).await;
    let dir = tempfile::tempdir().unwrap();
    assert!(
        client
            .fetch(&artifact, dir.path(), "file", &CancellationToken::new())
            .await
            .is_err()
    );
    assert_eq!(count.load(Ordering::SeqCst), 3);
    assert!(!dir.path().join("file").exists());
    server.abort();
}
#[tokio::test]
async fn pre_cancelled_download_does_not_connect() {
    let (client, artifact, count, server) = server(b"good", false, false).await;
    let dir = tempfile::tempdir().unwrap();
    let token = CancellationToken::new();
    token.cancel();
    assert!(
        client
            .fetch(&artifact, dir.path(), "file", &token)
            .await
            .is_err()
    );
    assert_eq!(count.load(Ordering::SeqCst), 0);
    server.abort();
}
#[test]
fn trust_policy_rejects_lookalikes_and_credentials() {
    let client = DownloadClient::new(Arc::new(|_| {})).unwrap();
    for url in [
        "http://piston-meta.mojang.com/file",
        "https://piston-meta.mojang.com.evil.com/file",
        "https://user@piston-meta.mojang.com/file",
        "https://piston-meta.mojang.com:123/file",
    ] {
        assert!(client.validate_url(url).is_err(), "{url}");
    }
}
#[test]
fn paths_reject_traversal_and_windows_special_names() {
    for path in [
        "../bad",
        "/abs",
        "C:/bad",
        "a/../../bad",
        "a\\b",
        "file:stream",
        "CON.txt",
        "a/NUL",
        "foo.",
        "foo//bar",
    ] {
        assert!(validate_relative(path).is_err(), "{path}");
    }
    assert!(validate_relative("libraries/org/example/1.0/example.jar").is_ok());
}
#[tokio::test]
async fn independent_instances_and_legacy_schema() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().into());
    let a = store
        .create("One", "1.8.9", Loader::Vanilla, false)
        .await
        .unwrap();
    let b = store
        .create("One", "1.8.9", Loader::Vanilla, false)
        .await
        .unwrap();
    assert_ne!(a.id, b.id);
    let game = store
        .directory(&a.id)
        .await
        .unwrap()
        .join("game/saves/keep.txt");
    fs::write(&game, b"save").await.unwrap();
    store.status(&a, "INSTALLING", "testing").await.unwrap();
    assert_eq!(store.read(&a.id).await.unwrap().status, "INTERRUPTED");
    assert!(
        !store
            .directory(&b.id)
            .await
            .unwrap()
            .join("game/saves/keep.txt")
            .exists()
    );
    let removed = store.quarantine(&a.id).await.unwrap();
    assert_eq!(
        fs::read(removed.join("game/saves/keep.txt")).await.unwrap(),
        b"save"
    );
}
#[tokio::test]
async fn default_instances_are_reused_and_pairs_validated() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().into());
    let a = store
        .create("Aero Default", "26.3", Loader::Fabric, true)
        .await
        .unwrap();
    let b = store
        .create("Aero Default", "26.3", Loader::Fabric, true)
        .await
        .unwrap();
    assert_eq!(a.id, b.id);
    assert!(
        store
            .create("Bad", "1.8.9", Loader::Fabric, false)
            .await
            .is_err()
    );
    assert!(
        store
            .create("Bad", "26.3", Loader::Forge, false)
            .await
            .is_err()
    );
    assert!(
        store
            .create("Bad", "99.9", Loader::Vanilla, false)
            .await
            .is_err()
    );
}
#[tokio::test]
async fn existing_java_preferences_are_migrated_without_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("settings.properties"),
        "theme=DARK\nmemoryMiB=8192\n",
    )
    .await
    .unwrap();
    let settings = Settings::load(dir.path()).await.unwrap();
    assert_eq!(settings.memory_mib, 8192);
    assert_eq!(settings.appearance, "Aero Dark");
    assert!(!dir.path().join("launcher-settings.json").exists());
    settings.save(dir.path()).await.unwrap();
    assert_eq!(Settings::load(dir.path()).await.unwrap(), settings);
}
#[test]
fn settings_reject_invalid_memory_and_theme() {
    let mut settings = Settings::default();
    settings.memory_mib = 1;
    assert!(settings.validate().is_err());
    settings.memory_mib = 4096;
    settings.appearance = "bad".into();
    assert!(settings.validate().is_err());
}
#[tokio::test]
async fn duplicate_copies_saves_without_sharing_writable_files() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().into());
    let original = store
        .create("World", "1.8.9", Loader::Vanilla, false)
        .await
        .unwrap();
    let source = store
        .directory(&original.id)
        .await
        .unwrap()
        .join("game/saves/level.dat");
    tokio::fs::write(&source, b"original world").await.unwrap();
    let copy = store
        .duplicate(&original.id, &CancellationToken::new())
        .await
        .unwrap();
    assert_ne!(copy.id, original.id);
    let destination = store
        .directory(&copy.id)
        .await
        .unwrap()
        .join("game/saves/level.dat");
    assert_eq!(
        tokio::fs::read(&destination).await.unwrap(),
        b"original world"
    );
    tokio::fs::write(destination, b"modified copy")
        .await
        .unwrap();
    assert_eq!(tokio::fs::read(source).await.unwrap(), b"original world");
}
#[test]
fn java_versions_are_metadata_compatible() {
    assert_eq!(runtime::major_version("1.8.0_51").unwrap(), 8);
    assert_eq!(runtime::major_version("21.0.13").unwrap(), 21);
    assert_eq!(runtime::major_version("25.0.1+8").unwrap(), 25);
}
#[test]
fn maven_coordinates_and_windows_rules() {
    assert_eq!(
        maven_path("org.example:core:1.0:natives").unwrap(),
        "org/example/core/1.0/core-1.0-natives.jar"
    );
    assert!(maven_path("org.example:../bad:1").is_err());
    assert!(
        !library_allowed(&json!({"rules":[{"action":"allow","os":{"name":"linux"}}]})).unwrap()
    );
    assert!(
        library_allowed(&json!({"rules":[{"action":"allow","os":{"name":"windows"}}]})).unwrap()
    );
}
#[tokio::test]
async fn native_zip_slip_is_rejected() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.zip");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    zip.start_file("../escape.dll", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"not executable").unwrap();
    zip.finish().unwrap();
    assert!(
        extract_natives(
            path,
            dir.path().join("native"),
            HashSet::new(),
            CancellationToken::new()
        )
        .await
        .is_err()
    );
    assert!(!dir.path().join("escape.dll").exists());
}

#[tokio::test]
async fn instance_overrides_persist_and_are_validated() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().to_path_buf());
    let instance = store
        .create("Overrides", "1.21.11", Loader::Fabric, false)
        .await
        .unwrap();
    assert_eq!(instance.memory_mib, None);
    store
        .configure(&instance.id, Some(2048), "C:/Java/bin/java.exe")
        .await
        .unwrap();
    let saved = store.read(&instance.id).await.unwrap();
    assert_eq!(saved.memory_mib, Some(2048));
    assert_eq!(saved.java_path, "C:/Java/bin/java.exe");
    assert!(store.configure(&instance.id, Some(512), "").await.is_err());
    assert!(
        store
            .configure(&instance.id, None, "bad\npath")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn manifests_without_instance_overrides_still_load() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().to_path_buf());
    let instance = store
        .create("Legacy", "1.8.9", Loader::Forge, false)
        .await
        .unwrap();
    let manifest = serde_json::json!({
        "schema": 1,
        "id": instance.id,
        "name": instance.name,
        "version": instance.version,
        "loader": "FORGE",
        "status": "READY",
        "detail": "Legacy manifest"
    });
    let file = store
        .directory(&instance.id)
        .await
        .unwrap()
        .join("instance.json");
    fs::write(&file, serde_json::to_vec_pretty(&manifest).unwrap())
        .await
        .unwrap();
    let read = store.read(&instance.id).await.unwrap();
    assert_eq!(read.status, "READY");
    assert_eq!(read.memory_mib, None);
    assert!(read.java_path.is_empty());
}

// ---------------------------------------------------------------- M3 additions

#[test]
fn launch_arguments_substitute_known_variables_only() {
    use crate::launch::substitute_for_test;
    let result = substitute_for_test(
        "--username ${auth_player_name} --uuid ${auth_uuid} --x ${unknown}",
        &[("auth_player_name", "Player"), ("auth_uuid", "abc")],
    );
    assert_eq!(result, "--username Player --uuid abc --x ");
}

#[test]
fn launch_rules_follow_the_host_platform() {
    use crate::launch::rules_for_test;
    let windows = serde_json::json!([{"action": "allow", "os": {"name": "windows"}}]);
    assert_eq!(rules_for_test(Some(&windows)), cfg!(target_os = "windows"));
    let features = serde_json::json!([{"action": "allow", "features": {"is_demo_user": true}}]);
    assert!(!rules_for_test(Some(&features)));
}

#[test]
fn memory_overrides_fall_back_to_settings() {
    use crate::{Instance, Loader, launch::effective_memory};
    let instance = Instance {
        schema: 1,
        id: uuid::Uuid::new_v4().to_string(),
        name: "x".into(),
        version: "1.21.11".into(),
        loader: Loader::Vanilla,
        status: "READY".into(),
        detail: String::new(),
        memory_mib: None,
        java_path: String::new(),
    };
    assert_eq!(effective_memory(&instance, 4096), 4096);
    let mut overridden = instance.clone();
    overridden.memory_mib = Some(6144);
    assert_eq!(effective_memory(&overridden, 4096), 6144);
}

#[test]
fn account_selection_and_removal_are_consistent() {
    use crate::auth::{Accounts, test_account};
    let mut accounts = Accounts::default();
    accounts.upsert(test_account("11111111-1111-1111-1111-111111111111", "One"));
    accounts.upsert(test_account("22222222-2222-2222-2222-222222222222", "Two"));
    assert_eq!(accounts.accounts.len(), 2);
    assert_eq!(
        accounts.active_account().map(|a| a.name.clone()),
        Some("Two".into())
    );
    accounts
        .select("11111111-1111-1111-1111-111111111111")
        .unwrap();
    assert_eq!(
        accounts.active_account().map(|a| a.name.clone()),
        Some("One".into())
    );
    accounts.remove("11111111-1111-1111-1111-111111111111");
    assert_eq!(
        accounts.active_account().map(|a| a.name.clone()),
        Some("Two".into())
    );
    assert!(
        accounts
            .select("33333333-3333-3333-3333-333333333333")
            .is_err()
    );
}

#[test]
fn skin_composition_produces_a_valid_png() {
    use crate::skin::{Skin, encode_png, model};
    let texture = Skin::fallback();
    let composed = model(&texture);
    assert_eq!(&composed[..8], b"\x89PNG\r\n\x1a\n");
    assert!(composed.len() > 1000);
    let decoded = Skin::decode(&composed);
    assert!(
        decoded.is_err(),
        "the composed character is not a 64x64 skin"
    );
    assert_eq!(
        &encode_png(1, 1, &[255, 0, 0, 255])[..8],
        b"\x89PNG\r\n\x1a\n"
    );
}

#[test]
fn onboarding_answers_and_loader_pairs_are_validated() {
    use crate::onboarding::Onboarding;
    let answers = Onboarding::default();
    answers.validate().unwrap();
    let mut bad = answers.clone();
    bad.version = "0.0.0".into();
    assert!(bad.validate().is_err());
    let mut incompatible = answers;
    incompatible.loader = "Forge".into();
    assert!(incompatible.validate().is_err());
}

#[test]
fn every_valid_wizard_step_advances_exactly_once() {
    use crate::onboarding::{Direction, LAST_STEP, Progress};
    let answers = crate::onboarding::Onboarding::default();
    let mut progress = Progress::default();
    for expected in 0..LAST_STEP {
        progress = progress
            .navigate(expected, Direction::Next, answers.clone())
            .unwrap();
        assert_eq!(progress.current_step(), expected + 1);
    }
    assert!(progress.finish(LAST_STEP, answers).unwrap().completed);
}

#[test]
fn wizard_back_skip_and_repeated_next_are_safe() {
    use crate::onboarding::{Direction, Progress};
    let answers = crate::onboarding::Onboarding::default();
    let first = Progress::default()
        .navigate(0, Direction::Next, answers.clone())
        .unwrap();
    assert!(
        first.navigate(0, Direction::Next, answers.clone()).is_err(),
        "a queued second click must not advance a second step"
    );
    let skipped = first.navigate(1, Direction::Skip, answers.clone()).unwrap();
    assert_eq!(skipped.current_step(), 2);
    let backed = skipped
        .navigate(2, Direction::Back, answers.clone())
        .unwrap();
    assert_eq!(backed.current_step(), 1);
    assert!(
        Progress {
            step: 3,
            ..Default::default()
        }
        .navigate(3, Direction::Skip, answers)
        .is_err()
    );
}

#[test]
fn invalid_minecraft_selection_stays_on_its_step() {
    use crate::onboarding::{Direction, Progress};
    let progress = Progress {
        step: 4,
        ..Default::default()
    };
    let mut answers = progress.answers.clone();
    answers.loader = "Forge".into();
    assert!(progress.navigate(4, Direction::Next, answers).is_err());
    assert_eq!(progress.current_step(), 4);
}

#[test]
fn hardware_or_metadata_failure_does_not_lock_setup() {
    use crate::onboarding::{Direction, Progress};
    let answers = crate::onboarding::Onboarding::default();
    // Neither transition depends on a Hardware value or a fetched metadata response.
    // The checked-in supported target list is the recoverable fallback.
    let hardware_unavailable = Progress {
        step: 2,
        ..Default::default()
    };
    assert_eq!(
        hardware_unavailable
            .navigate(2, Direction::Next, answers.clone())
            .unwrap()
            .current_step(),
        3
    );
    let metadata_unavailable = Progress {
        step: 4,
        answers: answers.clone(),
        ..Default::default()
    };
    assert_eq!(
        metadata_unavailable
            .navigate(4, Direction::Next, answers)
            .unwrap()
            .current_step(),
        5
    );
}

#[tokio::test]
async fn interrupted_setup_resumes_and_completion_persists() {
    use crate::onboarding::{self, Direction, Progress};
    let dir = tempfile::tempdir().unwrap();
    let answers = onboarding::Onboarding::default();
    assert_eq!(onboarding::load(dir.path()).await, Progress::default());
    let interrupted = Progress::default()
        .navigate(0, Direction::Next, answers.clone())
        .unwrap();
    onboarding::save(dir.path(), &interrupted).await.unwrap();
    assert_eq!(onboarding::load(dir.path()).await.current_step(), 1);

    let mut progress = interrupted;
    for expected in 1..onboarding::LAST_STEP {
        progress = progress
            .navigate(expected, Direction::Next, answers.clone())
            .unwrap();
    }
    let completed = progress.finish(onboarding::LAST_STEP, answers).unwrap();
    onboarding::save(dir.path(), &completed).await.unwrap();
    let restarted = onboarding::load(dir.path()).await;
    assert!(restarted.completed);
    assert!(
        !restarted.needs_setup(),
        "completed setup must route to the launcher"
    );
}

#[tokio::test]
async fn setup_later_opens_launcher_without_mutating_instances() {
    use crate::onboarding::{self, Progress};
    let dir = tempfile::tempdir().unwrap();
    let before = std::fs::read_dir(dir.path()).unwrap().count();
    let deferred = Progress::default()
        .defer(0, onboarding::Onboarding::default())
        .unwrap();
    onboarding::save(dir.path(), &deferred).await.unwrap();
    assert!(!onboarding::load(dir.path()).await.needs_setup());
    assert_eq!(before + 1, std::fs::read_dir(dir.path()).unwrap().count());
    assert!(!dir.path().join("instances").exists());
}

#[test]
fn hardware_recommendations_leave_the_system_room() {
    use crate::hardware::Hardware;
    let small = Hardware {
        memory_mib: 4_000,
        cores: 2,
        ..Default::default()
    };
    assert_eq!(small.recommended_memory_mib(), 1952);
    let large = Hardware {
        memory_mib: 32_000,
        cores: 12,
        ..Default::default()
    };
    assert_eq!(large.recommended_memory_mib(), 4096);
    assert!(large.recommended_memory_mib() as u64 + 2048 <= large.memory_mib);
}
