Revision: `ae6d4d932d1c382aabed6e415e2a5e2e0b21698d`

Files: PR #67 file list from `gh pr view 67 --json headRefOid,baseRefOid,files`, verified against `git diff --name-only eb5ce9180459fa8dba5b40bc7dcea2d8277d4b18..ae6d4d932d1c382aabed6e415e2a5e2e0b21698d`:

- `crates/engine/src/lib.rs`
- `crates/engine/src/movement.rs`
- `crates/engine/src/schedule.rs`
- `crates/engine/tests/schedule.rs`
- `docs/evidence/README.md`
- `docs/evidence/movement-v1.md`
- `docs/evidence/schedule-v1.md`

Target proof: local `HEAD` equals the PR head above; `git merge-base main <head>` equals the PR-reported base `eb5ce9180459fa8dba5b40bc7dcea2d8277d4b18`; the diff file list equals the PR file list.

Reviewer: GPT-6 Luna  
Model: `opencode-go/gpt-6-luna#high`  
Mode: OpenCode

## Findings

1. **blocking** — `crates/engine/src/schedule.rs:176-183, 198-205`: A valid schedule with zero origin dwell never records its departure. `Train::new` initializes the origin departure to `None`; `step` only fills it inside the positive-dwell countdown. Reproduced with `Schedule::new(... origin dwell_ticks: 0 ...)`: after movement begins, the origin departure remains `None`, so the dump says `null` for a non-final stop instead of tick `0`. Record the zero-dwell departure and add a regression assertion.

2. **non-blocking** — `crates/engine/src/schedule.rs:105, 168-170`: The endpoint tolerance accepts a final stop short of the path end, but `Train::finished()` then becomes true without `Movement::stopped`. Reproduced with path length `30.0` and final distance `29.9999995`: the train finishes at `29.9999995` with `movement.stopped == false`. Reject the mismatch or normalize an accepted near-equal endpoint to the exact path length.

3. **non-blocking** — `docs/evidence/schedule-v1.md:69`: The evidence says consist capacities “are declared,” but `Consist` in `crates/engine/src/schedule.rs:24-46` has no capacity field or capacity rule; the PR body correctly says no capacity is inferred. Clarify that capacity is not modeled, rather than described as declared.

## Verification

- `make fmt`, `make clippy`, `make test`, and `make evidence-check` passed using the supplied install and fixtures. No test skipped: schedule 5/5, movement 5/5, `rt3-kit` 68/68, and the remaining workspace suites passed. The CI workflow and GitGuardian checks are green.
- Independently reproduced the synthetic dump and digest `073f73bf46ce7b2aa6f3c192e0e5210610b1dac3a94fc30e4faa0ea4b5591dd8`; replayed dumps were equal.
- Independently parsed `review-tmp/saves/b1.gms`: 61-piece path, three station records, declared slots 0/1 Bath/Gloucester, typed locomotive `PlanetL`, and the first four sorted `.car` records used by the test (`242_A1_L.car`, `242_A1_T.car`, `2D2L.car`, `88 Tender.car`). The run produced Bath departure 5, Gloucester arrival 196, and digest `b272a9a6a88cb2fc39c5dc2c20924fde728eccda0b89d65e0157ac5833e9a419`. No save bytes were emitted or committed.
- The C2 movement test retained its golden hash. Temporarily changing `DEFAULT_DWELL_TICKS` from 5 to 4 made the fixture test fail at arrival (`195` vs `196`); after restoring it, the focused fixture test passed and the worktree was clean.
- Evidence describes Bath/Gloucester slots, labels, distances and car pairing as declared, not recovered. The changed evidence files contain no save bytes, installed-save content, or machine-specific absolute paths.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
