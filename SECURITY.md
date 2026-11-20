# Security policy

The latest 0.1.x source release receives fixes. Milestone 1 is a development foundation,
not a supported game-authentication implementation. No security guarantees are implied.

When the public GitHub repository is available, use Security > Report a vulnerability
(private vulnerability reporting). Maintainers must enable this before public release.
If unavailable, open a minimal issue requesting a private contact without exploit
details, tokens or personal data. No monitored email address is claimed at this stage.

Include affected versions, reproduction steps, impact and a minimal sanitized example
in the private report. Coordinated disclosure is preferred; maintainers will acknowledge
and investigate as capacity permits, with no promised response SLA.

Threat boundaries: untrusted manifests/downloads/mods; authentication redirects and
tokens; local filesystem paths; CI dependencies and release credentials. Future game
downloads must validate official origins, redirect destinations, paths and integrity.
Never log tokens or pass refresh tokens on command lines. Authentication must fail
closed when ownership cannot be established. Mods execute code and require explicit
user choice; no silent remote code or module installation.

CI uses read-only permissions except the dependency graph job. Release workflows
produce artifacts without automatic publication. Store any future signing keys in
protected CI secrets or external signing services, never in this repository. Rotate
accidentally exposed secrets immediately; deleting a file does not revoke a secret.
