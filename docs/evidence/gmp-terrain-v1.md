# GMP terrain format (v1 — framing, dims, raw-grid subset)

Claim level: **framing proven; dims-driven grid decode, 8 verified
maps**. Established 2026-11-22/23, milestone 3b, issues
[#8](https://github.com/diegoami/rt3engine/issues/8) and
[#9](https://github.com/diegoami/rt3engine/issues/9). Implemented in
`rt3-kit::terrain` (`crates/rt3-kit/src/terrain.rs`); exposed as
`rt3ctl map terrain <name>` and `rt3ctl map terrain --all --json`.
Per-map results: [terrain-scan-v1.json](terrain-scan-v1.json)
(40 current maps: 8 verified grids, 1 candidate window, 31 with no
grid; 13 with a dims header).

## Proven: save-path section framing

- `RT3.exe 0x00444dd0` is the map **save** orchestrator. Disassembly at
  `0x00444f83`…`0x00444fdb`: `MOV EDX,0x32c8; CALL 0x00531340` writes a
  4-byte section tag, then `PUSH 0x0; PUSH 0x4f2c; LEA EDX,[EBX+4];
  CALL 0x00531030` writes the 20,268-byte map record — the same sizes
  the prelude parser reads. `0x00531340(this, tag)` = tag write;
  `0x00531030(this, src, size)` = payload write. The prelude's "skipped"
  u32 words are section tags.
- The terrain section family (written by `0x00449520`, entered after
  tag `0x32dd`): `0x2ee2` + nine dims, `0x2ee3`/`0x2ef4` + presence
  bitmask, `0x2ef5`/`0x2ef6` + arrays, `0x2ee4` + bitmask, `0x2ee5` +
  f32 block, `0x2f43`/`0x2f44` + per-tile records (`0x0042b4a0`).
  `0x00517d90` is the level-masked quadtree writer used before it.
- On `Crossing the Alps.gmp` the chain sits at `0x1224b8`; its bitmask
  is `0x20000` bytes = `(256·256 + 1024·1024 − 256·256)/8` bits — a
  two-level presence mask over the 1024×1024 fine grid.

## Proven: per-map terrain dims vary

The nine dims are **not constant across maps**. `terrain::read_dims`
recovers them from the `0x2ee2` header (13 of 40 maps carry one; the
header's own framing differs in the rest — open). Observed groups
(values[0..3]=tiles, [4..5]=lattice, [6..7]=fine):

```text
Crossing the Alps   64,64,64,64,   256,256,   1024,1024
Germany             128,128,128,128, 512,512, 2048,2048
France              112,112,112,112, 448,448, 1792,1792
East Coast, USA     80,64,80,64,   320,256,   1280,1024
Alternate USA       256,160,256,160, 1024,640, 4096,2560
Southern Pacific    256,96,256,96, 1024,384,  4096,1536
…full list in terrain-scan-v1.json
```

Across the 13 header-bearing maps the lattice is consistently
tiles×4 and the fine grid lattice×4, and the maps are not square in
general. The coarse height grid layout — `(lattice+1)²` corner samples
— is verified on the 256- and 512-lattices (Crossing the Alps,
Germany); for other lattices it is an unverified inference from those
two maps.

## Proven: record extent word anchors the lattice height

The map record's first extent word (file+12) equals `lattice_h × 256`
on every header-bearing map checked — 13 of 13, including non-square
lattices (Alternate USA 640, Orient Express/Southern Pacific 384,
Greenland Growing 192). `terrain::lattice_h_from_record` reads it, and
`scan_with_dims` uses it to pick the orientation of non-square
lattices (the header's `values[5]` is the height; both orientations
validating no longer forces an ambiguity refusal when the anchor
speaks). The word also gives the lattice height of every headerless
map (256…832 across the 40), which suppresses fixed-geometry candidate
windows on maps whose height is not 256 — reporting a known-wrong
window would mislead.

## Proven: dims-driven raw f32 coarse grids (8 verified maps)

The coarse heights are stored **uncompressed** as little-endian f32
metres in `(lattice+2)`-cell rows — `(lattice+1)` height cells plus one
trailing cell (semantics open) — forming the `(lattice+1)²` corner
grid, with the lattice read from each map's own `0x2ee2` dims header
(`terrain::scan_with_dims`). Non-square lattices evaluate both
orientations; when both validate, the correct one cannot be
distinguished and no verified result is claimed — the map falls back
to the candidate path. The artifact records the rows/cols each hit
was read with. Water is 0.0 or NaN. The scanner validates windows by
value range, ≥5% land, ≤80% NaN, smoothness and ≥10 m relief;
geometries too small for the sampler's probe lattice (< 33 cells per
side) are rejected rather than mis-sampled.

The row rule (`lattice+2` cells) is corroborated on two maps by
independent evidence: Crossing the Alps (256-lattice; ASCII relief
matches the in-game view captured from the oracle — median ≈ 564 m,
p95 ≈ 1,448 m, max ≈ 2,344 m) and Germany (512-lattice; renders the
expected relief — central uplands, coastline, southern high ground —
and aligns with its 4 KiB-windowed height-dense span: the span
`[0x8000..0x10a000)` covers the scanned grid `0x89bc..0x10a1c4`
(513 × 2,056 = 1,054,728 bytes) except for the grid's final 452
bytes, which fall in the first 4 KiB window past the span — a window
whose density fell below the scan's threshold, most plausibly the
grid's low-relief trailing rows). On that rule, the square-lattice
maps scan to **verified** grids:
Crossing the Alps, Germany, State of Germany, France, Third Republic.
The three non-square maps whose both orientations validate (Alternate
USA, Orient Express, Southern Pacific) are recorded as candidates —
orientation ambiguity is refused rather than guessed.

## Candidate windows (not established as terrain)

One further window validates on a **headerless** map whose record
anchors the height at 256 (Central Pacific, `−145…0 m` — seafloor
relief or a look-alike; unresolved): the width is guessed at 256, so
it stays a candidate. Windows previously reported on Chicago to New
York, Dutchlantis and Tex-Mex are withdrawn — their records anchor
heights of 512/384/576, so the fixed-geometry windows were known-wrong
mangled reads.

## Open

- **31 of 40 maps have no verified terrain decode.** Thirty-one maps
  produce no validated window under their anchored geometry — five of
  them despite a dims header (East Coast, Greenland Growing, Italy,
  Mississippi Valley, New Beginnings). That is consistent with a
  second storage form, but the checks only establish what was not
  found; the question is unresolved. Headerless maps now have a known
  lattice height from the record word, but no known width — anchoring
  the width (another record field, or a constrained search over the
  height-anchored geometries) is the natural next step. For three window-less maps (Britain,
  East Coast, Ireland) a separate whole-file f32-density scan also
  found no dense height-like spans. These checks do not establish how
  those maps store their heights — only what was not found. The
  per-tile records (`0x2f43`/`0x2f44`, writer `0x0042b4a0` — 33-ish
  short arrays per tile) and the quadtree levels (`0x00517d90`) are
  candidate carriers; the read-side decoder is unidentified (the
  loader function is still unknown — only the save path is mapped).
- The trailing cell per row; which validated window is canonical when
  several fit (zero padding blends into leading water rows).
- The 27 maps without a `0x2ee2` dims header — their terrain framing
  differs (four of them still produce candidate windows).
- LOD decomposition beyond the coarse grid; legacy 9609/9610 layouts
  (their sections sit 2-mod-4 after the odd aux records).
- The `0x407FC0 | k<<24` field at file+0x3008 (from
  [gmp-format-v1.md](gmp-format-v1.md)) — relation to terrain unproven.

## Oracle corroboration (Wine, capture-lock protocol)

With the game parked at the loaded-map state, menu-state and map-state
dumps of the same process differ in 11 new regions; one
(`05db2000_07130000`, new at load) carries f32 spans with height-like
statistics (median 0, p95 ≈ 900 m, 43% zeros) and long runs of the
`.gmp`'s own bytes verbatim (a file cache, `mem@+X == file@(X−0xF583A)`
on Tutorial_1) — heights can reach the renderer without decompression.

## Corrected claims (supersede parts of gmp-format-v1.md)

- `0x00444dd0` writes maps; it does not read them. The layout facts in
  gmp-format-v1.md stand — a writer emits exactly the record sequence
  its loader consumes — but the loader function is unidentified. The
  earlier "chunk stream" and "terrain size at record+0x3008" corrections
  remain in force.
- `0x2ee0` (12000) at file+4 of shipped maps is a section tag of the
  12000 family (fresh in-game saves use the 13000 family, e.g. record
  tag `0x32c8`); the family appears to mark format generation, matching
  the 9609/9701 version split.
- Earlier session-local scan tables (NaN-tolerant, denormals counted as
  land) were majority false positives; only the windows listed in
  terrain-scan-v1.json passed the tightened gates (denormals excluded,
  full-cell range check on extraction).

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
cargo test -p rt3-kit terrain            # fixtures + reference map
cargo run -q -p rt3ctl -- map terrain "Crossing the Alps"
cargo run -q --release -p rt3ctl -- map terrain --all   # the artifact's producer
make evidence-check                      # includes terrain-scan-v1.json
```
