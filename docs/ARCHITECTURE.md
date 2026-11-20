# Architecture

## Implemented foundation

```
launcher (Java 21, Swing)         client (Java 21 library)
  AeroLauncher                    ClientSettings
       |
  SettingsStore -> local non-secret preferences
```

There is no dependency between these modules and no runtime network code. The
launcher can build without the client and vice versa. The client artifact is a
platform-neutral customization core, not a Minecraft-loadable mod. Swing supplies
an initial desktop shell without runtime dependencies. UI construction runs on the
event dispatch thread; short preference reads/writes are currently synchronous.

Settings validate before persistence. Writes use a temporary sibling and atomic
replacement where supported, with a non-atomic fallback. Corrupt settings produce a
visible warning and defaults; they are replaced only on explicit Save. This store
must never hold tokens. Multiple concurrent launcher instances are last-writer-wins.

## Planned trust boundaries (not implemented)

Launcher UI -> authentication service -> Microsoft/Xbox/Minecraft entitlement checks.
Secure credential-store adapter holds tokens outside settings and logs. An install
service consumes official metadata, validates origins/redirects and integrity, and
writes only under a user-selected game directory. A launch service accepts a verified,
non-expired session and validated install. No public boolean flag may substitute for
ownership verification. All blocking authentication/download work must leave the UI thread.

Client core -> separately reviewed mod-loader adapter -> public Minecraft/mod APIs.
Only original client code and license-compatible dependencies may be distributed.
Minecraft game code/assets remain separate downloads through official services.

## Architecture governance

ADRs record context, decision, consequences and status. Significant changes to module
boundaries, auth, storage, networking, dependencies, release or privacy require an ADR
before implementation. Supersede old records rather than rewriting historical rationale.
See [decision index](adr/README.md).
