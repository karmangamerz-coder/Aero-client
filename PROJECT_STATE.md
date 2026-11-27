# Project state

## Current release

Version: 2.0.0
Primary platform: Windows 10/11 x64
Original-source license: GPL-3.0-only

The native Rust/Freya launcher is primary; the Java client and JavaFX fallback remain
independently buildable. M4–M9 contain the implemented foundations listed in ROADMAP.md;
a source implementation does not imply all live game adapters or benchmarks are finished.

## 2.0.0 refinement

The owner requested keeping this update at version 2.0.0. Source/package metadata is
aligned accordingly; the existing published v2.0.0 tag and artifacts have not been replaced.

- Silver glass chrome, labeled sidebar, original vector background and blue inset Play panel.
  Existing Dark/Lite preferences remain respected.
- Removed setup skipping/defer commands. Valid completed records stay completed even if
  an upgrade retires their saved version/schema; interrupted setup still resumes.
- Modrinth browser defaults to unfiltered version/loader; project detail ignores Vanilla
  as a loader filter. Mod installation refuses Vanilla instances before fetching files.
  Packs/shaders omit mod-loader filters throughout search/detail/install. Discover loads on
  navigation and surfaces empty results/errors and the selected installation destination.
- OAuth callback no longer implies full success. Generation-guarded verification progress
  and persistent in-app failures explain why an account has not been saved.
  The notice also appears outside the setup scroll area on every step; its height is reserved
  before laying out the body. Completion is published before optional skin downloads and
  failures before diagnostic disk writes. A subsequent attempt in the isolated preview's
  data directory again logged registration rejection, not successful sign-in.
- Local user log reported Minecraft registration rejection after OAuth. This is an external
  authorization gate, not evidence of successful login. Never bypass it or borrow an ID.
- Local formatting can run; local tests cannot link without MSVC. New regression tests and
  platform packaging must be verified by the feature PR's GitHub Actions before release.

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
