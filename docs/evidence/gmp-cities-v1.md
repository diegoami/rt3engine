# GMP city collections (v1)

Claim level: **structural parse and raw label extraction for all 43
shipped maps**. Established 2026-11-24, milestone 3b, issue
[#8](https://github.com/diegoami/rt3engine/issues/8). Implemented as
`rt3_kit::cities`; per-map summaries are the `cities` objects in
[terrain-scan-v1.json](terrain-scan-v1.json).

## Provenance

Main map loader `RT3.exe 0x00443a50` seeks `0x332d` at `0x0044483d` and
calls `0x00421510` at `0x00444887`. The matching save-side label is
`gpdCityDB`. The collection reader seeks `0x5209`, invokes the generic
collection reader `0x00518680`, seeks `0x520a`, performs per-entry
virtual reads, and finally seeks `0x520b`. This evidence establishes
the city collection, not every field's gameplay meaning.

## Proven collection layout

All 43 maps use the same fixed collection area:

```text
u32 0x332d
u32 0x5209
u32 0, 6, 10, 20
u32 capacity
u32 active
u32 0
u32 level[4][3] = (1,0,0), (0,0,0), (0,0,0), (0,0,0)
u8 absent[ceil(capacity/8)]
repeat capacity slots, skipping absent bits:
    u8 prefix[6]
    u16 blob_bytes = 905
    u8 blob[905]
u32 0x520a
```

`cities::read` checks the full header, level triples, sparse slot count,
every blob length and the exact terminator. Capacity is bounded and all
allocations are fallible. Every shipped collection matches: **4,130
slots, 3,726 active records**. Ireland has 90 slots and 76 records;
British Isles has 70 and 68. Every active blob is exactly 905 bytes.

## Raw labels

Each blob has a 48-byte label field at offset 854. `CityRecord` retains
the complete blob and exposes this field as raw bytes. Known labels
include Dublin in Ireland and Manchester in British Isles.

Three shipped labels have leading control bytes: Louisville in
Germantown, plus South Bend and Louisville in Mississippi Valley. The
parser does not silently trim or normalize them. A regression checks
this exact count as well as Dublin, Manchester, all map totals and every
blob size.

## Open

- Meanings of the six prefix bytes and other blob fields.
- Per-entry virtual-read data between `0x520a` and `0x520b`.
- City coordinates, population, industries and relationships to other
  map object collections.

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
cargo test -p rt3-kit cities
cargo run -q --release -p rt3ctl -- map terrain --all --json
make evidence-check
```
