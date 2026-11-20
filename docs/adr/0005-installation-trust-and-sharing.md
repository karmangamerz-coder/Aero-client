# ADR 0005: Verified installation, sharing and runtime management

Status: accepted, 2026-11-20.

Context: M2 must install actual supported versions, recover safely, isolate user data
and avoid manual Java installation on Windows 10/11 x64 without implementing auth.

Decision: use official Mojang version/runtime metadata, Fabric metadata and Forge's
official legacy installer data. Allowlisted TLS endpoints, explicit redirect validation,
checksummed artifacts, bounded metadata/archive inputs and atomic final writes. Three
downloads at a time are intentionally conservative on the reference low-end hardware.
An expiring worker handles all I/O; cancellation closes blocking transfers off the FX thread.

UUID instance directories hold mutable game data. Verified assets/libraries/clients and
game runtimes are shared; runtime identities use manifest hashes, not guessed Java versions.
Forge installer JSON is read but installer code/processors are never executed. Fabric
loader selection is pinned per instance after compatibility resolution. No auth tokens exist.

Consequences: repairs hash files and can take time, but avoid redundant transfers. Existing
compatible Java is validated by major/architecture, not guaranteed to be vendor-patched.
Mojang's official checksums use SHA-1; these are integrity checks under trusted HTTPS,
not an independent signature scheme. Unavailable metadata fails clearly without fabricated
versions. Windows x64 only is explicit; unsupported platforms fail before installation.

Alternatives: running downloaded installers exposes command execution; plain-text filenames
as instance identities risk traversal; copying runtimes into every instance wastes disk;
unbounded parallel downloads harm responsiveness and resource usage.
