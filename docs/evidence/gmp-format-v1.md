# GMP map format (v1 — prelude)

Claim level: **parse (prelude only)**. Established 2026-11-22, milestone 3,
issues [#8](https://github.com/diegoami/rt3engine/issues/8) and
[#9](https://github.com/diegoami/rt3engine/issues/9). Implemented in
`rt3-kit::map` (`crates/rt3-kit/src/map.rs`); inventory evidence in
[maps-inventory-v1.json](maps-inventory-v1.json) (43 maps: 40 current,
3 legacy).

## Proven layout (version 9701)

```text
u32 version = 9701
map record, 20,268 bytes (fixed raw read in the loader):
  extent/config words at record+8 .. record+0x27 (semantics open)
  scenario intro text at record+0x78 (NUL-terminated, 8-bit codepage
    prose — length recorded, content never committed)
u32 skipped
auxiliary record, 3,001 bytes (largely 0xCC debug fill in shipped files)
u32 skipped
auxiliary record, 3,001 bytes
map-specific tail (terrain + objects; grammar open — see below)
```

## Provenance

- Loader: Ghidra `0x00444dd0` (located via the unique version-constant
  computation `0x25E5 + (flag ? 100 : 0)`; note 0x25E5 = 9701). Read
  primitives: `0x00531340` skips 4 bytes; `0x00531030(this, dest, size)`
  reads raw bytes — the .gmp is a fixed-record file the loader cherry-picks
  through, not a chunk stream.
- Read order in the driver: skip4, RAW 20,268, skip4, RAW 3,001, skip4,
  RAW 3,001, skip4, then section handlers (`FUN_00482160` and others) own
  the tail.
- Byte validation: all 40 version-9701 maps parse; all 3 legacy maps
  (9609 ×2 tutorial, 9610 ×1 Pacific Coastal) are recognized with a
  different pre-text layout (their text at 0x78 lands mid-sentence).

## Corrected claims (supersede issue #8 comments 1–2)

- There is no length-prefixed chunk stream: the file is a cherry-picked
  fixed record. The earlier "universal chunk prelude" reading was a
  misinterpretation of the fixed record's zero region.
- Offset bases must be stated precisely (round-1 review finding):
  - `u32` at **file offset 0x3008** (= record+0x3004, inside the fixed
    map record): nonzero in all 43 shipped maps — 4,227,072 in 36 maps,
    37,781,504 / 21,004,288 / 54,558,720 in the others. A genuine
    per-map, size-like field; semantics open (terrain scale unproven).
  - `u32` at **record offset 0x3008** (file offset 0x300C): 0 in every
    shipped map — the value my original check (mis-labeled as file+0x3008)
    actually measured.

## Open

- The map-specific tail: terrain encoding — step 1 (the coarse
  heightfield) is proven, see [gmp-terrain-v1.md](gmp-terrain-v1.md);
  objects, cities/industries, resources, events remain. The section
  handlers' own reads must be mapped.
- Legacy 9609/9610 non-prelude/non-terrain sections; their coarse grids
  share the explicit frame described in gmp-terrain-v1.md.
- Semantics of the extent/config words.

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
make evidence-check                 # includes maps-inventory-v1.json
cargo test -p rt3-kit map           # fixtures + install-backed check
cargo run -q -p rt3ctl -- map list
```
