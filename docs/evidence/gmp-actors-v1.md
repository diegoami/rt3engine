# Company and player collections (v1)

Claim level: **structural parse and raw name fields**. Established 2026-11-25,
milestone 4. Implemented by `rt3-kit::fixed`, `companies` and `players`.
No biographies, original records or save bytes are committed.

## Provenance and shared framing

Static RE identifies `gpdCompanyDB` at `RT3.exe 0x00429b90` and `gpdPlayerDB` at
`0x004777e0`. Both use the generic collection reader at `0x00517d90`, with
fixed-size records rather than the per-record lengths used by buildings/events.

All offsets below are relative to the outer section tag:

```text
+0x00 u32 outer tag
+0x04 u32 inner tag
+0x08 u32 one = 1
+0x0c u32 record_bytes
+0x10 u32 nominal_slots (5 companies, 4 players; not an active limit)
+0x14 u32 open
+0x18 u32 capacity (level payload and mask sizing)
+0x1c u32 active serialized record count
+0x20 u32 active repeated for companies; zero for players
+0x24 4 × 12-byte level records
      level 0 = [1,0,0]
      level 1 optionally owns a capacity × 12-byte payload
      level 2 flag is zero in all shipped maps
      fourth/last triple is not a boolean in every map; semantics open
      level 1 payload, when present
      occupancy/mask bytes: (capacity + 8) >> 3
      active fixed-size records, with no per-record length prefix
      two ending tags
```

The mask formula is visible at `0x00517d90` and matters at exact multiples of
eight. Mask semantics are deliberately not inferred. The retained `end_offset`
points at the first ending tag. Level tables beyond the observed level-1
payload and the remaining record fields remain open.

## Concrete collections

| Collection | Tags | Record bytes | Raw name field |
|---|---|---:|---|
| Companies | `0x32CE / 0x61A9`; end `0x61AA / 0x61AB` | 30,340 | offset 4, 64 bytes |
| Players | `0x32D0 / 0x5209`; end `0x520A / 0x520B` | 3,243 | offset 8, 31 bytes |

All **43 shipped maps** parse both collections: 40 version-9701 maps plus the
three legacy 9609/9610 maps. Totals are **37 company records** and **145 player
records**; every raw name field is nonempty printable ASCII.

Both original saves also parse:

| Save | Companies | Players |
|---|---|---|
| Controlled British Isles `.gms` | 3; third is `Great Republic` | 3; first is `George Hudson` |
| Supplied `.gmc` | 0 | 4; fourth is `Unassigned` |

The controlled save also contains liquidated AI companies. Their presence is
structural evidence only; no economic state is inferred from a name suffix.

## Evidence and reproduction

`docs/evidence/terrain-scan-v1.json` now includes per-map `companies` and
`players` offsets, capacities, counts and ending offsets. The two save inventory
artifacts contain the corresponding save-side structures.

```sh
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
export RT3_SAVE="$PWD/review-tmp/saves/Central west Pacific 1.gmc"
export RT3_CONTROL_SAVE="$PWD/review-tmp/saves/rt3engine-bi-18290101-202.gms"
cargo test -p rt3-kit fixed
cargo test -p rt3-kit companies
cargo test -p rt3-kit players
cargo test -p rt3-kit save
make evidence-check
```
