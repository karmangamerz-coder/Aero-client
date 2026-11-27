# Aero Client 2.0.0

Native Windows launcher release with Microsoft browser authorization-code PKCE (S256),
Xbox/XSTS/Minecraft library integration and OS credential-vault refresh storage.
Includes account switching, cancellation, local logout/removal, session renewal before
launch, skin avatar/body presentation and safe authentication errors.

The release also includes the responsive Freya UI, setup-navigation, Modrinth filtering
and full-window artwork fixes from the preceding refinement branch.

## Before signing in

Aero's own Microsoft Application ID must be configured and authorized for Minecraft API
access. No project-owned approved ID was available during this release's development.
The build therefore explains missing configuration; successful live owning-account login,
family-policy handling against Microsoft and actual game launch remain unverified.
See README.md for registration, desktop redirect URI and configuration instructions.
Players should not each create Azure registrations: this is one project-owner setup.

Account storage changed incompatibly from the older DPAPI file. Sign in again to store
credentials in Windows Credential Manager. Instances and game saves remain intact.
Sign out removes the active local credential, not Microsoft browser cookies or other devices.

## Downloads and verification

- AeroClient-Setup.exe: per-user Windows installer.
- AeroClient-Portable.zip: standalone Windows launcher and CLI.
- SHA256SUMS.txt: SHA-256 checksums for both artifacts.

Original source is GPL-3.0-only; bundled dependency notices are included. Minecraft binaries
are downloaded from official services and are not redistributed. Windows binaries are
unsigned and may be blocked by application-control policy; signing remains future work.
Linux/macOS have compile checks, not supported game-launch binaries in this release.

Automated verification covers core security cases and a synthetic Windows Credential Manager
roundtrip, plus Gradle tests and dependency audits. It is not a substitute for the live
account and platform tests noted above.
