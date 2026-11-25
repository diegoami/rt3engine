# Implementation review — M5 C5 (round 2)

- **Revision:** `f7d6eb072152082531f1d3d0ffa8b05ac8565a24`
- **Pull request:** [#49](https://github.com/diegoami/rt3engine/pull/49), base `main` at `65758edc653d41b768f5f1a37c9592da1cc923c0`
- **Files checked (PR file list):**
  - `crates/engine/src/world.rs`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/sim_cmd.rs`
  - `crates/rt3ctl/tests/sim_compare.rs`
  - `docs/evidence/economy-v1.md`
  - `reviews/38-sim-compare-impl-01.md`
- **How obtained:** Fetched `origin/main` and `origin/m5/compare`; `gh pr view 49 --json headRefOid,baseRefOid,files` and local `HEAD` reported revision `f7d6eb072152082531f1d3d0ffa8b05ac8565a24`, with base `65758edc653d41b768f5f1a37c9592da1cc923c0`. `git merge-base origin/main HEAD` equals the reported base, and the local merge-base diff names match GitHub's six paths. The four original implementation paths remain; `crates/engine/src/world.rs` is additionally changed for `WorldError::detail()`, and the PR also contains the round-1 review record. Read the full merge-base diff and the round-1-to-round-2 changes.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Round 1 follow-up

1. **Resolved.** `sim_compare.rs` now resolves the documented repo-relative fixture against the repository root. `cargo test -p rt3ctl --test sim_compare` passed with `RT3_CONTROL_SAVE=review-tmp/saves/rt3engine-bi-18290101-202.gms`.
2. **Resolved.** The C5 evidence list now includes the stock-update formula and its clamps.
3. **Resolved.** The CLI and current C5 evidence describe only the structural fact that the 520 records are present in the save but absent from the map. No runtime-placement or industry-semantics claim remains in those files; the round-1 review preserves the old wording only as history.
4. **Partly resolved; see finding 1.** Missing-map and invalid-save diagnostics no longer print the installation root or full save path. A map-load diagnostic can still leak an absolute path embedded in `WorldError::detail()`.

## Verification

- The relative-fixture install-backed comparison test passed and asserted the controlled-save counts and divergences.
- The evidence lists every declared economy-model rule without original-constant provenance: tick length, initial stock and cap, production assignment and rate, demand, stock update/clamps, noise and price formula/bounds. The `Dump` row describes output serialization rather than an economy behavior rule.
- Reproduced missing-map, invalid-save and different-map cases. The missing-map error names the requested map and says it is not found “under the installation”; the invalid-save error names only `Cargo.toml`; the different-map JSON reports the British Isles source-map mismatch with no stderr path. The structural wording contains no industry or runtime-placement interpretation.
- Also probed a malformed map supplied by absolute path. Its error output includes the absolute path because the wrapped parser error is included in `WorldError::detail()`; see finding 1.
- `$HOME/tools/bin/make fmt`, `make clippy`, and `make test` passed. `make test` used `RT3_HOME` and absolute paths to the three fixtures under `review-tmp/saves/`; the workspace tests, install-backed comparison test and evidence check passed. CI `check` passed for this revision (run `45677176779`). No emulator was booted.

## Findings

1. **blocking — a map-load diagnostic still leaks an absolute path.** `crates/rt3ctl/src/sim_cmd.rs:380` prints `error.detail()`, but `crates/engine/src/world.rs:71-72` puts the wrapped `GmpMap` error's display text into that detail. Reproduced with a malformed direct map path: stderr included the full absolute map path despite printing the basename separately. Strip the path from the wrapped parser detail (and add a regression assertion that stderr contains no absolute input path) before treating round-1 finding 4 as resolved.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
