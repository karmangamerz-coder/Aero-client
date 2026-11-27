# ADR 0012: Browser PKCE, Minecraft library and OS credential vault

Status: accepted, 2026-11-27. Supersedes ADR 0007's device-code and DPAPI-file decisions.

## Context

Aero 2.0 requires browser authorization-code PKCE, original application identity, secure
refresh storage and useful family/approval errors within the existing Rust/Tokio stack.

## Decision

- Use oauth2 5.0.0 for S256 verifier/challenge generation, random CSRF state, code exchange
  and refresh. No client secret, password form, embedded browser or borrowed application ID.
- Use minecraft-msa-auth 0.5.0 for Xbox Live, XSTS and Minecraft Services token exchange.
  An async adapter reuses Aero's Reqwest 0.12 client with redirects disabled and rustls,
  avoiding another HTTP/TLS stack. Aero retains explicit ownership/profile checks and
  converts library errors to safe fixed messages. All selected libraries are MIT/Apache-2.0.
- Bind an ephemeral IPv4 loopback listener before opening the default browser via
  webbrowser. The registered desktop redirect is http://localhost/callback; runtime ports
  are ignored by Microsoft's localhost matching. Accept only a bounded GET on /callback
  with one matching state and one code. Reject duplicates, time out after five minutes,
  and close the single-use listener on cancellation/completion. Invalid probes do not
  consume a valid transaction. Valid consent denial ends it immediately.
- Codes, verifiers, state and token-bearing errors never enter UI snapshots or logs.
  Auth jobs return completion through the backend command queue. Generation guards reject
  late results after cancellation/logout. Account writes and launch refresh share a mutex.
- Use keyring 3.6.3 with explicit native backends: Windows Credential Manager, macOS
  Keychain, Linux Secret Service. Blocking vault calls run on spawn_blocking. No automatic
  plaintext or mock fallback. Only refresh tokens persist in the vault; account JSON is
  metadata. Application ID binding prevents refreshing under another registration.
- Logout deletes the active local credential; removal deletes the chosen credential.
  Failed deletions are tracked in token-free metadata and retried before restoring accounts.
  Microsoft browser cookies and other-device sessions remain outside Aero's local logout.
- Replace older DPAPI files through a fresh login, not silent token import. Preserve game
  instances and saves. A successful vault save removes the obsolete credential file.
- Version 2.0.0 reflects the incompatible account-storage and login transition. Release
  artifacts may be published with explicit unverified-live-auth notes if no project ID
  is available; no ownership/approval bypass is introduced to complete a release.

## Consequences and alternatives

The OS vault must be available and unlocked. Linux builds need Secret Service and its
DBus development library. Network calls and vault failures produce actionable UI errors.
Live account, family-policy and vault testing remain separate from synthetic unit tests.
Device codes do not meet the requested flow. A webview creates unnecessary password and
cookie custody. Reusing another launcher's registration cannot represent Aero reliably.
Keeping DPAPI blobs would fail the requested OS-vault storage model.
