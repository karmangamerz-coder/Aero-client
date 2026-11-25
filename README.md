# Aero Client

Aero Client is a completely free and open-source Minecraft: Java Edition launcher and
client project. Original source is licensed GPL-3.0-only; third-party components keep
their own compatible licenses. Aero has no premium tier, subscriptions, paid cosmetics,
mandatory account, advertising, telemetry, or proprietary modules.

This is an independent community project, not an official Minecraft product and not
affiliated with Mojang or Microsoft. It does not contain Minecraft binaries. Minecraft is
downloaded from official distribution services after the player signs in with a Microsoft
account that owns Java Edition.

## 1.0.0-beta.1

The primary launcher is native Rust using Freya/Skia, Tokio, Reqwest and Serde. It provides
the Aero Light, Dark and Lite interface, resumable first-launch setup, official Microsoft
device-code authentication, encrypted Windows account storage, verified Minecraft and Java
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

Beta means that source and packaging are ready for community testing, not that unperformed
live checks are being claimed. Real Microsoft sign-in/game launch, version-specific
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

## Privacy and authentication

Microsoft sign-in uses OAuth 2.0 device code in the system browser; Aero never receives a
password. Xbox Live, XSTS, Minecraft services and entitlement verification are required.
Tokens are protected with Windows DPAPI and excluded from logs, UI text and crash reports.
Network access happens for an action the player requested; there is no telemetry or news
polling. Read PRIVACY.md and SECURITY.md before reporting sensitive issues.

### Configure Microsoft sign-in

Aero deliberately does not reuse another launcher's Microsoft application identity. Before
testing sign-in, the project owner must register an Azure app:

1. In Microsoft Entra **App registrations**, create a registration that supports personal
   Microsoft accounts (or both organizational and personal accounts).
2. Under **Authentication > Advanced settings**, enable public client flows. Do not create
   or commit a client secret; device-code applications are public clients.
3. Copy the Application (client) ID into **Settings > Microsoft application client ID**,
   or set `AERO_MS_CLIENT_ID` for a developer run. The value is a public GUID, not a token.
4. Press **Add account**, follow the displayed `microsoft.com/link` instructions, and use a
   Microsoft account that owns Minecraft: Java Edition.

Tokens and signing keys must remain outside the repository. A build without a configured
client ID can still use every offline launcher-management feature, but Add account explains
what must be configured instead of attempting a misleading legacy identity.

## Contributing and architecture

Community contributions are welcome. Read CONTRIBUTING.md, CODE_OF_CONDUCT.md, AGENTS.md,
ROADMAP.md and the decisions under docs/adr. Do not copy proprietary code or assets from
Lunar Client, OneClient or any other closed client. OneLauncher and other public projects
may inform architecture only when their licenses are respected.
