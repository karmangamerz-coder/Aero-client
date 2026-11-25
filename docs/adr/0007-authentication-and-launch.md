# 0007: Authentication, token storage and game launching

- Status: accepted (2026-11-21)
- Applies to: `native/core/src/auth.rs`, `native/core/src/launch.rs`

## Context
M3 requires official Microsoft sign-in, Xbox/Minecraft authentication, ownership
verification, secure token storage, refresh, multiple accounts, correct launch arguments
and supervised game processes — without collecting passwords, without bypassing ownership
and without exposing tokens.

## Decision
1. **OAuth 2.0 device-code flow.** Aero asks Microsoft for a short code, opens the user's
   system browser at `microsoft.com/link`, and polls the token endpoint. No embedded web
   view, no password field, no client secret. Polling is a supervised Tokio task that
   honours cancellation and the server-provided interval.
2. **Aero owns its client identity.** The owner registers an Azure public-client application
   supporting personal Microsoft accounts and enables public client flows. Its Application
   (client) ID is provided in Settings or `AERO_MS_CLIENT_ID`. Aero has no built-in identity
   and never borrows the legacy Minecraft Launcher ID or another launcher's registration.
   A client ID identifies an application, never a user, and is not a secret; client secrets
   are neither needed nor accepted by this device-code design.
3. **Ownership is verified, never bypassed.** `/entitlements/mcstore` must contain
   `product_minecraft` or `game_minecraft`; otherwise sign-in fails with an explicit
   message. No offline or "demo" mode.
4. **Token storage.** The account list (Microsoft refresh token + Minecraft session) is
   serialised and sealed with `CryptProtectData` (DPAPI, user scope, UI forbidden) on
   Windows. On non-Windows development hosts the file is written with the same atomic
   replacement but not encrypted, which is documented and never shipped. Tokens are never
   logged: error bodies are redacted and launch commands are described without secrets.
5. **Launch command from verified metadata only.** Arguments are built from
   `installation/version.json`, the loader profile stored during installation, and
   `installation/plan.json` (library paths, client jar, asset index). Modern `arguments`
   and legacy `minecraftArguments` are both supported, rules filter by host platform, and
   placeholders are substituted from a fixed map. Unknown placeholders abort the launch
   instead of starting a half-configured game.
6. **Java and memory.** Java comes from the managed Mojang runtime (matching the version
   manifest's `javaVersion`) or an explicit per-instance/custom executable; memory comes
   from the instance override, then settings. Values are validated to 1024–32768 MiB.
7. **Process supervision.** The child runs with `kill_on_drop(false)`, stdout/stderr are
   read by two tasks, logs are bounded, the command line is written redacted to
   `logs/last-command.txt`, cancellation kills the process, and the newest file in
   `game/crash-reports` is surfaced when the game exits abnormally.

## Consequences
- Users never type a Microsoft password into Aero, and Aero holds no client secret.
- Sign-in needs a network connection and a browser; the code expires after ~15 minutes.
- Until the owner configures an Azure public-client application, sign-in remains unavailable
  with an actionable explanation; offline instance and settings management still work.
- Non-Windows builds can run the whole chain but store accounts unencrypted; the shipping
  target is Windows, where DPAPI is used.
- Launch correctness depends on the installation plan and metadata, so a partially
  installed instance refuses to launch instead of guessing.

## Alternatives considered
- Embedded web view / password form: forbidden by the ownership and privacy policy.
- Storing tokens in the credential manager per account: DPAPI on a single sealed file is
  simpler to make atomic and to remove cleanly.
- Building the launch command from the raw version JSON at launch time: would re-fetch and
  re-parse remote metadata on every launch; the verified plan is faster and auditable.
