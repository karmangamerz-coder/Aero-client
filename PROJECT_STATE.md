# Project state

## Current milestone
M2.1 (launcher redesign), M3 (authentication and launching) and M3-S (first-launch setup)
are **implemented and type/test verified** in the Rust launcher. They are not yet verified
by a human on Windows: sign-in against Microsoft, an actual game launch and the visual
result still need the reference machine, so the JavaFX launcher stays as a fallback.

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
  - `onboarding.rs`: setup answers, resumable step and completion flag (`onboarding.json`).
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
- `cargo test --locked -p aero-core`: **24 passed, 0 failed** (previous 17 plus argument
  substitution, platform rules, memory resolution, account store behaviour, skin
  composition/PNG encoding, onboarding validation and hardware recommendations).
- `cargo check --locked --workspace`: clean, **zero warnings**, Rust 1.100.0.
- `cargo fmt --all -- --check`: clean.
- Live official metadata re-verified previously: `1.8.9`, `1.21.11`, `26.2`, `26.3`.
- **Not verified:** Windows-native build and launch, real Microsoft sign-in, a real game
  session, DPAPI behaviour, visual QA and idle memory/CPU measurements. The Windows release
  workflow remains the authority for linking and packaging.

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
1. Run `cargo run --release -p aero-launcher` on Windows 11 x64 and complete sign-in for an
   account that owns Minecraft.
2. Visual QA of the three themes, the 880×560 minimum window and DPI scaling; fix spacing.
3. Launch 1.8.9 Forge and 26.3 Fabric, confirm logs, cancellation and crash handling.
4. Record idle memory/CPU on the Ryzen 5 5500U reference machine, then retire the JavaFX
   launcher once parity is recorded.
5. Then stop: M4 requires explicit approval.
