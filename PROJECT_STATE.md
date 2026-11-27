# Project state

## Current release

Version: 2.0.0
Primary platform: Windows 10/11 x64
Original-source license: GPL-3.0-only

The native Rust/Freya launcher is primary; the Java client and JavaFX fallback remain
independently buildable. M4–M9 contain the implemented foundations listed in ROADMAP.md;
a source implementation does not imply all live game adapters or benchmarks are finished.

## Authentication architecture (2026-11-27)

- native/core/src/oauth_pkce.rs: oauth2 5.0.0, authorization-code flow with S256 PKCE,
  random state, bounded single-use IPv4 loopback listener, five-minute timeout/cancellation,
  safe token/refresh exchanges and explicit application identity.
- native/core/src/auth.rs: minecraft-msa-auth 0.5.0 through the existing async Reqwest/rustls
  client, Xbox/XSTS/Minecraft exchange, Java ownership/profile validation and skin retrieval.
- native/core/src/vault.rs: keyring 3.6.3, Windows Credential Manager/macOS Keychain/Linux
  Secret Service refresh storage; token-free account metadata; scoped vault entries and
  retryable deletions. No plaintext fallback. Older DPAPI accounts require fresh sign-in.
- native/launcher/src/backend.rs: generation-guarded auth completion, cancellation, visible
  errors, multiple accounts, switching, local logout/removal and launch-time renewal.
  Account writes/refresh serialize through a mutex; vault calls use blocking workers.
- native/launcher/src/pages.rs and shell.rs: Frutiger Aero sign-in/waiting state, skin head,
  username/status and account selector. Body skins continue to compose once per change.
- native/core/src/launch.rs: verified launch session and arguments, memory/runtime selection,
  safe diagnostic formatting and redaction of known tokens from captured game output.
- README.md, PRIVACY.md, SECURITY.md and ADR 0012 describe registration, approval,
  storage/logout semantics, upgrade behavior and security boundaries.
- Cargo/Gradle, Windows installer and user agent align at 2.0.0. Auth library license texts
  ship under third-party/auth. Tagged builds publish installer/portable/SHA-256 artifacts.

## Verification

- Local Rust formatting and locked dependency metadata checks pass. This host lacks the
  MSVC linker; GitHub Windows CI is authoritative for compilation, tests and packaging.
- PR #14's initial Linux/macOS native checks passed. Windows core tests passed, including a
  synthetic real Credential Manager save/reload/delete test. Final release checks must pass
  after the last source/documentation commit and before publishing the version tag.
- Tests cover CSRF mismatch, duplicate callback fields, callback cancellation/denial,
  listener closure/replay rejection, refresh rotation, application binding, metadata secrecy,
  Java entitlements, family/approval errors, head rendering and launch-token redaction.
- The owner supplied a Microsoft Application ID on 2026-11-27. It is configured in GitHub's
  AERO_MS_CLIENT_ID repository variable for release builds. Microsoft recognized the ID in
  an initial request and routed the PKCE request to sign-in. A device-flow diagnostic said
  the registration was not enabled for that feature; Aero uses browser PKCE instead.
  Minecraft API approval, desktop redirect configuration, successful live Microsoft login,
  real family-policy responses and actual game launches still require owning-account testing.
- Earlier visual testing exposed and corrected flex sizing, setup footer and SVG coverage.
  Complete 1280x720/1920x1080 and multi-DPI screenshot verification is still open; do not
  describe clipped desktop captures as full-resolution evidence.

## Remaining release/platform gates

- Validate the supplied Application ID's desktop callback and Minecraft authorization with
  an owning account. End users should not need separate Azure registrations.
- Run native macOS/Linux vault tests in unlocked user sessions; current CI checks compilation.
- Validate 1.8.9/modern Minecraft sessions, version-specific game adapters, media integration,
  modpack import and comparative performance benchmarks as tracked in ROADMAP.md.
- Stable Windows signing needs a publisher certificate in external CI secrets. Current
  unsigned builds can be blocked by Windows application-control policy.
