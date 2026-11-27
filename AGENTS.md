# Permanent Aero Client requirements

Read this file, PROJECT_STATE.md and ROADMAP.md before each task. Preserve existing
working features and local changes; inspect relevant files only. Do not recreate or
unnecessarily restructure the repository. Prefer medium reasoning and targeted work;
never sacrifice correctness or security to reduce cost.

## Architecture, license and security
- All original Aero source is GPL-3.0-only. Do not add a dependency whose license is
  incompatible with GPLv3 distribution; preserve every third-party notice.
- M2-R supersedes the launcher stack: Rust + Freya + Skia in a Cargo workspace, Tokio,
  Reqwest and Serde. Keep the in-game client Java. Retain JavaFX fallback until Rust
  parity is validated; no Tauri, Electron, Chromium or JavaFX in the replacement.
- Launcher layout: `native/core` owns services and safety; `native/launcher/src` splits
  into `design.rs` (tokens + custom controls), `shell.rs` (rail/top bar/status strip),
  `pages.rs` (pure page composition, incl. the setup wizard) and `ui.rs` (single owner of
  reactive state). Keep page functions hook-free and event-driven; never add a render
  timer, polling loop or periodic metadata request. Per-instance advanced settings belong
  in the instance manifest (`memory_mib`, `java_path`), not in global preferences.
- Authentication lives in `native/core/src/auth.rs`: Microsoft OAuth 2.0 authorization code with PKCE in
  the user's browser, Xbox Live, XSTS, Minecraft services, entitlement verification,
  profile and skin. Never collect a Microsoft password, never bypass ownership and never
  log, print or persist tokens in plaintext; refresh tokens belong in the OS credential vault — no token in UI text,
  logs, crash reports, telemetry or error bodies.
- Launching lives in `native/core/src/launch.rs`: arguments are built only from verified
  metadata written during installation, the token is passed as an argument (never on a
  command line that gets logged), Java comes from the managed runtime or an explicit
  user choice, and the process is supervised with bounded log buffers, cancellation and
  crash-report discovery. Never run arbitrary commands from remote metadata.
- The first-launch setup wizard (`native/core/src/onboarding.rs`, `pages::wizard`) is a
  permanent feature: it must resume after a restart, persist completion, never show
  again once completed, and be re-runnable from Settings. No Skip/defer action; browsing without sign-in is allowed,
  but every setup step must be visited and game launch still requires ownership.

## Permanent visual system (2.0.0 refinement)
- Follow the user's silver Vista reference with original assets: neutral silver glass,
  an Explorer-style labeled sidebar, blue inset Play panel, subtle static folds, Segoe UI,
  thin gray rims and restrained gloss. Dark and Lite presets remain available.
- Keep Aero branding, account selector, skin preview, instance/version/loader selection
  and the single Launch action. Do not copy Windows logos or add fake window buttons.
- No news section, no promotional headings, no dashboard cards, no badges or decorations
  that do not carry information. No excessive glow, lens flares, bubbles, oversaturation,
  live blur or copied branding. Static original vector artwork only; no per-frame animation.
- Appearance presets: Aero Light, Aero Dark, Aero Lite (flat, no photo, cheapest).
- The same system applies to every page, the wizard, and future mod menus, HUD editors
  and music overlays.

## Performance and distribution
- Reference: Ryzen 5 5500U, integrated Radeon, 16 GB DDR4, Windows 11. Never invent benchmarks.
- Never block the UI thread with disk/network work, extraction, hashing, subprocesses or
  authentication polling; the game process and sign-in polling are supervised tasks.
- Idle work stops: no continuous rendering, no render timers, no background polling.
  Character artwork is composed once per skin change; hardware detection runs once.
- End users must not install Java, Rust, Gradle, IntelliJ or developer tools.
- Use jlink for a minimal launcher runtime and jpackage for the M9 Windows installer.
  Bundle required JavaFX; manage game runtimes independently and automatically in M2.
- M9 artifacts: AeroClient-Setup.exe, AeroClient-Portable.zip where practical, SHA256SUMS.txt.
  Installer: optional desktop shortcut, Start Menu entry, uninstaller, icon, version and
  required license notices. Do not prematurely implement the full production release system.

## GitHub workflow for every Aero task
- Check branch, status and configured remote first; preserve user changes. Safely synchronize
  the appropriate branch of `https://github.com/karmangamerz-coder/Aero-client.git`.
- After each meaningful completed stage: run relevant tests, review, descriptive commit,
  push source/tests/docs/build configuration to this remote when permitted.
- Never force-push, rewrite remote history or overwrite unrelated changes. Honor protection;
  use a feature branch and PR if direct main writes are unavailable. Verify remote SHA/content
  before claiming success. Report required user approvals/merges accurately.
- Update PROJECT_STATE with real features, paths, tests, limitations and next steps; ROADMAP
  with product requirements and milestone status. Use SemVer and maintain CHANGELOG.
- Targeted tests during development; full build at milestone boundary. Minimize redundant
  scans/downloads and keep reporting concise. Ask only when genuinely blocked.
