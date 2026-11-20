# Project state

## Current milestone
M2 (installation, runtimes, instances) is complete in code and verified by tests plus a
live official-metadata check. M2-R (native Rust migration) is **code complete and
type-verified**; Windows native launch, visual QA and measured idle usage are still
pending, so the JavaFX launcher is retained as a fallback and is not removed yet.

Version: 0.3.0 development. Target: Windows 10/11 x64. Original-source license: MIT by the
owner's M2 instruction; third-party components keep their own licenses.

## What exists
- `native/core` — `aero-core`: official metadata, verified downloads, libraries/natives,
  Fabric and legacy Forge profiles, managed Java runtimes, UUID instances, settings.
- `native/launcher` — `aero-launcher`: Freya 0.5.3 + Skia desktop UI.
  - `design.rs`: three Aero palettes (Light/Dark/Lite), glass surfaces, custom buttons with
    hover/press/focus/disabled states, chips, progress, rail icons, original SVG artwork.
  - `shell.rs`: 64 px icon-only rail with tooltips, 56 px top bar with account/instance/
    version state, 26 px status strip with live progress.
  - `pages.rs`: Play (character preview + contextual launch/install + collapsible 24 % news
    panel), Library (create/search/list), instance detail (installation, runtime overrides,
    verified plan, rename/duplicate/trash), Discover (Modrinth search), Downloads (current
    activity + session log), Settings (theme, news, memory, Java, accessibility).
  - `backend.rs`: single-threaded command queue over Tokio (2 workers) publishing snapshots
    through a watch channel; the UI thread never performs disk, network or hashing work.
- `launcher/`, `client/` — retained JavaFX launcher and independent Java client settings
  library, buildable with the Gradle wrapper.
- Installation safety: HTTPS host allowlist, redirect re-validation, SHA-1 verification,
  `.part` resume, bounded retries, cancellation, atomic final writes, path/link/archive
  checks, exclusive installation lock, trash-based removal.

## Verification evidence (2026-11-20)
- `cargo test -p aero-core`: **17 passed, 0 failed** (download verification, resume,
  retry, corruption, cancellation, path/zip safety, trust policy, Maven/Windows rules,
  instance isolation/duplication, settings validation, per-instance overrides, legacy
  manifests).
- `cargo check --workspace`: clean, zero warnings, Rust 1.100.0 (pinned, edition 2024).
- `cargo fmt --all -- --check` clean.
- Live metadata (`piston-meta` version manifest): `1.8.9` (2015-12-03), `1.21.11`
  (2025-12-09), `26.2` (2026-06-16), `26.3` (2026-09-15) all exist as releases; current
  latest release is `26.3`. Fabric pins come from `meta.fabricmc.net` at install time.
- Native binary link was **not** possible in the Linux development container: the GUI
  development libraries (`freetype`, `fontconfig`, `EGL`, `GL`, `wayland-egl`, `GLESv2`)
  are absent and there is no root access. Type-checking passes for the same code; the
  Windows workflow in `.github/workflows/native.yml` produces the release build.

## Known limitations
- No Microsoft login, entitlement check, game launch, classpath resolution (M3).
- Play shows a disabled glossy `Launch` button until M3; installation still works.
- Windows-only game installation (`install` asserts `cfg!(windows)`); other hosts build
  and test the shell only.
- No idle-RAM/idle-CPU measurement yet: needs the Ryzen 5 5500U reference machine.
- No visual QA of the Freya window yet (headless container).
- rustls is used instead of the OS trust store; see ADR 0006.
- News items are release metadata and repository release titles only — never fabricated.

## Next steps
1. Run `cargo run --release -p aero-launcher` on Windows 11 x64, complete a 1.8.9 Forge and
   a 26.3 Fabric install through the UI, and record idle memory/CPU.
2. Visual QA of the three themes and small-window layout; fix any spacing issues found.
3. Only after parity is recorded: remove the JavaFX launcher, keep the Java client.
4. Then stop: M3 requires explicit approval.
