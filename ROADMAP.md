# Roadmap

Milestone 1 scope was defined at repository initialization; it is a foundation,
not a claim of Minecraft gameplay compatibility. Future dates are intentionally unset.

## Milestone 1 — 0.1.0: open foundation
- [x] Original public-ready source, GPL license and contribution/security/privacy policies.
- [x] Architecture and ADRs; independent launcher/client build targets.
- [x] Offline desktop shell with validated local settings; client customization model.
- [x] Unit tests and pinned, verified build inputs.
- [x] CI definitions for three operating systems, native images and release checksums.
- [ ] Publish repository publicly and enable GitHub security/community settings.
- [ ] Observe hosted CI on all target OSes before distributing a release.

## Milestone 2 — authenticated official game installation
- Register an approved public Microsoft OAuth application; no embedded client secret.
- Document and implement the permitted authentication flow, entitlement/profile
  checks, expiry handling, cancellation, logout and secure token storage.
- Download manifests, libraries/assets and game files from official services only;
  validate URLs, redirects, integrity, paths, interrupted downloads and disk limits.
- Require verified ownership before download/launch; no offline bypass.
- Integration tests use isolated fakes; no real credentials in fixtures or CI.

## Milestone 3 — original client integration
- Select Minecraft version and compatible open-source mod loader through an ADR.
- Review GPL compatibility and Minecraft linkage/distribution before shipping mods.
- Implement original FPS/HUD and accessibility customization using public APIs.
- Measure startup time, frame-time overhead and memory against a documented baseline.

## Milestone 4 — hardening and community release
- Accessible UI, applied themes, profiles and explicit mod permission boundaries.
- Audit authentication, download handling and license compliance.
- Signed/notarized release pipeline where maintainers can support it.
- Independent reproducibility checks, contributor documentation and community governance.
