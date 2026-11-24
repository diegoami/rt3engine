# Implementation re-review — legacy frames

- **Revision reviewed:** `d00008e7ff6d5d43e0f32aaa0d1dad2eba016c02` (PR #22 head; branch `m3b/legacy-frames`).
- **Target proof:** `gh pr view 22 --json headRefOid,baseRefOid,files` reported this exact head and base `9cd2a3096e2882b9eea44379ae85b7c9c596e327`; local `HEAD` and resolved `d00008e` match. The current PR file list is the eight paths below, matching the local base-to-HEAD file list.
- **Files checked:**
  - `crates/rt3-kit/src/render.rs`
  - `crates/rt3-kit/src/terrain.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/gmp-format-v1.md`
  - `docs/evidence/gmp-terrain-v1.md`
  - `docs/evidence/terrain-scan-v1.json`
  - `reviews/15-legacy-frames-impl-01.md`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`), same reviewer session for the implementation re-review.
- **Mode:** OpenCode implementation review; project slot is `design: none`, with delegated merge on AGREE.

## Findings

1. **Non-blocking, resolved from round 1.** `docs/evidence/gmp-terrain-v1.md:92-93` now scopes the 10/18/6/6 alignment counts to the 40 current maps' framed grids. Lines 122-124 now state that full pixel buffers are validated separately for all 40 current maps and all three legacy maps. This resolves the prior scope ambiguity.

No new findings. The implementation and evidence artifact have no changes since the round-1 target; `git diff c2525f5d529ac6e09326d8189cd8a74d0dd0fe02..HEAD` contains only the documentation clarification and the round-1 review record.

## Verification

- Confirmed the current PR head and eight-file list with `gh pr view 22 --json headRefOid,baseRefOid,files`; the local HEAD equals the PR head.
- Inspected the complete delta from the round-1 target. The finding's two documentation statements are now explicitly scoped as above; implementation and artifact files are unchanged.
- Gates rerun on this checkout:
  - `~/.cargo/bin/cargo fmt --all -- --check` — passed.
  - `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
  - `RT3_HOME`-backed `~/.cargo/bin/cargo test --workspace` — passed: 41 `rt3-kit` tests and 5 `rt3ctl` tests.
  - `RT3_HOME`-backed `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed.
  - Release CLI Pacific Coastal — `grid at 0x884c (321x833, -4.215068 m..3457.0273 m, NaN 0 / zero 75220) [verified]`.
- No emulator boot was performed; no capture lock was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
