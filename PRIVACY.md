# Privacy

Aero has no accounts of its own, telemetry, analytics, advertising, tracking IDs or automatic
crash uploads. Startup reads local settings, instance manifests, cached skins and account
metadata; it does not poll news or refresh remote metadata.

When you explicitly sign in, the system browser connects to Microsoft. Aero uses HTTPS
with login.microsoftonline.com, user.auth.xboxlive.com, xsts.auth.xboxlive.com and
api.minecraftservices.com to obtain your session, verify Java ownership and fetch the
Minecraft profile. Public skin textures may be fetched from textures.minecraft.net.
Those services receive your network address and information necessary for the request.
A short-lived callback listener binds IPv4 loopback only; the browser returns to
http://localhost:<port>/callback. Callback codes, state and PKCE verifiers stay in memory.

Refresh tokens live in the operating system credential vault: Windows Credential Manager,
macOS Keychain or Linux Secret Service. The account metadata file contains username, UUID,
skin URL, Application ID and active-account selection, with no session secrets. Access
tokens remain in memory. The vault service is scoped to the data directory, so isolated
developer runs do not share normal accounts. Vault errors never trigger plaintext storage.
Sessions refresh on demand before launching, not through background polling.

Sign out cancels pending sign-in and deletes the active account's local vault credential.
Remove account deletes the chosen saved account. Browser sign-in cookies and sessions on
other devices are unaffected. Old DPAPI account files require a new sign-in and are removed
after a successful vault save. Logs use fixed auth errors and redact known session tokens.

Requested downloads use Mojang/Minecraft, Fabric, Forge, Maven Central and Modrinth services.
Requested update checks use GitHub. Services receive your IP address, request path and an
AeroClient/2.0.0 user agent. No Minecraft files are redistributed by Aero.
Local game files, Java runtimes, preferences, caches and instance saves remain under the
configured data directory. Back up saves before manually removing an instance or directory.

Developer builds contact Cargo, Gradle and Maven registries. GitHub receives source, PR and
CI data submitted there. Credentials, signing keys and tokens must never be committed.

