// SPDX-License-Identifier: GPL-3.0-only
//! Microsoft / Xbox / Minecraft authentication.
//!
//! Browser OAuth PKCE → Xbox → XSTS → Minecraft; refresh tokens live only in the OS vault.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::fs;

/// A client id is public application metadata, not a secret. Aero intentionally
/// does not borrow Minecraft's retired legacy id or another launcher's identity.
pub use crate::oauth_pkce::{BrowserLogin, begin};

pub fn client_id(configured: &str) -> Result<String> {
    let value = std::env::var("AERO_MS_CLIENT_ID")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            if configured.trim().is_empty() {
                option_env!("AERO_MS_CLIENT_ID")
                    .unwrap_or("")
                    .trim()
                    .to_owned()
            } else {
                configured.trim().to_owned()
            }
        });
    ensure!(
        !value.is_empty(),
        "Microsoft sign-in needs Aero's Azure public-client ID. Add it in Settings or set AERO_MS_CLIENT_ID; see README.md."
    );
    ensure!(
        uuid::Uuid::parse_str(&value).is_ok(),
        "Microsoft client ID must be an Azure application GUID"
    );
    Ok(value)
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or_default()
}

fn http() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(20))
        .read_timeout(Duration::from_secs(30))
        .user_agent("AeroClient/2.0.0 (+https://github.com/karmangamerz-coder/Aero-client)")
        .build()?)
}

pub(crate) fn oauth_error(code: &str) -> String {
    match code {
        "invalid_grant" | "interaction_required" | "login_required" => "Your Microsoft session expired or was revoked. Sign in again.",
        "unauthorized_client" | "invalid_client" => "Microsoft rejected Aero's application ID. Configure Aero's own approved public-client registration.",
        "access_denied" | "consent_required" => "Microsoft sign-in was declined or requires consent. Family account restrictions must be resolved by the family organizer.",
        _ => "Microsoft could not complete sign-in. Check the app registration and connection, then retry.",
    }.into()
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MsTokens {
    #[serde(default)]
    pub application_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub uuid: String,
    /// Minecraft username, the only identity shown in the interface.
    pub name: String,
    pub skin_url: String,
    #[serde(default = "default_true")]
    pub owns_minecraft: bool,
    pub ms: MsTokens,
    /// Minecraft services session token, renewed from the Microsoft chain.
    pub access_token: String,
    pub access_expires_at: u64,
    #[serde(default)]
    pub added_at: u64,
}
fn default_true() -> bool {
    true
}
impl Account {
    pub fn is_active_token(&self) -> bool {
        self.access_expires_at > now() && !self.access_token.is_empty()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Accounts {
    pub accounts: Vec<Account>,
    pub active: Option<String>,
}
impl Accounts {
    pub fn active_account(&self) -> Option<&Account> {
        let uuid = self.active.as_deref()?;
        self.accounts.iter().find(|account| account.uuid == uuid)
    }
    pub fn select(&mut self, uuid: &str) -> Result<()> {
        ensure!(
            self.accounts.iter().any(|a| a.uuid == uuid),
            "Unknown account"
        );
        self.active = Some(uuid.to_owned());
        Ok(())
    }
    pub fn remove(&mut self, uuid: &str) {
        self.accounts.retain(|a| a.uuid != uuid);
        if self.active.as_deref() == Some(uuid) {
            self.active = self.accounts.first().map(|a| a.uuid.clone());
        }
    }
    pub fn upsert(&mut self, account: Account) {
        self.accounts.retain(|a| a.uuid != account.uuid);
        self.active = Some(account.uuid.clone());
        self.accounts.push(account);
    }
}

/// Only non-secret account metadata is stored in the data directory.
pub async fn load_accounts(root: &Path) -> Result<Accounts> {
    crate::vault::load(root).await
}
pub async fn save_accounts(root: &Path, accounts: &Accounts) -> Result<()> {
    crate::vault::save(root, accounts).await
}
impl std::fmt::Debug for MsTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MsTokens([redacted])")
    }
}
impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("uuid", &self.uuid)
            .field("name", &self.name)
            .field("session", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub skin_url: String,
}

/// Xbox → XSTS → Minecraft, returning the game session token and profile.
async fn minecraft_chain(client: &reqwest::Client, ms: &MsTokens) -> Result<(Account, Profile)> {
    let game = minecraft_msa_auth::MinecraftAuthorizationFlow::new(MinecraftHttp(client.clone()))
        .exchange_microsoft_token(&ms.access_token)
        .await
        .map_err(chain_error)?;
    let access_token = game.access_token().as_ref().to_owned();
    let access_expires_at = now() + u64::from(game.expires_in()).saturating_sub(120);

    let entitlements = client
        .get("https://api.minecraftservices.com/entitlements/mcstore")
        .bearer_auth(&access_token)
        .send()
        .await
        .context("Cannot verify Minecraft ownership")?;
    let status = entitlements.status();
    let value: serde_json::Value = entitlements.json().await.unwrap_or_default();
    ensure!(
        status.is_success(),
        "Cannot verify Minecraft ownership (HTTP {status})"
    );
    verify_ownership(&value)?;
    let owns_minecraft = true;

    let profile = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(&access_token)
        .send()
        .await
        .context("Cannot load the Minecraft profile")?;
    let profile_status = profile.status();
    let value: serde_json::Value = profile.json().await.unwrap_or_default();
    ensure!(
        profile_status.is_success(),
        "No Minecraft profile for this account (HTTP {profile_status}). Sign in to minecraft.net once, then retry."
    );
    let profile = Profile {
        uuid: value["id"].as_str().context("Profile has no id")?.into(),
        name: value["name"]
            .as_str()
            .context("Profile has no name")?
            .into(),
        skin_url: value["skins"]
            .as_array()
            .and_then(|skins| {
                skins
                    .iter()
                    .find(|s| s["state"] == "ACTIVE")
                    .or_else(|| skins.first())
            })
            .and_then(|skin| skin["url"].as_str())
            .unwrap_or("")
            .into(),
    };
    uuid::Uuid::parse_str(&profile.uuid).context("Minecraft returned an invalid profile UUID")?;
    ensure!(
        !profile.name.is_empty()
            && profile.name.len() <= 16
            && profile
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "Minecraft returned an invalid player name"
    );
    let account = Account {
        uuid: profile.uuid.clone(),
        name: profile.name.clone(),
        skin_url: profile.skin_url.clone(),
        owns_minecraft,
        ms: ms.clone(),
        access_token,
        access_expires_at,
        added_at: now(),
    };
    Ok((account, profile))
}

/// Complete a PKCE sign-in and return the ready account.
pub async fn finish(client: &reqwest::Client, tokens: &MsTokens) -> Result<(Account, Profile)> {
    minecraft_chain(client, tokens).await
}

/// Renew a Minecraft session token from the stored Microsoft refresh token.
pub async fn renew(
    client: &reqwest::Client,
    account: &Account,
    configured_client_id: &str,
) -> Result<Account> {
    ensure!(
        account.ms.application_id == client_id(configured_client_id)?,
        "This session belongs to a different application registration. Sign in to Aero again."
    );
    ensure!(
        !account.ms.refresh_token.is_empty(),
        "This account needs to sign in again"
    );
    let ms =
        crate::oauth_pkce::refresh(client, configured_client_id, &account.ms.refresh_token).await?;
    let (fresh, _) = minecraft_chain(client, &ms).await?;
    ensure!(
        fresh.uuid == account.uuid,
        "Signed-in account no longer matches the saved profile"
    );
    Ok(fresh)
}

pub fn client() -> Result<reqwest::Client> {
    http()
}

/// Where the launcher caches downloaded skins (public texture CDN, no account data).
pub fn skin_cache(root: &Path, uuid: &str) -> PathBuf {
    root.join("skins").join(format!("{uuid}.png"))
}

/// Fetch and cache the account skin; returns raw PNG bytes.
pub async fn skin_bytes(
    client: &reqwest::Client,
    account: &Account,
    root: &Path,
) -> Result<Vec<u8>> {
    let cache = skin_cache(root, &account.uuid);
    if let Ok(bytes) = fs::read(&cache).await {
        if !bytes.is_empty() && bytes.len() <= 1024 * 1024 {
            return Ok(bytes);
        }
    }
    if account.skin_url.is_empty() {
        bail!("No skin on this profile");
    }
    let url = reqwest::Url::parse(&account.skin_url)?;
    ensure!(
        url.scheme() == "https"
            && matches!(
                url.host_str().unwrap_or(""),
                "textures.minecraft.net" | "api.minecraftservices.com"
            ),
        "Unexpected skin host"
    );
    let response = client
        .get(url)
        .send()
        .await
        .context("Cannot download the skin")?;
    ensure!(
        response.status().is_success(),
        "Skin download failed (HTTP {})",
        response.status()
    );
    let bytes = response.bytes().await?;
    ensure!(
        !bytes.is_empty() && bytes.len() <= 1024 * 1024,
        "Skin file is too large"
    );
    if let Some(parent) = cache.parent() {
        fs::create_dir_all(parent).await.ok();
    }
    fs::write(&cache, &bytes).await.ok();
    Ok(bytes.to_vec())
}

#[cfg(test)]
pub fn test_account(uuid: &str, name: &str) -> Account {
    Account {
        uuid: uuid.into(),
        name: name.into(),
        skin_url: String::new(),
        owns_minecraft: true,
        ms: MsTokens {
            application_id: "11111111-2222-3333-4444-555555555555".into(),
            access_token: "ms".into(),
            refresh_token: "refresh".into(),
            expires_at: now() + 3600,
        },
        access_token: "mc".into(),
        access_expires_at: now() + 3600,
        added_at: 0,
    }
}
#[derive(Clone)]
struct MinecraftHttp(reqwest::Client);
#[async_trait::async_trait]
impl minecraft_msa_auth::HttpClient for MinecraftHttp {
    type Error = reqwest::Error;
    async fn call(
        &self,
        request: minecraft_msa_auth::HttpRequest,
    ) -> std::result::Result<minecraft_msa_auth::HttpResponse, Self::Error> {
        let response = self.0.execute(reqwest::Request::try_from(request)?).await?;
        let status = response.status();
        let body = response.bytes().await?.to_vec();
        let mut result = minecraft_msa_auth::HttpResponse::new(body);
        *result.status_mut() = status;
        Ok(result)
    }
}
fn chain_error<E: std::error::Error>(
    error: minecraft_msa_auth::MinecraftAuthorizationError<E>,
) -> anyhow::Error {
    use minecraft_msa_auth::MinecraftAuthorizationError as Error;
    let message = match error {
        Error::AddToFamily => {
            "Microsoft requires this account to belong to a family. Ask the family organizer to resolve the restriction."
        }
        Error::NoXbox => {
            "This account has no Xbox profile. Create a profile at xbox.com, then retry."
        }
        Error::XboxLive {
            code: 2148916236 | 2148916237,
        } => {
            "Xbox requires age or adult verification. Complete the official account steps with your family organizer."
        }
        Error::XboxLive { code: 2148916235 } => {
            "Xbox Live is unavailable in this account's region."
        }
        Error::HttpStatus(status) if status.as_u16() == 403 => {
            "Minecraft rejected Aero's application registration. Its client ID may need Minecraft API approval."
        }
        Error::HttpStatus(status) if status.as_u16() == 401 => {
            "Microsoft session expired or was rejected. Sign in again."
        }
        _ => {
            "Xbox or Minecraft authentication failed. Check the connection and account permissions, then retry."
        }
    };
    anyhow::anyhow!(message)
}

#[cfg(test)]
mod security_tests {
    use super::*;
    #[test]
    fn ownership_requires_a_java_entitlement() {
        for value in [
            serde_json::json!({}),
            serde_json::json!({"items":[]}),
            serde_json::json!({"items":[{"name":"demo"}]}),
        ] {
            assert!(verify_ownership(&value).is_err());
        }
        for name in ["product_minecraft", "game_minecraft"] {
            assert!(verify_ownership(&serde_json::json!({"items":[{"name":name}]})).is_ok());
        }
    }
    #[test]
    fn family_and_application_errors_are_actionable_and_safe() {
        use minecraft_msa_auth::MinecraftAuthorizationError as Error;
        assert!(
            chain_error::<std::io::Error>(Error::AddToFamily)
                .to_string()
                .contains("family")
        );
        assert!(
            chain_error::<std::io::Error>(Error::NoXbox)
                .to_string()
                .contains("Xbox profile")
        );
        let status = reqwest::StatusCode::FORBIDDEN;
        assert!(
            chain_error::<std::io::Error>(Error::HttpStatus(status))
                .to_string()
                .contains("approval")
        );
        let transport = Error::Http(std::io::Error::other("synthetic-secret"));
        assert!(
            !chain_error(transport)
                .to_string()
                .contains("synthetic-secret")
        );
        assert!(oauth_error("invalid_grant").contains("expired"));
    }
    #[tokio::test]
    async fn refresh_rejects_a_different_application_before_network_access() {
        let mut account = test_account("11111111222233334444555555555555", "Player");
        account.ms.application_id = "different-app".into();
        let result = renew(
            &client().unwrap(),
            &account,
            "11111111-2222-3333-4444-555555555555",
        )
        .await;
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("different application")
        );
    }
}
fn verify_ownership(value: &serde_json::Value) -> Result<()> {
    let owns_minecraft = value["items"]
        .as_array()
        .map(|items| {
            items.iter().any(|item| {
                matches!(
                    item["name"].as_str().unwrap_or(""),
                    "product_minecraft" | "game_minecraft"
                )
            })
        })
        .unwrap_or(false);
    ensure!(
        owns_minecraft,
        "This Microsoft account does not own Minecraft: Java Edition. Aero never bypasses ownership checks."
    );
    Ok(())
}
