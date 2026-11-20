# Privacy

No Aero accounts, telemetry, analytics, advertising, tracking IDs or crash uploading.
M2 implements no authentication and stores no access/refresh tokens. Startup reads local
settings/instance manifests only; it does not refresh metadata or start downloads.

When you click Refresh, Install, Retry or Repair, Aero connects over HTTPS to the
following services as needed. They receive your IP address, requested version/artifact
paths and the `AeroClient/0.2.0` user agent under their own policies:

- Mojang/Minecraft: piston-meta.mojang.com, piston-data.mojang.com,
  launchermeta.mojang.com, launcher.mojang.com, libraries.minecraft.net,
  resources.download.minecraft.net.
- Fabric (if selected): meta.fabricmc.net, maven.fabricmc.net.
- Forge (if selected): maven.minecraftforge.net, files.minecraftforge.net.
- Maven Central (allowlisted dependency source): repo.maven.apache.org.

Local storage: `~/.aero-client/settings.properties` holds theme/memory preferences;
`custom-java.txt` holds an optional executable path. `data/` holds cached metadata,
downloaded files/partial files, Java runtimes and UUID instances with installation status.
These records are not uploaded. No automatic retention/deletion runs. Remove unneeded
instances or caches manually only while Aero is closed; protect your saves first.

Only an explicitly selected or detected local Java executable is probed for its version.
Managed Java files are checksum-verified before execution. Runtime probe output is a
temporary local file removed after the probe. No game process is launched in M2.

Developer builds contact Gradle/Maven Central; GitHub receives source/PR/CI information
voluntarily submitted there. M3 must document Microsoft endpoints, token storage,
refresh/logout and deletion before auth is implemented. Never put tokens in settings,
repository files, logs or diagnostics. Future diagnostics require explicit revocable consent.
