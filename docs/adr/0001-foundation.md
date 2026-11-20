# ADR 0001: Java foundation and independent modules

Status: accepted, 2026-11-20.

Context: no existing repository or agreed Minecraft version is available. Milestone 1
needs verifiable foundations without inventing authentication or game compatibility.

Decision: JDK 21, pinned Gradle wrapper, launcher and client subprojects with no mutual
dependency. Use Swing for an offline shell and a plain Java client customization core.
Defer mod-loader choice and game-specific adapters until a compatibility ADR is approved.

Consequences: both artifacts compile independently; no external runtime dependencies.
The client is not a playable mod and theme settings do not yet style the shell.
Swing reduces packaging complexity but needs later visual/accessibility work.

Alternatives: JavaFX adds runtime modules; Electron adds a separate runtime and stack;
immediate loader integration commits to game/loader licensing before requirements exist.
