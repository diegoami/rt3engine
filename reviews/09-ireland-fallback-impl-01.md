# Implementation review — Ireland pitch fallback

- **Revision reviewed:** `652f000980c1a6a40f6f1a6a49287965843d71ee`
- **PR/base:** #16, base `5aeb83c0a020a19caac513806fa682d52351b16a`
- **Files:** `crates/rt3-kit/src/terrain.rs`; `docs/evidence/gmp-terrain-v1.md`; `docs/evidence/terrain-scan-v1.json`. Obtained with `gh pr view 16 --json headRefOid,baseRefOid,files`; confirmed against `git diff --name-only <base>...<head>`.
- **Reviewer/model:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`), fresh-context subagent
- **Mode/round:** OpenCode, implementation round 1; project slot records `design: none` and delegated merge on AGREE.

## Findings

None.

## Review

The checked-out branch and `HEAD` matched the PR head reported by GitHub. The PR API file list matched the complete merge-base diff; all three changed files and the full diff were inspected.

In `terrain.rs`, `alignment_rank`'s byte-alignment values are applied consistently by `densest_span` (`0x3000 + ((alignment + 4) & 3)`) and the existing scanner. The new `measure_pitch` retains the prior primary behavior by returning the measurement at `order[0]` immediately when present. Only if that measurement is absent does it consider the other alignments, and `unique_fallback_measurement` returns a result only for exactly one measurable fallback. The deterministic test covers zero, one, and competing measurements. No accept-side shortcut was added: `scan_geo` still uses the rejection-only probe/prefilter and full `validate` before extraction.

`scan_headerless` still returns directly from the pitch-anchored scan after a measurement, so a failed validation does not fall through to the fixed-geometry candidate. The install-backed reference includes Ireland and passed. The release CLI reports Ireland at `0x839d`, 257×321, −32 m…434.7794 m, pitch-anchored. California Island still reports 257×257 as `[candidate]`, not pitch-anchored.

The JSON artifact parses as 40 rows: 13 `verified`, 26 `pitch-anchored`, 1 `candidate`, and 0 missing results. All 13 header-bearing rows' grid dimensions match their header lattice dimensions in one of the two allowed orientations. Ireland is at offset 33693 (`0x839d`), 321 rows × 257 columns, residue 1. Residues are 10/18/6/6 for 0/1/2/3. Comparing row-by-row with the merge-base artifact shows Ireland is the only changed row.

The evidence document's revised totals agree with the artifact and code: 39 validated decodes (13 verified + 26 pitch-anchored) and one candidate window, not 40 validated decodes. Ireland's reported pitch 258 and rounded range −32…435 m match the measured CLI result.

## Gates

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME=<original installation, read-only> ~/.cargo/bin/cargo test --workspace -- --nocapture` — passed: 34 `rt3-kit` tests and 3 `rt3ctl` tests; the Ireland install-backed test and competing-fallback test ran.
- `RT3_HOME=<original installation, read-only> CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed.
- `RT3_HOME=<original installation, read-only> ~/.cargo/bin/cargo run -q --release -p rt3ctl -- map terrain Ireland` — passed; output reports 257×321, pitch-anchored, offset `0x839d`, range −32 m…434.7794 m.
- Same release CLI for `California Island` — passed; output remains `[candidate]`.

No game or emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

Landed with merge commit `b8b56d0` (delegated merge per decision 0002,
2026-11-23). Done-when items and evidence:

- Ireland recovered through measured pitch and exact-geometry validation:
  256×320 lattice at `0x839d`, pitch 258, range −32.0…434.8 m.
- Every established result preserved: the regenerated artifact changed
  only Ireland and now records 13 verified, 26 pitch-anchored and one
  candidate (California Island).
- Competing fallback measurements are refused; the deterministic
  regression and Ireland install-backed reference were both observed
  failing against their respective broken behavior.
- `make fmt`, `make clippy`, `make test` and the install-backed evidence
  check passed; CI and GitGuardian succeeded on the reviewed head.
- Final AGREE covers implementation revision
  `652f000980c1a6a40f6f1a6a49287965843d71ee` and is posted on PR #16.

— Implementer (zai-coding-plan/glm-5.3)
