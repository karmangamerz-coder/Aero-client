// SPDX-License-Identifier: MIT
//! Microsoft / Xbox / Minecraft authentication.
//!
//! Flow: OAuth 2.0 device code (official Microsoft endpoint, browser based — Aero
//! never sees the password) → Xbox Live → XSTS → Minecraft services → entitlement
//! check → profile and skin. Tokens are cached encrypted with DPAPI on Windows and
//! are never logged, printed or sent anywhere except the official endpoints.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::fs;

/// Public OAuth client id. Never a secret: it identifies the application, not a
/// user. Owners can override it (ADR 0007) with their own Azure public client.
pub const DEFAULT_CLIENT_ID: &str = "00000000402b5328";
const AUTHORITY: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0";
const SCOPE: &str = "XboxLive.signin offline_access";
const ACCOUNTS_FILE: &str = "accounts.bin";

pub fn client_id() -> String {
    std::env::var("AERO_MS_CLIENT_ID")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_owned())
}

fn now() -> u64 {
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
        .user_agent("AeroClient/0.4.0 (+https://github.com/karmangamerz-coder/Aero-client)")
        .build()?)
}

fn redact(text: &str) -> String {
    // API errors can echo a token back; keep messages short and never log bodies
    // that contain authorization material.
    if text.contains("access_token") || text.contains("refresh_token") || text.contains("Token") {
        "authentication failed; details withheld".into()
    } else {
        text.chars().take(240).collect()
    }
}

async fn post_json(
    client: &reqwest::Client,
    url: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value> {
    let response = client
        .post(url)
        .json(body)
        .send()
        .await
        .with_context(|| format!("Cannot reach {url}"))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    let value: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({"error": redact(&text)}));
    ensure!(
        status.is_success(),
        "{}",
        auth_error(status.as_u16(), &value)
    );
    Ok(value)
}

fn auth_error(status: u16, value: &serde_json::Value) -> String {
    if let Some(code) = value["XErr"].as_u64() {
        return match code {
            2148916233 => "This Microsoft account has no Xbox profile. Create one at xbox.com, then sign in again.".into(),
            2148916235 => "Xbox Live is not available in this account's country or region.".into(),
            2148916236 | 2148916237 => "This account needs adult verification on the Xbox website before it can be used.".into(),
            2148916238 => "This is a child account. A parent must add it to a Microsoft family first.".into(),
            _ => format!("Xbox services rejected the sign-in (XErr {code})."),
        };
    }
    let code = value["error"].as_str().unwrap_or("unknown_error");
    let detail = value["error_description"]
        .as_str()
        .or_else(|| value["message"].as_str())
        .unwrap_or("")
        .chars()
        .take(200)
        .collect::<String>();
    format!("Microsoft sign-in error {code} (HTTP {status}). {detail}")
        .trim_end()
        .to_owned()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DevicePrompt {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

/// Step 1: ask Microsoft for a browser sign-in code. The user types it on
/// microsoft.com/link and approves; Aero never handles the password.
pub async fn begin(client: &reqwest::Client) -> Result<DevicePrompt> {
    let response = client
        .post(format!("{AUTHORITY}/devicecode"))
        .form(&[("client_id", client_id()), ("scope", SCOPE.into())])
        .send()
        .await
        .context("Cannot reach Microsoft sign-in")?;
    let status = response.status();
    let value: serde_json::Value = response.json().await.unwrap_or_default();
    ensure!(
        status.is_success(),
        "{}",
        auth_error(status.as_u16(), &value)
    );
    Ok(DevicePrompt {
        device_code: value["device_code"]
            .as_str()
            .context("Missing device code")?
            .into(),
        user_code: value["user_code"]
            .as_str()
            .context("Missing user code")?
            .into(),
        verification_uri: value["verification_uri"]
            .as_str()
            .unwrap_or("https://microsoft.com/link")
            .into(),
        expires_in: value["expires_in"].as_u64().unwrap_or(900),
        interval: value["interval"].as_u64().unwrap_or(5).max(1),
        message: value["message"].as_str().unwrap_or("").into(),
    })
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MsTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

fn parse_ms(value: &serde_json::Value) -> Result<MsTokens> {
    Ok(MsTokens {
        access_token: value["access_token"]
            .as_str()
            .context("No access token")?
            .into(),
        refresh_token: value["refresh_token"].as_str().unwrap_or_default().into(),
        expires_at: now()
            + value["expires_in"]
                .as_u64()
                .unwrap_or(3600)
                .saturating_sub(60),
    })
}

/// Step 2: poll until the user approves in the browser.
pub async fn poll(
    client: &reqwest::Client,
    prompt: &DevicePrompt,
    cancel: &crate::CancellationToken,
) -> Result<MsTokens> {
    let deadline = now() + prompt.expires_in;
    while now() < deadline {
        ensure!(!cancel.is_cancelled(), "Sign-in cancelled");
        tokio::time::sleep(Duration::from_secs(prompt.interval)).await;
        let response = client
            .post(format!("{AUTHORITY}/token"))
            .form(&[
                (
                    "grant_type",
                    "urn:ietf:params:oauth:grant-type:device_code".to_owned(),
                ),
                ("client_id", client_id()),
                ("device_code", prompt.device_code.clone()),
            ])
            .send()
            .await
            .context("Cannot reach Microsoft sign-in")?;
        let status = response.status();
        let value: serde_json::Value = response.json().await.unwrap_or_default();
        if status.is_success() {
            return parse_ms(&value);
        }
        match value["error"].as_str().unwrap_or("") {
            "authorization_pending" => continue,
            "slow_down" => {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            _ => bail!("{}", auth_error(status.as_u16(), &value)),
        }
    }
    bail!("The sign-in code expired. Start again to get a new one.")
}

async fn refresh_tokens(client: &reqwest::Client, refresh: &str) -> Result<MsTokens> {
    let response = client
        .post(format!("{AUTHORITY}/token"))
        .form(&[
            ("grant_type", "refresh_token".to_owned()),
            ("client_id", client_id()),
            ("refresh_token", refresh.to_owned()),
            ("scope", SCOPE.to_owned()),
        ])
        .send()
        .await
        .context("Cannot reach Microsoft sign-in")?;
    let status = response.status();
    let value: serde_json::Value = response.json().await.unwrap_or_default();
    ensure!(
        status.is_success(),
        "{}",
        auth_error(status.as_u16(), &value)
    );
    let mut tokens = parse_ms(&value)?;
    if tokens.refresh_token.is_empty() {
        tokens.refresh_token = refresh.to_owned();
    }
    Ok(tokens)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

/// Steam-style token file: DPAPI on Windows, owner-only permissions elsewhere.
pub async fn load_accounts(root: &Path) -> Result<Accounts> {
    let file = root.join(ACCOUNTS_FILE);
    if !fs::try_exists(&file).await.unwrap_or(false) {
        return Ok(Accounts::default());
    }
    let bytes = fs::read(&file).await?;
    let plain = unseal(&bytes).context("Cannot decrypt the saved accounts on this device")?;
    Ok(serde_json::from_slice(&plain).unwrap_or_default())
}

pub async fn save_accounts(root: &Path, accounts: &Accounts) -> Result<()> {
    let plain = serde_json::to_vec(accounts)?;
    let sealed = seal(&plain)?;
    let path = root.join(ACCOUNTS_FILE);
    crate::storage::write_atomic(&path, &sealed).await
}

#[cfg(windows)]
fn seal(plain: &[u8]) -> Result<Vec<u8>> {
    dpapi(plain, true)
}

#[cfg(not(windows))]
fn seal(plain: &[u8]) -> Result<Vec<u8>> {
    Ok(plain.to_vec())
}

#[cfg(not(windows))]
fn unseal(sealed: &[u8]) -> Result<Vec<u8>> {
    Ok(sealed.to_vec())
}

#[cfg(windows)]
fn unseal(sealed: &[u8]) -> Result<Vec<u8>> {
    dpapi(sealed, false)
}

#[cfg(windows)]
fn dpapi(input: &[u8], protect: bool) -> Result<Vec<u8>> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
    };
    ensure!(input.len() <= 8 * 1024 * 1024, "Token file is too large");
    let mut data = input.to_vec();
    let mut output = CRYPT_INTEGER_BLOB::default();
    let input_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_mut_ptr(),
    };
    // CRYPTPROTECT_UI_FORBIDDEN keeps DPAPI silent: no prompts, ever.
    let _ = CRYPTPROTECT_UI_FORBIDDEN;
    let ok = unsafe {
        if protect {
            CryptProtectData(
                &input_blob,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                1,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input_blob,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                1,
                &mut output,
            )
        }
    };
    ensure!(ok != 0, "Windows DPAPI rejected the request");
    let bytes = unsafe {
        let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
        let copy = slice.to_vec();
        LocalFree(output.pbData as *mut core::ffi::c_void);
        copy
    };
    Ok(bytes)
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
    let xbl = post_json(
        client,
        "https://user.auth.xboxlive.com/user/authenticate",
        &serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", ms.access_token),
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        }),
    )
    .await
    .context("Xbox Live sign-in failed")?;
    let xbl_token = xbl["Token"]
        .as_str()
        .context("Xbox Live returned no token")?
        .to_owned();
    let uhs = xbl["DisplayClaims"]["xui"]
        .as_array()
        .and_then(|list| list.first())
        .and_then(|entry| entry["uhs"].as_str())
        .context("Xbox Live returned no user hash")?
        .to_owned();

    let xsts = post_json(
        client,
        "https://xsts.auth.xboxlive.com/xsts/authorize",
        &serde_json::json!({
            "Properties": {"SandboxId": "RETAIL", "UserTokens": [xbl_token]},
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
        }),
    )
    .await
    .context("Xbox security token request failed")?;
    let xsts_token = xsts["Token"]
        .as_str()
        .context("Xbox returned no security token")?
        .to_owned();

    let game = post_json(
        client,
        "https://api.minecraftservices.com/authentication/login_with_xbox",
        &serde_json::json!({"identityToken": format!("XBL3.0 x={uhs};{xsts_token}")}),
    )
    .await
    .context("Minecraft services rejected the sign-in")?;
    let access_token = game["access_token"]
        .as_str()
        .context("Minecraft returned no token")?
        .to_owned();
    let access_expires_at = now()
        + game["expires_in"]
            .as_u64()
            .unwrap_or(86_400)
            .saturating_sub(120);

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

/// Complete a device-code sign-in and return the ready account.
pub async fn finish(client: &reqwest::Client, tokens: &MsTokens) -> Result<(Account, Profile)> {
    minecraft_chain(client, tokens).await
}

/// Renew a Minecraft session token from the stored Microsoft refresh token.
pub async fn renew(client: &reqwest::Client, account: &Account) -> Result<Account> {
    ensure!(
        !account.ms.refresh_token.is_empty(),
        "This account needs to sign in again"
    );
    let ms = refresh_tokens(client, &account.ms.refresh_token).await?;
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
            access_token: "ms".into(),
            refresh_token: "refresh".into(),
            expires_at: now() + 3600,
        },
        access_token: "mc".into(),
        access_expires_at: now() + 3600,
        added_at: 0,
    }
}
