# GMP building and event collections (v1)

Claim level: **structural parse for all 43 shipped maps**, with raw
building labels and event names/text. Established 2026-11-24, milestone
3b, issue [#8](https://github.com/diegoami/rt3engine/issues/8).
Implemented as `rt3_kit::buildings` and `rt3_kit::events` over the shared
bounded sparse-blob reader. Per-map summaries are the `buildings` and
`events` objects in [terrain-scan-v1.json](terrain-scan-v1.json).

## Provenance

- Main loader `0x00443a50` enters `gpdBuildingDB` through tag `0x3315`
  and reader `0x00413280` at `0x00444467`. Its nested collection uses
  `0x36b1` and terminates at `0x36b2`; per-entry virtual reads follow.
- It enters `gpdEventDB` through `0x32dc` and reader `0x00433130` at
  `0x00444959`. The nested `0x4e99` collection has a 1001 word before
  its header and terminates its fixed area at `0x4e9a`.

The labels come from original executable save/load references, not from
guesses based on record text.

## Shared sparse-blob layout

Both collections have the header `(0, 6, 10, 20, capacity, active, 0)`
and four level triples. Level 0 is `(1,0,0)`. Each populated later level
carries `capacity × 12` bytes before the absence mask; this is used by
events, while the shipped building levels carry no such payload.

After `(capacity + 8) >> 3` allocated mask bytes (including a following byte
at exact multiples of eight), each active slot stores six prefix bytes, a
16-bit length and exactly that many blob bytes. The parser
checks the active count and terminator, uses checked arithmetic and
fallible reservations, and rejects excess active records before growth.

| Collection | Blob bytes | Fixed-area end | Slots | Active records |
|---|---:|---:|---:|---:|
| Buildings | 993 | `0x36b2` | 2,450 | 2,044 |
| Events | 2,191 | `0x4e9a` | 1,810 | 1,169 |

All 43 maps match. Eighteen maps have nonempty building collections;
every map has at least one event.

## Raw labels and event text

- Building blobs carry a 48-byte label beginning 51 bytes before the
  blob end (offset 942). All 2,044 labels are nonempty with no leading
  control byte. Examples include `Warehouse03`, `Barracks`, `Windmill`,
  `Farm Sheep` and `Farm Cattle`.
- Event blobs carry a 200-byte name field at offset 24, with message
  text beginning at offset 224. All 1,169 names are nonempty. Raw bytes
  are retained: Spanish Mainline includes 37 Latin-1 names containing
  José/Martí, with no forced UTF-8 conversion.

The city parser's raw label rule uses the same trailing 48-byte layout
as buildings. Its non-blocking review follow-up is included here: the
active-count guard, borrowed mask and direct truncation/count/length
regressions are now explicit.

## Open

- Prefix, level and remaining blob field semantics.
- Building types, coordinates, production and links to cities.
- Event conditions, actions and post-`0x4e9a` virtual-read details.
- Semantic interpretation of names/text beyond the raw fields above.

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
cargo test -p rt3-kit buildings
cargo test -p rt3-kit events
cargo run -q --release -p rt3ctl -- map terrain --all --json
make evidence-check
```
