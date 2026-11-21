# Prior art and reuse policy

Survey updated 2026-11-21 (milestone 1, issue
[#3](https://github.com/diegoami/rt3engine/issues/3); GitHub repository
search — broader web search was unavailable at survey time).

## Policy

- This project's code is MIT, written here. **No code is copied from GPL or
  proprietary sources**; such projects are reference-only at most.
- Reverse-engineering findings (file formats, constants, algorithms) are
  facts and are documented freely with their method (Ghidra project, oracle
  session). Third-party code remains off-limits regardless.
- **Clean-room discipline for unlicensed reimplementations:** where a prior
  project reverse-engineers the same game (e.g. OpenRT3), implementers do
  not read its source or generated decompilations. Our findings are derived
  independently with our own Ghidra project on the GOG `RT3.exe`; where
  topics overlap, our evidence records the independent provenance (the PK4
  format record predates any contact with that project's material).
- Before adopting any third-party tool, library or dataset: license audit,
  provenance record, and an explicit decision under `docs/decisions/`.
- The original `RT3.exe` and `d8tx.dll` are analyzed, never executed at
  runtime by our product and never distributed.

## Survey

| Candidate | Scope | License | Verdict |
| --- | --- | --- | --- |
| [MichaelMcDonnell/rt3conf](https://github.com/MichaelMcDonnell/rt3conf) | Rust writer for `Data/Configuration/engine.cfg` with RE'd field offsets (modern resolutions); documents a diff-based field method | MIT | Reference. Format facts usable; milestone 2 input. Code reusable with attribution if ever adopted. |
| [FlorianFFF/OpenRT3](https://github.com/FlorianFFF/OpenRT3) | Function-at-a-time reimplementation of `RT3_mp.exe` (2004-09-08 build) in C++ (re3/OpenRCT2 method: DLL hooks, differential tests); includes PK4 readers and format docs; ~5,100 game functions estimated | **None** | Reference-only. Existence and method noted; **no source reading by implementers** (clean-room, see policy). |
| [atshaw1994/RRT3SaveManager](https://github.com/atshaw1994/RRT3SaveManager) | Cross-platform save-game manager (C#) | None | Reference-only; potentially relevant to milestone 4 (save format). No source reading beyond scope facts. |
| [FlorianFFF/RT3Multiplayer](https://github.com/FlorianFFF/RT3Multiplayer) | Community multiplayer lobby for RT3 | not audited | Behavior notes only. |
| [PariPariKai/rt3-modern-1440p](https://github.com/PariPariKai/rt3-russian) (rt3-modern-1440p) | Steam 1.05 compatibility package (DXVK, resolutions) | NOASSERTION | Behavior notes only. |
| [pedryvo/rrt2-decomp](https://github.com/pedryvo/rrt2-decomp) | Railroad Tycoon **II** decompilation | not audited | Different game; context only. |
| [bluddy/rails](https://github.com/bluddy/rails) | Original Railroad Tycoon (1990) remake in OCaml | not audited | Different game; context only. |

## Open items

- Broader web survey (community forums, legacy tools) once web search is
  available; record any additional candidates here with the same table.
- Version identity: OpenRT3 targets `RT3_mp.exe` (2004-09-08); our reference
  is the GOG `RT3.exe` (2,330,624 bytes). Confirm the patch level of our
  executable (expected 1.05/1.06) when the save/economy RE begins, and
  record it in `docs/environment.md`.
