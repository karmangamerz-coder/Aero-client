# Releasing and verification

Use SemVer `MAJOR.MINOR.PATCH`; while below 1.0, minor releases may change APIs and
patch releases fix compatible behavior. This initial version is 0.1.0. The current
native packager accepts stable numeric versions only; prerelease packaging needs an ADR.

1. Publish all source to a public GitHub repository. Enable dependency graph,
   Dependabot alerts, private vulnerability reporting and protected main/release tags.
   Set up private moderation contact and review required CI checks.
2. Review licenses, notices, dependency audit, changelog and ADRs. Ensure no credentials
   or proprietary game files appear in source or artifacts.
3. Set `gradle.properties` version and CHANGELOG; commit and review. Tag `vX.Y.Z`
   matching that version. Never move a published tag.
4. GitHub Actions builds/tests and packages all three OSes; retrieve `aero-*` artifacts.
   Verify every build passed. Test the native launcher on each OS before release.
5. Each bundle has SHA256SUMS.txt, binary/source JARs, a complete original-source archive
   and native app image. Keep OS bundles/checksum manifests in distinct directories
   to avoid duplicate filenames. Verify with `sha256sum -c SHA256SUMS.txt` on Linux,
   `shasum -a 256` on macOS or `Get-FileHash -Algorithm SHA256` on Windows.
6. Publish reviewed artifacts with a public source/tag link and exact Java runtime
   source release link matching BUILD-INFO.txt. Preserve runtime/legal and all notices.
   Checksums detect corruption; they are not publisher authentication/signatures.

The workflow intentionally generates artifacts without creating a public release.
Signing/notarization is not configured. Future signing must use protected external
secrets and a reviewed workflow, never repository files or PR-accessible secrets.

## Reproducibility

JARs use fixed timestamps and stable ordering. CI rebuilds and compares their hashes
using the same checkout/toolchain. For independent verification use the same commit,
JDK patch/vendor, Gradle distribution and locked/verified dependencies on a clean host;
compare `*/build/libs/*.jar` hashes. Builds emit no build time or host path into JARs.

Native app images, archive metadata, source ZIP commit timestamps and changing hosted
runner/JDK patches mean entire release bundles are not promised byte-identical.
BUILD-INFO records the native environment. Tool updates need review; the JDK 21 patch
floats in CI to receive security fixes. Keep the wrapper distribution checksum and
verification metadata under source control. OSV auditing fails on network errors and
known findings but is not a complete security audit or a license compatibility check.

References: https://docs.gradle.org/current/userguide/dependency_verification.html,
https://docs.gradle.org/current/userguide/dependency_locking.html,
https://docs.oracle.com/en/java/javase/21/docs/specs/man/jpackage.html,
https://google.github.io/osv.dev/api/.
