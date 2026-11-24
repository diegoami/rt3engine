# GMP terrain format (v1 — coarse heightfield)

Claim level: **framing and coarse-grid decode proven for all 40 current
and three legacy maps, all verified**. Established 2026-11-22/24, milestone 3b, issues
[#8](https://github.com/diegoami/rt3engine/issues/8) and
[#9](https://github.com/diegoami/rt3engine/issues/9). Implemented in
`rt3-kit::terrain` (`crates/rt3-kit/src/terrain.rs`); exposed as
`rt3ctl map terrain <name>` and `rt3ctl map terrain --all --json`.
Per-map results: [terrain-scan-v1.json](terrain-scan-v1.json)
(43 shipped maps: 40 current 9701 and three legacy 9609/9610, all
verified.)

## Proven: save-path section framing

- `RT3.exe 0x00444dd0` is the map **save** orchestrator (corrects
  gmp-format-v1.md's earlier loader claim). Disassembly at
  `0x00444f83`…`0x00444fdb`: `MOV EDX,0x32c8; CALL 0x00531340` writes a
  4-byte section tag, then `PUSH 0x0; PUSH 0x4f2c; LEA EDX,[EBX+4];
  CALL 0x00531030` writes the 20,268-byte map record. `0x00531340(this,
  tag)` = tag write; `0x00531030(this, src, size)` = payload write.
- The terrain section family (written by `0x00449520`, entered after
  tag `0x32dd`): `0x2ee2` + nine dims, `0x2ee3`/`0x2ef4` + presence
  bitmask, `0x2ef5`/`0x2ef6` + arrays, `0x2ee4` + bitmask, `0x2ee5` +
  f32 block, `0x2f43`/`0x2f44` + per-tile records (`0x0042b4a0`).
  `0x00517d90` is the level-masked quadtree writer.
- The tag family marks format generation (shipped maps carry 12000-
  family tags; fresh in-game saves use 13000-family, e.g. record tag
  `0x32c8`).

## Proven: explicit coarse-grid frame on all 43 maps

Every shipped map carries this frame at an arbitrary byte alignment.
Current maps use subsection tag `0x2ee2`; legacy 9609/9610 maps use
`0x2ee1`, with the remaining layout unchanged:

```text
u32 0x32dd                  terrain section
u32 0x2ee2 / 0x2ee1         current / legacy coarse-grid subsection
u32 11001                   framing constant; semantics open
u32 3002                    framing constant; semantics open
u32 cols                    stored height columns = lattice_w + 1
u32 rows                    stored height rows    = lattice_h + 1
u32 11002                   framing constant; semantics open
u32 11004                   framing constant; semantics open
f32 heights[rows][cols+1]   starts immediately at section + 32
```

`terrain::read_framed` searches for the full signature, bounds-checks
the geometry, validates at exactly `section + 32`, and extracts every
cell. All 43 shipped maps match the signature and pass sampled
validation plus the full-cell range check. The artifact records the
resulting offsets, dimensions and statistics. The extra `cols + 1`th
cell in each stored row is the trailing cell whose semantics remain
open; it is not part of the corner-height grid.

This supersedes the earlier scan-derived offsets: those validated
windows overlapped the real grids but began **84–96 bytes too early**
(21–24 f32 cells). The explicit frame, not a smoothness preference or a
per-map offset override, determines the corrected starts. California
Island is the clearest correction: its section is at `0x845b`, its real
grid starts at `0x847b` with 257×257 cells, and its former candidate
window at `0x841f` began 92 bytes earlier. It is now verified like the
other current maps; no current candidate remains.

## Older nine-word dims header

Thirteen maps also carry the older nine-`u32` dims header read by
`terrain::read_dims`. Its observed groupings are tiles, tiles, tiles,
tiles, lattice_w, lattice_h, fine_w = 4·lattice_w,
fine_h = 4·lattice_h, 0:

```text
Crossing the Alps   256×256 lattice, 1024×1024 fine
Germany             512×512, 2048×2048
East Coast, USA     320×256, 1280×1024
Alternate USA       1024×640, 4096×2560
Greenland Growing   256×192, 1024×768
…full list in terrain-scan-v1.json
```

The older header and record-word orientation path remain as fallbacks
for inputs without the explicit frame. They no longer determine the
current-map starts.

## Proven: byte alignment is data-dependent

Odd-length sections (train-roster strings; the legacy 3,001-byte aux
records) shift a map's post-prelude stream by 1–3 bytes, so sections
and raw rows do not always sit 0-mod-4. Italy's grid sits at **2 mod
4**. The framed reader checks every byte offset; fallback scans try all
four alignments ranked by land-like float density. Residues across the
40 framed grids: 10 at 0, 18 at 1, 6 at 2, 6 at 3 (mod 4).

## Proven: record extent word anchors fallback height

The map record's first extent word (file+12) equals `lattice_h × 256`
on every older-header map checked — 13 of 13, including every
non-square lattice. `terrain::lattice_h_from_record` reads it for the
fallback paths; it also gives the lattice height of every map lacking
the older header (256…768 across the current maps).

## Proven: raw f32 coarse grids

The coarse heights are stored **uncompressed** as little-endian f32
metres. The frame's `rows` and `cols` are the `(lattice+1)` corner
samples; each stored row has one additional trailing cell. Water is
0.0 or NaN. Windows validate by value range, ≥5% land, ≤80% NaN,
smoothness and ≥10 m relief; geometries below the sampler's 33-cell
floor or above the bounded parser limit are rejected rather than
mis-sampled.

## 2D PNG render

`rt3_kit::render::encode_png` renders every framed grid as a north-up,
8-bit RGB PNG with no external dependencies. Display rows reverse the
stored order (the south-to-north orientation corroborated on three
maps); zero and NaN use the oracle's water color `(41, 64, 96)`, and
other heights use a green-brown-white gradient
scaled between the grid's 5th and 95th height percentiles. The PNG uses
standard uncompressed-deflate blocks, with CRC-32 and Adler-32
checksums. Tests decode its stored pixels, check exact orientation and
water handling, and validate the full pixel buffer for all 40 maps.

```sh
rt3ctl map terrain "California Island" --png review-tmp/california.png
```

The CLI requires an existing output parent, rejects every output
symlink (including dangling links), and refuses to write inside the
canonical installation root.
Generated images stay in the gitignored working area; no original game
bytes are embedded. These are terrain visualizations, not original-game
renderings. All 40 current maps were rendered and inspected in indexed
contact sheets: every output shows coherent north-up geography with no
blank, misaligned or transposed result. Examples include California
Island's diagonal island chain, Argentina's Andes on the western edge
with plains east, the British Isles coastline, Italy and Japan. Shared
scenario geography (for example Britain/British Isles and France/Third
Republic) renders consistently. The three legacy maps were also rendered
and inspected: Pacific Coastal shows the western coastline and mountain
spine, and both tutorials show coherent Italy geography.

## Oracle corroboration (2026-11-23)

The original executable's sandbox picker independently confirms every
current map that lacks the older nine-word header: the 26 formerly
pitch-anchored maps plus California Island, all now decoded by their
explicit frames.

| Map | Decoded lattice | Original picker |
|---|---:|---:|
| Argentina Opens Up | 640×384 | 640×384 |
| Britain | 320×512 | 320×512 |
| British Isles | 320×512 | 320×512 |
| California Island | 256×256 | 256×256 |
| Central Pacific | 512×256 | 512×256 |
| Chicago to New York | 640×512 | 640×512 |
| Coast to Coast | 1024×640 | 1024×640 |
| Dutchlantis | 512×384 | 512×384 |
| East of Mississippi | 640×576 | 640×576 |
| Eastern Canada | 448×384 | 448×384 |
| Eastern China | 448×576 | 448×576 |
| Germantown | 576×576 | 576×576 |
| Go West! | 384×256 | 384×256 |
| Ireland | 256×320 | 256×320 |
| Japan Trembles | 320×256 | 320×256 |
| Louisiana | 512×512 | 512×512 |
| Mexico | 640×640 | 640×640 |
| Pacific NW | 512×512 | 512×512 |
| Poland | 448×320 | 448×320 |
| Rhodes Unfinished | 576×576 | 576×576 |
| Russia | 640×640 | 640×640 |
| South East Australia | 320×384 | 320×384 |
| South East USA | 640×768 | 640×768 |
| Spanish Mainline | 448×384 | 448×384 |
| Tex-Mex | 512×576 | 512×576 |
| Texas Tea | 512×576 | 512×576 |
| War Effort | 320×256 | 320×256 |

The scenario picker also independently confirms East Coast, USA as
320×256. These are exact geometry confirmations, not independent checks
of every height sample.

For British Isles, Italy and Ireland, the in-game minimap also
corroborates row orientation. The comparison resized each decoded grid
to its minimap with nearest neighbour, classified grid water as `0` or
NaN and oracle water by its blue channel ordering (`B > G > R`), then
tested all four axis-aligned reflections. A **vertical flip** — stored
rows run south to north — was best on every map:

| Map | Vertical flip | Next-best reflection |
|---|---:|---:|
| British Isles | 75.7% | 61.9% |
| Italy | 76.0% | 65.4% |
| Ireland | 77.2% | 73.5% |

The scores measure water/land agreement; coast shading, cities and the
camera rectangle limit them. They are corroboration on these three
maps, not a general acceptance threshold or a row-order claim for maps
not compared.

The picker scroll widget did not respond to scripted input, so the
complete sweep presented each map in turn under the sandbox's default
Argentina filename in the writable oracle copy. Each substituted
file's SHA-256 matched its source before capture. In a separate run,
Ireland loaded this way showed Ireland and Dublin in game. Exit traps
restored Argentina after the sessions, and byte-for-byte comparisons
verified the restoration. No original installation file was modified.
Every oracle session held the capture lock; screenshots and original
bytes are not committed.

## Current decode and fallback ladder

`rt3ctl map terrain` uses this order:

1. **verified (43 shipped maps: 40 current, three legacy):** exact start
   and geometry from `read_framed`.
2. **verified fallback:** geometry from the older nine-word header,
   with the record extent word resolving non-square orientation.
3. **pitch-anchored fallback:** record height plus a clearly separated
   measured row pitch (`measure_pitch`: densest height span, lag
   autocorrelation over observed `(w+2)` candidates, winner ≤ 60% of
   the runner-up), validated at exactly that geometry. The primary
   measurement uses the densest-ranked alignment; only when it yields
   no measurement do the other alignments run, and exactly one fallback
   measurement is required. A measured pitch that fails validation
   **ends the ladder**.
4. **candidate fallback:** only with no measurable pitch and record
   height 256 does the fixed-256 geometry run. No current map needs or
   retains this classification.

Legacy 9609/9610 maps share the frame with subsection tag `0x2ee1`.
Pacific Coastal decodes at `0x884c` with 321×833 stored cells; Tutorial
1 and Tutorial 2 decode at `0x87e6` with 385×449. All three are verified
in the artifact. This supersedes Pacific Coastal's earlier fallback-only
measurement while leaving non-terrain legacy sections open.

## Open

- Semantics of the frame constants (11001, 3002, 11002, 11004) and of
  each stored row's trailing cell. Fallback scans can still validate
  overlapping windows; current maps use the explicit frame instead.
- Minimap row-order comparison currently covers only British Isles,
  Italy and Ireland.
- Per-tile records (`0x2f43`/`0x2f44`, writer `0x0042b4a0`) and
  quadtree levels (`0x00517d90`) beyond the coarse grids; the
  read-side decoder is still unidentified (only the save path is
  mapped).
- Legacy 9609/9610 non-terrain sections and other generation
  differences; the coarse-grid frame itself is shared and decoded.
- The `0x407FC0 | k<<24` field at file+0x3008 (from
  [gmp-format-v1.md](gmp-format-v1.md)) — relation to terrain unproven.

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
cargo test -p rt3-kit terrain            # fixtures + framed references and fallback regressions
cargo run -q -p rt3ctl -- map terrain "California Island"
cargo run -q -p rt3ctl -- map terrain "Pacific Coastal"
cargo run -q --release -p rt3ctl -- map terrain --all   # the artifact's producer
make evidence-check                      # includes terrain-scan-v1.json
```
