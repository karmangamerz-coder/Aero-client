# Permanent Aero Client requirements

Read this file, PROJECT_STATE.md and ROADMAP.md before each task. Preserve existing
working features and local changes; inspect relevant files only. Do not recreate or
unnecessarily restructure the repository. Prefer medium reasoning and targeted work;
never sacrifice correctness or security to reduce cost.

## Architecture, license and security
- M2-R supersedes the launcher stack: Rust + Freya + Skia in a Cargo workspace, Tokio,
  Reqwest and Serde. Keep the in-game client Java. Retain JavaFX fallback until Rust
  parity is validated; no Tauri, Electron, Chromium or JavaFX in the replacement.
- Do not copy GPL OneLauncher source or branding. Architecture reference only.
- Standalone native distribution uses cargo-packager/NSIS in M9; no launcher Java needed.
- Original code is MIT as directed by the owner for M2; third-party licenses remain intact.
- Free/open-source project: no paid gates, subscriptions, cosmetics, proprietary Aero
  modules, mandatory Aero accounts/telemetry, tracking or source obfuscation.
- Never copy proprietary Lunar Client or OneClient source. Use public APIs and reviewed
  open-source dependencies with notices, version locks and verified checksums.
- Keep launcher and client independently buildable. Java 21 toolchain; wrapper only.
- M2 installs official game files; M3 owns Microsoft authentication, ownership checks
  and authenticated launch. No offline ownership bypass. Do not start M3 automatically.
- Never commit credentials, tokens, signing keys, game binaries, downloaded game Java,
  caches or user data. Use HTTPS allowlisted distributions, validated redirects, integrity
  verification, safe paths, atomic final writes and bounded inputs. Never run installers
  or arbitrary commands from remote metadata. Custom Java is explicit user choice.
- UUID instance manifests own only their directories. Never overwrite unrelated instances
  or user content. Share verified assets/libraries and compatible runtimes, not saves/configs.
- Record every major architecture decision in an ADR; retain superseded records.

## Permanent visual system
- Modern Modrinth-inspired usability with original Frutiger Aero / Windows 7 Aero styling.
- Compact Home, Library, Discover, Downloads, Settings navigation; clean layout, consistent
  typography/spacing, restrained outline SVG icons only where useful.
- Subtle glossy glass, aqua/sky-blue accents, natural green and a glossy green Play button.
  Nature-inspired backgrounds where appropriate; accessible contrast and responsive layout.
- No generic dark gaming dashboard, unnecessary cards/badges/decorations, excessive
  rounding/gradients/animations, expensive live blur or copied branding/interface.
- Apply this system to future pages, mod menus, HUD editors and music overlays.

## Performance and distribution
- Reference: Ryzen 5 5500U, integrated Radeon, 16 GB DDR4, Windows 11. Never invent benchmarks.
- Never block the JavaFX thread with disk/network work, extraction, hashing or subprocesses.
  Bound downloads and worker queues; reuse metadata/files/runtimes. Idle work stops; no
  continuous rendering, unnecessary threads/dependencies, Electron or Chromium.
- End users must not install Java, JavaFX, Gradle, IntelliJ or developer tools.
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
