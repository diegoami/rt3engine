# RT3Engine delivery plan

Owner direction, 2026-11-21; structure follows the sibling project
[totengine](https://github.com/diegoami/totengine).
[Mission](mission.md) → this plan → [milestones](milestones.md) →
[backlog](../BACKLOG.md). Owner decisions live in
[decisions/](decisions/); start with [0001](decisions/0001-founding-decisions.md).

## Principles

1. **One authoritative native engine.** Clients, observer, headless runners
   and the MCP interface are surfaces over the same Rust simulation and
   versioned persistence. The original executable is an oracle and a
   reverse-engineering reference, not a runtime requirement.
2. **Faithful baseline, explicit enhancements.** Original behavior is the
   default; QoL, overlays and automation are opt-in settings with their own
   contracts, never silent rule changes.
3. **Every format claim needs evidence.** Findings live under
   `docs/evidence/` as small derived artifacts re-derivable from `RT3_HOME`
   (`make evidence-check`). No folklore without verification against real
   data; Ghidra/wine-oracle findings record their method.
4. **Keep original resources private and read-only.** Configurable loading
   behind `RT3_HOME`; never original paths in library code; never original
   bytes in Git; preserve the owner's installation.
5. **Determinism by construction.** The original is real-time; the engine is
   a fixed-tick, seeded simulation. Comparable outcomes against oracle
   evidence is the acceptance bar; bit-exact original parity is a separate
   program (decision 0001).
6. **MCP is staged, and acting agents are recorded.** Read-only headless
   tools first; client-attached advisory access filtered to the human's view;
   agent play submits validated orders and replays without the agent.

## Baseline

Bootstrap (milestone 0, 2026-11-21): harness process adopted (r5.1), Rust
workspace skeleton (`rt3-kit`, `engine`, `rt3ctl`), metadata-only
installation inventory (`rt3ctl inventory`) and its evidence artifact,
planning documents, CI. No format decoding, simulation or client exists yet.

| Capability | Available | Remaining target |
| --- | --- | --- |
| Formats | Aggregate install inventory | PK4 archives, typed data, config, language, maps, saves (milestones 1–4) |
| Simulation | — | Fixed-tick kernel, economy, trains, companies, campaigns (milestones 5–10) |
| Surfaces | CLI | Observer, then playable client after its route decision (milestones 7, 9) |
| Persistence | — | Native checkpoints/replay; original `.sav` read, then write (milestones 6, 8, 13) |
| MCP | — | Staged interface (milestones 11–12) |
| Enhancements | — | QoL/analysis packages after the faithful baseline (milestone 14+) |

## Workstreams

| Stream | Deliverables and acceptance | Tracking |
| --- | --- | --- |
| W1 Formats (`rt3-kit`) | PK4 reader/extractor; typed-data model (`.cty/.bty/.bca/.car/.lco`, txt tables); `RT3.lng`; `Configuration/*.cfg`; `.gmp` + `GrayscaleMaps`; `.sav`. Gate per family: parse real data, retain evidence, load into typed structures | [milestones](milestones.md) 1–4 |
| W2 Reverse engineering | Ghidra project on `RT3.exe`/`d8tx.dll`; file IO, RNG, economy constants, save layout; Wine oracle with scripted saves at known dates (save-diff method) | milestones 1–5, 8, 13 |
| W3 Simulation (`engine`) | Fixed-tick loop; cargo economy; cities/industries; track and trains; stations/buildings; maintenance; stock market; competitor AI; campaign objectives. Gates: determinism, oracle comparability where measurable | milestones 5–10 |
| W4 Surfaces | Observer (browser) over live headless runs; playable client **after** the route decision package (decision 0001); browser play is a stretch goal, not a gate | milestones 7, 9 |
| W5 Persistence | Native checkpoints and order logs (exact replay); original `.sav` read (bounded entry) and later write (original round-trip) | milestones 6, 8, 13 |
| W6 MCP (`rt3ctl mcp`) | Stage 1 headless read-only (inspection, campaign runs, observer queries — byte-identical to CLI, bounded roots, read-only filesystem); stage 2 client-attached advisory; stage 3 headless experimental agent play | milestones 11–12 |
| W7 Authoring | Map/scenario inspection → lint/dry-run → editing and creation assistance | milestone 15+ |
| W8 Process | CI, evidence corpus, prior-art audit, decisions, releases | continuous |

## Delivery order and gates

1. **Formats before simulation** (milestones 1–4): each format family lands
   with its evidence artifact and typed loader. PK4 first (it gates artwork
   and sound for every later surface).
2. **Simulation before surfaces** (5–6): the headless kernel and command
   layer with deterministic replay; no client work before this.
3. **Observer before client** (7): the browser observer exercises the
   engine and assets without the client-route decision.
4. **Client route, then playable slices** (9): the owner decides the
   rendering route (native 3D vs 2D-first) on a reviewed design package;
   headless-first until then (decision 0001).
5. **Compatibility depth** (8, 13): bounded original-save entry, then
   original-save write with round-trip evidence.
6. **MCP after campaigns, QoL last** (11–12, 14+): the interface stages on
   the milestone ladder; enhancements are opt-in packages on the faithful
   baseline.

## Acceptance and evidence

| Claim | Required check |
| --- | --- |
| Format compatibility | Re-derivable evidence from `RT3_HOME`; declared coverage; originals outside Git |
| Simulation behavior | Deterministic replay from checkpoints; oracle comparisons where measurable, divergence documented |
| Observer | Live/replay agreement; seeking never executes gameplay |
| Playable client | Real input routes on the actual client; checkpoint continuation; owner acceptance |
| Save compatibility | Original loads/re-saves our output (oracle) with retained differential evidence; native checkpoints never stand in for this gate |
| MCP | Tool results byte-identical to CLI; advisory mode cannot mutate; agent runs replay without the agent |
| Release | Coherent scope, gates green, applicable human acceptance, owner-launched independent review; tag the exact reviewed SHA |

No calendar estimates or pre-assigned version numbers; versions attach when a
milestone's scope freezes.
