// SPDX-License-Identifier: GPL-3.0-only
//! Token-free metadata on disk; refresh credentials in Windows Credential Manager,
//! macOS Keychain, or Linux Secret Service. No plaintext or mock fallback.
use crate::auth::{Account, Accounts, MsTokens};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

const METADATA: &str = "accounts.json";

#[derive(Default, Serialize, Deserialize)]
struct Metadata {
    #[serde(default)]
    pending_removals: Vec<String>,
    active: Option<String>,
    accounts: Vec<Record>,
}
#[derive(Serialize, Deserialize)]
struct Record {
    application_id: String,
    uuid: String,
    name: String,
    skin_url: String,
    owns_minecraft: bool,
    added_at: u64,
}
impl From<&Accounts> for Metadata {
    fn from(accounts: &Accounts) -> Self {
        Self {
            pending_removals: vec![],
            active: accounts.active.clone(),
            accounts: accounts
                .accounts
                .iter()
                .map(|a| Record {
                    application_id: a.ms.application_id.clone(),
                    uuid: a.uuid.clone(),
                    name: a.name.clone(),
                    skin_url: a.skin_url.clone(),
                    owns_minecraft: a.owns_minecraft,
                    added_at: a.added_at,
                })
                .collect(),
        }
    }
}
trait Vault {
    fn put(&self, id: &str, secret: &str) -> Result<()>;
    fn get(&self, id: &str) -> Result<String>;
    fn delete(&self, id: &str) -> Result<()>;
}
struct OsVault {
    service: String,
}
impl OsVault {
    fn new(root: &Path) -> Result<Self> {
        use sha2::{Digest, Sha256};
        let root = std::fs::canonicalize(root).context("Cannot resolve account storage")?;
        let key = hex::encode(Sha256::digest(root.to_string_lossy().as_bytes()));
        Ok(Self {
            service: format!("org.aeroclient.accounts.{}", &key[..24]),
        })
    }
    fn entry(&self, id: &str) -> Result<keyring::Entry> {
        ensure!(
            uuid::Uuid::parse_str(id).is_ok(),
            "Invalid Minecraft account UUID"
        );
        keyring::Entry::new(&self.service, id)
            .map_err(|_| anyhow::anyhow!("The operating system credential vault is unavailable. Unlock your keychain and retry."))
    }
}
impl Vault for OsVault {
    fn put(&self, id: &str, secret: &str) -> Result<()> {
        ensure!(
            !secret.is_empty(),
            "Session expired. Sign in with Microsoft again."
        );
        self.entry(id)?.set_password(secret)
            .map_err(|_| anyhow::anyhow!("Cannot securely save the Microsoft session in the operating system credential vault."))
    }
    fn get(&self, id: &str) -> Result<String> {
        self.entry(id)?.get_password()
            .map_err(|_| anyhow::anyhow!("The saved Microsoft credential is missing or locked. Unlock the credential vault or sign in again."))
    }
    fn delete(&self, id: &str) -> Result<()> {
        match self.entry(id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => anyhow::bail!(
                "Cannot remove the Microsoft credential from the operating system vault. Unlock it and retry."
            ),
        }
    }
}
async fn metadata(root: &Path) -> Result<Metadata> {
    let file = root.join(METADATA);
    if !fs::try_exists(&file).await? {
        return Ok(Metadata::default());
    }
    let bytes = fs::read(file).await?;
    ensure!(bytes.len() <= 1024 * 1024, "Account metadata is too large");
    serde_json::from_slice(&bytes).context("Saved account metadata is invalid")
}
fn restore(metadata: Metadata, vault: &impl Vault) -> Result<Accounts> {
    let mut accounts = Accounts {
        active: metadata.active,
        accounts: vec![],
    };
    for record in metadata.accounts {
        let refresh_token = vault.get(&record.uuid)?;
        accounts.accounts.push(Account {
            uuid: record.uuid,
            name: record.name,
            skin_url: record.skin_url,
            owns_minecraft: record.owns_minecraft,
            added_at: record.added_at,
            ms: MsTokens {
                application_id: record.application_id,
                refresh_token,
                ..Default::default()
            },
            access_token: String::new(),
            access_expires_at: 0,
        });
    }
    ensure!(
        accounts.active.is_none() || accounts.active_account().is_some(),
        "Unknown active account in metadata"
    );
    Ok(accounts)
}
pub async fn load(root: &Path) -> Result<Accounts> {
    fs::create_dir_all(root).await?;
    if !fs::try_exists(root.join(METADATA)).await?
        && fs::try_exists(root.join("accounts.bin")).await?
    {
        anyhow::bail!(
            "Aero 2.0 uses the operating system credential vault. Sign in again to replace the older account file."
        );
    }
    let mut metadata = metadata(root).await?;
    if !metadata.pending_removals.is_empty() {
        let removed = metadata.pending_removals.clone();
        let vault_root = root.to_owned();
        tokio::task::spawn_blocking(move || {
            let vault = OsVault::new(&vault_root)?;
            for id in removed {
                vault.delete(&id)?;
            }
            Ok::<_, anyhow::Error>(())
        })
        .await
        .context("Credential vault task failed")??;
        metadata.pending_removals.clear();
        crate::storage::write_atomic(&root.join(METADATA), &serde_json::to_vec(&metadata)?).await?;
    }
    let root = root.to_owned();
    tokio::task::spawn_blocking(move || restore(metadata, &OsVault::new(&root)?))
        .await
        .context("Credential vault task failed")?
}
pub async fn save(root: &Path, accounts: &Accounts) -> Result<()> {
    fs::create_dir_all(root).await?;
    let previous = metadata(root).await?;
    let mut next = Metadata::from(accounts);
    let root_owned = root.to_owned();
    let accounts_owned = accounts.clone();
    tokio::task::spawn_blocking(move || {
        let vault = OsVault::new(&root_owned)?;
        for account in &accounts_owned.accounts {
            vault.put(&account.uuid, &account.ms.refresh_token)?;
        }
        Ok::<_, anyhow::Error>(())
    })
    .await
    .context("Credential vault task failed")??;
    let mut removed: Vec<_> = previous
        .accounts
        .into_iter()
        .filter(|record| !accounts.accounts.iter().any(|a| a.uuid == record.uuid))
        .map(|record| record.uuid)
        .collect();
    removed.extend(previous.pending_removals);
    removed.retain(|id| !accounts.accounts.iter().any(|a| &a.uuid == id));
    removed.sort();
    removed.dedup();
    next.pending_removals = removed.clone();
    crate::storage::write_atomic(&root.join(METADATA), &serde_json::to_vec(&next)?).await?;
    let root_owned = root.to_owned();
    tokio::task::spawn_blocking(move || {
        let vault = OsVault::new(&root_owned)?;
        for id in removed {
            vault.delete(&id)?;
        }
        Ok::<_, anyhow::Error>(())
    })
    .await
    .context("Credential vault task failed")??;
    next.pending_removals.clear();
    crate::storage::write_atomic(&root.join(METADATA), &serde_json::to_vec(&next)?).await?;
    // An explicit successful sign-in replaces the obsolete token-file format.
    if fs::try_exists(root.join("accounts.bin")).await? {
        fs::remove_file(root.join("accounts.bin")).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[derive(Default)]
    struct MemoryVault(RefCell<std::collections::HashMap<String, String>>);
    impl Vault for MemoryVault {
        fn put(&self, id: &str, secret: &str) -> Result<()> {
            self.0.borrow_mut().insert(id.into(), secret.into());
            Ok(())
        }
        fn get(&self, id: &str) -> Result<String> {
            self.0
                .borrow()
                .get(id)
                .cloned()
                .context("Missing credential")
        }
        fn delete(&self, id: &str) -> Result<()> {
            self.0.borrow_mut().remove(id);
            Ok(())
        }
    }
    #[test]
    fn metadata_never_serializes_tokens_and_restart_requires_refresh() {
        let account = crate::auth::test_account("11111111222233334444555555555555", "Player");
        let accounts = Accounts {
            active: Some(account.uuid.clone()),
            accounts: vec![account.clone()],
        };
        let json = serde_json::to_string(&Metadata::from(&accounts)).unwrap();
        assert!(!json.contains("refresh"));
        assert!(!json.contains("access_token"));
        let vault = MemoryVault::default();
        vault.put(&account.uuid, &account.ms.refresh_token).unwrap();
        let restored = restore(serde_json::from_str(&json).unwrap(), &vault).unwrap();
        assert_eq!(
            restored.accounts[0].ms.refresh_token,
            account.ms.refresh_token
        );
        assert!(!restored.accounts[0].is_active_token());
        vault.delete(&account.uuid).unwrap();
        assert!(restore(Metadata::from(&accounts), &vault).is_err());
        assert!(!format!("{account:?}").contains("refresh"));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn windows_credential_manager_roundtrip_and_removal() {
        let root = tempfile::tempdir().unwrap();
        let mut accounts = Accounts::default();
        let account = crate::auth::test_account("11111111222233334444555555555555", "Player");
        accounts.upsert(account.clone());
        struct Cleanup {
            root: std::path::PathBuf,
            id: String,
        }
        impl Drop for Cleanup {
            fn drop(&mut self) {
                if let Ok(vault) = OsVault::new(&self.root) {
                    let _ = vault.delete(&self.id);
                }
            }
        }
        let _cleanup = Cleanup {
            root: root.path().to_owned(),
            id: account.uuid.clone(),
        };
        save(root.path(), &accounts).await.unwrap();
        let disk = fs::read_to_string(root.path().join(METADATA))
            .await
            .unwrap();
        assert!(!disk.contains("refresh"));
        let restored = load(root.path()).await.unwrap();
        assert_eq!(
            restored.accounts[0].ms.refresh_token,
            account.ms.refresh_token
        );
        assert!(!restored.accounts[0].is_active_token());
        accounts.remove(&account.uuid);
        save(root.path(), &accounts).await.unwrap();
        assert!(load(root.path()).await.unwrap().accounts.is_empty());
        assert!(
            OsVault::new(root.path())
                .unwrap()
                .get(&account.uuid)
                .is_err()
        );
    }
}
