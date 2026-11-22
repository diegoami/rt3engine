# GMP terrain format (v1 — framing, dims, raw-grid subset)

Claim level: **framing proven; decode proven for a subset of maps**.
Established 2026-11-22, milestone 3b, issues
[#8](https://github.com/diegoami/rt3engine/issues/8) and
[#9](https://github.com/diegoami/rt3engine/issues/9). Implemented in
`rt3-kit::terrain` (`crates/rt3-kit/src/terrain.rs`); exposed as
`rt3ctl map terrain <name>` and `rt3ctl map terrain --all --json`.
Per-map results: [terrain-scan-v1.json](terrain-scan-v1.json)
(40 current maps: 12 with a decoded raw grid, 13 with a dims header).

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

The lattice is always tiles×4 and the fine grid lattice×4; the maps are
not square in general. The coarse height grid carries `(lattice+1)²`
corner samples.

## Proven: raw f32 coarse grids (12 of 40 maps)

In 12 maps the coarse heights are stored **uncompressed**, as
little-endian f32 metres in 1,032-byte rows — 257 height cells plus one
trailing cell (semantics open) — forming the 257×257 corner grid of the
256-lattice maps; on larger-lattice maps (e.g. Germany, France) the
scanner validates 257-wide windows inside the bigger grid. Water is 0.0
or NaN. `terrain::scan` locates them by validation (value range, ≥5%
land, ≤80% NaN, smoothness, ≥10 m relief); `rt3ctl map terrain
"Crossing the Alps"` renders the ASCII relief — alpine statistics
median ≈ 564 m, p95 ≈ 1,448 m, max ≈ 2,344 m, visually matching the
in-game view captured from the oracle the same session.

Central Pacific and Dutchlantis scan to all-negative ranges
(−145…0 m, −150…0 m) — seafloor-relief or a look-alike window; recorded
as-is, unresolved.

## Open

- **The majority of maps (28 of 40) store their heights some other
  way**: no validated raw grid, and no dense f32 spans anywhere in the
  file (checked for Britain, East Coast, Ireland). The per-tile records
  (`0x2f43`/`0x2f44`, writer `0x0042b4a0` — 33-ish short arrays per
  tile) and the quadtree levels (`0x00517d90`) are the likely carriers;
  the read-side decoder is unidentified (the loader function is still
  unknown — only the save path is mapped).
- Which validated window is the canonical grid when several fit
  (sub-windows of larger grids); the trailing cell per row.
- The 27 maps without a `0x2ee2` header that still carry raw grids
  (Tex-Mex, Chicago to New York, …) — their framing tags differ.
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
