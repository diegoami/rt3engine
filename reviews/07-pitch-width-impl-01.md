# Implementation review: pitch-anchored headerless width

- **Revision covered:** `b32952f8758140cee86bb325962060d000d8767e`
- **Files checked (full PR #14):** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/06-record-anchor-impl-02.md`.
- **How obtained:** `gh pr view 14 --json headRefName,headRefOid,baseRefName,files` reported head `m3b/pitch-anchored-width`, this revision, and base `main`. `git merge-base main HEAD` was `9c01758246ad447f93fcbce5d55bef169880386a`; the PR file list matched `git diff --name-only main...HEAD`. Checked-out `HEAD` matched the revision covered. Inspected the full diff and read every changed file at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Findings

1. **blocking** — `docs/evidence/gmp-terrain-v1.md:3-4,10-11,119-127`. The headline says “grid decode for 11 maps” but its parenthetical counts only 8 verified plus 2 pitch-anchored maps (10); the artifact's 11th non-null grid is the Central Pacific **candidate window**, which the document itself says is not an established decode. Reconcile the headline as either 10 grid decodes plus one candidate, or 11 windows with all three statuses listed.

2. **blocking** — `docs/evidence/gmp-terrain-v1.md:113-117`; `crates/rt3-kit/src/terrain.rs:472-508`. The section says only Germantown, Britain and South East Australia additionally measure a pitch without a validating window. A direct call to the checked-out `measure_pitch` on the installed headerless maps also measures British Isles (258 cells, scores 1.470/2.515) and Coast to Coast (1026 cells, 70.101/127.932); both have no validated grid in the artifact. Thus five, not three, headerless maps besides Tex-Mex and Chicago to New York measure a pitch without a hit. Also, the code can return `None` before testing separation when fewer than two candidates score (`:494-499`), so the blanket statement that the rest are “refused on separation” overstates what the code establishes.

3. **blocking** — `crates/rt3ctl/src/map_cmd.rs:218-223`; `crates/rt3-kit/src/terrain.rs:17-22`. `scan_pitch_anchored` returns `None` both when no pitch is measurable and when a measured geometry fails full validation. `scan_map` then falls through to the fixed-width candidate scan for record height 256 (and even when the record height is unavailable). That contradicts the stated rule that the candidate runs only when no pitch is measurable and the record anchors height 256; it can guess a different geometry after the measured one failed. Preserve the distinction: only run this candidate fallback when pitch measurement returned `None` and the record height is exactly 256, or make and document a different explicit rule.

4. **non-blocking** — `docs/evidence/gmp-terrain-v1.md:137-140`. The Open section still says headerless maps have no known width and that anchoring width is the natural next step, despite this change measuring row pitches and accepting two pitch-anchored grids. Scope that statement to the remaining unresolved maps / pitches that do not validate.

## Verification

- Inspected the new pitch path: `is_land_like`'s precedence admits only bounded finite signed heights; span runs flush both on a cold window and at EOF; the lag loop stays in-span, samples at stride 83, requires more than 40 pairs, and compares the best score to the runner-up at 60%. `scan_pitch_anchored` uses the record height, checked `row_cells - 2`, then `scan_geo`'s existing probe/prefilter/full-validation/extraction path. The existing probe-subset and ambiguity regressions remain in the suite.
- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME`-backed `~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 15 passed, 0 failed. Includes the synthetic pitch fixture, both install-backed pitch references, and the earlier probe-subset, orientation-ambiguity, and unsupported-geometry regressions.
- `RT3_HOME`-backed `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no output or drift.
- Parsed `terrain-scan-v1.json`: 40 maps, 8 verified, 2 pitch-anchored, 1 candidate, 29 with no grid, and 13 dims headers. These artifact counts match the detailed count in lines 10-11, but not the “11 grid decode” headline.
- The release Tex-Mex CLI reported `grid at 0x876c (513x577, ...) [pitch-anchored]`; the artifact records 577 rows and 513 columns.
- No game or emulator was booted; no capture was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
