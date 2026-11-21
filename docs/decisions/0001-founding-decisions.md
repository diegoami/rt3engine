# Decision 0001 — founding decisions

Date: 2026-11-21. Owner: diegoami. Status: decided.

Established at project founding (session of 2026-11-21) from the owner's
direction and four explicit owner answers.

## Decisions

1. **License: MIT.** Matches totengine; `LICENSE` at the root; original game
   assets are never distributed regardless of license.
2. **Headless-first; client route decided later.** Formats, simulation,
   observer and MCP come before any playable client. The rendering route —
   native 3D (wgpu) vs 2D-first — is decided by a reviewed design package at
   milestone 9's opening, not now.
3. **Browser is an observer surface only.** The playable client is native
   desktop. Browser play (wasm/webgpu) is a stretch goal, never a release
   gate — unlike totengine's both-client rule.
4. **MCP ships as laddered (milestone 11).** Stage 1 (headless read-only)
   comes after campaign breadth so the tools expose real runs; stages 2–3
   follow. The owner may pull stage 1 earlier by explicit decision.
5. **Workspace layout:** Rust workspace with `crates/rt3-kit` (format
   decoders), `crates/engine` (simulation), `crates/rt3ctl` (CLI, later MCP
   server). `rt3ui` and further crates are added when their milestone opens.
   Zero runtime dependencies at bootstrap; additions are justified per change.
6. **Determinism and fidelity baseline:** the engine is a fixed-tick, seeded
   simulation; acceptance is *comparable outcomes* against oracle evidence.
   Bit-exact original reproduction is a separate later program, not a
   prerequisite for any milestone.
7. **Resource policy:** `RT3_HOME` names the installation (default: the
   owner's GOG path, see [environment](../environment.md)); it is read-only;
   no original bytes are committed; evidence artifacts are small derived
   aggregates re-derivable from it.
8. **References:** Ghidra (static analysis of `RT3.exe`/`d8tx.dll`) and an
   isolated Wine oracle running the original are development references.
   Their findings are facts (documented freely); their code is never copied.

## Consequences

- Milestone 9 opens with a design package and an owner decision (see
  [milestones](../milestones.md) rule 3).
- Browser play claims never appear in acceptance gates.
- `docs/evidence/` artifacts carry only aggregate/derived metadata.
