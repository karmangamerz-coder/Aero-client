# ADR 0013: Vista refinement, required setup and explicit auth outcomes

Date: 2026-11-27
Status: accepted

The user requested silver Vista-style chrome, no setup skip, clearer authentication and
working Modrinth browsing. Preserve Rust/Freya services and OS-vault security boundaries.

Use original static SVG folds, neutral silver surfaces, labeled navigation and a blue
inset Play panel. Native window controls stay native. No copied Microsoft branding,
continuous animation or rendering timer is added. Dark and flat Lite remain supported.

Setup is required once, with no defer command. Account authorization remains optional
during setup so a service outage cannot block local configuration; playing still requires
verified ownership. Persisted completion is independent of answer validity after an upgrade:
retired answer values reset to defaults without reopening setup or mutating instances.

OAuth callback acceptance is only one authentication stage. The callback explains the
remaining Xbox/Minecraft/vault checks; the app shows progress and persistent failures.
Progress events carry the attempt generation so cancelled attempts cannot update a newer
login. Accounts appear only after verification and vault save. Reported HTTP 403 application
rejection requires legitimate authorization for Aero's ID; no borrowed ID or bypass.

Discovery defaults to no version/loader restrictions. Project details omit the nonexistent
Vanilla loader filter. Installation still selects exact instance metadata, requires a
modded loader for mods and retains artifact verification. These choices favor useful
browsing without representing every discovered project as compatible with every instance.

Regression coverage includes retired completed setup records, navigation persistence and
Modrinth version URL semantics. Live service queries verify API availability but do not
replace installing content and launching Minecraft with an owning account.
