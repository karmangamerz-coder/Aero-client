# Project state

## Current milestone
M2.1 (launcher redesign), M3 (authentication and launching) and M3-S (first-launch setup)
are **implemented, regression-tested and interactively verified** in the Rust launcher on
the Windows reference machine. Real Microsoft sign-in, an actual game launch and complete
multi-DPI visual QA remain, so the JavaFX launcher stays as a fallback.

Version: 0.4.0 development. Target: Windows 10/11 x64. Original-source license: MIT.

## What exists
- `native/core` — `aero-core` services:
  - `install.rs`: official metadata, verified downloads, libraries/natives, Fabric and
    legacy Forge profiles, managed Java runtimes, UUID instances.
  - `auth.rs` (M3): Microsoft OAuth 2.0 **device-code** sign-in in the browser (password
    never touches Aero), Xbox Live, XSTS, Minecraft services session, entitlement check,
    profile and skin retrieval, automatic refresh, multi-account store encrypted with
    Windows DPAPI (`accounts.bin`), switch/remove.
  - `launch.rs` (M3): authenticated launch command built from the verified installation
    plan and version metadata (modern `arguments` and legacy `minecraftArguments`,
    placeholder substitution, platform rules, loader `inheritsFrom`), managed or explicit
    Java, `-Xmx` from the instance/settings, process supervision, bounded live logs,
    cancellation, crash-report discovery and token-redacted command logging.
  - `skin.rs`: Minecraft skin decoding, original Aero fallback skin, CPU-composed standing
    character PNG (no GPU work, no per-frame rendering).
  - `hardware.rs`: one-shot CPU/RAM/storage/OS detection and a read-only Windows WMI GPU
    query, plus conservative memory/performance recommendations.
  - `onboarding.rs`: validated setup state machine, resumable step and completion flag
    (`onboarding.json`), including guarded transitions and a safe defer path.
- `native/launcher` — `aero-launcher` (Freya 0.5.3 + Skia):
  - `design.rs`: Aero Light/Dark/Lite palettes, glass surfaces, glossy green Launch control,
    custom buttons, chips, progress, outline icons, static landscape and character view.
  - `shell.rs`: 58 px icon-only rail, compact top bar with Aero branding and the Microsoft
    account selector, status strip shown only while work runs.
  - `pages.rs`: Play (character, username/online state, instance/version/loader selectors,
    Launch + instance settings, progress inside the launch control), Instances and instance
    detail (installation, overrides, plan), Discover, Downloads (downloads + live game logs
    + cancellation), Settings (themes, memory, Java, accounts, setup/storage), and the
    seven-step setup wizard with live theme previews and resumable progress.
  - `backend.rs`: command queue over Tokio (2 workers), watch-channel snapshots, supervised
    sign-in polling and game process; the UI thread never performs blocking work.
- `launcher/`, `client/` — retained JavaFX launcher and independent Java client settings
  library, buildable with the Gradle wrapper.

## Verification evidence (2026-11-21)
- `cargo test --locked -p aero-core`: **30 passed, 0 failed**, including setup validation,
  guarded Back/Next/Skip transitions, completion and defer regression coverage.
- `cargo check --locked --workspace`: clean, **zero warnings**, Rust 1.100.0.
- `cargo fmt --all -- --check`: clean.
- Live official metadata re-verified previously: `1.8.9`, `1.21.11`, `26.2`, `26.3`.
- GitHub Actions Windows release build `44888560348`: release workspace compiled, tests
  passed and the portable artifact plus SHA-256 manifest were generated successfully.
- Interactive Windows 11 verification: Next, Back, Skip, appearance selection, version and
  loader selection, performance, Summary and **Create instance only** all worked. The main
  launcher opened, and restarting with the same data opened the launcher directly rather
  than repeating setup. Persisted setup and instance records matched the selected values.
- Reference-machine idle sample after restart: 130.8 MiB working set, 119 MiB private
  memory and about 0.16% of one CPU core over 10 seconds. This is a single observation,
  not a benchmark; GPU use was not measured.
- **Not verified:** real Microsoft sign-in and token refresh/DPAPI storage, a real game
  session, or visual QA at 125% and 150% scaling. UI transitions were checked through the
  Windows accessibility tree at the current display scale; screenshot capture was not
  reliable enough to claim pixel-level clipping verification.

## Known limitations
- Sign-in uses public client id `00000000402b5328` by default (the public Minecraft launcher
  client). Owners should register their own Azure public client and set
  `AERO_MS_CLIENT_ID` before a public release; see ADR 0007.
- Setup step 5 shows the instance folder but a user-selectable game folder is deferred to
  M4 instance management; the wizard says so instead of pretending it works.
- Discover lists Modrinth projects only; installation is M4. News was removed entirely.
- Performance presets store the player's choice and the game memory; automatic mod
  installation arrives with M7. No benchmark numbers are published yet.
- Crash reports are located and surfaced (path shown in the interface and log); uploading
  or parsing them into a report UI is not implemented.
- Game installation and launching still assert Windows x64; other hosts build and test the
  shell and services only.

## Next steps
1. Complete sign-in on Windows 11 x64 with an account that owns Minecraft and verify DPAPI
   storage, refresh, switching and removal without exposing tokens.
2. Perform visual QA of all three themes at 880×560 and 1180×760 at 100%, 125% and 150%.
3. Launch 1.8.9 Forge and 26.3 Fabric, confirming logs, cancellation and crash handling.
4. Repeat performance measurements under a documented protocol, then decide whether the
   JavaFX fallback can be retired.
5. Then stop: M4 requires explicit approval.
