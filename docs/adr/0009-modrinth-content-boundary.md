# ADR 0009: Explicit, verified Modrinth content

Status: accepted — 2026-11-24

## Decision

Modrinth discovery is read-only until the player presses **Install** for a selected
instance. Aero then requests a release matching that instance's exact Minecraft version
and loader, walks required dependencies with a bounded graph, downloads only from trusted
Modrinth endpoints, and verifies the published SHA-1 before an artifact becomes visible
under mods, resourcepacks, or shaderpacks.

Installed projects and their exact versions/checksums are recorded in content.json.
Disable is a reversible filename move and removal moves files to content-trash. Aero
does not silently update mods or infer consent from a search.

## Consequences

- A failed or cancelled transfer remains resumable and cannot publish an unverified file.
- Modpacks require a separate new-instance import flow; dropping a pack into the mods
  folder is intentionally rejected.
- Third-party licenses remain attached to their projects. Inclusion in a preset is not
  relicensing or endorsement.

