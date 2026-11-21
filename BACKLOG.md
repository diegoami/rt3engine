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

1. Milestone 0 (bootstrap) is complete and accepted; CI is green.
2. Milestone 1 (PK4 asset archives) is **implemented on branch `m1/pk4-re`**
   pending review/merge: format decoded from `RT3.exe` (see
   `docs/evidence/pk4-format-v1.md`), reader in `rt3-kit::pk4`, `rt3ctl pk4
   list` with aggregate evidence. Prior-art survey [#3](https://github.com/diegoami/rt3engine/issues/3)
   remains open.
3. Next: milestone 2 — typed data tables (`.cty/.bty/.bca/.car/.lco`),
   `RT3.lng`, `Configuration/*.cfg`.

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
- The owner merges every pull request; the owner launches milestone reviews.
- No calendar estimates; versions attach when scope freezes.
