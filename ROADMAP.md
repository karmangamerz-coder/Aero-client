# Product roadmap

This roadmap separates implemented source from live validation. A checked implementation
item has code and automated coverage; it does not imply a real account, game session,
hardware benchmark or signed installer was tested when the project lacks those resources.

## M1 — launcher foundation (implemented)

- [x] Independent launcher and client builds, persistence, navigation and project policies.
- [x] Original Aero interface and documented architecture.

## M2 / M2-R / M2.1 — installation and native launcher (implemented)

- [x] Official metadata, verified game files, assets, libraries and native extraction.
- [x] Fabric and legacy 1.8.9 Forge profiles without executing remote installer code.
- [x] Managed compatible Java, integrity, cancellation, retry, repair and isolated instances.
- [x] Rust/Freya/Skia launcher and Aero Light, Dark and Lite design system.
- [x] Responsive layout foundation for 1280x720 and 1920x1080, with internal setup scrolling.
- [ ] Complete 100%, 125% and 150% multi-DPI visual QA on Windows.

## M3 / M3-S — authentication, launch and setup (implemented)

- [x] Microsoft authorization-code PKCE, Xbox, XSTS, ownership, profile and skin.
- [x] OS credential-vault refresh storage, cancellation, multiple accounts, switching and logout.
- [x] Aero-owned configurable public-client identity and automatic expired-token renewal.
- [x] Verified launch arguments, supervised process, bounded logs and crash discovery.
- [x] Resumable seven-step setup with persisted completion and no Skip/defer path.
- [ ] Validate the full chain with an owning account and real 1.8.9/modern game sessions.

## M4 — instances and Modrinth (beta implementation)

- [x] Create, rename, duplicate, repair and recoverably remove instances.
- [x] Search mods, modpacks, resource packs and shaders with version/loader facets.
- [x] Project details, compatible version listing and installed-content management UI.
- [x] Explicit hash-verified mod/resource-pack/shader install with required dependencies.
- [x] Per-instance installed-content manifest, reversible enable/disable and removal APIs.
- [ ] Modrinth modpack import/export, individual update UI and external game directories.

## M5 — in-game client foundation (beta implementation)

- [x] Adapter-neutral Java registry, categories, search-ready metadata and favorites-ready IDs.
- [x] Atomic named profiles and settings persistence.
- [x] Original server-safe module catalog; no cheat or anti-cheat bypass surface.
- [ ] Ship and validate version-specific Forge/Fabric adapters and the Right Shift menu.

## M6 — HUD and utilities (beta implementation)

- [x] Resolution-independent draggable HUD placement model and scale validation.
- [x] FPS, CPS, keystrokes, ping, armor/effects, coordinates/direction, toggle input,
  zoom, crosshair and chat appearance definitions.
- [ ] Connect every definition to rendered game adapters and complete interactive HUD QA.

## M7 — performance optimisation (beta implementation)

- [x] Potato, Maximum FPS, Balanced and Visual Quality manifests using maintained Modrinth
  project identities and exact compatibility resolution.
- [x] No bundled mod binaries; incompatibility fails safely.
- [ ] Run documented comparative benchmarks and validate each target-version combination.

## M8 — music and lyrics (foundation implemented)

- [x] Event-driven track, artist, artwork, progress, playback state and capability model.
- [x] No background service polling and graceful empty state.
- [ ] Windows media-session adapter, in-game overlay and a documented lawful lyrics source.

## M9 — distribution and public release (beta implementation)

- [x] AeroClient-Setup.exe NSIS source, per-user install, optional desktop shortcut,
  Start Menu entries and uninstaller.
- [x] AeroClient-Portable.zip and SHA256SUMS.txt generation in Windows CI.
- [x] Tagged GitHub release upload, Linux/macOS checks, Gradle builds and dependency audits.
- [x] User-requested semantic-version update discovery; no silent replacement.
- [ ] Code-sign a stable build and validate clean install, upgrade and uninstall on Windows.

## 2.0.0 refinement

- [x] Silver Vista-inspired native UI with original assets and labeled navigation.
- [x] Required once-only setup, persisted completion and no Skip/defer action.
- [x] Modrinth default/filter correction and Vanilla mod-install guard.
- [x] Explicit distinction between OAuth authorization and verified Minecraft sign-in.
- [x] Sign-in result panel remains visible outside setup's scrolling content on every step.
- [ ] Resolve Minecraft Services application rejection and verify owning-account login.
- [ ] Finish full-window/multi-DPI visual QA of the revised interface.
