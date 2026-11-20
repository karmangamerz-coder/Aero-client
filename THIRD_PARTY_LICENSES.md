# Third-party inventory and redistribution

Original current Aero source: MIT. Third-party components keep their own terms;
MIT does not relicense Java, JavaFX, Forge, Fabric or Minecraft.

| Component | Version | License / use |
| --- | --- | --- |
| Gradle wrapper | 8.14.3 | Apache-2.0; build-only. License/NOTICE in third-party/gradle. |
| Gson | 2.13.2 | Apache-2.0; JSON parser. License in third-party/gson; source https://github.com/google/gson/tree/gson-parent-2.13.2. |
| OpenJFX base/graphics/controls | 21.0.13 | GPL-2.0 with Classpath Exception and component notices; runtime UI. Notices in third-party/javafx and JARs. Source https://github.com/openjdk/jfx21u/tree/21.0.13%2B1. |
| Eclipse Temurin / OpenJDK | Java 21 (CI resolves patch) | GPL-2.0 with Classpath Exception and bundled notices. jlink preserves legal/; record exact version/source in build info. Source https://github.com/adoptium/temurin21-binaries/releases. |
| JUnit Jupiter / BOM | 5.11.4 | EPL-2.0; test-only, https://github.com/junit-team/junit5/tree/r5.11.4. |
| JUnit Platform | 1.11.4 | EPL-2.0; test-only. |
| OpenTest4J | 1.3.0 | Apache-2.0; test-only, https://github.com/ota4j-team/opentest4j. |
| API Guardian | 1.1.2 | Apache-2.0; test-only, https://github.com/apiguardian-team/apiguardian. |
| Error Prone annotations | 2.41.0 | Apache-2.0; Gson annotation dependency, https://github.com/google/error-prone. Preserve embedded notices if bundled. |

Java/JavaFX Classpath Exceptions permit independent application linking; bundled runtime
components keep their licenses. Unmodified corresponding OpenJFX source and its release
tag must remain available with distributions, together with all component notices.
JavaFX graphics JPEG and Mesa notices are retained in third-party/javafx; Apache-2.0
text for Error Prone annotations is retained in third-party/error-prone.
No JavaFX web/media modules are used. Review native bundles before public binary release.

## User-requested game installations (not bundled in Git or Aero releases)
- Minecraft client/assets/libraries: official Mojang metadata; Minecraft is proprietary,
  https://www.minecraft.net/en-us/usage-guidelines. Do not redistribute game files.
- Game Java: Mojang runtime metadata chooses versions/components and retains all provided
  legal files. Vendor licenses apply; never redistribute these through the repository.
- Fabric Loader: stable version from official compatibility metadata; Apache-2.0,
  https://github.com/FabricMC/fabric-loader. ASM (BSD-3-Clause) and Mixin (MIT) retain their
  notices in downloaded artifacts. Fabric API/mods are not automatically installed.
- Forge 1.8.9 / 11.15.1.2318: official Forge Maven installer/universal; LGPL-2.1,
  https://github.com/MinecraftForge/MinecraftForge/tree/1.8.9. Loader/library artifacts
  retain embedded notices. No installer code is copied or executed.

Downloads are installed locally at the user's request. Installing a loader does not
relicense it or establish Minecraft ownership. Future mods need individual license,
compatibility, source and notice review. GitHub Actions are not application dependencies.
