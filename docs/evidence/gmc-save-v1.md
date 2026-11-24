# Original GMC save inventory (v1)

Claim level: **structural parse of one original version-9701 save**.
Established 2026-11-24, milestone 4. Implemented as `rt3_kit::save` and
`rt3ctl save inspect --path <file> [--json]`.

## Fixture identity and read-only handling

- File: `Central west Pacific 1.gmc`, an original save supplied by the
  owner after the oracle-input investigation on issue #30.
- Size: 8,490,851 bytes.
- SHA-256: `3c11cbbdde844273c499086a8a4c23df9e1cfe9837910696fa13bf32d121d99b`.
- The working copy stays in the gitignored `review-tmp/saves/` area.
  `RT3_SAVE` names it for tests and evidence regeneration. Original bytes
  are never committed. CI self-skips the optional fixture when unset.

## Proven structural inventory

The container reports version 9701 and satisfies every existing framed
parser: coarse terrain, per-cell/tile records, and sparse city, building
and event collections.

| Section | Offset | Records / dimensions |
|---|---:|---:|
| Coarse terrain | 35,804 | 257 rows × 385 columns; −12.0…1821.4 m |
| Tiles | 2,601,345 | 6,144 records; 0 dynamic entries |
| Buildings | 3,579,183 | 430 slots; 410 records |
| Cities | 4,084,035 | 70 slots; 59 records |
| Events | 4,244,640 | 30 slots; 17 records |

The exact artifact, including all terminator offsets, is
[save-inventory-v1.json](save-inventory-v1.json). Its producer is
`rt3ctl save inspect --path "$RT3_SAVE" --json` and is checked by
`tools/evidence.sh` whenever `RT3_SAVE` is set.

A dependency-free SHA-256 implementation identifies the fixture and has
a standard `abc` known-vector test. Save parsing rejects other versions
and any missing proven section. The original-save regression checks the
file size, hash and every count above.

## Open

- Save-specific date, company, economy, train/track and campaign fields.
- Meanings of the retained terrain/tile/blob fields beyond the already
  documented structural layouts.
- A second controlled save and a differential state walk.
- Original save *writing* is not claimed.

The filename and terrain dimensions do not establish the source map's
identity; no such inference is made.

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
export RT3_SAVE="review-tmp/saves/Central west Pacific 1.gmc"
cargo test -p rt3-kit save
cargo run -q --release -p rt3ctl -- save inspect --path "$RT3_SAVE" --json
make evidence-check
```
