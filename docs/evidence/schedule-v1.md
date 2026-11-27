# Stations, consists and schedules (v1 — declared model)

Claim level: **declared station/consist/schedule model**. Station names and
positions, train composition and timetable data are not recovered original
semantics. Established 2026-11-27, milestone 6, claim C3, over the movement
model in [movement-v1.md](movement-v1.md) and the track graph in
[rail-v1.md](rail-v1.md).

## Declared rules

- **Consist.** Built from one parsed `Locomotive` and parsed `CarType`
  records: the locomotive name and the cars' display names, in train order. No
  capacity, weight or performance is inferred from the catalogue.
- **Station.** A reference to an existing station-DB record slot plus a
  **declared label**. The record's own name and position are undecoded, so the
  label and the path distance belong to the model, not to the format.
- **Schedule.** At least two stops, strictly increasing along the path, the
  first at the path start and the last at the path end, all within the path.
  `Schedule::between` builds the two-stop case from two station-DB records.
- **Run.** Each stop carries a declared dwell; a scheduled run dwells there
  before departing. `Schedule::between` uses `DEFAULT_DWELL_TICKS` (5) at the
  origin and no dwell at the final station. The train travels under the
  declared movement model and stops at the final station. There is no
  randomness.

The schedule dump is one fixed-order JSON line per reached stop:
`{"stop","slot","station","distance","arrival","departure"}`, with `null`
departure at the final station. Its golden hash is SHA-256 over those lines,
one trailing newline each.

## Synthetic result

A 90-unit, four-piece straight path with stops `A` at 0, `B` at 30 and `C` at
90, dwell 2 at the first two:

```text
{"stop":0,"slot":0,"station":"A","distance":0.000,"arrival":0,"departure":2}
{"stop":1,"slot":1,"station":"B","distance":30.000,"arrival":8,"departure":10}
{"stop":2,"slot":2,"station":"C","distance":90.000,"arrival":18,"departure":null}
```

Digest `073f73bf46ce7b2aa6f3c192e0e5210610b1dac3a94fc30e4faa0ea4b5591dd8`.
Replaying the same run produces identical stop records and digest.

## Fixture result

The populated fixture has **3 station-DB records**; the run uses slots `0` and
`1`, declared as Bath and Gloucester, at the declared path start and end. The
consist is the typed `PlanetL` locomotive plus the first four typed `.car`
records in catalogue order (declared v0 composition; the save's own car-to-type
assignment is not decoded). On the C2 path:

```text
{"stop":0,"slot":0,"station":"Bath","distance":0.000,"arrival":0,"departure":5}
{"stop":1,"slot":1,"station":"Gloucester","distance":2054.776,"arrival":196,"departure":null}
```

Digest `b272a9a6a88cb2fc39c5dc2c20924fde728eccda0b89d65e0157ac5833e9a419`.
The 196-tick arrival is the 5-tick origin dwell plus the 191 ticks measured for
the same path in C2.

## Open / divergences

- Station record names, positions and type meanings remain open in
  [#62](https://github.com/diegoami/rt3engine/issues/62); the Bath/Gloucester
  labels, their slots and their path distances are declared.
- The fixture's car-to-type assignment is declared; the save's own composition
  is not decoded.
- Dwell times, consist capacities and per-consist performance are declared, not
  recovered. Revenue and operating costs belong to C5.

## Reproduce

```sh
export RT3_HOME="${RT3_HOME:?set RT3_HOME to the installation root}"
export RT3_TRACK_SAVE="review-tmp/saves/b1.gms"
cargo test -p engine --test schedule
```
