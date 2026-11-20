# Changelog

This project uses Semantic Versioning. Dates use ISO 8601.

## 0.3.0 — 2026-11-20

### Added
- Native Freya/Skia launcher with an Aero Light, Aero Dark and Aero Lite design system:
  glass surfaces, glossy green launch control, 64 px icon rail, compact top bar and a
  status strip with live installation progress.
- Home with original pointer-rotated explorer preview and a collapsible 24 % news panel;
  dedicated instance management screen holding the advanced installation settings and the
  verified installation plan.
- Per-instance memory and Java overrides stored in the instance manifest, with validation
  and backward-compatible loading of older manifests.
- Session activity log and read-only installation plan view (`clientSha1`, asset index,
  library count, chosen runtime).

### Changed
- Rust workspace is the primary launcher implementation; the JavaFX launcher is retained
  as a fallback until Windows parity is confirmed.
- `reqwest` now uses rustls instead of the platform OpenSSL (ADR 0006).
- News, discovery and downloads pages never fabricate content; empty states explain when
  nothing was fetched.

### Fixed
- Launcher UI no longer borrows page state across the app root, removing a hook-order risk
  in the prototype.
- Core tests cover manifest migration and per-instance override validation.

## 0.2.0 — 2026-11-20

### Added
- JavaFX Aero interface with five navigation pages and asynchronous Downloads controls.
- Official metadata, Windows game files, Fabric and legacy Forge installation/repair.
- Automatic compatible Java management and UUID-isolated instance foundations.
- Download integrity, resume/retry/cancel and path/archive safety tests.
- Persistent project state and M1–M9 roadmap with standalone distribution requirements.

### Changed
- Original current source uses MIT per the owner's M2 instruction.
- Authentication and game launching explicitly remain M3; production installer remains M9.
- The M1 preference format and independent client core are retained.

## 0.1.0 — 2026-11-20

### Added
- GPL-3.0-only project policies and architectural decision records.
- Independent Java 21 launcher and client core Gradle modules.
- Offline desktop shell and validated local preference storage.
- Unit tests, dependency locks and artifact verification metadata.
- Cross-platform CI, native app images, release bundles and SHA-256 checksums.

### Limitations
- No Microsoft authentication, game downloading, game launch or Minecraft mod integration.
- Native images are unsigned and their reproducibility is not guaranteed.
