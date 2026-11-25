# Evidence

Every format/mechanics claim in this repository needs a retained artifact
here that re-derives it from `RT3_HOME`. Artifacts are **small, derived
metadata only** (JSON summaries, counts, renders we made) — never original
bytes, sprites, maps, saves or sounds.

## Regeneration

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
make evidence-check   # verify committed artifacts against the installation
make evidence-regen   # regenerate after an intentional change; audit the diff
```

Checks self-skip without `RT3_HOME` (CI).

## Artifacts

| Artifact | Claim | Producer |
| --- | --- | --- |
| `install-inventory-v1.json` | Aggregate installation inventory: 1,176 files, ~2.02 GB; per-top-level-entry and per-extension counts/sizes (metadata only) | `rt3ctl inventory --json` |
| `pk4-archives-v1.json` | PK4 aggregate: 18 archives, format version 1003, 8,375 entries, ~387 MB raw payload, per-archive totals (metadata only) | `rt3ctl pk4 list --json` |
| `typed-tables-v1.json` | Typed-data aggregate: 41 cargo, 190 building types, 67 behaviors (99 subrecords, 35 scripts), 66 locomotives, 145 car types, 268 tree rows, 3,868 language strings, engine.cfg 800×600 | `rt3ctl typed list --json` |
| `maps-inventory-v1.json` | Map inventory: 43 shipped maps (40 version-9701 current, 3 legacy 9609/9610) with text lengths and tail sizes (metadata only) | `rt3ctl map list --json` |
| `terrain-scan-v1.json` | Terrain and sparse-map collections: all 43 shipped maps verified, with exact offsets, dimensions, height statistics and per-map tile/city/building/event capacities and counts | `rt3ctl map terrain --all --json` |
| `save-inventory-v1.json` | One owner-supplied version-9701 `.gmc` save: identity, source-map basename, framed terrain and tile/city/building/event structural inventory; optional `RT3_SAVE` fixture | `rt3ctl save inspect --path "$RT3_SAVE" --json` |
| `controlled-save-inventory-v1.json` | One independently generated and read-back-verified version-9701 `.gms` save at a recorded scenario/company start; optional `RT3_CONTROL_SAVE` fixture | `rt3ctl save inspect --path "$RT3_CONTROL_SAVE" --json` |

Format findings with method and provenance live beside them:
[pk4-format-v1.md](pk4-format-v1.md) (PK4 layout; claim level: parse) and
[typed-formats-v1.md](typed-formats-v1.md) (typed records and text/config)
and [gmp-format-v1.md](gmp-format-v1.md) (map prelude; claim level: parse).
City collection findings live in [gmp-cities-v1.md](gmp-cities-v1.md);
building/event collections are in
[gmp-collections-v1.md](gmp-collections-v1.md).
Original save inventory findings are in
[gmc-save-v1.md](gmc-save-v1.md).
Milestone 3 adds map findings; later milestones add save
findings, each with its producer command and method note (Ghidra address,
oracle session, or both).
