# Implementation review: map-record height anchor

- **Revision covered:** `a8ee75f806ce65879f947004ee02c82018b8e29b`
- **Files checked (full PR #13):** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/05-dims-terrain-impl-03.md`.
- **How obtained:** `gh pr view 13 --json headRefName,headRefOid,baseRefName,files` reported head `m3b/record-height-anchor`, base `main`, and the five files above. Inspected `git diff main...HEAD --stat`, `--name-only`, and the full diff; merge base was `6b9012369090e0008d2c92e40d8c18ff088cade2`. Checked-out `HEAD` matched the PR head.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Findings

1. **blocking** — `docs/evidence/gmp-terrain-v1.md:75-78,94-99,131-132`. The document retains the prior-round ambiguity story: it says non-square maps with both orientations validating cannot be verified and fall back to candidates, then lists Alternate USA, Orient Express, and Southern Pacific as candidates. The record-anchor path now selects the matching orientation, and the artifact marks all three verified. The open section also says four headerless maps produce candidates, while the artifact has only Central Pacific after Chicago to New York, Dutchlantis, and Tex-Mex were withdrawn. Reconcile these stale classifications with the code and artifact.

2. **blocking** — `docs/evidence/gmp-terrain-v1.md:65,113-114`. The headline says 31 of 40 have no verified decode, but there are 8 verified maps, so 32 of 40 are not verified (1 candidate plus 31 with no grid). Thirty-one is the no-grid count, which the following sentence describes. Also, the claimed headerless record-height range `256…832` does not match the installed maps: their record words yield heights from 256 to 768.

3. **non-blocking** — `crates/rt3-kit/src/terrain.rs:735-753`. `scans_reference_maps` still covers only Crossing the Alps and Germany; it does not include the newly promoted non-square Alternate USA (`1024×640`). The evidence gate below does exercise and compare the all-map producer output, so this is not blocking, but an install-backed assertion for Alternate USA would keep this specific regression visible in the terrain test.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 13 passed, 0 failed. Includes `record_word_anchors_height`, `record_anchor_resolves_orientation`, and `ambiguous_orientation_is_refused`; the last still tests both orientations without an anchor.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no drift.
- Independently checked the installed files for all 13 header-bearing maps: each file+12 word is divisible by 256, and its quotient equals the artifact's `dims.values[5]` (including Alternate USA 640, Orient Express/Southern Pacific 384, and Greenland Growing 192). The withdrawn headerless maps yield record heights 512, 384, and 576 respectively.
- Parsed `terrain-scan-v1.json`: 40 maps, 8 verified, 1 candidate, 31 with no grid, and 13 dims headers; every grid hit records rows and cols. The verified list is Crossing the Alps, Alternate USA, France, Germany, Orient Express, Southern Pacific, State of Germany, and Third Republic; Central Pacific is the sole candidate.
- The release CLI reported Alternate USA as `1025x641 [verified]` (artifact rows/cols: `641×1025`).
- No game or emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
