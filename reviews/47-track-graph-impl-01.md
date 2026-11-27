# Implementation review — PR #65 (round 1)

- **Revision:** `4e02d57009587646436b5e71db41216a07a00dae`
- **Pull request/base:** [#65](https://github.com/diegoami/rt3engine/pull/65),
  base `46a003c35ee1d77705aa692d529209c1c5bfc206`
- **Files checked (PR file list):**
  - `crates/rt3-kit/src/rail.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/rail-v1.md`
  - `docs/milestones.md`
- **How obtained:** `gh pr view 65 --json headRefOid,baseRefOid,files`
  reports this head, base and file list; local `HEAD` equals the reported head.
  `git merge-base main 4e02d57009587646436b5e71db41216a07a00dae` equals the
  reported base, and `git diff --name-only <base>..<head>` matches the PR list.
  I read the full merge-base diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; the project slot records
  `design: none` and merge delegated on AGREE.

## Verification

- C1's declared rule matches the fixed scope: split on origin steps **above
  100 units**, use the median within-run step as `piece_length`, and join run
  ends within one `piece_length`. The implementation makes no station-link or
  recovered original-connectivity claim; axis, orientation/endpoint and station
  fields remain open.
- An independent raw walk of the private fixture (not using the parser)
  verified the expected SHA-256 prefix, 61 sparse 599-byte blobs, and the
  three little-endian f32 origin values at blob `+12`. The measured runs are
  `0..5`, `5..10`, `10..61`; the median 3D step is `34.234776`; the joins are
  `(4, 10)` and `(60, 5)`; and there are 60 edges. The evidence's grouped
  coordinate ranges also match the fixture.
- The synthetic test exercises both a break and a join. Temporarily changing
  `RUN_BREAK_UNITS` from 100 to 1 made the populated fixture test fail at its
  expected-runs assertion; I restored the source and confirmed the worktree was
  clean.
- `$HOME/tools/bin/make fmt`, `clippy`, `test` and `evidence-check` passed with
  `RT3_HOME` and all save fixtures supplied. The workspace test totals were
  5 economy, 4 world, 68 `rt3-kit`, 10 `rt3ctl`, 1 compare and 2 sim-run tests;
  no fixture or evidence check skipped.
- PR CI `check` and GitGuardian Security Checks passed. No emulator was booted.
- PR #64's completion record says M6 claims were fixed before M6 work. This
  change records the owner's 2026-11-27 opening in the plan heading; no version
  is assigned, and C2–C6 are not implemented here.

## Findings

1. **Blocking — the reproduction snippet publishes a machine-specific
   absolute installation path** (`docs/evidence/rail-v1.md:49`). Keep the
   reproduction portable by requiring the caller to set `RT3_HOME` rather than
   committing the local machine's path.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
