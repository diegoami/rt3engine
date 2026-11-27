# Implementation review — PR #63 (round 3)

- **Revision:** `ccb44f6c396d5fb5db0c51d24c64aa303e84f1fc`
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
  - `reviews/45-rail-sections-impl-01.md`
  - `reviews/45-rail-sections-impl-02.md`
- **How obtained:** Re-fetched `origin/main` and `origin/feat/rail-sections`.
  `gh pr view 63 --json headRefOid,baseRefOid,files` reported this revision,
  base and 17 paths; local `HEAD` and `origin/feat/rail-sections` match the
  reported head. `git merge-base main ccb44f6c396d5fb5db0c51d24c64aa303e84f1fc`
  matches the reported base, and `git diff --name-only <base>..<head>` matches
  the PR list. The delta since round two is only
  `docs/evidence/gmc-save-v1.md`.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; project slot records `design: none`
  and delegated merge on AGREE.

## Round 2 finding

1. **Resolved — round-two blocking finding.** The reproduction block now uses
   two valid commands at `docs/evidence/gmc-save-v1.md:165-166`. With
   `RT3_TRACK_SAVE` and the other documented fixtures set,
   `cargo test -p rt3-kit save` passed 6 tests including
   `save::tests::reads_populated_track_save`; `cargo test -p rt3-kit rail`
   passed 5 tests including
   `rail::tests::reads_all_track_sections_from_the_populated_fixture`. Neither
   emitted the populated-fixture skip message.

## Verification

- `$HOME/tools/bin/make fmt`, `make clippy`, `make test` and
  `make evidence-check` passed with `RT3_HOME` and all four save fixtures set.
  The full suite passed 5 economy, 4 world, 66 `rt3-kit`, 10 `rt3ctl`,
  1 `sim_compare` and 2 `sim_run` tests. Install-backed and populated-fixture
  checks did not skip; evidence matched its producers.
- PR CI `check` completed successfully on this revision
  (run `46004837877`). The PR body retains its four required sections and
  `Refs #62`; `closingIssuesReferences` is empty. No save bytes or newly added
  machine-specific paths are committed. No emulator was booted and no
  temporary edits were made.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
