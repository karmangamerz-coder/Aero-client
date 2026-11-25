# ADR 0010: Adapter-neutral in-game client foundation

Status: accepted — 2026-11-24

## Decision

The Java client module owns stable module metadata, profiles, normalized HUD placement
and media snapshots. Version-specific Forge/Fabric adapters consume that API; the launcher
never depends on game classes. Built-in modules are original, local, and server-safe:
FPS/CPS/keystrokes/ping, armor/effects, coordinates/direction, toggle sprint/sneak, zoom,
crosshair, chat appearance, and now-playing.

Profiles use atomic local persistence. The media API is push-based: an operating-system
adapter sends snapshots and capabilities when they change. Aero does not scrape players,
poll streaming services, or ship lyrics. A future lyrics provider must document a lawful
source and degrade to no lyrics.

## Consequences

- Core behavior is testable without redistributing Minecraft or mappings.
- Adapters can target incompatible game versions without forking profile formats.
- No reach, combat automation, packet manipulation, anti-cheat bypass, or similar cheat
  surface belongs in the registry.

