# Headless orders, order log and checkpoints (v1 — declared model)

Claim level: **declared order model**. The order set, the built track geometry
and the checkpoint format are declared; original build costs, placement rules
and command semantics are not recovered. Established 2026-11-28, milestone 6,
claim C4.

## Declared rules

- **Station references.** `sim init` creates a checkpoint over station
  references: explicit `--station <slot>:<label>` entries, or the records of a
  save's station section (`--stations-from <save>`, label `station-<slot>` by
  default). Labels and the slot-to-station meaning are declared, not decoded.
- **Build track.** `sim build-track` builds a straight declared path between
  the two referenced station slots, with evenly spaced pieces at most 30 units
  apart and a declared length in `2.0 ..= 100_000.0` units. The from and to
  slots must exist and differ. This is not original geometry.
- **Buy train.** `sim buy-train` records a locomotive name and car names from
  the command line. No price, capacity or performance is inferred.
- **Assign schedule.** `sim assign-schedule` builds the two-stop schedule over
  the built path, with the declared dwell at the origin and no dwell at the
  destination, and validates it with the C3 run model.
- Building a track or buying a train clears any existing schedule; a schedule
  must be assigned after both.

## State, log and checkpoints

The canonical state text (`engine::orders::state_line`, fixed order and six
decimal places for distances) escapes spaces, `|`, backslashes and control
characters, so distinct names cannot produce the same text. It is hashed once
with SHA-256. The checkpoint is JSON:

```json
{"version":1,"seq":3,"log_sha256":"…","state_sha256":"…","state":{…}}
```

The order log is one JSON line per command:

```json
{"seq":1,"order":{"build-track":{"from":0,"to":1,"length":120.0}},"state_sha256":"…"}
```

Each command verifies the log against the checkpoint's `log_sha256`, applies
the order, appends the line with the new state hash and advances the
checkpoint. `sim replay` loads a checkpoint, verifies the log prefix against
`log_sha256`, re-applies every later line in sequence, and checks each line's
recorded `state_sha256`. A resumed replay and a replay from the initial
checkpoint therefore end byte-identically.

## Synthetic result

Two declared stations, a 120-unit track, `PlanetL` with cars `242 A1` and
`2D2`, dwell 2:

| Seq | Order | State SHA-256 |
|---|---|---|
| 0 | init | `0de50086fa86eaec311d60d5e634d54c47a31bf53d318607c14c50657527dbcd` |
| 1 | build-track | `10532b067a3673f7a7b2789ccff300256fab1c81aa84cb123ab0762d87ef82e3` |
| 2 | buy-train | `7bd417b98953f661eedaa54b01d9cf8af08289ee05bc396816353bce93de4762` |
| 3 | assign-schedule | `9c526411bf691fbedc5f756d9d302ff271489f6593c57caf2c7fb56f0829a23c` |

A replay from the seq-1 checkpoint and a replay from the seq-0 checkpoint print
identical output and the same final hash.

## Fixture result

The populated fixture's station section supplies 3 references; slots 0 and 1
are declared Bath and Gloucester, slot 2 is declared Tower. With the C3
fixture consist (`PlanetL` plus the first four typed car display names
`242 A1`, `242 A1`, `2-D-2`, `E-88`) and the default 5-tick origin dwell,
building a 2,054.776-unit track between slots 0 and 1 reaches final hash
`c53351cdb88921370785c32a2b9857f718c5147f15a3c9c3739cb0e5f463f857`. The
built path's measured length is `2054.77587890625` under f32 piece origins,
not the decoded C2 path length; the declared geometry is not the original
track.

## Open / divergences

- Original build rules, costs, station placement and command semantics are not
  recovered; the straight track and all labels are declared.
- The commands record and validate a schedule but do not run it; movement and
  the C5 cash flow are separate claims.
- File writes are not transactional: a failure between the log and checkpoint
  writes leaves the pair inconsistent and the next command detects it through
  the log hash.
- A checkpoint is rejected unless its state re-validates: non-empty station
  labels with unique slots, non-empty consist names, track endpoints that
  exist and differ, and schedule stops that reference existing stations and
  match the track endpoints. Duplicate station labels are allowed; the slot is
  the identity.
- No economy, company or AI state is part of the order checkpoint.

## Reproduce

```sh
export RT3_TRACK_SAVE="review-tmp/saves/b1.gms"
cargo test -p rt3ctl --test sim_orders
```
