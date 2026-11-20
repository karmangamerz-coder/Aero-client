# Product roadmap

## M2-R — native Rust migration (code complete, Windows verification pending)
- [x] Port installation/runtime/instance/preferences parity in Rust (`native/core`).
- [x] Freya/Skia Aero design: 64 px icon rail, glass header, character-focused home.
- [x] Light/Dark/Lite themes, custom controls with hover/press/focus/disabled states.
- [x] Cached factual news, Modrinth discovery, downloads and settings pages.
- [x] Advanced installation settings in a dedicated per-instance management screen.
- [ ] Windows-native launch, visual QA and measured idle memory/CPU on the reference
  machine; then remove the JavaFX launcher (Java client stays).
- Keep no auth/game launch (M3). No OneLauncher source or assets copied.
- Permanent design: 58–68 px rail, selected instance/version/account state in the header,
  central character and glossy contextual launch control, collapsible 22–25 % news panel.
  Original assets only; sky/hills/water as static SVG artwork, no live environmental blur.
- Idle animation off by default; low-performance and reduced-motion presets exist.
- Native distribution: cargo-packager + NSIS in M9. Target <150 MB idle and <1 % CPU;
  publish only measured values.

## M1 — foundation and visual refinement
- [x] Independent launcher/client builds; settings validation and persistence retained.
- [x] Public repository, source policies and architecture records.
- [x] JavaFX shell: Home, Library, Discover, Downloads, Settings.
- [x] Original modern Frutiger Aero / Windows 7 Aero styling, restrained outline icons.
- Permanent: apply this language to future launcher pages, mod menus, HUD editors and music.
  No generic dark dashboard, copied branding, excessive decoration, blur or animation.

## M2 — installation, runtimes and instance foundations (complete)
- [x] Official metadata, caching, live availability for 1.8.9, 1.21.11, 26.2, 26.3
  (verified against `piston-meta` on 2026-11-20).
- [x] Verified client, libraries, Windows natives and asset installation.
- [x] Compatible modern Fabric; legacy Forge 1.8.9 without executing installer code.
- [x] Progress, cancel, retry, interrupted-download recovery and repair.
- [x] Java detection, version validation, automatic metadata-driven download, verification,
  cache/repair/reuse, explicit custom executable and incompatible-selection fallback.
- [x] UUID-based Aero Default/custom instances; isolated user folders; shared verified files;
  per-instance memory/Java overrides in the manifest.
- [x] Rust port of every item above with 17 passing tests.
- [x] Publication of the implementation and verification evidence (see PROJECT_STATE).
- Scope: Windows 10/11 x64. Authentication and game launching are not part of M2.

## M3 — authentication and authenticated launch (do not start automatically)
- Approved Microsoft public-client OAuth flow, entitlement/profile validation, secure token
  storage, refresh, logout, cancellation and safe process launch. Require legitimate ownership.
- Resolve inherited loader/vanilla launch arguments and classpath overrides correctly.
- No embedded secret, Aero account requirement, offline bypass or token logging.

## M4 — full instances and Modrinth
- Full instance lifecycle and explicit Modrinth marketplace integration; licensing review,
  explicit mod installation, compatibility checks and user-data-safe management.

## M5 — original client integration
- Select/review client adapters for target game versions; original performance and HUD modules.
- Respect every mod license; no proprietary copied implementations.

## M6 — customization and accessibility
- HUD editor, in-game mod menus, accessible controls and original Aero visual system.

## M7 — music overlay
- Privacy-conscious optional music integration using compatible public APIs; Aero visuals.

## M8 — performance and hardening
- Profile on Ryzen 5 5500U / integrated Radeon / 16 GB / Windows 11; publish measured results.
- Security, cancellation, disk-space, accessibility and upgrade/recovery audits.
- Permanent: bounded async I/O, no UI-thread blocking, caches and immutable sharing,
  low memory/disk usage, idle background work suspended; no Electron/Chromium/live blur.

## M9 — standalone Windows production distribution
- cargo-packager + NSIS for the native launcher; `AeroClient-Setup.exe`,
  `AeroClient-Portable.zip` where practical, `SHA256SUMS.txt`.
- Optional desktop shortcut, Start Menu shortcut, uninstaller, icon, version information,
  full third-party notices, signing/notarization as appropriate and tested upgrades.
- End users need no Rust, Cargo, Java, Gradle, IntelliJ or developer tools.
- Game runtimes are downloaded only when needed and managed independently by M2.
