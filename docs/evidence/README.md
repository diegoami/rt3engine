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

Format findings with method and provenance live beside them:
[pk4-format-v1.md](pk4-format-v1.md) (PK4 layout; claim level: parse).
Milestone 2 adds typed-data tables; later milestones add map and save
findings, each with its producer command and method note (Ghidra address,
oracle session, or both).
