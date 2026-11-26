# Implementation review — PR #55 (round 2)

- **Revision:** `095bfa6f85c25add9fbeb24c4fe88d6669d265f3`
- **Pull request:** [#55](https://github.com/diegoami/rt3engine/pull/55),
  base `main` at `e0f5f0978007907ddb91532c99a9fd5bd0eee61a`
- **Files checked (PR file list):**
  - `crates/rt3-kit/src/cities.rs`
  - `crates/rt3-kit/src/events.rs`
  - `crates/rt3-kit/src/sparse.rs`
  - `reviews/41-sparse-rejection-impl-01.md` (round 1 record)
- **How obtained:** Re-fetched `origin/test/sparse-rejection`;
  `gh pr view 55 --json headRefOid,baseRefOid,files` reported this revision,
  base and four paths, matching local `HEAD` and `FETCH_HEAD`.
  `git merge-base main 095bfa6f85c25add9fbeb24c4fe88d6669d265f3` equals the
  reported base, and `git diff --name-only` from that merge base matches the
  PR file list. Compared with round 1, the implementation update is confined
  to `crates/rt3-kit/src/sparse.rs`; the other paths are unchanged.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Findings

No new findings.

## Round 1 follow-up

1. **Resolved — round 1 blocking finding 1.** `crates/rt3-kit/src/sparse.rs:234-243`
   now uses mask `0x0f` for capacity 4 and `active = 2`, with the terminator
   immediately after the mask. The parser treats set bits as absent
   (`crates/rt3-kit/src/sparse.rs:100-103`), so no records are read and the
   end tag is correctly aligned. The final `records.len() != active` check at
   `crates/rt3-kit/src/sparse.rs:118` is the rejection. Removing only that
   check made the test fail with `accepted a mask with fewer records than
   declared`.

## Verification

- The round-2 diff changes only sparse-reader test setup and assertions; the
  parser, format claims and shipped fixtures are unchanged. The refactored
  sample retains its level-table, payload and mask offsets.
- The distinct blob-length fixture has one active slot, declares five bytes,
  supplies all five, and aligns the end tag. Removing only the length guard at
  `crates/rt3-kit/src/sparse.rs:109` made the targeted test fail with
  `accepted a five-byte blob`.
- The slot-limit fixture has capacity 1025, no active records, all 129 mask
  bytes present and set, and an aligned end tag. Removing only the capacity
  limit at `crates/rt3-kit/src/sparse.rs:72` made the targeted test fail with
  `accepted capacity over the slot limit`.
- The under-count, wrong-length and slot-limit removals all ran only
  `sparse::tests::rejects_bad_counts_wrong_blob_length_and_oversized_capacity`;
  each failed at its named assertion. Removing only the active-over-capacity
  bound left that test green because the final count check rejects its buffer,
  matching the stated regression-coverage limitation. All temporary edits
  were reverted.
- `$HOME/tools/bin/make fmt`, `make clippy` and `make test` passed with
  `RT3_HOME` and the documented repo-relative fixtures under
  `review-tmp/saves/`. Results match the PR body: 5 economy, 4 world,
  61 `rt3-kit`, 10 `rt3ctl`, 1 `sim_compare` and 2 `sim_run` tests passed.
  The install-backed city, building and event tests passed; none skipped.
- CI `check` and GitGuardian Security Checks passed. The PR body’s three new
  guard-removal rows G, H and I match the reproduced failures. No emulator was
  booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
