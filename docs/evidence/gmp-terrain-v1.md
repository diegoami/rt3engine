# GMP terrain format (v1 — coarse heightfield)

Claim level: **framing proven; validated coarse-grid decode for 39 of
40 current maps, plus one candidate window**. Established
2026-11-22/23, milestone 3b, issues
[#8](https://github.com/diegoami/rt3engine/issues/8) and
[#9](https://github.com/diegoami/rt3engine/issues/9). Implemented in
`rt3-kit::terrain` (`crates/rt3-kit/src/terrain.rs`); exposed as
`rt3ctl map terrain <name>` and `rt3ctl map terrain --all --json`.
Per-map results: [terrain-scan-v1.json](terrain-scan-v1.json)
(40 current maps: 13 verified, 26 pitch-anchored, 1 candidate —
California Island; every current map has a grid result. Pacific
Coastal is legacy 9610 and outside this scan.)

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

## Proven: per-map terrain dims vary

The nine dims are **not constant across maps**; `terrain::read_dims`
recovers them from the `0x2ee2` header (13 of 40 maps carry one).
Observed groupings (tiles, tiles, tiles, tiles, lattice_w,
lattice_h, fine_w = 4·lattice_w, fine_h = 4·lattice_h, 0):

```text
Crossing the Alps   256×256 lattice, 1024×1024 fine
Germany             512×512, 2048×2048
East Coast, USA     320×256, 1280×1024
Alternate USA       1024×640, 4096×2560
Greenland Growing   256×192, 1024×768
…full list in terrain-scan-v1.json
```

The coarse height grid carries `(lattice+1)²` corner samples —
verified on the 256- and 512-lattices (Crossing the Alps, Germany);
for other lattices it is an unverified inference from those two maps.

## Proven: byte alignment is data-dependent

Odd-length sections (train-roster strings; the legacy 3,001-byte aux
records) shift a map's post-prelude stream by 1–3 bytes, so raw rows
do not always sit 0-mod-4: Italy's heights sit at **2 mod 4**, exactly
where its roster implies. The scan tries all four byte alignments
ranked by land-like float density (`alignment_rank`), and hits record
their geometry so the chosen alignment stays visible. Residues across
the 40 grid results: 10 at 0, 18 at 1, 6 at 2, 6 at 3 (mod 4).

## Proven: record extent word anchors the lattice height

The map record's first extent word (file+12) equals `lattice_h × 256`
on every header-bearing map checked — 13 of 13, including every
non-square lattice (Alternate USA 640, Orient Express/Southern Pacific
384, Greenland Growing 192). `terrain::lattice_h_from_record` reads it
and `scan_with_dims` uses it to anchor the orientation of non-square
lattices; it also gives the lattice height of every headerless map
(256…768 across the current maps).

## Proven: raw f32 coarse grids

The coarse heights are stored **uncompressed** as little-endian f32
metres in `(lattice+2)`-cell rows — `(lattice+1)` height cells plus
one trailing cell (semantics open) — forming the `(lattice+1)²` corner
grid. Water is 0.0 or NaN. Windows validate by value range, ≥5% land,
≤80% NaN, smoothness and ≥10 m relief; geometries below the sampler's
33-cell floor are rejected rather than mis-sampled.

## Scan ladder and classification

- **verified** (13): header-bearing maps, geometry from their own
  record-anchored dims header (`scan_with_dims`; both orientations
  considered for non-square lattices, the record word picks the
  height). Includes Italy at alignment 2.
- **pitch-anchored** (26): headerless maps whose height comes from the
  record word and whose width comes from a clearly separated measured
  row pitch (`measure_pitch`: densest height span, lag
  autocorrelation over the observed `(w+2)` candidates, winner ≤ 60%
  of the runner-up), validated at exactly that geometry. The primary
  measurement uses the densest-ranked alignment; only when it yields
  no measurement do the other alignments run, and exactly one fallback
  measurement is required. Ireland needs that fallback: alignment 1
  measures pitch 258 (11.0 versus 27.9 for the runner-up), giving a
  validated 256×320 grid at `0x839d`, `−32…435 m`. Renders corroborate
  the established set: Germany's Mittelgebirge, Tex-Mex's Sierra
  Madre, Chicago–New York's Appalachian diagonal, Argentina's Andes
  edge (max 4,966 m), Rhodes' island relief.
- **candidate** (1): California Island — no measurable pitch and
  record height 256, so the fixed-256 guess applies (`−454…3051 m` at
  257×257); not established as terrain.
- A measured pitch that fails validation **ends the ladder** — no
  fallback after the map has spoken (`terrain::scan_headerless`).
  With alignment coverage and the unique fallback, 26 of the 27
  headerless maps measure a pitch and validate at exactly that
  geometry; California Island (record height 256, no measurable pitch
  at any alignment) takes the fixed-geometry candidate.

The legacy 9610 map Pacific Coastal is outside the current-map scan;
a direct check shows it measures (320×832) and validates — evidence
the raw-row layout reaches the legacy format too (not claimed beyond
that).

## Open

- Which validated window is canonical when several fit (zero padding
  blends into leading water rows); the trailing cell per row.
- Per-tile records (`0x2f43`/`0x2f44`, writer `0x0042b4a0`) and
  quadtree levels (`0x00517d90`) beyond the coarse grids; the
  read-side decoder is still unidentified (only the save path is
  mapped).
- Legacy 9609/9610 layouts (only Pacific Coastal touched: it measures
  and validates).
- The `0x407FC0 | k<<24` field at file+0x3008 (from
  [gmp-format-v1.md](gmp-format-v1.md)) — relation to terrain unproven.

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
cargo test -p rt3-kit terrain            # fixtures + Alps/Germany/Italy/Argentina/Ireland
cargo run -q -p rt3ctl -- map terrain "Crossing the Alps"
cargo run -q --release -p rt3ctl -- map terrain --all   # the artifact's producer
make evidence-check                      # includes terrain-scan-v1.json
```
