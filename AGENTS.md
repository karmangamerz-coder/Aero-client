# Instructions for contributors and coding agents

- Preserve the free, open-source, GPL-3.0-only project requirements.
- Never introduce paid gates, proprietary Aero modules, mandatory accounts/telemetry,
  tracking, source obfuscation or ownership bypasses.
- Never copy proprietary Lunar Client or OneClient code. Record third-party provenance.
- Do not redistribute Minecraft binaries/assets. Use official distribution services only.
- Keep launcher and client independently buildable; client must not depend on launcher.
- Use JDK 21 and the Gradle wrapper. Run `./gradlew clean build` for code changes and
  `./gradlew :launcher:packageNative` for packaging changes on supported hosts.
- Add focused behavioral tests. Keep runtime startup free of network work in Milestone 1.
- Update architecture docs and add an ADR for every major architectural decision.
- Review dependency licenses, notices, locks and verification hashes together.
- Keep credentials/tokens/signing keys out of source, fixtures, logs and artifacts.
- Use semantic versioning and update the changelog for user-visible changes.
- Never describe untested OS builds, stubbed auth or future modules as implemented.
- Public repository setup and hosted CI must be verified before an actual release.
