# ADR 0004: M2 scope, MIT and JavaFX presentation

Status: accepted, 2026-11-20. Supersedes conflicting portions of ADRs 0001–0003.

Context: the owner's refinement request specifies MIT, JavaFX, Windows installation
in M2 and authentication in M3. The actual repository had a Swing shell, GPL policy and
no PROJECT_STATE. Pretending JavaFX already existed would hide a necessary UI change.

Decision: preserve the two modules, preference behavior and client library; replace
only the presentation entry point with JavaFX. Apply MIT to original current source as
directed; retain upstream notices and historical ADRs. Keep original Aero visual styling
in CSS/outline SVGs, with no raster/blur/render-loop dependency. M2 prepares files only;
game authentication and launch stay disabled until M3. No production setup installer yet.

Consequences: JavaFX platform dependencies and Gson are new reviewed runtime libraries.
The source runner needs dependency JARs. Development app images include JavaFX and a
jlink runtime. Existing old GPL releases are not retroactively removed from history.

Alternatives: continuing Swing contradicts the requested platform; recreating the repository
would lose working state. Implementing auth now would violate the M2-only boundary.
