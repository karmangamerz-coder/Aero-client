# Aero Client

Free, open-source Minecraft Java Edition launcher and client foundations. **MIT**
for original source; third-party components retain their own licenses.
Not an official Minecraft product; not affiliated with Mojang or Microsoft.

## 0.2.0 — M1 visual refinement and M2 installation

The JavaFX launcher has Home, Library, Discover, Downloads and Settings, with a
lightweight modern Frutiger Aero design. M1 local preferences and the independent
client settings library are preserved. Discover/Modrinth marketplace work belongs to M4.

M2 installs official Minecraft files for versions actually present in live metadata:
1.8.9, 1.21.11, 26.2 and 26.3. Choose Vanilla, compatible stable Fabric for modern
versions, or Forge 11.15.1.2318 for 1.8.9. Installation supports **Windows 10/11 x64**.
Metadata determines the required Java runtime. Aero detects compatible game Java,
honors a compatible custom executable, or downloads and verifies a shared runtime.
No manual game Java installation is required.

**Play is intentionally disabled.** Microsoft authentication, ownership verification
and authenticated game launch belong to M3 and are not implemented here. Installation
does not imply ownership and provides no offline/cracked launch path. No game files,
downloaded runtimes, credentials or user data are redistributed in this repository.

Downloads support integrity checks, cancellation, bounded retries, partial-file resume,
cache reuse and repair. Instances isolate saves, configs, mods, resource packs and
shader packs. Repair preserves user content and rechecks required shared files.

## Run from source (developers)

Use JDK 21 and the checked-in wrapper. Windows:

```powershell
.\gradlew.bat :launcher:run
.\gradlew.bat clean build
.\gradlew.bat :launcher:run --args=--verify-metadata
.\gradlew.bat :launcher:packageNative
```

Linux/macOS developers use `./gradlew`; the shell builds there, but M2 game installation
is Windows x64 only. Launcher/client build independently with `:launcher:build` and
`:client:build`. The launcher now needs its dependency JARs; use Gradle run, the generated
`launcher/build/install/launcher/bin/launcher.bat` after `:launcher:installDist`, or
the portable application image. A bare launcher JAR alone is not a distribution.

Developer-only CLI: `--install 1.8.9 FORGE` or `--install 26.3 FABRIC` installs without
launching. Override storage using `-Daero.dataDir=<absolute path>` when invoking Java
or Gradle. Never point this at an existing unrelated game directory.

## Local data and privacy

Preferences remain in `~/.aero-client/settings.properties`; optional custom Java is
in `~/.aero-client/custom-java.txt`. Game data defaults to `~/.aero-client/data/`.
Startup does not fetch metadata or download files: use Refresh official versions.
No telemetry, accounts, premium features, subscriptions, paid cosmetics or tracking.
See [PRIVACY](PRIVACY.md), [architecture](docs/ARCHITECTURE.md),
[PROJECT_STATE](PROJECT_STATE.md), [ROADMAP](ROADMAP.md) and [contributing](CONTRIBUTING.md).

## Distribution

Current `:launcher:packageNative` produces a development portable app image using
jlink + jpackage, with a bundled launcher runtime and JavaFX dependencies. End users
of that image need no Java, JavaFX, Gradle or IDE. Production `AeroClient-Setup.exe`,
portable ZIP, signing, shortcuts and installer polish remain M9 work. Preserve all
image directories and third-party notices. Game runtimes are managed separately.
