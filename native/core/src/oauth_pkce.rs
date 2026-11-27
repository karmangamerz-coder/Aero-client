// SPDX-License-Identifier: GPL-3.0-only
//! Single-use loopback OAuth transaction. Codes, verifier and state never enter UI state.
use anyhow::{Context, Result, bail, ensure};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

type Client = BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

pub struct BrowserLogin {
    listener: TcpListener,
    client: Client,
    verifier: PkceCodeVerifier,
    state: CsrfToken,
    application_id: String,
    pub authorization_url: String,
}

fn client(id: &str, redirect: Option<String>) -> Result<Client> {
    let client = BasicClient::new(ClientId::new(crate::auth::client_id(id)?))
        .set_auth_uri(AuthUrl::new(
            "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize".into(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://login.microsoftonline.com/consumers/oauth2/v2.0/token".into(),
        )?);
    Ok(if let Some(redirect) = redirect {
        client.set_redirect_uri(RedirectUrl::new(redirect)?)
    } else {
        client
    })
}

pub async fn begin(id: &str) -> Result<BrowserLogin> {
    let application_id = crate::auth::client_id(id)?;
    // Bind before opening the browser; Windows chooses an unused loopback port.
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .context("Cannot open the local Microsoft sign-in callback")?;
    let redirect = format!(
        "http://localhost:{}/callback",
        listener.local_addr()?.port()
    );
    let client = client(id, Some(redirect))?;
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let (url, state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("XboxLive.signin".into()))
        .add_scope(Scope::new("offline_access".into()))
        .add_extra_param("prompt", "select_account")
        .set_pkce_challenge(challenge)
        .url();
    Ok(BrowserLogin {
        listener,
        client,
        verifier,
        state,
        application_id,
        authorization_url: url.to_string(),
    })
}

/// Parse only a bounded GET on the exact callback path, reject duplicated security fields.
fn callback(target: &str, expected_state: &str) -> Result<String> {
    ensure!(
        target.starts_with("/callback?"),
        "Unexpected sign-in callback path"
    );
    let url = reqwest::Url::parse(&format!("http://127.0.0.1{target}"))
        .map_err(|_| anyhow::anyhow!("Invalid sign-in callback"))?;
    let pairs: Vec<_> = url.query_pairs().collect();
    let unique = |name: &str| -> Result<Option<String>> {
        let values: Vec<_> = pairs.iter().filter(|(key, _)| key == name).collect();
        ensure!(values.len() <= 1, "Duplicate sign-in callback field");
        Ok(values.first().map(|(_, value)| value.to_string()))
    };
    let state = unique("state")?.context("Missing sign-in state")?;
    // Compare fixed-size SHA-256 digests without early exit.
    use sha2::{Digest, Sha256};
    let actual = Sha256::digest(state.as_bytes());
    let expected = Sha256::digest(expected_state.as_bytes());
    use subtle::ConstantTimeEq;
    ensure!(
        bool::from(actual.as_slice().ct_eq(expected.as_slice())),
        "Sign-in state did not match. Start sign-in again."
    );
    if let Some(error) = unique("error")? {
        return Err(Denied(crate::auth::oauth_error(&error)).into());
    }
    let code = unique("code")?.context("Microsoft returned no authorization code")?;
    ensure!(
        !code.is_empty(),
        "Microsoft returned an empty authorization code"
    );
    Ok(code)
}

#[derive(Debug)]
struct Denied(String);
impl std::fmt::Display for Denied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Denied {}

impl BrowserLogin {
    pub async fn complete(
        self,
        http: &reqwest::Client,
        cancel: &crate::CancellationToken,
    ) -> Result<crate::auth::MsTokens> {
        let operation = async {
            let code = loop {
                let (mut stream, peer) = self
                    .listener
                    .accept()
                    .await
                    .map_err(|_| anyhow::anyhow!("Sign-in callback connection failed"))?;
                if !peer.ip().is_loopback() {
                    continue;
                }
                let read = tokio::time::timeout(Duration::from_secs(3), async {
                    let mut bytes = Vec::new();
                    loop {
                        let mut part = [0u8; 1024];
                        let count = stream.read(&mut part).await?;
                        if count == 0 {
                            break;
                        }
                        bytes.extend_from_slice(&part[..count]);
                        if bytes.len() > 8192 || bytes.windows(4).any(|p| p == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Ok::<_, std::io::Error>(bytes)
                })
                .await;
                let Ok(Ok(bytes)) = read else { continue };
                if bytes.len() > 8192 {
                    continue;
                }
                let Ok(header) = std::str::from_utf8(&bytes) else {
                    continue;
                };
                let mut line = header.lines().next().unwrap_or("").split_whitespace();
                if line.next() != Some("GET") {
                    continue;
                }
                let target = line.next().unwrap_or("");
                if !target.starts_with("/callback?") {
                    continue;
                }
                let result = callback(target, self.state.secret());
                let body = if result.is_ok() {
                    "Microsoft authorization received. Sign-in is not finished yet. Return to Aero Client: it must still verify Xbox access, Minecraft Java ownership and secure account storage. Aero will show the final result there. You may close this tab."
                } else {
                    "Sign-in could not be verified. Return to Aero Client and try again."
                };
                let reply = format!(
                    "HTTP/1.1 {}\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nContent-Security-Policy: default-src 'none'\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
                    if result.is_ok() {
                        "200 OK"
                    } else {
                        "400 Bad Request"
                    },
                    body.len()
                );
                let _ = tokio::time::timeout(
                    Duration::from_secs(2),
                    stream.write_all(reply.as_bytes()),
                )
                .await;
                // Invalid probes must not consume the transaction or abort a valid browser login.
                match result {
                    Ok(code) => break code,
                    Err(error) if error.downcast_ref::<Denied>().is_some() => return Err(error),
                    Err(_) => continue,
                }
            };
            let response = self
                .client
                .exchange_code(AuthorizationCode::new(code))
                .set_pkce_verifier(self.verifier)
                .request_async(http)
                .await
                .map_err(oauth_failure)?;
            tokens(&response, None, &self.application_id)
        };
        tokio::select! {
            _ = cancel.cancelled() => bail!("Microsoft sign-in cancelled."),
            result = tokio::time::timeout(Duration::from_secs(300), operation) => {
                result.map_err(|_| anyhow::anyhow!("Microsoft sign-in timed out. Please try again."))?
            }
        }
    }
}

fn oauth_failure<E: std::error::Error + 'static>(
    error: oauth2::RequestTokenError<E, oauth2::basic::BasicErrorResponse>,
) -> anyhow::Error {
    // Never format library errors: transports can include callback URLs or response bodies.
    let message = match error {
        oauth2::RequestTokenError::ServerResponse(response) => {
            crate::auth::oauth_error(response.error().as_ref())
        }
        _ => "Cannot complete Microsoft sign-in. Check your connection and try again.".into(),
    };
    anyhow::anyhow!(message)
}

fn tokens(
    response: &oauth2::basic::BasicTokenResponse,
    previous: Option<&str>,
    application_id: &str,
) -> Result<crate::auth::MsTokens> {
    let refresh_token = response
        .refresh_token()
        .map(|value| value.secret().as_str())
        .or(previous)
        .context("Microsoft returned no refresh token. Sign in again.")?
        .to_owned();
    Ok(crate::auth::MsTokens {
        application_id: application_id.to_owned(),
        access_token: response.access_token().secret().clone(),
        refresh_token,
        expires_at: crate::auth::now()
            + response
                .expires_in()
                .unwrap_or(Duration::from_secs(3600))
                .as_secs()
                .saturating_sub(60),
    })
}

pub async fn refresh(
    http: &reqwest::Client,
    id: &str,
    refresh: &str,
) -> Result<crate::auth::MsTokens> {
    ensure!(
        !refresh.is_empty(),
        "Session expired. Sign in with Microsoft again."
    );
    let response = client(id, None)?
        .exchange_refresh_token(&RefreshToken::new(refresh.into()))
        .add_scope(Scope::new("XboxLive.signin".into()))
        .add_scope(Scope::new("offline_access".into()))
        .request_async(http)
        .await
        .map_err(oauth_failure)?;
    tokens(&response, Some(refresh), &crate::auth::client_id(id)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_rejects_state_mismatch_duplicates_and_wrong_paths() {
        assert!(callback("/callback?code=secret&state=bad", "expected").is_err());
        assert!(callback("/callback?code=secret&state=x&state=x", "x").is_err());
        assert!(callback("/other?code=secret&state=x", "x").is_err());
        assert!(callback("/callback?state=x&code=a&code=b", "x").is_err());
        assert!(callback("/callback?state=x&code=", "x").is_err());
        assert_eq!(callback("/callback?state=x&code=abc", "x").unwrap(), "abc");
    }
    #[tokio::test]
    async fn authorize_uses_random_state_s256_and_loopback() {
        let a = begin("11111111-2222-3333-4444-555555555555").await.unwrap();
        let b = begin("11111111-2222-3333-4444-555555555555").await.unwrap();
        let url = reqwest::Url::parse(&a.authorization_url).unwrap();
        let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["code_challenge_method"], "S256");
        assert!(query["redirect_uri"].starts_with("http://localhost:"));
        assert_ne!(a.state.secret(), b.state.secret());
        assert!(!a.authorization_url.contains(a.verifier.secret()));
        let cancel = crate::CancellationToken::new();
        cancel.cancel();
        assert!(
            a.complete(&crate::auth::client().unwrap(), &cancel)
                .await
                .is_err()
        );
    }

    #[test]
    fn denied_signin_does_not_echo_provider_details() {
        let error = callback(
            "/callback?state=x&error=access_denied&error_description=secret",
            "x",
        )
        .unwrap_err();
        assert!(error.downcast_ref::<Denied>().is_some());
        assert!(!error.to_string().contains("secret"));
    }

    #[test]
    fn refresh_rotation_retains_previous_token_only_when_omitted() {
        let response: oauth2::basic::BasicTokenResponse =
            serde_json::from_value(serde_json::json!({
                "access_token": "synthetic-access", "token_type": "Bearer", "expires_in": 3600
            }))
            .unwrap();
        let tokens = tokens(&response, Some("previous-refresh"), "app").unwrap();
        assert_eq!(tokens.refresh_token, "previous-refresh");
        assert_eq!(tokens.application_id, "app");
        let rotated: oauth2::basic::BasicTokenResponse =
            serde_json::from_value(serde_json::json!({
                "access_token": "synthetic-access", "refresh_token": "new-refresh",
                "token_type": "Bearer", "expires_in": 3600
            }))
            .unwrap();
        assert_eq!(
            super::tokens(&rotated, Some("old"), "app")
                .unwrap()
                .refresh_token,
            "new-refresh"
        );
        assert!(super::tokens(&response, None, "app").is_err());
    }

    #[tokio::test]
    async fn valid_denial_closes_listener_and_cannot_replay() {
        let login = begin("11111111-2222-3333-4444-555555555555").await.unwrap();
        let address = login.listener.local_addr().unwrap();
        let state = login.state.secret().clone();
        let cancel = crate::CancellationToken::new();
        let task = tokio::spawn(async move {
            login
                .complete(&crate::auth::client().unwrap(), &cancel)
                .await
        });
        let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
        let request = format!(
            "GET /callback?state={state}&error=access_denied HTTP/1.1\r\nHost: localhost\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut reply = String::new();
        stream.read_to_string(&mut reply).await.unwrap();
        assert!(reply.starts_with("HTTP/1.1 400"));
        assert!(!reply.contains(&state));
        assert!(
            tokio::time::timeout(Duration::from_secs(5), task)
                .await
                .unwrap()
                .unwrap()
                .is_err()
        );
        assert!(tokio::net::TcpStream::connect(address).await.is_err());
    }
}
