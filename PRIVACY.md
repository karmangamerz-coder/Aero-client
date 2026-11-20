# Privacy

Milestone 1 sends no network requests at runtime and includes no telemetry, analytics,
advertising, tracking identifiers, crash uploading or Aero accounts. It collects no
authentication tokens. The launcher stores only theme and planned memory allocation
in `~/.aero-client/settings.properties` when the user explicitly saves. These settings
remain local; deleting the file resets them. No log file is created by this implementation.

Builds contact Gradle and Maven Central to fetch tools and test dependencies; those
services receive ordinary connection information such as IP address. GitHub receives
information users voluntarily submit in issues, PRs and CI. Their own policies apply.

Future Microsoft authentication and official game downloads will require connections
to Microsoft/Mojang services and legitimate ownership. Before implementation, this
document must specify endpoints, data flows, secure token retention and deletion.
Tokens must never be stored in plain settings files, committed, or uploaded to Aero.
There will be no mandatory Aero accounts or telemetry. Any future optional diagnostics
require explicit, revocable consent, data minimization and a documented retention policy.
