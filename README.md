# Aero Client

Free, open-source Minecraft Java Edition launcher and client foundations. **MIT** for
original source; third-party components retain their own licenses.
Not an official Minecraft product; not affiliated with Mojang or Microsoft.

## 0.3.0 — native launcher, installation and instances

The launcher is a native Rust desktop application (Freya + Skia, Tokio, Reqwest, Serde)
with an Aero Light, Aero Dark and Aero Lite design system: a 64 px icon rail, compact top
bar, home screen with an original pointer-rotated explorer preview, a collapsible news
panel, a library with per-instance management, Modrinth discovery, downloads and settings.
The JavaFX launcher is retained temporarily as a fallback until Windows parity is recorded.

M2 installs official Minecraft files for versions present in live metadata: 1.8.9,
1.21.11, 26.2 and 26.3. Choose Vanilla, compatible stable Fabric for modern versions, or
Forge 11.15.1.2318 for 1.8.9. Installation supports **Windows 10/11 x64**. Metadata
determines the required Java runtime; Aero detects compatible game Java, honours an
explicit custom executable, or downloads and verifies a shared runtime. No manual game
Java installation is required.

**Play is intentionally disabled.** Microsoft authentication, ownership verification and
authenticated game launch belong to M3 and are not implemented. Installation does not
imply ownership and provides no offline/cracked launch path. No game files, downloaded
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
cargo test -p aero-core                       # 17 installation and safety tests
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
same folder. Instances, shared assets and managed runtimes are stored there too. Startup
performs at most one cached news request when news is enabled; metadata is refreshed only
on request. No telemetry, accounts, premium features, paid cosmetics or tracking.
See [PRIVACY](PRIVACY.md), [architecture](docs/ARCHITECTURE.md),
[PROJECT_STATE](PROJECT_STATE.md), [ROADMAP](ROADMAP.md) and [contributing](CONTRIBUTING.md).

## Distribution

The native launcher is a single executable today. cargo-packager + NSIS stand-alone
installers (`AeroClient-Setup.exe`, portable ZIP, `SHA256SUMS.txt`) remain M9 work. The
JavaFX `:launcher:packageNative` task still produces a development app image with a bundled
launcher runtime.
