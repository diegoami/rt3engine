# Original save inventories (v1)

Claim level: **structural parse of two original version-9701 saves**, one
owner-supplied and one independently generated at a controlled state.
Established 2026-11-24, milestone 4. Implemented as `rt3_kit::save` and
`rt3ctl save inspect --path <file> [--json]`.

## Fixture identity and read-only handling

- Supplied file: `Central west Pacific 1.gmc`, provided by the owner after the
  oracle-input investigation on issue #30.
- Size: 8,490,851 bytes.
- SHA-256: `3c11cbbdde844273c499086a8a4c23df9e1cfe9837910696fa13bf32d121d99b`.
- The working copy stays in the gitignored `review-tmp/saves/` area.
  `RT3_SAVE` names it for tests and evidence regeneration. Original bytes
  are never committed. CI self-skips the optional fixture when unset.

## Supplied-save structural inventory

The container reports version 9701 and satisfies every existing framed
parser: coarse terrain, per-cell/tile records, and sparse city, building,
company, player and event collections. Company/player framing is documented in
[gmp-actors-v1.md](gmp-actors-v1.md).

| Section | Offset | Records / dimensions |
|---|---:|---:|
| Coarse terrain | 35,804 | 257 rows × 385 columns; −12.0…1821.4 m |
| Tiles | 2,601,345 | 6,144 records; 0 dynamic entries |
| Buildings | 3,579,183 | 430 slots; 410 records |
| Companies | 4,063,347 | 5 capacity; 0 records |
| Players | 4,063,823 | 6 capacity; 4 records |
| Cities | 4,084,035 | 70 slots; 59 records |
| Events | 4,244,640 | 30 slots; 17 records |

The exact artifact, including all terminator offsets, is
[save-inventory-v1.json](save-inventory-v1.json). Its producer is
`rt3ctl save inspect --path "$RT3_SAVE" --json` and is checked by
`tools/evidence.sh` whenever `RT3_SAVE` is set.

## Source-map field

Both saves contain a NUL-terminated source-map basename at fixed file offset
`0x73d0`, inside the 51-byte region ending before the following display-label
field at `0x7403`:

- supplied save: `Go West!.gmp`;
- controlled save: `British Isles.gmp`, independently confirmed by the native
  scenario launch and subsequent original-game read-back.

The same offset is empty in the shipped British Isles `.gmp`, so this is a
save-populated field rather than coincidental scenario prose. The parser
accepts only a printable ASCII `.gmp` basename without path separators.
Both JSON artifacts expose it as `source_map`.

## Independently generated control save

Native Windows AutoHotkey automation started **British Isles**, dismissed the
briefing, created the default company **Great Republic**, and saved while the
game displayed **January 1829** and **$1,100K**. No track, train or economic
action followed company creation. The save dialog truncated the requested
long name; the resulting unique file is `rt3engine-bi-18290101-202.gms`.
This is an independently generated **`.gms`**, not the previously assumed
`.gmc` extension.

- Size: 10,285,120 bytes.
- SHA-256: `f892bc28ea1fb3bb1ff60f3ed443e82de7818d3fd7a7c26a93e974cfb69ac6bc`.
- Private working copy: `review-tmp/saves/rt3engine-bi-18290101-202.gms`;
  `RT3_CONTROL_SAVE` names it for tests and evidence regeneration.

| Section | Offset | Records / dimensions |
|---|---:|---:|
| Coarse terrain | 33,069 | 513 rows × 321 columns; 0.0…760.5 m |
| Tiles | 3,574,574 | 10,240 records; 0 dynamic entries |
| Buildings | 5,096,090 | 530 slots; 520 records |
| Companies | 5,710,149 | 5 capacity; 3 records |
| Players | 5,801,645 | 4 capacity; 3 records |
| Cities | 5,817,066 | 70 slots; 68 records |
| Events | 5,992,027 | 50 slots; 39 records |

The exact artifact is
[controlled-save-inventory-v1.json](controlled-save-inventory-v1.json), with
producer `rt3ctl save inspect --path "$RT3_CONTROL_SAVE" --json`.
The native save has been privately transferred and parsed. Original-game
read-back was verified on 2026-11-25 through the reviewed elevated disposable
launcher: **Single Player → Load Game → Start** returned to British Isles at
January 1829 with $1,100K. A second controlled save remains open; Save-button
and Enter attempts after loading did not create a file.

A copy of the game at `RT3_HOME` is not by itself sufficient to isolate native
writes: this RT3 installation uses GOG's registry `SAVEGAMEFOLDER`. Future
native runs must redirect and verify that setting before saving, keep the
original installation read-only, and restore its registry values immediately
after startup. No save bytes or original assets are committed.

## Shared parser checks

A dependency-free SHA-256 implementation identifies each fixture and has a
standard `abc` known-vector test. Save parsing rejects other versions and any
missing proven section. Fixture regressions check file size, hash, source-map
basename, dimensions and every collection count above.

## Open

- Save-specific date, company, economy, train/track and campaign fields.
- Meanings of the retained terrain/tile/blob fields beyond the already
  documented structural layouts.
- A second controlled save and a differential state walk.
- Original save *writing* is not claimed.

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
export RT3_SAVE="review-tmp/saves/Central west Pacific 1.gmc"
export RT3_CONTROL_SAVE="review-tmp/saves/rt3engine-bi-18290101-202.gms"
cargo test -p rt3-kit save
cargo run -q --release -p rt3ctl -- save inspect --path "$RT3_SAVE" --json
cargo run -q --release -p rt3ctl -- save inspect --path "$RT3_CONTROL_SAVE" --json
make evidence-check
```
