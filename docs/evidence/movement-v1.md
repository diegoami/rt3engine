# Consist movement (v1 — declared model)

Claim level: **declared movement model**. The speed, acceleration and braking
rules are a declared model, not recovered original train physics. Established
2026-11-27, milestone 6, claim C2, on the track graph from
[rail-v1.md](rail-v1.md).

## Declared rule

| Parameter | Value | Meaning |
|---|---|---|
| `TICK_SECONDS` | 1.0 | simulated seconds per tick |
| `MAX_SPEED` | 12.0 | cruise speed in track-world units per second |
| `ACCELERATION` | 2.0 | acceleration and braking rate, units per second squared |

One tick: the consist targets cruise speed unless the remaining path is inside
its stopping distance `v² / (2a)`, in which case it brakes at `a`; the speed
changes by at most `a` per tick, the distance advances by `speed × 1 s`, and
the position clamps to the path end where the consist stops. A scheduled run
may brake to an intermediate limit instead of the path end; the same rule
applies, and the consist then dwells and departs again (C3). There is **no
randomness**, so replaying the same path for the same number of ticks is
byte-identical and a seed has no effect.

The path is the unique walk between the graph's two degree-1 endpoints; it is
rejected unless the graph has exactly two such endpoints and the walk visits
every piece. Travel starts at the endpoint with the lower piece index — a
declared direction, not a recovered travel direction. One JSON state line per
tick records `tick`, `piece` (the decoded track record index), `distance`,
`speed`, interpolated `x`/`y`/`z`, and `stopped`. The golden hash is SHA-256
over those lines, one trailing newline each.

## Fixture result

On the private populated fixture (British Isles, SHA-256 `2a8f6e72…`, 61 track
pieces), the graph yields a 61-piece path from piece `0` (Bath end) to piece
`9` (Gloucester end). The declared model runs **2,054.776 units in 191 ticks**
and stops at the path end. The state-line digest is:

```text
9bbdfd39b16df265f5650b32178d72b45fc2239d085bad20bfac6bee13176cf6
```

The same path replayed twice produces identical lines and the same digest.

## Synthetic checks

- A four-piece 90-unit straight path accelerates by 2 units/s per tick to the
  12-unit cruise, never exceeds it, stays within the path, and stops at the
  end.
- An eight-piece toy with two runs and two declared joins follows the join
  order, visits every piece, and ends on the last piece.
- Branched, disconnected and empty graphs are rejected.

## Open / divergences

- The constants are declared; the original's train performance, power, mass,
  friction and braking data are not recovered.
- Distances are planar 3D distances between decoded piece origins, under the
  declared connectivity rule. The origin axis order and the piece
  orientation/endpoint fields remain open in
  [#62](https://github.com/diegoami/rt3engine/issues/62); grades, curves and
  per-piece geometry are not modelled.
- Every consist uses the same parameters. Per-consist performance and
  schedules belong to C3.

## Reproduce

```sh
export RT3_TRACK_SAVE="review-tmp/saves/b1.gms"
cargo test -p engine --test movement
```
