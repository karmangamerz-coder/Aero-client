# 0008: First-launch setup and character rendering

- Status: accepted (2026-11-21)
- Applies to: `native/core/src/onboarding.rs`, `native/core/src/skin.rs`,
  `native/launcher/src/pages.rs`

## Context
M3-S requires a permanent, resumable first-launch experience using the same Aero design,
and M2.1 requires a large character preview that shows the player's real skin when signed
in, with an original fallback when signed out, on hardware as modest as a Ryzen 5 5500U
with integrated graphics.

## Decision
1. **Setup state is a separate record.** `onboarding.json` stores the completion flag,
   the current step and the answers (version, loader, memory, appearance, performance,
   automatic Java). Every Back/Next/Skip writes it atomically, so a crash or a close
   resumes at the same step. Completion hides the wizard permanently; Settings can reset
   it on purpose.
2. **The wizard never blocks on the network.** Sign-in is optional and runs as the same
   supervised task as the main window; installation is explicit and can be deferred, in
   which case only the instance is created. Hardware detection runs once on a blocking
   worker.
3. **Character rendering is CPU-only.** A skin (64×64 PNG, or legacy 64×32) is decoded
   once, a standing character is composed from its faces with a hint of side and top
   geometry, and the result is encoded as a PNG that Skia decodes and caches. There is no
   3D engine, no GPU skinning and no per-frame work; the image changes only when the
   account or skin changes. A procedural original Aero skin is used while signed out.
4. **No compression dependency.** The PNG encoder writes stored deflate blocks with correct
   CRCs/Adler-32, which keeps the renderer dependency-light; the composed image is small
   enough that the size cost is irrelevant.

## Consequences
- Setup is resumable and idempotent; running it again never duplicates the default instance
  (the create call reuses an existing instance with the same version and loader).
- The wizard's Minecraft step cannot yet choose a custom game folder: that belongs to M4
  instance management, and the interface says so rather than silently ignoring the field.
- Character artwork is intentionally a stylised composition rather than a textured 3D
  render; it is original, cheap and consistent across all three themes.
- Skin download failures fall back to the Aero skin and never block the interface.

## Alternatives considered
- Rendering the skin with Skia shaders or a 3D engine: heavier, GPU-dependent and prone to
  continuous rendering on low-end hardware.
- Shipping fixed artwork per account: cannot show real skins.
- Keeping setup answers in the general settings file: would mix transient wizard state with
  durable preferences and make resumption harder to reason about.
