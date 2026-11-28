# Milestones

Owner-approved ladder, 2026-11-21. [Mission](mission.md) ·
[Delivery plan](plan.md) · [Backlog](../BACKLOG.md). Versions attach when a
milestone's scope freezes; no calendar claims. A milestone closes only on its
gate — including owner acceptance where named — plus the release process in
`PRINCIPLES.md` for tagged releases.

| Milestone | Theme | Key tasks | Gate |
| --- | --- | --- | --- |
| 0 | **Bootstrap and environment** | Harness adoption; workspace skeleton (`rt3-kit`, `engine`, `rt3ctl`); `rt3ctl inventory` + evidence v1; planning docs; CI | Gates green; owner accepts structure |
| 1 | **PK4 asset archives** | Ghidra-assisted format RE on `RT3.exe` file IO; Rust reader/list/extract for `Data/{2D,3D,Sound}/*.PK4`; DDS/TGA identification inside packs | Every shipped PK4 listed and entries verifiable; evidence artifact; no assets committed |
| 2 | **Game data tables and text** | Typed model for `.cty/.bty/.bca/.car/.lco` and txt tables; `RT3.lng`; `Configuration/*.cfg` decode | Typed data loads from the real installation; cross-check against the manual |
| 3 | **Maps — prelude and inventory** | `.gmp` prelude decode (version ladder, record, scenario text, aux records); map inventory CLI + evidence. Re-scoped by owner decision 2026-11-23 (was terrain+render) | Landed: PR #10, merge 942aab5 |
| 3b | **Terrain decode and render** | `.gmp` terrain encoding (quadtree/LOD class) via static RE + Wine-oracle dynamic calibration; objects/cities; 2D render | Every shipped v9701 map renders with recognizable geography |
| 4 | **Save-game decode** | Wine oracle produces saves at known dates; save-diff structure walk; read-only loader; section inventory | Save set loads; field provenance documented |
| 5 | **Headless economy kernel** | Fixed-tick loop; cargo supply/demand/prices on loaded maps; seeded determinism; month-by-month dumps; oracle comparisons where measurable | Deterministic multi-year run on ≥2 maps; divergence documented |
| 6 | **Trains, track and commands** | Track graph; movement/physics; consists and schedules; headless player commands with order log and checkpoints; AI competitor v0 | Scripted company reaches cash flow; replay exact |
| 7 | **Observer** | Browser observer over live headless runs: map, trains, economy panels; run controls | Owner watches a full campaign run |
| 8 | **Original-save entry** | Boot a bounded original `.sav` into the native sim; capability/gaps report | Bounded save continues; differences visible and documented |
| 9 | **Playable client, first slice** | Rendering-route design package and **owner decision** (decision 0001); interactive track laying and trains on an original map; checkpoint save/resume | Owner plays and saves/resumes on the actual client |
| 10 | **Campaign breadth** | Stations/buildings; industry upgrades; stock market; competitor depth; campaign objectives | One shipped scenario playable end-to-end |
| 11 | **MCP stage 1 — headless read-only** | `rt3ctl mcp`: installation/map/save inspection, headless campaign runs, observer queries; bounded roots; read-only filesystem; byte-identical to CLI | An external assistant inspects and runs a campaign via MCP |
| 12 | **MCP stages 2–3** | Client-attached advisory (filtered view, pending decisions); headless experimental agent play (validated, recorded orders) | Advisory survives reconnect; agent runs replay without the agent |
| 13 | **Original save write** | Export `.sav` from native state | Original (oracle) loads/re-saves our output; differential evidence retained |
| 14 | **Enhanced playing package** | Opt-in QoL: overlays (cargo flow, profitability), schedule aids, presentation/settings, recording/annotations | Owner acceptance; faithful defaults preserved |
| 15+ | **Authoring and beyond** | Map/scenario inspection → lint/dry-run → editing; AI plugins/tournaments; original-fidelity program | scoped per package when opened |

## Milestone 5 — headless economy kernel (scope fixed 2026-11-25)

**Promise:** a headless economy kernel over loaded original maps — a fixed-tick,
seeded simulation of cities, placed buildings and cargo supply, demand and
prices under a declared industry model, with monthly dumps, deterministic
replay, and divergence from measurable original state documented.

Claims, each naming the check that proves it:

- **C1 — Map-grounded world.** The kernel loads two shipped maps with their
  city records, placed building records, the typed cargo catalogue and the
  coarse terrain heightfield, reporting counts identical to the structural
  parsers. Check: `cargo test -p engine` with `RT3_HOME` on `British Isles.gmp`
  and `California Island.gmp`, asserting the loaded city and building record
  counts against `rt3-kit`'s collections, the 41-type cargo catalogue against
  `typed-tables-v1.json` (including a named type such as `Alcohol`), and the
  terrain dimensions and height range against `terrain-scan-v1.json`. Building
  and city *semantics* (industry types, production, demand links) are not
  claimed here; only parser-proven structural records.
- **C2 — Fixed-tick supply, demand and prices.** One simulated month is a
  declared tick count; each tick applies documented production, consumption and
  price rules, with the stock accounting identity and documented bounds
  asserted. Check: engine unit tests on a committed synthetic scenario for the
  identity, bounds and a golden monthly dump hash, plus the rule table in
  `docs/evidence/economy-v1.md`.
- **C3 — Seeded determinism.** The same map, seed, configuration and tick count
  produce byte-identical monthly dumps, and a different seed changes them.
  Check: the engine golden-hash replay test asserts both the identical-dump and
  changed-seed cases; `rt3ctl sim run` twice on a shipped map with the same
  inputs is compared by SHA-256, and a changed seed changes the digest.
- **C4 — Multi-year runs on two maps.** `rt3ctl sim run` completes at least one
  in-game decade on two shipped maps, emitting one JSON line per month and an
  explicit stop reason. Check: install-backed integration runs of a decade on
  both maps, each run repeated and the two dumps compared byte-for-byte, with
  the dump schema validated and the stop reason asserted.
- **C5 — Measurable divergence documented.** The kernel's British Isles start
  state is compared with the controlled original save at January 1829 (city
  collection, placed building collection, cargo catalogue); every difference,
  and every economy rule without original-constant provenance, is listed in
  `docs/evidence/economy-v1.md`. Check: `rt3ctl sim compare` reproduces the
  comparison, and the evidence doc enumerates each divergence with its cause.

**Not in this release:** track, trains, consists and schedules (milestone 6);
player or AI company operations, company finance beyond a starting balance and
maintenance; observer (milestone 7); original-save entry (milestone 8); client
(milestone 9); buildable stations and buildings, industry upgrades, stock
market, bonds, competitor depth and campaign objectives (milestone 10); MCP
surfaces (milestones 11–12); original save writing (milestone 13); bit-exact
original fidelity; original economy constants that reverse engineering has not
recovered.

**Gate:** C1–C5 met on the candidate, including the deterministic multi-year
run on two maps and the documented divergence.

## Milestone 6 — trains, track and commands (scope fixed 2026-11-27; opened by the owner 2026-11-27)

**Promise:** a headless rail layer over loaded original maps — the decoded
track and consist structures feed a declared planar track graph; a consist
moves on fixed ticks under a documented movement model; stations, consists and
schedules are executed; headless player commands carry an order log and
checkpoints; a scripted company reaches positive cash flow while a declared
AI competitor v0 operates on the same scenario; replay is byte-identical and
every declared rule and divergence is documented.

Claims, each naming the check that proves it:

- **C1 — Track graph decoder.** Over the five rail readers landed as prior
  baseline (PR #63, not claimed here), decode the track piece-origin field and
  build a planar graph with a declared connectivity rule: split where the
  origin step exceeds 100 units, then join run ends within one piece length.
  The claim is the graph decoder and its measured origin field; it does **not**
  claim a station-to-path link, and matching the path ends to station records
  remains open in #62. Check: graph tests on `RT3_TRACK_SAVE` and a committed
  synthetic graph, with the measured origins and the declared rule in
  `docs/evidence/rail-v1.md`. Connectivity is a declared model, not a
  recovered game semantic; the piece orientation/endpoint fields remain open.
- **C2 — Consist movement on the graph.** A consist moves along the graph at a
  documented speed and acceleration model on fixed ticks, with positions
  bounded by the path and a golden state hash. The same inputs replay
  byte-identically, and a different seed changes the state only where the
  model declares randomness. Check: engine tests on a committed synthetic
  graph and a private-fixture run that emits one state line per tick.
- **C3 — Stations, consists and schedules.** A consist is assembled from the
  typed engine and car catalogue, a schedule is built between two station-DB
  records, and the run stops at each station in order. Check: engine tests on
  the synthetic graph for stop order and a golden schedule dump, plus the
  fixture run from C2.
- **C4 — Headless commands, order log and checkpoints.** `rt3ctl sim` gains
  commands to build a declared track between two station records, buy a train
  and assign a schedule. Every command is recorded in an order log with
  checkpoints, and replaying the log reproduces a byte-identical state. Check:
  CLI integration tests for the command path, the order log and checkpoint
  resume, and a replay comparison by SHA-256.
- **C5 — Scripted company cash flow.** On the private fixture's decoded track
  (not a claimed station link), a scripted company runs the installation
  catalogue's `PlanetL` consist (the first four typed car records) under the
  declared economy, earns revenue and pays operating costs, and reaches
  positive cash flow within a bounded run. The revenue and cost rules and
  every divergence from measurable original state are listed in
  `docs/evidence/rail-economy-v1.md`. Check: an install-backed scripted run
  whose `rt3ctl` output names the consist, plus the cash figures and the
  evidence table. Corrected 2026-11-28: the earlier wording said the fixture's
  Planet 2-2-0 consist; the fixture's car-to-type assignment is not decoded,
  so the run uses the declared catalogue consist.
- **C6 — AI competitor v0.** One declared competitor company operates in the
  C5 scenario under a fixed scripted policy (the declared v0 rule), with
  deterministic behavior and no recovered original AI. Check: the joint C5
  run with the competitor active, a replay hash, and the declared policy and
  its divergences in `docs/evidence/rail-economy-v1.md`.

**Not in this release:** original track geometry and physics constants,
original train performance or schedule data, AI competitor behavior beyond the
declared v0 policy, signalling, company finance beyond the scripted model,
buildable stations or industry changes, the observer (milestone 7),
original-save entry (milestone 8), the client (milestone 9), MCP surfaces
(milestones 11–12), original save writing (milestone 13), campaign breadth,
bit-exact original fidelity, and any original constant reverse engineering has
not recovered.

**Gate:** C1–C6 met on the candidate, including the scripted company's positive
cash flow in the joint run with the AI competitor v0 active, and byte-identical
command-log replay.

## Selection and completion rules

1. Work proceeds milestone by milestone in the order above unless the owner
   re-orders; a milestone opens only when its predecessor's gate is met or
   the owner waives it explicitly.
2. Versions are assigned when a milestone's scope freezes; the tag points at
   the exact reviewed SHA. The owner launches every milestone review.
3. Milestone 9's rendering-route decision (native 3D vs 2D-first; browser
   play as stretch) is an explicit owner gate recorded as a decision; the
   engine stays headless-first until then.
4. MCP stage 1 (milestone 11) may be pulled earlier by owner decision; until
   then it stays after campaign breadth so tools expose real runs.
5. Enhancements (14+) never block or replace faithful-baseline work, and are
   always opt-in.
6. Umbrella programs (authoring, fidelity) stay open across slices; a
   bounded story closes on its own evidence.
