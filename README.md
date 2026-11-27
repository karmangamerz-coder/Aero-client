# Aero Client

Aero Client is a completely free and open-source Minecraft: Java Edition launcher and
client project. Original source is licensed GPL-3.0-only; third-party components keep
their own compatible licenses. Aero has no premium tier, subscriptions, paid cosmetics,
mandatory account, advertising, telemetry, or proprietary modules.

This is an independent community project, not an official Minecraft product and not
affiliated with Mojang or Microsoft. It does not contain Minecraft binaries. Minecraft is
downloaded from official distribution services after the player signs in with a Microsoft
account that owns Java Edition.

## 2.0.0

The primary launcher is native Rust using Freya/Skia, Tokio, Reqwest and Serde. It provides
the Aero Light, Dark and Lite interface, resumable first-launch setup, official Microsoft
browser PKCE authentication, OS credential-vault storage, verified Minecraft and Java
installation, isolated instances, game launching, logs and crash discovery.

The v1 completion branch also includes:

- explicit Modrinth search and hash-verified installation for compatible mods, resource
  packs and shaders, including required dependency resolution and a recoverable manifest;
- instance create, rename, duplicate, repair and recoverable removal;
- an independent Java client foundation with original module metadata, profiles, atomic
  persistence, normalized HUD placement and push-based media state;
- auditable Fabric performance presets whose projects are resolved live for the exact game
  version instead of bundling third-party mod binaries;
- user-requested GitHub release discovery with strict semantic-version parsing;
- Windows portable and NSIS installer pipelines, Linux/macOS checks, dependency audits and
  SHA-256 release manifests.

Published artifacts are available for testing; automated checks do not establish successful
live account sign-in. Real Microsoft sign-in/game launch, version-specific
Forge/Fabric in-game adapters, Modrinth modpack import, Windows code signing, multi-DPI
visual QA and benchmark runs still require dedicated validation. See PROJECT_STATE.md.

## Build independently

Requirements for contributors are Rust 1.100, Java 21 and the checked-in Gradle wrapper.
End users use a release artifact and need none of these tools.

    cargo test --locked -p aero-core
    cargo build --locked --release --workspace
    ./gradlew clean build

Windows:

    .\gradlew.bat clean build

The launcher and client can be built separately:

    cargo build --locked --release -p aero-launcher
    ./gradlew :client:build

Set AERO_DATA_DIR to an empty test directory to isolate developer runs. Normal data is
stored under %USERPROFILE%\.aero-client\data on Windows. Never point it at unrelated
files.

Visual QA can start a deterministic window by setting `AERO_WINDOW_WIDTH` and
`AERO_WINDOW_HEIGHT`; invalid values fall back to 1180x760 and the supported minimum remains
1000x640. These variables affect only the initial window dimensions.

## Privacy and authentication

Microsoft sign-in uses OAuth 2.0 authorization code flow with PKCE (S256) in your default
browser. Aero never receives a Microsoft password or needs a client secret. Xbox Live,
XSTS, Minecraft Services, Java ownership and the Minecraft profile are checked before play.
Microsoft family restrictions remain enforced; Aero cannot change or bypass them.

Refresh tokens are stored in Windows Credential Manager, macOS Keychain or Linux Secret
Service via `keyring`. Minecraft and Microsoft access tokens stay in memory. The local
`accounts.json` contains only username, UUID, skin URL, application ID and selection.
Vault failures are visible; there is no plaintext fallback. Linux needs an unlocked Secret
Service implementation such as GNOME Keyring or KWallet. Headless Linux CI tests use fake
vaults rather than your real credentials.

Sessions refresh automatically before launch when expired, including after restarting Aero.
Multiple accounts can be selected in the top bar. **Sign out** cancels pending login and
removes the active account's local credential; **Remove** deletes a selected saved account.
Neither action signs you out of Microsoft's browser cookies or revokes other devices.
Use your Microsoft account security page if you want to revoke access everywhere.

Upgrading from the DPAPI account-file format requires signing in again. Your game instances
and saves are retained. The obsolete `accounts.bin` is removed after credentials are saved
successfully in the vault.

### Microsoft application registration — project owner setup

This is one registration for Aero, not a registration each player should create. You need
access to an existing Microsoft Entra directory and permission to register apps. A personal
Microsoft account without a directory cannot register an app from that screen. Azure signup
may ask for a card; Aero offers no bypass of Microsoft eligibility requirements.

1. Open [Microsoft Entra App registrations](https://entra.microsoft.com/) and select
   **App registrations → New registration**. Name it **Aero Client**.
2. Choose an audience supporting **personal Microsoft accounts** (personal accounts only,
   or organizational and personal accounts). Minecraft sign-in uses the consumers authority.
3. Under **Authentication**, add the **Mobile and desktop applications** platform with
   redirect URI `http://localhost/callback`. Aero binds an available IPv4 loopback port
   before opening the browser and uses `http://localhost:<port>/callback` for that attempt.
   Microsoft ignores the port when matching localhost desktop redirects. Do not register
   a Web/SPA redirect, enable implicit grants, add wildcard redirects or create a secret.
4. Copy the **Application (client) ID**, not the directory ID or object ID. Set it in
   **Settings → Microsoft client ID**, save preferences, then press **Sign in with Microsoft**.
   Developers may set `AERO_MS_CLIENT_ID` at runtime. Release maintainers can set the GitHub
   repository Actions variable `AERO_MS_CLIENT_ID`; native release builds embed that public
   ID so players do not need to configure it. Runtime configuration takes precedence.
5. Obtain any required Xbox/Minecraft API authorization for this application through
   Microsoft's official developer process. Creating an Entra registration alone does not
   guarantee Minecraft Services access. Aero displays an application-approval error for
   rejection; it never substitutes another launcher's client ID.
6. Test an owning account, an expired session, cancellation, account switching and local
   logout. A family-restricted account must be resolved by the family organizer through
   Microsoft's official settings. Complete password entry and consent only on Microsoft.

Official references: [register an app](https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app),
[desktop app configuration](https://learn.microsoft.com/en-us/entra/identity-platform/scenario-desktop-app-registration),
[redirect URI rules](https://learn.microsoft.com/en-us/entra/identity-platform/reply-url),
[authorization code and PKCE](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow).

### Authentication troubleshooting

| Message | Next step |
| --- | --- |
| Missing/invalid client ID | Configure Aero's registered Application ID; save and retry. |
| Application rejected / HTTP 403 | Check registration and Minecraft API approval with Microsoft. |
| Session expired or revoked | Sign in again; no cached session bypass is offered. |
| No Java ownership | Use the Microsoft account that owns Java Edition. |
| Xbox profile or family restriction | Complete the official Xbox profile/family steps. |
| Vault unavailable or locked | Unlock the OS credential store; Linux requires Secret Service. |
| Browser failed / timeout | Check the default browser, retry, and finish within five minutes. |
| Redirect mismatch | Register the exact desktop localhost callback above. |

Tokens, callback codes and verifiers are never logged. The callback is single-use, state
checked, bounded in size and time, and closed on completion/cancellation. Auth errors use
fixed descriptions rather than echoing provider response bodies. See ADR 0012 and PRIVACY.md.

## Contributing and architecture

Community contributions are welcome. Read CONTRIBUTING.md, CODE_OF_CONDUCT.md, AGENTS.md,
ROADMAP.md and the decisions under docs/adr. Do not copy proprietary code or assets from
Lunar Client, OneClient or any other closed client. OneLauncher and other public projects
may inform architecture only when their licenses are respected.
