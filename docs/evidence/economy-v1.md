# Declared cargo economy (v1)

Claim level: **declared model over parser-proven map structure** — milestone 5,
claim C2. Established 2026-11-25. Implemented by `engine::economy`.

The original's economy constants are not recovered. This document records the
rules the native kernel actually runs, so the model is inspectable and its
divergences can be named rather than implied. It is not a claim that the
original game computes these formulas.

## Inputs and provenance

| Input | Source | Evidence |
|---|---|---|
| Cargo base price and factor | `CargoTypes/*.cty` (`price`, `factor`) | [typed-formats-v1.md](typed-formats-v1.md), [typed-tables-v1.json](typed-tables-v1.json) |
| City count | map city collection | [terrain-scan-v1.json](terrain-scan-v1.json) |
| Placed building count | map building collection | [terrain-scan-v1.json](terrain-scan-v1.json) |
| Cargo count and order | `CargoTypes/*.cty` records | [typed-tables-v1.json](typed-tables-v1.json) |

City and building *semantics* — which building makes what, which city wants
what — are not established by the format evidence. The rules below assign them
by a declared index convention and must not be read as game behaviour.

## Rule table

| Rule | Formula or value | Status |
|---|---|---|
| Tick length | 1 simulated month = 30 ticks | declared |
| Initial stock | 10,000 units per cargo | declared scale |
| Stock cap | 10 × initial stock (100,000 units) | declared |
| Production assignment | building index `j` produces cargo `j mod cargo_count` | declared convention |
| Production rate | 1 unit per building per tick | declared |
| Demand | each city consumes 0.04 units of every cargo per tick | declared |
| Stock update | `stock' = clamp(stock + production - demand + noise, 0, cap)` | declared |
| Noise | SplitMix64(seed), uniform in ±2 units, one draw per cargo per tick in cargo-index order | declared |
| Price | `base × (1 + factor × (1 - stock / initial_stock))` | declared; base and factor from `.cty`, factor clamped to [0, 1] when loaded |
| Price bounds | 0.5 × base to 1.5 × base | declared |
| Dump | one JSON line per month: month, then per-cargo name, stock and price to two decimals in cargo-index order | declared |

## Determinism

The stock scale is chosen so the declared model does not saturate within a
decade on the shipped maps; it is a model parameter, not a recovered original
constant.

The same world, seed, rules and month count produce byte-identical monthly
dumps. Cargo is processed in index order and the generator draw order follows
it, so no hash-map iteration or wall-clock input can change the result. The
engine tests assert the same-seed replay, a changed-seed divergence, the stock
identity with noise disabled, bounds over 120 months, the stock cap and both
price clamps reached by the committed overproduction and overconsumption
scenarios (`economy-cap.txt`, `economy-drain.txt`), and a stable 12-month
golden hash on the committed synthetic scenario
`crates/engine/tests/fixtures/economy-scenario.txt`.

## Divergence (C5)

`rt3ctl sim compare "British Isles" --save <controlled save> --json` compares the
kernel's start state with the controlled original save at January 1829. Measured
on the M5 candidate with `rt3engine-bi-18290101-202.gms`:

| Item | Kernel (map) | Controlled save | Cause |
|---|---:|---:|---|
| Terrain dimensions | rows 513, cols 321 | rows 513, cols 321 | same source map; no divergence |
| City records | 68 | 68 | counts agree; city attributes are not modelled |
| Placed building records | 0 | 520 | the map's placed-building collection is empty; the 520 records are present only in the save |
| Companies | 0 (not modelled) | 3 | company records are save state and out of M5 |
| Cargo catalogue | 41 types | not in the save | shared typed data from the installation |

Model rules with no original-constant provenance, all of which are divergences
from the original game rather than reproductions of it:

- production assignment (building index `j` → cargo `j mod cargo_count`) and the
  1-unit-per-building-per-tick rate;
- the stock update `stock' = clamp(stock + production - demand + noise, 0, cap)`
  and its clamps;
- demand of 0.04 units per city per cargo per tick;
- 10,000 initial stock and the 10× stock cap;
- ±2 units of SplitMix64(seed) noise per cargo per tick;
- the price formula and its 0.5×–1.5× base bounds;
- the `factor` clamp to [0, 1] when the catalogue is loaded (all 41 shipped
  values lie inside it);
- 30-tick months.

Consequence on British Isles: the map carries no placed building records, so
declared production is zero and every cargo drains over the decade; the save's
520 building records and three companies are not modelled. The
comparison establishes agreement in terrain dimensions and city record counts
and names the remaining differences rather than reproducing the original
economy.

## Reproduce

```sh
cargo test -p engine --test economy
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
export RT3_CONTROL_SAVE="review-tmp/saves/rt3engine-bi-18290101-202.gms"
cargo test -p rt3ctl --test sim_compare
cargo run -q --release -p rt3ctl -- sim compare "British Isles" \
  --save "$RT3_CONTROL_SAVE" --json
```
