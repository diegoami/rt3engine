# Scripted rail company and competitor v0 (v1 — declared economy)

Claim level: **declared company economy**. The fare, operating costs and the
competitor's behavior are declared; the original's finance, passenger demand
and AI are not recovered. Established 2026-11-28, milestone 6, claims C5 and
C6, over the schedule model in [schedule-v1.md](schedule-v1.md) and the track
graph in [rail-v1.md](rail-v1.md).

## Declared rules

| Rule | Value | Meaning |
|---|---|---|
| `fare_per_car_trip` | 250.0 | revenue per car for one completed one-way trip |
| `cost_per_tick` | 0.5 | operating cost per simulated tick, including dwell |
| `cost_per_unit` | 0.05 | operating cost per track-world unit travelled |
| `initial_cash` | 0.0 | cash before the first trip |

A trip is one scheduled run from Bath to Gloucester under the C3 model: the
consist dwells at the origin, runs the path and stops at the destination. Each
trip is independent; the consist is reset at Bath rather than returning along
the line. Cash is `initial_cash + revenue - costs` after every trip; the first
1-based trip with positive cash is recorded.

The C5 scenario uses the populated fixture's declared track and stations and
the typed `PlanetL` consist (first four catalogue car display names `242 A1`,
`242 A1`, `2-D-2`, `E-88`), with the default 5-tick origin dwell.

## Competitor v0 (C6)

One declared competitor company runs the same path, schedule, consist and
rules. Its fixed policy is:

| Rule | Value |
|---|---|
| first trip start tick | 300 |
| trip interval | 300 ticks |

Trip `n` (0-based) starts at `start_tick + n × interval`; a trip lasts the same
196 ticks as the player's on the fixture. The companies do not interact: they
share the declared scenario but neither changes the other's revenue, costs or
path. This is a scripted v0, not recovered original AI.

## Results

Synthetic check (120-unit path, two cars, 5-tick dwell, 3 trips, competitor
start 40 and interval 50): each trip takes 20 ticks and costs 16.0 (10 tick
cost + 6 distance cost); revenue is 500.0 per trip. Player and competitor both
end with cash 1,452.0, first profitable trip 1, horizon 160 ticks, digest
`cf2fbff569f06dee8dc1948a8384d80adfb4bcc8f3395a51f3597135c663eb0e`.

Fixture check (`rt3ctl sim company`, 5 trips): each trip takes 196 ticks and
costs about 200.74 (98 tick cost plus 102.73879013449423 distance cost);
revenue is 1,000.0 per trip. Player and competitor both end with revenue
5,000.0, costs 1,003.6939506724713 and cash 3,996.3060493275293; first
profitable trip 1; horizon 1,696 ticks (player end 980; competitor end
300 + 4×300 + 196); joint digest
`2d517847f5b1250fd9fc75cc3f8962b7605df9b92868153337894be8056f7e4c`.

The scripted company therefore reaches positive cash flow within a bounded
run while the competitor operates on the same scenario.

## Divergences from measurable original state

- The fare, per-tick and per-distance costs are declared; no original ticket
  price, crew, fuel, maintenance or overhead data is recovered.
- Revenue is not coupled to the declared cargo supply/demand economy
  (`docs/evidence/economy-v1.md`); no passenger counts, demand or load factor
  are modelled.
- The line carries a single declared consist; the save's own car-to-type
  assignment is not decoded.
- Trips are one-way and teleport the consist back to Bath; no return leg,
  turnaround or timetable conflict exists.
- The competitor has no recovered AI, no revenue stealing and no interaction;
  its only declared behavior is the delayed fixed-interval schedule above.
- Station identity, positions and the station-to-path link remain declared and
  open in [#62](https://github.com/diegoami/rt3engine/issues/62).

## Reproduce

```sh
export RT3_HOME="${RT3_HOME:?set RT3_HOME to the installation root}"
export RT3_TRACK_SAVE="review-tmp/saves/b1.gms"
cargo test -p engine --test company
cargo test -p rt3ctl --test sim_company
target/debug/rt3ctl sim company --track-save "$RT3_TRACK_SAVE" --trips 5 --json
```
