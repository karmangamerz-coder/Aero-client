# ADR 0011: GPL source, release artifacts and updates

Status: accepted — 2026-11-24

## Decision

All original Aero Client code is GPL-3.0-only from version 1.0.0-beta.1 onward. Third-party
components retain their own licenses and notices. This restores the project's original
open-source requirement and prevents proprietary Aero modules.

Windows CI produces a portable ZIP, a per-user NSIS installer and SHA256SUMS.txt. The
installer has Start Menu entries, an optional desktop shortcut and an uninstaller. Tagged
builds publish these exact CI artifacts. Linux and macOS validate the portable code where
supported; game installation remains Windows x64 until separately implemented and tested.

Update discovery is an explicit user action against the GitHub Releases API. Aero validates
semantic versions and exposes release URLs, but never silently replaces a running binary.
Release signing keys and API credentials stay outside Git. Checksums are integrity metadata,
not a substitute for a future code-signing certificate.

## Consequences

- Contributors and redistributors receive the complete corresponding source under GPLv3.
- Unsigned beta builds may show Windows SmartScreen; this limitation must be disclosed.
- A production signing identity can be injected by CI secrets without changing source.

