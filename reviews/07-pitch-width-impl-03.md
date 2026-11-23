# Implementation review: pitch-anchored headerless width (round 3)

- **Revision covered:** `14b4dbc81a456665c9af6ebd5fd46e145ce249cb`
- **Files checked (full PR #14):** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/06-record-anchor-impl-02.md`, `reviews/07-pitch-width-impl-01.md`, `reviews/07-pitch-width-impl-02.md`.
- **How obtained:** `gh pr view 14 --json headRefName,headRefOid,baseRefName,baseRefOid,files` reported branch `m3b/pitch-anchored-width`, this head, and base `main` at `9c01758246ad447f93fcbce5d5bef169880386a`; its seven-file list matched `git diff --name-only 9c01758..HEAD`. `git merge-base main HEAD` returned that base, and checked-out `HEAD` matched the revision covered. The incremental `git diff f38cb44..14b4dbc` contains `docs/evidence/gmp-terrain-v1.md` and the posted round-2 record. Inspected that diff, read the evidence document and JSON artifact in full, and checked the relevant current scanner/CLI code and prior review records.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-2 findings rechecked

1. **Resolved.** `docs/evidence/gmp-terrain-v1.md:124-127` now distinguishes the 20 maps with no measurable pitch: none lack a height-dense span; nine have too little land-pair data to score any lag, so separation is not tested, while eleven fail the separation threshold. This matches the round-2 diagnostic.
2. **Resolved.** `docs/evidence/gmp-terrain-v1.md:145-149` counts 30 maps without a verified or pitch-anchored decode, distinguishes the 29 with no validated window from the Central Pacific candidate, and explicitly says that candidate is not a decode. The artifact has 8 verified, 2 pitch-anchored, 1 candidate, and 29 with no grid.

## Round-1 findings rechecked

1. **Resolved.** The headline at `docs/evidence/gmp-terrain-v1.md:3-11` states 10 decodes (8 verified and 2 pitch-anchored) plus one candidate.
2. **Resolved.** The funnel lists two measured-and-validated maps, all five measured-but-not-validated maps, and the 20 no-measurable-pitch maps with the diagnostic split above.
3. **Resolved.** `terrain::scan_headerless` scans the measured geometry and returns directly if a pitch exists; only when pitch measurement is absent and record height is 256 does it run the candidate scan. `scan_map` delegates to that ladder, and `ladder_never_guesses_after_a_pitch` remains present.
4. **Resolved.** The stale future-work statement about anchoring headerless width is gone.

## Findings

1. **non-blocking** — `docs/evidence/gmp-terrain-v1.md:146-148`; `crates/rt3-kit/src/terrain.rs:530-540`. “Twenty-nine of them produce no validated window at all under their anchored geometry” can imply that each of those maps was scanned against an anchored geometry. But when pitch measurement returns `None` and the record height is not 256, `scan_headerless` returns without scanning any window geometry. The count of 29 with no validated window is supported by the artifact; remove or qualify “under their anchored geometry” so it does not overstate which maps were actually tested.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `~/.cargo/bin/cargo test -p rt3-kit terrain` — 16 passed, 0 failed.
- `git diff --check 9c01758..HEAD` and `git diff --check f38cb44..14b4dbc` — passed.
- Parsed `terrain-scan-v1.json`: 40 maps; 8 verified, 2 pitch-anchored, 1 candidate, 29 with no grid, and 13 dims headers. These counts match the headline, Open section, and status breakdown.
- The full evidence check was not run for this documentation-only delta. No game or emulator was booted; no capture was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
