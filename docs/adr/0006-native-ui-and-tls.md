# 0006 — Native Rust launcher UI and TLS backend

Date: 2026-11-20
Status: Accepted

## Context

M2-R replaces the JavaFX shell with Freya + Skia. The first native pass ported the
installation services but kept a single-file prototype UI, a hard-coded toolchain pin,
and `native-tls`, which requires a system OpenSSL toolchain to build.

## Decision

1. **Launcher UI is split by responsibility**: `design.rs` (tokens and the custom control
   set), `shell.rs` (rail, top bar, status strip), `pages.rs` (page composition) and
   `ui.rs` (state ownership and routing). All reactive state is created once in the app
   root; page functions are pure and receive state, so hook order can never drift.
2. **Rendering is event driven.** Static SVG artwork, no animation timer, no polling and
   no periodic metadata requests. The optional pointer-rotated character is a discrete
   state update per pointer move and is disabled in low-performance mode.
3. **TLS uses rustls** instead of the platform OpenSSL. It removes a system build
   dependency, keeps verification behaviour identical on Windows and CI Linux, and adds
   no runtime installer requirement. Certificate roots come from `webpki-roots`.
   Platform trust stores (corporate proxies, enterprise roots) are therefore **not**
   used; this is an accepted trade-off for M2 and must be revisited before M9 if the
   owner needs OS-managed roots.
4. **Advanced installation settings live per instance** (`memory_mib`, `java_path`) in the
   instance manifest, written only by an explicit save action. Older manifests without
   these fields keep loading because both are `#[serde(default)]`.
5. **JavaFX stays in the tree** until the Windows parity run converts the milestone from
   "code complete" to "verified".

## Consequences

- The launcher type-checks on any Rust host, but linking needs the platform GUI
  development packages (Windows: none beyond the MSVC toolchain).
- Google/GitHub/Mojang/Modrinth endpoints are validated by host allowlist as before; the
  TLS change does not widen the allowlist.
- Per-instance overrides do not affect M2 installation behaviour except choosing the Java
  executable, which was already an explicit user choice.
