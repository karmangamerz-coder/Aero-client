# Third-party inventory and redistribution

Original Aero source: GPL-3.0-only. Third-party components keep their own terms;
GPL-3.0-only does not relicense Java, JavaFX, Forge, Fabric, mods or Minecraft.

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

## Native Rust launcher dependencies (Cargo, GPLv3-compatible)

| Component | Version | License / use |
| --- | --- | --- |
| Freya | 0.5.3 | MIT; native egui-free UI toolkit over Skia (https://github.com/marc2332/freya). |
| Skia bindings (freya-skia-safe) | 0.104.0 | MIT/BSD-3-Clause; 2D rendering used by Freya. |
| Tokio | 1.54 | MIT; async runtime, process supervision. |
| Reqwest | 0.12 | MIT/Apache-2.0; HTTP client. TLS is rustls (below), not OpenSSL. |
| rustls / ring / webpki-roots | as locked | MIT/Apache-2.0/ISC; TLS with bundled web PKI roots (ADR 0006). |
| Serde / serde_json | 1.x | MIT/Apache-2.0; manifest serialisation. |
| PNG (`png`) | 0.17 | MIT/Apache-2.0; decodes Minecraft skin textures. |
| sysinfo | 0.33 | MIT; read-only CPU/memory/disk/OS detection for setup. |
| zip, sha1, hex, uuid, fs2, futures-util, regex, bytes, anyhow | as locked | Permissive licenses; downloads, integrity and hashing. |
| windows-sys | 0.61 | MIT/Apache-2.0; DPAPI token sealing and file-system calls on Windows. |
| oauth2 | 5.0.0 | MIT/Apache-2.0; PKCE, OAuth code exchange and refresh (https://github.com/ramosbugs/oauth2-rs). |
| minecraft-msa-auth | 0.5.0 | MIT/Apache-2.0; Xbox/XSTS/Minecraft exchange (https://github.com/minecraft-rs/minecraft-msa-auth). |
| keyring | 3.6.3 | MIT/Apache-2.0; native OS credential vault (https://github.com/hwchen/keyring-rs). |
| webbrowser | 1.2.7 | MIT/Apache-2.0; open system browser without shell string construction (https://github.com/amodm/webbrowser-rs). |

Rust dependency licenses are audited in CI and must be re-reviewed with cargo-deny or
cargo-license before a stable binary release. Their notices ship with the installer. Only official
Microsoft, Mojang, FabricMC, Forge, Modrinth and GitHub endpoints are contacted.
