# Project state

## Current release

Version: 1.0.0-beta.1 development
Primary platform: Windows 10/11 x64
Original-source license: GPL-3.0-only

M1–M3 remain implemented. This completion pass opened every M4–M9 track and delivered the
safe, independently testable foundations described in ROADMAP.md without claiming live
Minecraft, signing or benchmark evidence that does not exist.

## New in this pass

- native/core/src/content.rs: explicit Modrinth install, bounded required dependencies,
  exact version/loader resolution, trusted endpoints, SHA-1 verification, per-instance
  content.json, reversible enable/disable and recoverable removal.
- native/core/src/performance.rs: four transparent Fabric performance manifests resolved
  through the same compatibility and integrity boundary.
- native/core/src/update.rs: user-requested GitHub release discovery and SemVer validation.
- client module: original module registry, immutable profiles, atomic ProfileStore,
  normalized HudLayout and push-based MusicState.
- packaging/windows/AeroClient.nsi: per-user NSIS setup with shortcuts and uninstaller.
- native CI: Windows setup/portable/checksums, tagged release publishing and Linux/macOS
  workspace checks. Security CI now audits locked Maven and Rust dependencies.
- GPL-3.0-only migration and ADRs 0009–0011.
- Responsive Freya layout repair for 1280x720 and larger windows, including unclipped Play,
  Settings and setup content, true Lite surfaces, immediate theme changes and visible status.
- Configurable Aero-owned Microsoft public-client identity, immediate device-code feedback,
  expiry-aware token renewal and sanitized launcher/panic diagnostics.
- Faceted Modrinth discovery, project/version details, SHA-512 verification where published,
  and installed-content enable, disable and removal controls.

## Verification in this working copy (2026-11-25)

- Java 21 client unit tests: 8 passed, 0 failed.
- Local Rust formatting passes. This machine lacks the MSVC linker, so GitHub Actions is the
  authoritative clean Windows/Linux/macOS compile, test and Windows packaging environment.
- Linux and macOS portable checks passed for PR #11 at commit `9a58c4a`; the Windows package
  job was still running when this state was written. Final results must be recorded before
  merging.
- A live device-code request proved the former borrowed legacy client ID is rejected by
  Microsoft. Aero now requires its own public-client GUID; owning-account login and game
  launch remain pending that owner-provided registration.
- Earlier 0.4.0 evidence remains: 30 aero-core tests passed and GitHub Windows release run
  44888560348 succeeded before this pass.

## Release gates still requiring real environments

1. CI must pass after the final push, including Rust format, core tests, all-platform checks,
   Windows release build, NSIS packaging and dependency audits.
2. An owning Minecraft account must validate device-code login, DPAPI refresh and real
   launch for supported targets. Credentials and tokens must never be committed.
3. Version-specific in-game adapters, modpack import, media session integration and actual
   benchmark evidence remain open beta work; ROADMAP.md lists each boundary precisely.
4. Stable Windows distribution needs an external code-signing identity held only in CI
   secrets plus clean install/upgrade/uninstall testing.
