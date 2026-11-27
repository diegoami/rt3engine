# Implementation review — PR #63 (round 1)

- **Revision:** `3e9e4b71393e2d73584623466d516a36d0878d20`
- **Pull request/base:** [#63](https://github.com/diegoami/rt3engine/pull/63),
  base `317ec2290b0b1e8a8b1a1ba87a5597bf23c39c6a`
- **Files checked (PR file list):**
  - `crates/rt3-kit/src/buildings.rs`
  - `crates/rt3-kit/src/companies.rs`
  - `crates/rt3-kit/src/events.rs`
  - `crates/rt3-kit/src/fixed.rs`
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3-kit/src/players.rs`
  - `crates/rt3-kit/src/rail.rs`
  - `crates/rt3-kit/src/save.rs`
  - `crates/rt3-kit/src/sparse.rs`
  - `crates/rt3ctl/src/save_cmd.rs`
  - `docs/environment.md`
  - `docs/evidence/README.md`
  - `docs/evidence/controlled-save-inventory-v1.json`
  - `docs/evidence/gmc-save-v1.md`
  - `docs/evidence/save-inventory-v1.json`
- **How obtained:** `gh pr view 63 --json headRefOid,baseRefOid,files`
  reported this head, base and file list. Local `HEAD` and
  `origin/feat/rail-sections` matched the reported head;
  `git merge-base main 3e9e4b71393e2d73584623466d516a36d0878d20` matched the
  reported base, and `git diff --name-only <base>..<head>` exactly matched
  the PR file list.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; project slot records `design: none`
  and delegated merge on AGREE.

## Findings

1. **Non-blocking — text output is narrower than the PR description says.**
   `crates/rt3ctl/src/save_cmd.rs:31-48` prints the five rail record counts in
   text mode, but not their offsets, capacities or trailer sizes. Running
   `rt3ctl save inspect` on the controlled save reproduced counts only; JSON
   mode includes the other metadata. Correct the PR description or extend the
   text output.
2. **Non-blocking — the documented reproduction skips the populated fixture.**
   `docs/evidence/gmc-save-v1.md:152-163` runs `cargo test -p rt3-kit save`
   without exporting `RT3_TRACK_SAVE`, although the populated-save claim at
   `:130-135` depends on it. Reproducing that block's fixture environment with
   `RT3_TRACK_SAVE` unset printed
   `RT3_TRACK_SAVE unset; skipping populated track-save check`. Add the fixture
   export to the reproduction block so it verifies the new populated-save
   assertions.
3. **Non-blocking — trailer-bearing `end_offset` values have inconsistent
   semantics.** `crates/rt3-kit/src/fixed.rs:18-22,108-130` retains the first
   ending-tag offset, while `crates/rt3-kit/src/sparse.rs:133-152` advances
   `end_offset` to the second ending tag after finding a trailer. On the
   populated fixture, JSON reported the station offset as `5703718` (first
   ending tag) and the car offset as `5709353` (second ending tag); byte
   inspection found their respective first tags at `5703718` and `5708631`.
   Align or document what the shared `end_offset` field means.

## Verification

- Read issue #62 and its comments, including the map-reader qualification,
  the populated fixture's ground truth and the refined framing measurements.
  Verified `review-tmp/saves/b1.gms` is 10,323,174 bytes with SHA-256
  `2a8f6e723fffcdd7cf9d82fbcc5e9f5b309918108687f8ae985093803830a27a`.
- Independently walked the fixture bytes (without using the parser or printing
  save bytes). The section ranges and framing were:
  - Stations `[5698985,5704593)`: 3 × 1,549-byte records; 867-byte trailer.
  - Cars `[5704593,5709357)`: 4 sparse records of 980 bytes each; 718-byte
    trailer.
  - Trains `[5709357,5710639)`: 1 × 473-byte record, with the 50 × 12-byte
    level-1 payload; 110-byte trailer.
  - Track `[5801812,5839396)`: 61 sparse records of 599 bytes each;
    339-byte trailer.
  - Track lines `[5839396,5839677)`: header size 74, 5 stored records of
    36 bytes, no trailer.
- The supplied save and first controlled save passed the added zero-record
  assertions. `save inspect --json` on the second controlled save also
  reported zero stations, cars, trains, track records and track lines.
- Compared the shared-reader paths before and after: cities are unchanged;
  building/event sparse specs retain the prior 993/2,191-byte blob sizes,
  10/20 header constants and single end tag; company/player fixed specs retain
  the prior record/header sizes, 64-slot bound and adjacent two-tag check.
  Their new options preserve those parse conditions.
- All three guard-removal demonstrations made
  `rail::tests::reads_all_track_sections_from_the_populated_fixture` fail:
  track-line stored size 36→74, skipping the train level-1 payload, and
  disabling cars trailer capture. Each temporary edit was restored
  byte-for-byte; the worktree was clean afterward.
- The fixed, sparse and rail synthetic truncation loops assert rejection for
  every strict prefix. Trailer scans are bounded to 64 KiB; fixed capacities
  are per-spec bounded, sparse capacity is limited to 1,024, blob lengths are
  u16 and input-bounded, and reads/reservations use checked or fallible paths.
- `$HOME/tools/bin/make fmt`, `make clippy`, `make test` and
  `make evidence-check` passed with `RT3_HOME` and all four save fixtures set.
  `make test` passed 5 economy, 4 world, 66 `rt3-kit`, 10 `rt3ctl`,
  1 `sim_compare` and 2 `sim_run` tests. No install-backed or populated-fixture
  check skipped; the evidence artifacts matched their producers.
- PR CI `check` completed successfully. The PR body has the four required
  sections and `Refs #62`; `closingIssuesReferences` is empty. No save bytes or
  newly added machine-specific paths are in the change.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
