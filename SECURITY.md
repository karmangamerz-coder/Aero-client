# Security policy

The latest 2.0.x source release receives fixes. Live authentication still requires Aero's
own registered, approved Microsoft Application ID; no successful owning-account test is
claimed without evidence.

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

CI uses read-only permissions except explicitly authorized tagged-release publication.
Release workflows generate SHA-256 manifests and publish tagged artifacts. Store signing keys in
protected CI secrets or external signing services, never in this repository. Rotate
accidentally exposed secrets immediately; deleting a file does not revoke a secret.

Authentication uses browser authorization-code PKCE with a short-lived loopback callback,
random state, S256 and no client secret. Microsoft/Xbox family policy and Minecraft ownership
checks fail closed. Refresh tokens use the OS vault with no plaintext fallback. Local logout
deletes the selected credential; revoke Microsoft account consent to invalidate access
across devices. Never include callback URLs, authorization codes, verifier/state values or
credential-vault exports in a public report. See README.md and ADR 0012.
