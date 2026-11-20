# ADR 0002: Privacy and legitimate ownership

Status: accepted, 2026-11-20.

Context: users require privacy and legitimate Minecraft ownership without Aero accounts.

Decision: no runtime networking or telemetry in Milestone 1. Disable play until real
Microsoft authentication, entitlement validation and official installation exist.
Persist only non-sensitive settings locally. Future auth uses an approved public-client
flow and OS secure token storage; never embed a confidential client secret.

Consequences: foundation cannot play Minecraft, including offline. No mock auth path
can accidentally bypass ownership. Future endpoint details, refresh behavior and secure
storage require an additional ADR and threat review before implementation.

Alternatives: Aero account service creates unnecessary data custody; offline username
authentication does not establish ownership; plain-text token files expose credentials.
