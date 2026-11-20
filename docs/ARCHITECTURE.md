# Architecture (M2)

`launcher` and `client` remain independent Java 21 Gradle projects. The client settings
library is unchanged in behavior. The launcher now uses JavaFX instead of the M1 Swing
view; SettingsStore/LauncherSettings remain compatible with existing preferences.

## UI and work ownership
`AeroApplication` owns the five navigation pages and presentation only. A single bounded,
expiring worker serializes disk/network/hash/extraction/runtime-probe work; FX updates
are dispatched and transfer progress is throttled. Downloads are bounded (maximum
three active transfers); there is no idle polling or continuous visual animation. User
cancel sets a token and closes an active stream away from the UI thread.

## Installation boundary
`Installer` takes an installation lock shared across Aero processes. `Metadata` resolves
live versions and caches mutable metadata for six hours; content-addressed metadata
is checksum-verified. `Downloads` checks HTTPS hosts and every redirect, performs up to
three attempts, resumes `.part` files only with matching range responses, and publishes
only files matching declared size/hash. Repair re-verifies files before reuse.

`Libraries` evaluates Windows rules, resolves Maven coordinates and safely extracts
natives with path and size limits. `Loaders` consumes Fabric's compatible loader profiles
and the official Forge 1.8.9 legacy install profile; no installer code is executed.
Official SHA-1 checksums provide compatibility/integrity checks, not independent publisher
authentication. TLS and explicit trusted origins are also required. No checksum means fail.

`Runtimes` reads the game's Java component/major requirement, probes trusted selected or
detected executables, then installs a verified official Windows x64 runtime if needed.
Managed runtime paths are keyed by component and manifest hash, shared between instances.
The launcher's own Java home is excluded from auto-detection. Custom Java is explicit
user-trusted executable input; version probes time out. Managed files are verified before use.

## Storage
Default root: `~/.aero-client/data/`, overridable for developer tests only.

```
metadata/                 cached official/loader metadata
runtimes/<component>/<hash>/  shared verified game Java
shared/clients/<hash>.jar  official game client (never committed)
shared/libraries/         verified library artifacts
shared/assets/            content-addressed asset index/objects
instances/<uuid>/instance.json
instances/<uuid>/installation/  source version/loader profiles, natives, plan
instances/<uuid>/game/     independent saves/config/mods/resourcepacks/shaderpacks
```

Instance names never become filesystem paths. Schema and UUIDs are validated. Path
traversal, Windows reserved names/streams and links escaping roots are rejected. Status
records interrupted work without claiming it is ready. No instance deletion exists in M2.

## Deferred boundaries
M3 alone owns authentication/entitlements/token storage and launch commands. Stored M2
plans explicitly disable launch; no fake session or auth bypass exists. M4 owns the full
instance manager/Modrinth. M9 owns production setup installer/signing; current jlink and
jpackage app images remain development distribution output. See ADR 0004 and 0005.
