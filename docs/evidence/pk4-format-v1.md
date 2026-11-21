# PK4 archive format (v1)

Claim level: **parse** (read side). Established 2026-11-21, milestone 1,
issues [#1](https://github.com/diegoami/rt3engine/issues/1) and
[#2](https://github.com/diegoami/rt3engine/issues/2). Implemented in
`rt3-kit::pk4` (`crates/rt3-kit/src/pk4.rs`); aggregate evidence in
[pk4-archives-v1.json](pk4-archives-v1.json).

## Layout (format version 1003)

```text
header (8 bytes):   u32 version = 1003      u32 count
entries:            count × 74 bytes, in table order:
                      u32 hash       game-side lookup key (32 buckets, hash & 0x1F)
                      u32 size       stored payload size
                      u32 offset     payload start, relative to end of table
                      u8  flag       0 in every shipped archive; meaning unverified
                      char name[61]  NUL-padded entry name (ASCII)
data:               payload blocks at the given offsets, stored raw
```

- Payload region starts at `8 + 74 × count`.
- Payloads are **raw**: DDS payloads begin `DDS `, WAVs `RIFF/WAVE`, TGAs
  with the standard TGA header; no compression or encryption was found in
  any shipped archive.
- Empty archive = the 8-byte header with `count = 0` (shipped
  `rt3_TGAF.PK4` is exactly 8 bytes).
- Name comparison in the game is case-insensitive (`__stricmp`).

## Provenance

- Ghidra 12.2 analysis of `RT3.exe` (GOG release, PE32, sha256 recorded in
  the project at `~/ghidra/rt3engine`; binary analyzed from a copy, never
  committed):
  - `0x0053b090` — index loader: reads version+count; `version < 0x3EB`
    selects a 13-byte legacy entry stride, `version >= 0x3EB` (1003) reads
    13 record bytes + 0x3D name bytes per entry (74 total); hashes index
    `32` buckets per archive (`hash & 0x1F`); rejects `version < 0x3E9`.
  - `0x0053b390` — lookup by the u32 hash with a 10-entry cache; no name
    comparison on the hot path.
  - `0x0053bd40` — seek formula: `stride × count + 8 + entry.offset`,
    confirming the payload-region base.
- Byte-level validation of all 18 shipped archives behind `RT3_HOME`
  (18/18 parse; 8,375 entries; zero out-of-bounds, overlapping or gapped
  payloads; aggregate totals in the JSON artifact).

## Open

- The name→hash algorithm is unidentified (not CRC32 of any case form, nor
  djb2/sdbm/java31/fnv1/fnv1a/bkdr). Not required for reading; required
  before authoring archives (save/export milestone).
- The legacy pre-1003 variants (versions 1001–1002, 13-byte nameless
  records) are rejected by the reader; none ship with the game.
- The flag byte's meaning is unverified (always 0 in shipped data).

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
make evidence-check                 # includes pk4-archives-v1.json
cargo test -p rt3-kit pk4           # unit fixtures + install-backed check
cargo run -q -p rt3ctl -- pk4 list --archive "$RT3_HOME/Data/2D/rt3_2WIN.PK4"
```
