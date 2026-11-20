# Milestone 1 verification — 2026-11-20

Local environment: Windows x64, Eclipse Temurin 21.0.13+9, Gradle 8.14.3.

- `clean build :launcher:packageNative`: passed.
- Checked-in Gradle wrapper build with normal dependency verification: passed.
- Seven JUnit tests: passed, zero failures/errors/skips.
- Launcher JAR `--self-check`: passed.
- Windows packaged `AeroClient.exe --self-check`: exit code 0.
- Four binary/source JARs rebuilt with `--rerun-tasks`: SHA-256 values identical.
- Independent launcher/client task graphs: neither schedules the other module.
- Twenty resolved dependency artifact/metadata SHA-256 values match Maven Central
  checksum sidecars; wrapper JAR matches Gradle's published checksum. Sidecar checking
  confirms repository consistency, not an independent security audit of upstream code.
- OSV query of ten exact locked Maven versions: no known findings at check time.
- Both GitHub workflow files pass actionlint; Python scripts compile.

Limitations: no hosted GitHub CI has run; Linux/macOS execution and interactive UI
behavior have not been manually verified. No game login/download/play is implemented.
Same-host JAR reproducibility does not establish cross-host or native-image reproducibility.
Public GitHub publication and security settings remain repository-owner setup work.
