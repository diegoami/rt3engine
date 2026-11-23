# Implementation review: pitch-anchored headerless width (round 2)

- **Revision covered:** `f38cb44c0d3db47f3a829780cf79216410563b7a`
- **Files checked (full PR #14):** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/06-record-anchor-impl-02.md`, `reviews/07-pitch-width-impl-01.md`.
- **How obtained:** `gh pr view 14 --json headRefName,headRefOid,baseRefName,baseRefOid,files` reported branch `m3b/pitch-anchored-width`, this head, and base `main` at `9c01758246ad447f93fcbce5d55bef169880386a`. `git merge-base main HEAD` returned that base; `git diff --name-only <merge-base>..HEAD` matched the six files reported by the PR. Checked-out `HEAD` matched the revision covered. Inspected the full diff and read the changed source and evidence files at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-1 findings rechecked

1. **Resolved.** The headline separates 10 grid decodes (8 verified and 2 pitch-anchored) from the Central Pacific candidate window.
2. **Partially resolved.** The measured-and-validated count is 2; the measured-but-rejected list now includes all five maps; and 20 current headerless maps have no measurable pitch. The explanation of those 20 remains inaccurate; see finding 1 below.
3. **Resolved.** `scan_headerless` requires a usable record height first. When `measure_pitch` returns `Some`, it returns the pitched-geometry scan result directly, so the only outcomes are a `PitchAnchored` hit or `None`. The candidate branch is reached only when pitch measurement returns `None` and `h == 256`; a missing record word returns `None`. `ladder_never_guesses_after_a_pitch` is present.
4. **Resolved.** The stale future-work sentence about anchoring headerless width has been removed.

## Findings

1. **blocking** — `docs/evidence/gmp-terrain-v1.md:124`; `crates/rt3-kit/src/terrain.rs:498-499`. The document says the 20 maps without a measurable pitch have a missing span or a refused separation. The code returns before comparing separation when fewer than two lag candidates score. An independent diagnostic over the 27 current headerless maps found no missing spans among the 20: nine maps (East of Mississippi, Eastern China, Ireland, Louisiana, Mexico, Pacific NW, Poland, Russia, Spanish Mainline) had zero scoring lags, so separation was never tested; the other eleven had five scoring lags each and failed the separation threshold. State this distinction, or remove the unsupported cause summary.

2. **non-blocking** — `docs/evidence/gmp-terrain-v1.md:142-143`. “29 of 40 maps have no grid decode (verified, pitch-anchored or candidate)” still groups the candidate with decodes, although the headline and candidate section distinguish it as an unestablished window. Consider saying “no grid result/window” here, or giving the 30-without-a-decode and 29-without-a-grid-result counts separately.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 16 passed, 0 failed.
- `git diff --check 9c01758246ad447f93fcbce5d55bef169880386a..HEAD` — passed.
- Independently scanned the installed current maps using a temporary crate depending on `rt3-kit` by path: 40 current maps, 27 headerless; 7 measurable pitches, 2 validated and 5 rejected (Britain, British Isles, Coast to Coast, Germantown, South East Australia), and 20 with no measurable pitch. A separate diagnostic found the 9 zero-score and 11 separation-failure cases described above.
- Independently checked the legacy Pacific Coastal data point: v9610, pitch 322 cells, record height 832, validated 321×833 grid.
- Parsed `terrain-scan-v1.json`: 40 maps; 8 verified, 2 pitch-anchored, 1 candidate, 29 with no grid, and 13 dims headers. These counts match the revised headline and artifact.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no output or drift.
- No game or emulator was booted; no capture was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
