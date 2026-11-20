# Aero Client

A completely free, open-source Minecraft Java Edition launcher and client project.
Original code is licensed under **GPL-3.0-only**, subject to third-party compatibility.
The project vision includes Frutiger Aero aesthetics, performance optimizations,
customizable HUDs and Modrinth integration; these remain planned beyond the foundation.
Not an official Minecraft product; not approved by or associated with Mojang or Microsoft.

## Current status: 0.1.0 / Milestone 1

Implemented: a Java 21 Swing launcher shell, validated local preferences with atomic
replacement where supported, an independent platform-neutral client settings core,
unit tests, Gradle wrapper, reproducible JAR settings, and native app-image packaging.

**This is not yet a playable Minecraft client.** Authentication, entitlement checks,
official game downloads and a mod-loader adapter are future milestones. Play is
disabled. No mock authentication, offline ownership bypass or bundled game binaries.
Theme and memory choices are stored preferences for future integration.

## Commitments

- All original launcher and client source must be public before distribution.
- No premium features, subscriptions, paid cosmetics or proprietary Aero modules.
- No Aero account requirement, telemetry, tracking or source obfuscation.
- Community contributions and independently compiled launcher/client are welcome.
- Original implementations using public APIs and compatible open-source libraries;
  never copy proprietary Lunar Client or OneClient source code.
- Minecraft ownership must be verified through legitimate Microsoft authentication
  before game access; game files will come from official distribution services.
- Privacy, performance, customization and transparent community governance come first.

## Build and run

Install JDK 21, then run from this directory (Windows: use `gradlew.bat`):

```sh
./gradlew clean build
./gradlew :launcher:run
./gradlew :launcher:build       # does not build client
./gradlew :client:build         # does not build launcher
./gradlew :launcher:packageNative
java -jar launcher/build/libs/launcher-0.1.0.jar --self-check
```

The wrapper downloads the pinned Gradle distribution and validates its SHA-256.
Initial builds need network access to Gradle and Maven Central. Later builds may use
`--offline` with a populated cache. Runtime Milestone 1 makes no network requests.
Tests use JUnit; the application has no third-party runtime libraries.

Native output: `launcher/build/native/AeroClient/` (Windows/Linux) or
`launcher/build/native/AeroClient.app` (macOS). Windows contains `AeroClient.exe`;
keep its companion directories. These are portable application images, not installers.
Packaging runs on the target OS. CI covers Windows, Linux and macOS. Images bundle
a Java runtime with its legal notices and are currently unsigned/not notarized.

Preferences are written only when Save is clicked to
`~/.aero-client/settings.properties`; delete that file to reset them.

## Development and releases

See [CONTRIBUTING](CONTRIBUTING.md), [architecture](docs/ARCHITECTURE.md),
[ADRs](docs/adr/README.md), [ROADMAP](ROADMAP.md), [privacy](PRIVACY.md), and
[third-party notices](THIRD_PARTY_LICENSES.md).
Versioning follows [Semantic Versioning 2.0.0](https://semver.org/).
See [release instructions](docs/RELEASING.md) for public-source requirements,
checksums and reproducibility limits. CI configuration is supplied; hosted runs
require publishing this repository to GitHub.
