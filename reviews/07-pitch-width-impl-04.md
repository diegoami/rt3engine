# Implementation review: pitch-anchored headerless width (round 4)

- **Revision covered:** `de2905145da0c034fb1aa28b3b0f02ceb10ef59f`
- **Files checked (full PR #14):** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/06-record-anchor-impl-02.md`, `reviews/07-pitch-width-impl-01.md`, `reviews/07-pitch-width-impl-02.md`, `reviews/07-pitch-width-impl-03.md`.
- **How obtained:** `gh pr view 14 --json headRefName,headRefOid,baseRefName,baseRefOid,files` reported branch `m3b/pitch-anchored-width`, this revision, and base `main` at `9c01758246ad447f93fcbce5d55bef169880386a`; its eight-file list matched `git diff --name-only 9c01758246ad447f93fcbce5d55bef169880386a..HEAD`. Checked-out `HEAD` matched the revision. The requested `git diff 14b4dbc..de29051` contains only `docs/evidence/gmp-terrain-v1.md` and `reviews/07-pitch-width-impl-03.md`. Inspected that delta, the current evidence document and artifact, the relevant scanner path, and the prior review records.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-3 finding rechecked

1. **Resolved.** `docs/evidence/gmp-terrain-v1.md:145-152` no longer implies all 29 maps were tested against a window geometry. It explicitly says not all were window-tested and explains that the ladder scans no geometry when pitch is unmeasurable and record height is not 256. It separately retains the five dims-header maps scanned with their own anchored geometry.

## Prior resolutions rechecked

1. **Hold.** The headline and funnel distinguish 10 decodes (8 verified, 2 pitch-anchored) from the one candidate, and retain the five measured-but-unvalidated maps and the 20 no-measurable-pitch diagnostic split.
2. **Hold.** `terrain::scan_headerless` scans measured pitch geometry and returns directly; only when pitch is unmeasurable and record height is 256 can the candidate scan run. `scan_map` delegates to this ladder.
3. **Hold.** The artifact parses to 40 maps: 8 verified, 2 pitch-anchored, 1 candidate, 29 with no grid, and 13 dims headers. These counts reconcile with the headline and Open section.

## Findings

None.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `~/.cargo/bin/cargo test -p rt3-kit terrain` — 16 passed, 0 failed.
- `git diff --check 9c01758246ad447f93fcbce5d55bef169880386a..HEAD` and `git diff --check 14b4dbc..de29051` — passed.
- The full evidence check was not run for this documentation-only delta. No game or emulator was booted; no capture was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
