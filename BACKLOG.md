# Backlog and delivery conventions

**GitHub issues become the live queue as milestones open; GitHub milestones
are scoped releases.** This file maps programs and records working
conventions. [Mission](docs/mission.md) · [Delivery plan](docs/plan.md) ·
[Milestones](docs/milestones.md) · [Decision 0001](docs/decisions/0001-founding-decisions.md).

## Program map

| Product target | Stream (plan) | Milestones |
| --- | --- | --- |
| Original-format compatibility | W1 Formats (`rt3-kit`) | 1–4 |
| Reverse-engineering evidence | W2 RE (Ghidra + Wine oracle) | 1–5, 8, 13 |
| Native simulation and campaigns | W3 Simulation (`engine`) | 5–10 |
| Observer, then playable client | W4 Surfaces | 7, 9 |
| Persistence and save compatibility | W5 Persistence | 6, 8, 13 |
| MCP agent interface | W6 MCP (`rt3ctl mcp`) | 11–12 |
| Enhanced playing (QoL/analysis) | — | 14+ |
| Authoring tools | W7 | 15+ |

Shared mechanics (economy, track/trains, companies, objectives) are real
dependencies of every later stream; no client, MCP tool or enhancement
compensates for absent engine mechanics.

## Near-term selection

1. Milestone 0 (bootstrap) and milestone 1 (PK4 archives, PR #4, merge
   `ab827b5`) are complete; issues #1/#2 closed by evidence.
2. Milestones 1–3b are landed (PK4, typed data, map prelude + inventory,
   terrain decode/render). **Milestone 4 is released as `v0.5.0`** (issue #42,
   AGREE): original-save structural loading, two independently generated
   control saves, native read-back, source/company/player fields, a
   section/actor differential, and the native Windows companion decision
   (0004). Issues #30 and #43 are closed. Milestone 5 implementation is
   complete through PR #49 (claims C1–C5); the pre-existing fixture-path defect
   #50 is fixed. Next is the M5 milestone review; milestone 6 has not started.
3. The prior-art survey [#3](https://github.com/diegoami/rt3engine/issues/3) is
   closed by the 2026-11-25 web/forum survey recorded in
   [`docs/prior-art.md`](docs/prior-art.md); new candidates are added there as
   they are found. Issue #8 remains the open map reverse-engineering umbrella.

## Story contract

Before implementation work starts, a story names:

- Its milestone and the user/engineer route it serves.
- The admitted fixture (which real installation data it exercises) and the
  provenance of every format claim (Ghidra finding, oracle session, manual
  cross-check).
- Engine/API dependencies and what it deliberately excludes.
- Invalid/unsupported/missing-input behavior.
- Reproducibility effects (checkpoints, replay, evidence artifacts) and the
  acceptance check that closes it.
- Which claim level it establishes: parse, load, simulate, observe, human
  play, export, MCP tooling or fidelity.

A parse is not a simulation; a checkpoint is not an original `.sav` export;
a deterministic run is not a fidelity claim.

## Conventions

- Labels: `p0` urgent, `p1` next-candidate, `p2` later; `re` for
  reverse-engineering findings; `qol` for enhancement packages.
- Ordinary changes merge on independent AGREE per decision 0002; the owner
  launches milestone reviews, tags and human-acceptance gates.
- No calendar estimates; versions attach when scope freezes.
