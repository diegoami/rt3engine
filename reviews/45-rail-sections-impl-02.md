# Implementation review — PR #63 (round 2)

- **Revision:** `dae98ac48391e8aa0e1f0b558411d4f6d982b1aa`
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
- **How obtained:** Re-fetched `origin/main` and `origin/feat/rail-sections`.
  `gh pr view 63 --json headRefOid,baseRefOid,files` reported this revision,
  base and 16 paths; local `HEAD` and `origin/feat/rail-sections` match the
  reported head. `git merge-base main dae98ac48391e8aa0e1f0b558411d4f6d982b1aa`
  matches the reported base, and `git diff --name-only <base>..<head>` exactly
  matches the PR list: the original 15 implementation paths plus the
  round-one review record.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; project slot records `design: none`
  and delegated merge on AGREE.

## Round 1 findings

1. **Resolved — round-one non-blocking finding 1.** The PR body now says text
   mode prints counts and JSON includes offsets, capacities and trailer sizes.
   This matches the unchanged text formatter at
   `crates/rt3ctl/src/save_cmd.rs:31-48`.
2. **Blocking — round-one non-blocking finding 2 remains unresolved.**
   `docs/evidence/gmc-save-v1.md:164-169` now exports `RT3_TRACK_SAVE`, but its
   test command at `:165`, `cargo test -p rt3-kit save rail`, fails before
   running tests: Cargo reports `unexpected argument 'rail'`. Use valid
   separate test commands or one valid filter so the documented reproduction
   actually runs the populated-fixture check.
3. **Resolved — round-one non-blocking finding 3.** Both collection docs now
   define `end_offset` as the first ending tag before any trailer
   (`crates/rt3-kit/src/fixed.rs:19-20`,
   `crates/rt3-kit/src/sparse.rs:26-27`); `sparse.rs:134-152` captures that
   offset before scanning the trailer.

## Verification

- Reproduced the documented test command with `RT3_TRACK_SAVE` set; it exits
  with Cargo's `unexpected argument 'rail'` usage error. The remaining commands
  in the block ran: both save inventories, the controlled-save diff and
  `make evidence-check` completed successfully.
- Independently walked `b1.gms` again and compared first-tag offsets, counts
  and trailer lengths with `save inspect --json`: stations `5703718` / 3 /
  867 B; cars `5708631` / 4 / 718 B; trains `5710521` / 1 / 110 B; track
  `5839049` / 61 / 339 B; track lines `5839673` / 5 / 0 B. All five JSON
  offsets now match the byte walk. The fixture SHA-256 remains
  `2a8f6e723fffcdd7cf9d82fbcc5e9f5b309918108687f8ae985093803830a27a`.
- The regenerated save-inventory artifacts match their producers:
  `$HOME/tools/bin/make evidence-check` passed. The empty-save cars and track
  offsets now point to their first ending tags. The populated fixture still
  parses with the same counts and trailers.
- The station note at `docs/evidence/gmc-save-v1.md:137-140` describes
  `gpdStationDB` as including the trackside service tower, records type-like
  values 3, 8 and 23, and leaves the field meaning unproven.
- `$HOME/tools/bin/make fmt`, `make clippy`, `make test` and
  `make evidence-check` passed with `RT3_HOME` and all four save fixtures set.
  `make test` passed 5 economy, 4 world, 66 `rt3-kit`, 10 `rt3ctl`,
  1 `sim_compare` and 2 `sim_run` tests; install-backed and populated-fixture
  checks did not skip.
- PR CI `check` completed successfully on this revision. The PR body still has
  its four required sections and `Refs #62`; `closingIssuesReferences` is
  empty. No save bytes or newly added machine-specific paths are committed.
  No emulator was booted and no temporary edits were made.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
