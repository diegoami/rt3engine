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
| Initial stock | 1,000 units per cargo | declared |
| Stock cap | 10 × initial stock | declared |
| Production assignment | building index `j` produces cargo `j mod cargo_count` | declared convention |
| Production rate | 10 units per building per tick | declared |
| Demand | each city consumes 5 units of every cargo per tick | declared |
| Stock update | `stock' = clamp(stock + production - demand + noise, 0, cap)` | declared |
| Noise | SplitMix64(seed), uniform in ±2 units, one draw per cargo per tick in cargo-index order | declared |
| Price | `base × (1 + factor × (1 - stock / initial_stock))` | declared; base and factor from `.cty` |
| Price bounds | 0.5 × base to 1.5 × base | declared |
| Dump | one JSON line per month: month, then per-cargo name, stock and price to two decimals in cargo-index order | declared |

## Determinism

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

Claim C5 compares the kernel's British Isles start state with the controlled
January 1829 original save and lists every difference, including the declared
rules above that have no original-constant provenance. Those findings are
appended here when C5 lands; they are already model assumptions, not recovered
original behaviour.

## Reproduce

```sh
cargo test -p engine --test economy
```
