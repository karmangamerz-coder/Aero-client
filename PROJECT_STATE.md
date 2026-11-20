# Project state

## Active M2-R migration
Rust workspace under `native/core` and `native/launcher` is being validated alongside
the retained JavaFX launcher. Original implementation uses Freya 0.5.3/Skia, Tokio,
Reqwest and Serde; no OneLauncher source/assets were copied. OneLauncher README was
consulted only for high-level workspace/packaging architecture. Java client stays Java.
The previous Java 1.8.9 Forge installation completed with READY status. Rust end-to-end
parity and redesigned UI validation are pending; do not retire JavaFX yet.
Local C++ build toolchain installation was blocked by automatic approval policy;
native compilation/testing uses GitHub's Windows runner.

Version: 0.2.0 development. Target: Windows 10/11 x64. Original-source license: MIT
by the owner's M2 instruction; prior GPL-tagged history remains historical.

## Baseline and current implementation
The actual M1 repository contained a Swing shell, not JavaFX. The M2 UI layer replaces
that shell while retaining LauncherSettings, SettingsStore, client core and their tests.
No unrelated user changes were present at task start; main matched origin/main.

- JavaFX `AeroApplication` and `aero.css`: five pages, Aero design, local preferences.
- `install/Downloads`: allowlisted HTTPS, integrity, retries, resumable partial files, cancel.
- `install/Metadata`: official cached manifests and exact version lookup.
- `install/Libraries`, `Loaders`: Windows rules/natives, Fabric metadata, Forge legacy profile.
- `install/Runtimes`: detected/custom Java compatibility, official component-driven shared runtime.
- `install/Instances`, `Installer`: isolated UUID folders, process lock, status and repair.
- JavaFX is bundled as application dependencies; jlink supplies the launcher runtime.
- Game data is external to the repository. Play/authentication remain unimplemented (M3).

## Verification in progress
Official metadata checked 2026-11-20: 1.8.9 (Java 8), 1.21.11 (Java 21), 26.2/26.3
(Java 25) are release entries. Fabric metadata lists stable loaders for modern targets.
Do not treat this observation as permanent: the launcher checks actual metadata.
Initial compilation and inherited tests passed before the new failure/recovery suite.
Final test/build/integration results will be recorded before marking M2 complete.

## Known boundaries and next steps
- No Microsoft login, entitlement checking, game launch, marketplace or production installer.
- Installation targets Windows x64; other hosts build/test the shell only.
- Full installation verification and visual QA are pending during development.
- Future M3 must interpret inherited profiles and library overrides; M2 stores source
  profiles and dependency inventory, not an executable game launch command.
- Public GitHub: https://github.com/karmangamerz-coder/Aero-client, main.
- Complete M2 verification, commit and verified push; stop before M3.
