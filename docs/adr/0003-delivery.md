# ADR 0003: Open licensing and reproducible delivery

Status: accepted, 2026-11-20.

Context: all original source must remain open, and users must build independently.

Decision: GPL-3.0-only for original code, explicit dependency license review, pinned
Gradle/JUnit versions, locks and SHA-256 verification. Deterministic JAR ordering and
timestamps. Native app images built separately on each target OS using jpackage.
CI produces artifacts/checksums, with manual reviewed publication and public source.

Consequences: no installer-tool dependency or signing keys are needed for Milestone 1.
Native images are unsigned and not guaranteed bit-reproducible because runtimes and
host images vary. JAR comparisons require equal source, JDK, Gradle and dependencies.
Record build environment and preserve Java runtime legal files. Future signatures,
notarization and reproducible native builds require separate decisions.

Alternatives: shipping proprietary Minecraft binaries violates project scope; automatic
publication before license/source checks is rejected; obfuscation harms transparency.
