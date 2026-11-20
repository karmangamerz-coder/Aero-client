# Product roadmap

## M2-R — native Rust migration (active)
- [ ] Port and verify installation/runtime/instance/preferences parity in Rust.
- [ ] Freya/Skia character-focused Aero design, compact icon rail and glass header.
- [ ] Light/Dark/Lite themed controls; original landscape and pointer-rotated fallback avatar.
- [ ] Cached factual news, discovery browsing, accessible download/settings pages.
- [ ] Windows tests/build, native launch, measured idle usage, verified GitHub publication.
- Keep JavaFX until parity is validated; keep Java client; no auth/game launch (M3).
- Permanent design: 58–68 px rail, selected instance/version/account state in header,
  central character and glossy contextual launch control, collapsible 22–25% news panel.
  Original assets only; sky/clouds/hills/water as static artwork, no live environmental blur.
- Controls must be custom-themed, with hover/press/focus/disabled states and keyboard
  access. Idle animation off by default; reduced motion/static low-performance mode.
- Native distribution: cargo-packager + NSIS planned in M9, standalone portable executable.
  Target <150 MB idle memory and <1% CPU where achievable; report measured values only.

## M1 — foundation and visual refinement
- [x] Independent launcher/client builds; settings validation and persistence retained.
- [x] Public repository, source policies and architecture records.
- [x] JavaFX shell: Home, Library, Discover, Downloads, Settings.
- [x] Original modern Frutiger Aero / Windows 7 Aero styling: glossy glass, sky/aqua,
  green Play, restrained outline SVGs, accessible text, consistent spacing and responsive layout.
- Permanent: apply this language to future launcher pages, mod menus, HUD editors and music.
  No generic dark dashboard, copied branding, excessive decoration, blur or animations.

## M2 — installation, runtimes and instance foundations
- [x] Official metadata, caching, live availability for 1.8.9, 1.21.11, 26.2, 26.3.
- [x] Verified client, libraries, Windows natives and asset installation.
- [x] Compatible modern Fabric; legacy Forge 1.8.9 without executing installer code.
- [x] Progress, cancel, retry, interrupted-download recovery and repair in Downloads/Library.
- [x] Java detection, version validation, automatic metadata-driven download, verification,
  cache/repair/reuse, explicit custom executable and incompatible-selection fallback.
- [x] UUID-based Aero Default/custom instances; isolated user folders; shared verified files.
- [ ] Final integration/build/visual verification and publication: see PROJECT_STATE for evidence.
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
- Permanent: bounded async I/O, no FX thread blocking, caches and immutable sharing,
  low memory/disk usage, idle background work suspended; no Electron/Chromium/live blur.

## M9 — standalone Windows production distribution
- Minimal jlink launcher runtime + required JavaFX, packaged by jpackage.
- AeroClient-Setup.exe, AeroClient-Portable.zip where practical, SHA256SUMS.txt.
- Optional desktop shortcut, Start Menu shortcut, uninstaller, icon, version information,
  full third-party notices, signing/notarization as appropriate and tested upgrades.
- End users need no Java, JavaFX, Gradle, IntelliJ or developer tools.
- Game runtimes are downloaded only when needed and managed independently by M2.
- Preserve the current development image pipeline; full production installer work waits here.
