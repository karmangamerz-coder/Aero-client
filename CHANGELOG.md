# Changelog

This project uses Semantic Versioning. Dates use ISO 8601.

## 2.1.0 — 2026-11-27

### Changed
- Silver Vista-inspired chrome, labeled navigation, original vector backdrop and blue Play panel.
- First-launch setup has no Skip action; completed setup survives retired answer values.
- Browser callback explicitly distinguishes Microsoft authorization from verified Minecraft
  sign-in. A persistent notice shows pending verification and failures throughout the launcher.

### Fixed
- Modrinth project details no longer filter releases by the nonexistent "vanilla" loader.
- Discovery starts with all versions/loaders instead of inheriting setup filters.
- Mods cannot install into Vanilla instances; compatible Fabric/Forge instances are required.

### Known limitation
- The observed live sign-in reports Minecraft registration rejection. Aero's own client ID
  still needs valid Minecraft Services authorization; no borrowed ID or bypass is provided.

## 2.0.0 — 2026-11-27

### Added
- Microsoft authorization-code sign-in with S256 PKCE in the default browser, a single-use
  loopback callback, state validation, timeout and cancellation.
- OS credential-vault refresh storage, token-free account metadata, application ID binding,
  automatic expired-session renewal, multiple accounts, switching and local logout/removal.
- Minecraft authentication library integration, ownership/profile/skin checks, family and
  application-approval errors, skin-head display and visible waiting state.
- Tests for callback security, refresh rotation, metadata secrecy and safe error handling.
- Microsoft registration/configuration guide and ADR 0012.

### Changed
- Version aligned to 2.0.0 across Cargo, Gradle, user agent and Windows installer.
- Older DPAPI account files require sign-in again; instances and saves remain intact.
- Includes the responsive UI, Modrinth filters and setup fixes from the refinement branch.

### Security
- No password collection, embedded login, client secret or borrowed app ID.
- Auth library errors never echo token-bearing response bodies; launch diagnostics redact
  known secrets and account Debug output omits sessions.
- Live owning-account sign-in remains unverified until Aero's approved registration is supplied.

## 1.0.0-beta.1 — 2026-11-24

### Added
- Responsive Frutiger Aero launcher surfaces for small and full-HD windows, an always-visible
  status footer, immediate theme application and a static low-performance Lite mode.
- Configurable Microsoft public-client identity in Settings and `AERO_MS_CLIENT_ID`, with
  clear device-code progress and automatic access-token renewal before launch.
- Modrinth type, version, loader, category and sort filters; project details and compatible
  versions; installed-content enable, disable and removal controls.
- Explicit Modrinth installation for compatible mods, resource packs and shaders with
  bounded required-dependency resolution, trusted endpoints, checksum verification,
  per-instance manifests and reversible content management.
- Original Java client foundation: module registry, immutable profiles, atomic profile
  storage, normalized HUD layout and event-driven operating-system media snapshots.
- Transparent Fabric performance preset manifests and user-requested GitHub release
  discovery with semantic-version validation.
- NSIS installer source, Windows setup/portable/checksum automation, tagged GitHub release
  publishing, Linux/macOS native checks and locked Rust dependency auditing.
- ADRs 0009–0011 documenting content trust, client boundaries, GPL licensing and releases.

### Changed
- All original Aero source is now GPL-3.0-only. Third-party components retain their own
  licenses and notices.
- The project version is aligned across Cargo and Gradle at 1.0.0-beta.1.

### Security
- Removed the borrowed legacy Minecraft Launcher client ID. Aero sign-in now requires an
  owner-registered Azure public-client application and never needs a client secret.
- Modrinth downloads verify SHA-512 when the publisher supplies it, in addition to SHA-1.
- Content installation refuses incompatible releases and untrusted download hosts, verifies
  the publisher-provided SHA-1 and never silently installs or updates projects.
- Release credentials, signing identities and Microsoft tokens remain outside the repository.

### Fixed
- Play, Settings and first-run setup no longer overlap or clip at the supported minimum
  window size; background artwork no longer captures pointer interaction.
- Setup reserves a fixed responsive viewport for step content, keeping Next and Skip setup
  visible at the supported 640px client height while longer steps scroll internally; its
  navigation uses bounded controls instead of a spacer that could push actions off-screen.
- The landscape now covers the full root instead of exposing side bands, and Discover keeps
  its Search control inside the content area at 1280px and the supported minimum width.
- Sign-in reports missing configuration and browser-opening failures immediately instead of
  appearing unresponsive, and launcher/panic diagnostics omit token-bearing payloads.

## 0.4.0 — 2026-11-21

### Added
- Microsoft sign-in with the official OAuth 2.0 device-code flow in the system browser
  (Xbox Live, XSTS and Minecraft services), ownership verification, profile and skin
  retrieval, automatic token refresh, multiple accounts with switching and removal.
- Encrypted account storage: Windows DPAPI (`accounts.bin`); tokens are never logged or
  shown in the interface.
- Authenticated launching: arguments from verified metadata, managed or explicit Java,
  memory allocation, supervised game process, bounded live logs, crash-report discovery
  and launch cancellation.
- First-launch setup wizard with seven steps, live theme previews, hardware detection,
  resumable progress, persisted completion and a Settings entry to run it again.
- CPU/RAM/storage/OS detection with conservative memory and performance recommendations.
- Minecraft skin rendering: real skins when signed in, an original Aero fallback skin
  otherwise, composed on the CPU once per skin change.

### Changed
- Home screen redesigned to match the M2.1 reference: no news, no promotional copy, Aero
  branding in the top bar, compact account selector at the top-right, narrow icon-only
  rail, large character with instance/version/loader selectors beneath it and one glossy
  Launch control with an adjacent instance settings button.
- Installation progress is displayed inside the launch control instead of only in a strip.
- Themes consolidated into Aero Light, Aero Dark and Aero Lite and applied to every page.
- Downloads page now shows live Minecraft output and can stop the game process.

### Removed
- The unfinished news feature and its cached background request, including the
  `show_news` setting and the `NewsItem` content API.

### Fixed
- The launcher no longer holds a watch-channel borrow across an await point, so the
  backend future is `Send` and sign-in/launch tasks compile on Tokio.
- Setup navigation now has one durable source of truth. Expected-step guards make repeated
  clicks idempotent, validation covers every answer, errors are visible, and Finish writes
  completion only after settings and instance creation succeed.
- Setup later now exits safely without silently changing an instance; incompatible loader
  selections reset when the Minecraft version changes.
- Native Windows packaging removes stale fixed-name output before archiving and forces the
  archive write, avoiding failures caused by a restored Actions cache.
- Legacy jpackage arguments are passed as a flat command, and macOS uses a compatible
  bundle version while project artifacts retain Semantic Versioning.

## 0.3.0 — 2026-11-20

### Added
- Native Freya/Skia launcher with an Aero Light, Aero Dark and Aero Lite design system:
  glass surfaces, glossy green launch control, 64 px icon rail, compact top bar and a
  status strip with live installation progress.
- Home with original pointer-rotated explorer preview and a collapsible 24 % news panel;
  dedicated instance management screen holding the advanced installation settings and the
  verified installation plan.
- Per-instance memory and Java overrides stored in the instance manifest, with validation
  and backward-compatible loading of older manifests.
- Session activity log and read-only installation plan view (`clientSha1`, asset index,
  library count, chosen runtime).

### Changed
- Rust workspace is the primary launcher implementation; the JavaFX launcher is retained
  as a fallback until Windows parity is confirmed.
- `reqwest` now uses rustls instead of the platform OpenSSL (ADR 0006).
- News, discovery and downloads pages never fabricate content; empty states explain when
  nothing was fetched.

### Fixed
- Launcher UI no longer borrows page state across the app root, removing a hook-order risk
  in the prototype.
- Core tests cover manifest migration and per-instance override validation.

## 0.2.0 — 2026-11-20

### Added
- JavaFX Aero interface with five navigation pages and asynchronous Downloads controls.
- Official metadata, Windows game files, Fabric and legacy Forge installation/repair.
- Automatic compatible Java management and UUID-isolated instance foundations.
- Download integrity, resume/retry/cancel and path/archive safety tests.
- Persistent project state and M1–M9 roadmap with standalone distribution requirements.

### Changed
- Original current source uses MIT per the owner's M2 instruction.
- Authentication and game launching explicitly remain M3; production installer remains M9.
- The M1 preference format and independent client core are retained.

## 0.1.0 — 2026-11-20

### Added
- GPL-3.0-only project policies and architectural decision records.
- Independent Java 21 launcher and client core Gradle modules.
- Offline desktop shell and validated local preference storage.
- Unit tests, dependency locks and artifact verification metadata.
- Cross-platform CI, native app images, release bundles and SHA-256 checksums.

### Limitations
- No Microsoft authentication, game downloading, game launch or Minecraft mod integration.
- Native images are unsigned and their reproducibility is not guaranteed.
