# Typed game-data formats (v1)

Claim level: **parse** (read side, structural). Established 2026-11-22,
milestone 2, issues
[#5](https://github.com/diegoami/rt3engine/issues/5) and
[#6](https://github.com/diegoami/rt3engine/issues/6). Implemented in
`rt3-kit::typed` (`crates/rt3-kit/src/typed.rs`); aggregate evidence in
[typed-tables-v1.json](typed-tables-v1.json).

All records are little-endian. Names are NUL-padded fixed-width ASCII
fields; the original authoring tools left `0xCC`/`0xCD` debug fill in
unused space, which readers tolerate after the terminator. Field semantics
beyond those named below are open; every layout was validated against all
shipped files with exact-length (EOF) checks.

## Layouts

### `CargoTypes/*.cty` — 41 files, 62 bytes each, version 1002

```text
i32 version; char name[20]; f32 price; f32 factor; u8 category;
u32 icon; char car_type[21]; i32 cargo_index
```

The loader (Ghidra `0x0041ede0`) uppercases `car_type`, special-cases
`Mail`/`Passengers`/`Troops`, and derives display pricing from `price`
through two era-dependent constants. Observed: price 1e4..2.35e5, factor
0.1..0.3, category 0..11, icon 0x708..0x776, index 1..42 (file order).

### `BuildingTypes/*.bty` — 190 files, version 1004

```text
i32 version; char names[6][30]; u8 dim_w, dim_h, dim_t; u32 f[4]; u16 g[2];
blob w*h*t*t/2 bytes; (v>1000) u32 ×2; (v>1002) 12 × (u8 + u16 ×2);
(v>1003) char name[30]; u32; char tail[46]
```

Loader: Ghidra `0x00414490` (called per file by the `FindFirstFile` driver
`0x004196c0`); it special-cases `ServiceTower` and `Maintenance` and
allocates four `dim_w*dim_h*0x20` cell planes. Observed dims (6,6,48);
`g[2]` uses 0xFFFF sentinels.

### `BuildingTypes/*.bca` — 67 files, version 1010

```text
i32 version; char name[30]; fields...; u32 cost; ... u32 count @0x33
(unaligned); count × subrecord[188]; tail[169]
```

Header is 55 bytes; `size = 55 + count*188 + 169` holds for every file.
Subrecords carry cargo production/consumption rules including `{...}`
script strings (99 subrecords, 35 scripts across the installation; e.g.
`{Demand Only}`). Parser: Ghidra `0x004120b0` (driver `0x00412fb0`); the
full subrecord field map is open.

### `EngineTypes/*.lco` — 66 files, 91 bytes each, version 2005

```text
i32 version; char name[30]; f32 speed, power, price, reliability; u8;
u32 ×7; u8 ×4; i32 a (clamped 0..8); i32 b (clamped 0..9)
```

Loader: Ghidra `0x00461f10` (with legacy `version < 2005` handling).
Values are era-ordered (Adler 1835: 31 mph / 8 power / $1.3k; 242.A1:
95 mph / 175 / $22k).

### `EngineTypes/*.car` — 145 files, 272 bytes each, version 1002

```text
i32 version; u32 flag; u32 string_id; char display[60]; char lco_ref[60];
char id[30]; char short[30]; char tail_name[60]; u32; u8; u8; u32; u8; u8;
u32; u32
```

Loader: Ghidra `0x0041d7c0`; when `flag == 1` it opens the `.lco` named by
`lco_ref` and reads a 4+21-byte header from it. `EngineTypes` also holds
48 further files (`.cgo`, `.cct`, …) not covered by this milestone.

### Text families

- `TreeTypes/TreeTypes.txt`: whitespace rows `name class density` (268
  data rows), `;` comments.
- `Other/AirplaneTypes.txt`: 7 whitespace rows under a `;` column header.
- `Language/RT3.lng`: `id text` lines (`;` comments; `\n` escapes a
  newline; 3,868 strings). One shipped line glues the id to its text
  (`1"Warning"`); ids parse as leading digits. The file is 8-bit text
  (Windows codepage), decoded lossily here.

### `Configuration/*.cfg`

- `engine.cfg` / `default_engine.cfg`: 980 bytes; header `0x41B`, then
  `width`, `height` (u32) and a fixed field map cross-checked against
  [rt3conf](https://github.com/MichaelMcDonnell/rt3conf) (MIT), which
  documents sound/music/voice volumes and the render toggles by
  byte-offset. Our parser exposes width/height and validates length.
- `game.cfg`: 1,169 bytes; leading u32 (`1065` observed) — its exact
  semantics are open (it is **not** the file length).

## Provenance

- Ghidra 12.2 project on the GOG `RT3.exe` (same copy as the PK4 record;
  loaders located via format-string cross-references, decompiled
  headless; scripts under `~/ghidra/scripts`, not committed).
- Byte-level validation of every shipped file of every family (exact EOF
  consumption where the layout implies it); aggregate counts in the JSON
  artifact.
- rt3conf (MIT) used as a cross-check for `engine.cfg` only; no code
  copied.

## Open

- Field semantics marked open above (`.bty` u32/u16 groups, `.bca`
  subrecord internals, `.lco` u32 block, `.car` tail ints, `game.cfg`
  leading u32).
- The 48 non-`.car`/`.lco` files in `EngineTypes` (`.cgo`, `.cct`).
- Legacy versions below those shipped (loaders contain ladders).

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
make evidence-check                 # includes typed-tables-v1.json
cargo test -p rt3-kit typed         # fixtures + install-backed check
cargo run -q -p rt3ctl -- typed list
```
