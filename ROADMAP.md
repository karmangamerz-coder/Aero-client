# Product roadmap

## M2.1 — final launcher redesign (implemented, visual QA pending)
- [x] Home without news or promotional copy: compact top bar with Aero branding, Microsoft
  account selector at the top-right, narrow icon-only rail.
- [x] Large character preview using the real skin when signed in and an original Aero
  fallback skin otherwise, with username and online state.
- [x] Compact instance, Minecraft version and loader selectors beneath the character.
- [x] One glossy green Launch control with an instance settings button beside it and
  installation progress inside the control.
- [x] Aero Light / Dark / Lite presets applied consistently to every page.
- [x] Static original landscape, translucent blue glass, subtle gloss, moderate radii,
  Segoe UI and consistent outline icons; no glow, flares, bubbles or live blur.
- [ ] Visual QA at 1180×760, 880×560 and high DPI on Windows.
- Removed: the unfinished news feature (interface, settings, cached requests) is gone for
  good; Aero issues no background request that the player did not ask for.

## M3 — authentication and Minecraft launching (implemented, runtime verification pending)
- [x] Official Microsoft OAuth 2.0 device-code sign-in in the system browser (no embedded
  web view, no password collection), Xbox Live, XSTS and Minecraft services chain.
- [x] Minecraft ownership verification through the entitlements endpoint; no bypass.
- [x] Token storage encrypted with Windows DPAPI, automatic refresh, multiple accounts,
  switching and removal. Tokens never appear in logs, UI text or crash output.
- [x] Authenticated profile name, UUID and skin retrieval.
- [x] Launch arguments from verified metadata (modern and legacy formats, loader profiles,
  platform rules), version-compatible Java, memory allocation, process management.
- [x] Live game logs, crash-report discovery, launch cancellation and readable errors.
- [ ] Real sign-in and real 1.8.9 Forge / 26.3 Fabric launches on the reference machine.

## M3-S — first-launch setup (implemented and Windows runtime verified)
- [x] Seven steps: welcome, Microsoft account (skippable, never blocks browsing), hardware
  detection with recommendations, appearance with live previews, Minecraft version/loader/
  Java management, performance preset, summary with create/install/defer.
- [x] Back/Next/Skip controls, progress saved per step, resume after restart, completion
  persisted, re-runnable from Settings, never repeated automatically.
- [x] Backend-owned state machine with expected-step guards prevents repeated queued clicks;
  Finish persists a completed record and Setup later defers without mutating an instance.
- [x] Interactive Windows run completed through Summary, created an instance, entered the
  main launcher and confirmed setup did not repeat after restart.
- [x] All work happens off the UI thread; no Aero account, no ads and no telemetry.
- [ ] Custom game folder selection: deferred to M4 with instance management (the wizard
  states this instead of pretending the setting is wired up).

## M4 — instance management and Modrinth integration
- Full instance lifecycle: version/loader changes, per-instance folders, user-selectable
  game directory, import/export, repair of individual components.
- Explicit Modrinth marketplace: licensing review, compatibility checks, explicit install
  and user-data-safe management.

## M5 — in-game client foundation and mod menu
- Select and review client adapters for target game versions; original, customizable
  in-game mod menu. Respect every mod license; no copied proprietary implementations.

## M6 — HUD editor, PvP utilities, cosmetics and quality of life
- HUD editor, PvP-focused utilities, original cosmetics and QoL modules built on the same
  Aero visual system.

## M7 — performance mods, benchmarks and hardware optimisation
- Performance mod presets integrated with the setup wizard and Settings, real benchmarks on
  the reference machine, hardware-specific recommendations. Publish only measured values.

## M8 — music integration
- Privacy-conscious optional music integration with legally sourced, synchronised lyrics.

## M9 — updates, installers and public release
- Automatic updates, `AeroClient-Setup.exe`, `AeroClient-Portable.zip`, `SHA256SUMS.txt`,
  optional shortcuts, uninstaller, icon, version information, notices, signing as
  appropriate and tested upgrades. End users need no Rust, Cargo, Java or Gradle.

## M1 — foundation and visual refinement (complete)
- [x] Independent launcher/client builds; settings validation and persistence.
- [x] Public repository, source policies and architecture records.
- [x] JavaFX shell: Home, Library, Discover, Downloads, Settings with original Aero styling.

## M2 — installation, runtimes and instance foundations (complete)
- [x] Official metadata, live availability for 1.8.9, 1.21.11, 26.2 and 26.3.
- [x] Verified client, libraries, Windows natives and asset installation.
- [x] Compatible modern Fabric; legacy Forge 1.8.9 without executing installer code.
- [x] Progress, cancel, retry, interrupted-download recovery and repair.
- [x] Java detection, validation, automatic download, verification, cache/repair/reuse,
  explicit custom executable and incompatible-selection fallback.
- [x] UUID instances with isolated user folders, shared verified files and per-instance
  memory/Java overrides in the manifest.
- Scope: Windows 10/11 x64. Authentication and launching live in M3.
