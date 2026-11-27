# Track graph (v1 — declared connectivity)

Claim level: **declared track graph**. The connectivity rule is a declared
model, not a recovered game semantic. Established 2026-11-27, milestone 6,
from the rail readers landed in PR #63 and the field decode in
[#62](https://github.com/diegoami/rt3engine/issues/62).

## Proven layout

Each track record is a sparse blob of **599 bytes**. The first 12 bytes are a
three-word id header; the next three little-endian f32 values at `+12` are the
piece origin. The per-record reader is `RT3.exe FUN_0048dcf0`; the section and
its inner tag are documented in [gmc-save-v1.md](gmc-save-v1.md).

On the private populated fixture (British Isles, SHA-256 `2a8f6e72…`, one month
after company start, one Planet 2-2-0 with four cars, 61 track pieces, no
bridge or tunnel), the 61 origins form three contiguous runs in record order:

| Records | Run | y range |
|---|---|---|
| 0–4 | Bath-end pieces | 8,230.7 → 8,363.0 |
| 5–9 | Gloucester-end pieces | 10,052.5 → 10,185.8 |
| 10–60 | main line | 8,394.2 → 10,019.4 |

## Declared rule

1. Split runs where consecutive origins are more than **100 units** apart.
2. `piece_length` is the median consecutive-origin step within the runs.
3. Join the ends of any two runs whose origins are within one `piece_length`.

On the fixture this yields `piece_length ≈ 33`, two joins (piece 4 ↔ 10 and
60 ↔ 5) and **60 edges over 61 pieces** — a single path from the Bath end to
the Gloucester end. That is a declared reading of the measured origins; the
owner's ground truth (one line, no branch) agrees, but the game's own
connectivity is not claimed.

## Open

- The axis order and meaning of the three origin components.
- The piece orientation/endpoint fields at `+24` … `+60` and the `-131072`
  sentinel.
- Station positions. The track trailer holds 3D model names
  (`…CapST.3dp`, bridges, tunnels), not coordinates; the path-to-station link
  is not claimed.

## Reproduce

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
export RT3_TRACK_SAVE="review-tmp/saves/b1.gms"
cargo test -p rt3-kit rail
```
