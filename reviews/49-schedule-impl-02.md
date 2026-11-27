Revision: `89d92ad1c34ba037f1daefa09ad7cb6524717b9f`

Files: PR #67 file list from `gh pr view 67 --json headRefOid,baseRefOid,files`, verified against `git diff --name-only eb5ce9180459fa8dba5b40bc7dcea2d8277d4b18..89d92ad1c34ba037f1daefa09ad7cb6524717b9f`:

- `crates/engine/src/lib.rs`
- `crates/engine/src/movement.rs`
- `crates/engine/src/schedule.rs`
- `crates/engine/tests/schedule.rs`
- `docs/evidence/README.md`
- `docs/evidence/movement-v1.md`
- `docs/evidence/schedule-v1.md`
- `reviews/49-schedule-impl-01.md`

Target proof: fetched PR head and local `HEAD` equal the revision above; `git merge-base main <head>` equals the PR-reported base `eb5ce9180459fa8dba5b40bc7dcea2d8277d4b18`; the merge-base diff list equals the PR file list. The new commit changes only the three files from the implementation review that contain these fixes.

Reviewer: GPT-6 Luna  
Model: `opencode-go/gpt-6-luna#high`  
Mode: OpenCode

## Round 1 finding outcomes

1. **resolved (blocking)** — `crates/engine/src/schedule.rs:174-184`; regression test `crates/engine/tests/schedule.rs:192-223`. `Train::new` now records `Some(0)` for a zero-dwell origin. The test runs the train to completion and confirms that departure remains tick 0, there are exactly two stop records, and the next stop arrives later. An independent run reproduced this without a duplicate or overwritten origin departure.

2. **resolved (non-blocking)** — `crates/engine/src/schedule.rs:91-108, 168-172`; regression test `crates/engine/tests/schedule.rs:226-262`. `Train::new` normalizes an accepted near-equal final distance to `path.length()`. `Schedule::new` has already required finite positive path length, bounded stops, and strict ordering; the accepted final stop is never greater than the path end, so normalization can only move it forward to that boundary and cannot make it out of bounds or violate ordering. The 29.9999995-on-30.0 test now finishes with movement stopped at exactly 30.0 and a `30.000` dump. An independent check also confirmed a final distance above 30.0 is rejected.

3. **resolved (non-blocking)** — `docs/evidence/schedule-v1.md:20-24, 69-71`. The evidence now says capacity is not modelled, and documents zero-dwell departure at arrival tick, including the origin at tick 0.

No remaining findings.

## Verification

- `make fmt`, `make clippy`, `make test`, and the separate `make evidence-check` passed with the supplied install and fixtures. No tests skipped: schedule 7/7, movement 5/5, `rt3-kit` 68/68, and all remaining workspace suites passed.
- C2 movement and C3 fixture tests passed with their existing goldens. An independent run reproduced the synthetic digest `073f73bf46ce7b2aa6f3c192e0e5210610b1dac3a94fc30e4faa0ea4b5591dd8` and fixture digest `b272a9a6a88cb2fc39c5dc2c20924fde728eccda0b89d65e0157ac5833e9a419`.
- CI and GitGuardian checks on this revision are green. No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

- M6 C3 lands: `engine::schedule` assembles a consist from typed catalogue
  records, builds a validated schedule between two station-DB records and runs
  it as ordered stops with declared dwell, arrival/departure records and a
  golden dump. On the private fixture Bath departs at tick 5 and Gloucester is
  reached at tick 196.
- PR #67 merged as `3d2a06b`. C4–C6 are not started; no version is assigned.
- Reviews: `reviews/49-schedule-impl-01.md` (BLOCK),
  `-02.md` (AGREE). Gates and CI green.

— Implementer (zai-coding-plan/glm-5.3)
