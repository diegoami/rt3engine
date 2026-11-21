# RT3Engine mission

Owner direction, 2026-11-21. [Delivery plan](plan.md) · [Milestones](milestones.md) ·
[Decision 0001](decisions/0001-founding-decisions.md) · [Backlog](../BACKLOG.md).

**Build an open, native stand-in for Railroad Tycoon 3 that plays the game from
the user's original installation assets**, for play, observation, headless
experiments and AI-assisted (MCP) sessions. The original executable is never
needed at runtime; no proprietary assets are distributed.

## Product targets

| # | Target | Intended experience | Completion evidence |
| --- | --- | --- | --- |
| 1 | Playable stand-in | A native engine and client that play original maps with original assets, faithful to the original's rules with optional enhancements | Sustained human play of a shipped scenario; save/resume via native checkpoints |
| 2 | Original-format compatibility | Read and use the installation's data: `PK4` asset archives, `.gmp` maps, typed data (cargo/buildings/engines), language, configuration, saves | A format ladder per family — parse → load → simulate → (saves only) original round-trip |
| 3 | Headless campaigns | Deterministic all-AI company runs, parameter sweeps, comparable reports | Reproducible run manifests, outcome/event traces, explicit stop reasons |
| 4 | Observation and recording | Watch live and recorded runs; inspect economy, trains and events | Observer agrees with the headless run; recorded seeking never executes |
| 5 | MCP agent interface | AI assistants inspect installations, advise attached sessions and (staged) play headless campaigns | Headless read-only tools byte-identical to CLI; advisory attach filtered; agent runs replay without the agent |
| 6 | Enhanced playing | Optional QoL and analysis: overlays, schedule aids, automation, modern presentation, modding hooks | Explicit opt-in settings with separate contracts; faithful behavior remains available |

These are targets, not claims. Parsing a file, rendering a map, running
autoplay and completing a human scenario establish different levels of
support; current capability belongs to the [delivery plan](plan.md).

## Compatibility and resources

The main experience requires the user's original Railroad Tycoon 3
installation (GOG release verified; `RT3_HOME`, see
[environment](environment.md)). The installation is treated **read-only**;
ship no proprietary artwork, maps, saves, sounds or scenarios. Format and
behavior claims are measured against the installation and the original
executable via static analysis (Ghidra) and an isolated Wine oracle — the
original binary is a development reference, never a runtime dependency.

Fidelity baseline: the native engine is a **fixed-tick, seeded simulation**
whose acceptance bar is *comparable outcomes* against oracle evidence.
Bit-exact reproduction of the original is a separate later program, not a
prerequisite (decision 0001).

## Delivery discipline

Small themed milestones, each with an explicit gate; the owner merges every
change; releases are owner-launched, independently reviewed tags following
`PRINCIPLES.md`. Every format/mechanics claim needs retained, re-derivable
evidence under `docs/evidence/`. See the [delivery plan](plan.md).
