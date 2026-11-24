# Implementation review — legacy frames

- **Revision reviewed:** `c2525f5d529ac6e09326d8189cd8a74d0dd0fe02` (PR #22 head; branch `m3b/legacy-frames`).
- **Target proof:** `gh pr view 22 --json headRefOid,baseRefOid,files` reported this exact head and base `9cd2a3096e2882b9eea44379ae85b7c9c596e327`; the local `HEAD` and resolved `c2525f5` match. The PR file list is the seven paths below, matching `git diff --name-only` from the base/merge-base to the head.
- **Files checked:**
  - `crates/rt3-kit/src/render.rs`
  - `crates/rt3-kit/src/terrain.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/gmp-format-v1.md`
  - `docs/evidence/gmp-terrain-v1.md`
  - `docs/evidence/terrain-scan-v1.json`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`), fresh-context subagent.
- **Mode:** OpenCode implementation review; project slot is `design: none`, with delegated merge on AGREE.

## Findings

1. **Non-blocking — clarify the scope of the retained current-map summaries.** `docs/evidence/gmp-terrain-v1.md:92` still says “Residues across the 40 framed grids,” while the revised claim establishes 43 framed grids. The unchanged residue totals are correct for the original 40 current maps (10/18/6/6); the three legacy offsets add one residue-0 and two residue-2 grids, making the 43-map totals 11/18/8/6. Also, line 122 says the pixel-buffer test covers “all 40 maps,” although the text now covers 40 current maps plus three legacy maps. Label these summaries as current-only (and state legacy coverage separately) to avoid ambiguity. This does not affect decoding or the artifact.

## Verification

- Independently inspected the three original legacy maps read-only. Pacific Coastal is version 9610 and has one valid `0x32dd/0x2ee1` frame at `0x882c`, with the grid at `0x884c` (321×833). Tutorial_1 and Tutorial_2 are version 9609 and each has one valid frame at `0x87c6`, with the grid at `0x87e6` (385×449). All frame constants match; every stored cell, including each row’s trailing cell, is NaN or within the accepted height range. The artifact dimensions and computed cell statistics match all three.
- Inspected all 40 current maps: each has one valid `0x2ee2` frame and none has a `0x2ee1` frame. `read_framed` at `crates/rt3-kit/src/terrain.rs:398-426` admits only the exact pair `0x2ee1..=0x2ee2`; the synthetic frame test accepts `0x2ee1` at lines 935-957. No wider subsection range is accepted.
- Reviewed single-map and `--all` version gates at `crates/rt3ctl/src/map_cmd.rs:167-173` and `:307-309`: both admit only 9609, 9610, and 9701. A temporary synthetic version-9608 map was rejected by single-map mode and omitted by `--all`; the supported legacy originals decode successfully. `scan_map` and its fallback ordering are otherwise unchanged.
- Parsed `terrain-scan-v1.json`: all 43 rows are `verified`; all 40 merge-base rows compare exactly equal; the only additions are Pacific Coastal, Tutorial_1, and Tutorial_2 with the expected offsets, dimensions, and values. The documentation’s 27-row oracle table remains current-only with no legacy entries; the 43-map headline, legacy paragraph, open non-terrain scope, and evidence README producer row match the artifact.
- Visually inspected `review-tmp/render-legacy/Pacific Coastal.png`, `Tutorial_1.png`, and `Tutorial_2.png`: all are coherent geography at the expected dimensions (Pacific coastline/mountain spine; tutorial Italy terrain). The PNGs are gitignored, as is the containing `review-tmp/` working area.
- Required gates passed:
  - `~/.cargo/bin/cargo fmt --all -- --check`
  - `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings`
  - `RT3_HOME`-backed `~/.cargo/bin/cargo test --workspace`: 41 `rt3-kit` tests and 5 `rt3ctl` tests passed.
  - `RT3_HOME`-backed `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check`
  - Release CLI Pacific Coastal: `grid at 0x884c (321x833, -4.215068 m..3457.0273 m, NaN 0 / zero 75220) [verified]`.
- No emulator boot was performed; no capture lock was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
