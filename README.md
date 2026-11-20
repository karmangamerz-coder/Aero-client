# Aero Client

Free, open-source Minecraft Java Edition launcher and client foundations. **MIT** for
original source; third-party components retain their own licenses.
Not an official Minecraft product; not affiliated with Mojang or Microsoft.

## 0.4.0 — sign-in, launching and a first-launch setup

The launcher is a native Rust desktop application (Freya + Skia, Tokio, Reqwest, Serde)
with an Aero Light, Aero Dark and Aero Lite design system: a narrow icon-only rail, compact
top bar with Aero branding and the Microsoft account selector, a home screen built around
the player's own skin, instance/version/loader selectors and one glossy Launch control, a
library with per-instance management, Modrinth discovery, downloads with live game logs,
settings and a seven-step first-launch setup. The JavaFX launcher is retained temporarily
as a fallback until Windows parity is recorded.

Microsoft sign-in uses the official OAuth device-code flow in your browser: Aero never sees
your password. Xbox Live, XSTS, Minecraft services and the ownership check run in order;
tokens are sealed with Windows DPAPI and never logged. Launching uses verified metadata,
the managed Java runtime and the memory you choose, with live logs, cancellation and crash
report discovery.

M2 installs official Minecraft files for versions present in live metadata: 1.8.9,
1.21.11, 26.2 and 26.3. Choose Vanilla, compatible stable Fabric for modern versions, or
Forge 11.15.1.2318 for 1.8.9. Installation supports **Windows 10/11 x64**. Metadata
determines the required Java runtime; Aero detects compatible game Java, honours an
explicit custom executable, or downloads and verifies a shared runtime. No manual game
Java installation is required.

Authentication requires an account that owns Minecraft: Java Edition. Aero verifies
ownership with Microsoft and provides no offline or cracked launch path. Sign-in, launching
and the setup wizard are implemented and type/test verified but have not yet been exercised
on Windows by a human; see [PROJECT_STATE](PROJECT_STATE.md). No game files, downloaded
runtimes, credentials or user data are redistributed in this repository.

Downloads support integrity checks, cancellation, bounded retries, partial-file resume,
cache reuse and repair. Instances isolate saves, configs, mods, resource packs and shader
packs; verified assets, libraries and runtimes are shared. Advanced installation settings
(memory, custom Java) live in each instance's management screen.

## Run the native launcher (Windows 11 x64, developers)

Requires the pinned Rust toolchain from `rust-toolchain.toml` (installed automatically by
`rustup`) and, on Windows, the MSVC build tools. End users of a packaged build need none of
this; packaging is M9.

```powershell
cargo run --release -p aero-launcher          # desktop launcher
cargo test -p aero-core                       # 24 installation, auth, launch and skin tests
cargo run -p aero-core --bin aero-cli -- versions
cargo run -p aero-core --bin aero-cli -- install 26.3 fabric
cargo build --release --workspace             # aero-launcher.exe + aero-cli.exe
```

On Linux the code type-checks (`cargo check --workspace`), but linking the GUI needs
`freetype`, `fontconfig`, `EGL`, `GL` and `GLESv2` development packages. Game installation
itself is Windows-only and refuses to run elsewhere.

Override storage with `AERO_DATA_DIR=<absolute path>`; the default is
`%USERPROFILE%\.aero-client\data`. Never point this at an existing unrelated game directory.

## Legacy JavaFX launcher (fallback)

```powershell
.\gradlew.bat :launcher:run
.\gradlew.bat clean build
.\gradlew.bat :launcher:packageNative
```

`launcher` and `client` still build independently (`:launcher:build`, `:client:build`).
The launcher needs its dependency JARs: use Gradle run, the generated
`launcher/build/install/launcher/bin/launcher.bat` after `:launcher:installDist`, or the
portable application image. A bare launcher JAR is not a distribution.

## Local data and privacy

Preferences live in `~/.aero-client/data/launcher-settings.json`; optional custom Java in the
same folder. Instances, shared assets and managed runtimes are stored there too. Accounts
live in `accounts.bin`, encrypted with Windows DPAPI on Windows. Setup progress is stored in
`onboarding.json`, so an interrupted setup resumes. Aero makes no background metadata or
news request: everything is fetched when you ask for it. No telemetry, premium features,
paid cosmetics or tracking.
See [PRIVACY](PRIVACY.md), [architecture](docs/ARCHITECTURE.md),
[PROJECT_STATE](PROJECT_STATE.md), [ROADMAP](ROADMAP.md) and [contributing](CONTRIBUTING.md).

## Distribution

The native launcher is a single executable today. cargo-packager + NSIS stand-alone
installers (`AeroClient-Setup.exe`, portable ZIP, `SHA256SUMS.txt`) remain M9 work. The
JavaFX `:launcher:packageNative` task still produces a development app image with a bundled
launcher runtime.
