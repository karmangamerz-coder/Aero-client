# Changelog

This project uses Semantic Versioning. Dates use ISO 8601.

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
