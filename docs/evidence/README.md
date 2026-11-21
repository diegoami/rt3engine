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

Milestone 1 adds PK4 archive inventories; later milestones add typed-data,
map and save findings, each with its producer command and method note
(Ghidra address, oracle session, or both).
