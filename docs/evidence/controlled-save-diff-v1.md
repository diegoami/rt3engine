# Controlled original-save differential (v1)

Claim level: **structural comparison of two independently generated original
saves at the same recorded scenario/company start**. Established 2026-11-25,
milestone 4. Implemented by `rt3ctl save diff --path-a <first> --path-b
<second> --json`.

## Pair provenance

Both saves started a fresh British Isles scenario, dismissed the briefing,
created the default company Great Republic, and saved at January 1829 with
$1,100K and no deliberate track/train/economic action. The second save's final
Save button was activated by the owner after the automation left the map
waiting for confirmation. Save bytes remain private and untracked.

| Identity | First | Second |
|---|---|---|
| File | `rt3engine-bi-18290101-202.gms` | `rt3c-bi-1829-02.gms` |
| Bytes | 10,285,120 | 10,315,724 |
| SHA-256 | `f892bc28ea1fb3bb1ff60f3ed443e82de7818d3fd7a7c26a93e974cfb69ac6bc` | `225a98ff8939ae3b4ff5fb6f36cc4b730042017d83f379333d55d783ab0e75b6` |
| Source map | `British Isles.gmp` | `British Isles.gmp` |
| Companies | 3 | 3 |
| Players | 3 | 3 |
| Buildings | 520 records, 530 capacity | 543 records, 550 capacity |

The second file is 30,604 bytes larger. This is a measured difference, not a
claim that time or a particular action caused it.

## Section deltas (second minus first)

| Section | Start offset | End offset | Capacity | Records |
|---|---:|---:|---:|---:|
| Terrain | 0 (same dimensions) | — | — | — |
| Tiles | −138 | −1,210 | 0 | 0 |
| Buildings | +3,586 | +26,611 | +20 | +23 |
| Companies | +30,595 | +30,595 | 0 | 0 |
| Players | +30,595 | +30,595 | 0 | 0 |
| Cities | +30,595 | +30,595 | 0 | 0 |
| Events | +30,595 | +30,595 | 0 | 0 |

The shared +30,595 shift across companies/players/cities/events is separated
from the building and tile changes rather than treating the whole file as one
opaque blob.

## Actor record differences

Both liquidated AI companies are byte-identical. Great Republic has one changed
byte at record offset 55. Founding years remain `[1830, 1830, 1829]` and actor
names are unchanged.

| Actor record | Changed bytes | Differing offsets |
|---|---:|---|
| Edinburgh Express | 0 | — |
| London Rails | 0 | — |
| Great Republic | 1 | 55 |
| George Hudson | 3 | 485, 486, 487 |
| Isambard Kingdom Brunel | 2 | 486, 487 |
| Sir John Forrest | 0 | — |

The command reports only counts and offsets, not original record bytes. These
locations are differential anchors for later semantics; they are not assigned
cash, date, AI or economic meanings from coincidence.

## Reproduce

```sh
export RT3_CONTROL_SAVE="$PWD/review-tmp/saves/rt3engine-bi-18290101-202.gms"
export RT3_CONTROL_SAVE_2="$PWD/review-tmp/saves/rt3c-bi-1829-02.gms"
cargo run -q --release -p rt3ctl -- save diff \
  --path-a "$RT3_CONTROL_SAVE" --path-b "$RT3_CONTROL_SAVE_2" --json
make evidence-check
```

The exact derived artifact is
[controlled-save-diff-v1.json](controlled-save-diff-v1.json).
