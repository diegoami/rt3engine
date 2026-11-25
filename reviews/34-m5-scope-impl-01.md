# Implementation review — M5 scope

- **Revision:** `28c9ba63514a1a2896df2d7815f4995c81afe470`
- **Pull request:** [#45](https://github.com/diegoami/rt3engine/pull/45), base `main` at `d39ef8bb3c71dae8ee5ba3423973161bb3530a5f`
- **Files checked:** `docs/milestones.md`
- **How obtained:** `gh pr view 45 --json headRefOid,baseRefOid,files`; compared with the local diff from the merge base to the reported head. Local `HEAD` matched the PR head; the file list matched.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`.

## Findings

1. **blocking — C1's proving check does not cover the claimed world.** `docs/milestones.md:38-42` includes industry buildings, cargo catalogue and terrain, but the check only says it asserts city and building counts. It does not say how the engine's cargo catalogue or terrain are checked. Also, the cited structural building parser does not establish industry semantics: `docs/evidence/gmp-collections-v1.md:61-63` leaves building types, production and city links open. Narrow the claim to parser-proven structural records, or specify checks and evidence that establish the claimed industry, cargo and terrain data.
2. **blocking — the determinism checks do not prove C3's full claim or the gate.** `docs/milestones.md:49-56` says a different seed must change the dump, but C3 names only a golden-hash replay test and two same-map CLI runs; it does not explicitly require a changed-seed assertion. C4 checks a decade on both maps but names schema/stop-reason assertions, not a repeated run and byte comparison. Thus the checks do not establish the ladder's deterministic multi-year run on at least two maps (`docs/milestones.md:17`, `71-72`). Name checks that assert seed variation and compare repeated decade dumps on both maps.
3. **blocking — the not-in-release list omits milestone 8 save entry.** `docs/milestones.md:65-69` excludes original save writing, but not the distinct original-save entry in the ladder (`docs/milestones.md:20`). C5's comparison with a controlled save is not save entry; state that M8 capability is out of this release.
4. **non-blocking — the exclusion list misattributes stations.** `docs/milestones.md:65-67` assigns stations to milestone 6, but the ladder puts stations/buildings in milestone 10 (`docs/milestones.md:21`); milestone 6 includes schedules (`docs/milestones.md:18`). Correct the cross-reference.

## Verification

- `make fmt`, `make clippy`, and `make test` passed with the requested installation and fixtures. `make test` ran 59 `rt3-kit` tests and 10 `rt3ctl` tests; the evidence check passed. No emulator was booted.
- CI checks on this revision passed (`check` and GitGuardian Security Checks).
- The merge-base diff adds the M5 section only (45 additions, no deletions). The base has no prior M5 claim, so no earlier M5 claim was weakened or silently changed. C5's boundary is otherwise explicit: unproven original constants are to be documented, and bit-exact fidelity is out of scope.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
