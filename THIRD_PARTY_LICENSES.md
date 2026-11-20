# Third-party inventory and redistribution

Original Aero code: GPL-3.0-only. No third-party runtime libraries or Minecraft mods
are linked by Milestone 1. Minecraft itself is proprietary and is not included.

| Component | Version | License / source | Use and obligations |
| --- | --- | --- | --- |
| Gradle wrapper/build distribution | 8.14.3 | Apache-2.0; https://github.com/gradle/gradle/tree/v8.14.3 | Build tool; wrapper license and upstream NOTICE preserved under third-party/gradle. Distribution retains its own notices. |
| JUnit Jupiter / BOM | 5.11.4 | EPL-2.0; https://github.com/junit-team/junit5/tree/r5.11.4 | Test-only; not shipped in app images. |
| JUnit Platform | 1.11.4 | EPL-2.0; same JUnit source | Test-only transitive dependency. |
| OpenTest4J | 1.3.0 | Apache-2.0; https://github.com/ota4j-team/opentest4j | Test-only transitive dependency. |
| API Guardian | 1.1.2 | Apache-2.0; https://github.com/apiguardian-team/apiguardian | Test-only transitive dependency. |
| Eclipse Temurin / OpenJDK | Java 21 (CI resolves current patch) | GPL-2.0 with Classpath Exception plus bundled notices; https://adoptium.net/ and https://github.com/openjdk/jdk21u | jpackage bundles a reduced runtime. Preserve runtime/legal; record exact JDK in release build info; provide corresponding runtime source link/version with releases. |

Build/test tools are separate from the distributed Aero application; their presence
does not relicense Aero. Apache-2.0 wrapper redistribution retains license/NOTICE.
The Java Classpath Exception permits application linking; included runtime components
retain their own licenses. Exact bundled runtime license files are authoritative.
GitHub Actions run on hosted infrastructure and are not included in app binaries;
their pinned repositories retain their respective licenses.

Before adding any runtime library or mod, maintainers must review actual license text,
GPL-3.0-only compatibility, modifications, attribution, source availability and
redistribution requirements. If incompatible or unclear, do not bundle it. Selecting
a Minecraft mod loader and addressing game linkage/distribution remain Milestone 3
release gates. No blanket compatibility claim is made for future dependencies.

Sources: https://www.gnu.org/licenses/gpl-3.0.html,
https://www.minecraft.net/en-us/usage-guidelines,
https://openjdk.org/legal/gplv2+ce.html.
