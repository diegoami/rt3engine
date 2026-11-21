# Prior art and reuse policy

Status: charter only (milestone 0). Nothing audited or adopted yet.

## Policy

- This project's code is MIT, written here. **No code is copied from GPL or
  proprietary sources**; such projects are reference-only at most.
- Reverse-engineering findings (file formats, constants, algorithms) are
  facts and are documented freely with their method (Ghidra project, oracle
  session). Third-party code remains off-limits regardless.
- Before adopting any third-party tool, library or dataset: license audit,
  provenance record, and an explicit decision under `docs/decisions/`.
- The original `RT3.exe` and `d8tx.dll` are analyzed, never executed at
  runtime by our product and never distributed.

## Candidates to survey (milestone 1+)

Community Railroad Tycoon 3 tooling exists around PK4 archives, map editing
and save inspection. Survey, record license and reuse verdict per candidate:

- PK4 unpackers/archive tools (various community implementations).
- RT3 map/save editors and converters.
- GOG/Steam community fixes (resolution, compatibility) — behavior notes
  only, no code.

Survey results land in a table here with license, scope and verdict
(reference-only / audit-needed / adopted-under-license).
