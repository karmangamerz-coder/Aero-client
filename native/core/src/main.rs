// SPDX-License-Identifier: MIT
use aero_core::{CancellationToken, Installer, Loader, Store};
use std::{path::Path, sync::Arc};

#[tokio::main(worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let token = CancellationToken::new();
    let installer = Installer::new(
        Store::default_root(),
        Arc::new(|p| {
            if p.files_total > 0 {
                println!("{}", p.item);
            }
        }),
    )?;
    match args.first().map(String::as_str) {
        Some("versions") => {
            println!("Available: {:?}", installer.versions(true, &token).await?);
            for version in aero_core::TARGETS {
                println!(
                    "{version}: {}",
                    installer.version(version, &token).await?["javaVersion"]
                );
            }
        }
        Some("list") => println!(
            "{}",
            serde_json::to_string_pretty(&installer.store.list().await?)?
        ),
        Some("install") | Some("repair") => {
            let version = args.get(1).ok_or_else(|| {
                anyhow::anyhow!(
                    "Usage: aero-cli install VERSION vanilla|fabric|forge [custom-java]"
                )
            })?;
            let loader = match args.get(2).map(String::as_str).unwrap_or("vanilla") {
                "vanilla" => Loader::Vanilla,
                "fabric" => Loader::Fabric,
                "forge" => Loader::Forge,
                _ => anyhow::bail!("Unknown loader"),
            };
            let instance = installer
                .store
                .create(&format!("Aero Default {version}"), version, loader, true)
                .await?;
            let result = installer
                .install(&instance, args.get(3).map(Path::new), &token)
                .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some("verify-loaders") => {
            for version in aero_core::TARGETS {
                let loader = if version == "1.8.9" {
                    Loader::Forge
                } else {
                    Loader::Fabric
                };
                let instance = installer
                    .store
                    .create(&format!("Aero Default {version}"), version, loader, true)
                    .await?;
                let profile = installer.loader(&instance, &token).await?.unwrap();
                println!("{version}: {}", profile["id"]);
            }
        }
        _ => println!(
            "Aero CLI: versions | list | verify-loaders | install VERSION vanilla|fabric|forge [custom-java]\nSet AERO_DATA_DIR to use isolated validation storage. Authentication and game launch are unavailable."
        ),
    }
    Ok(())
}
