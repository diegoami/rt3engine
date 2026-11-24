# Implementation review: trailing-cell evidence

- **Revision reviewed:** `bcd261e662212c861294c89e459c412c1d7b5455`
- **Files checked:** `crates/rt3-kit/src/terrain.rs`; `docs/evidence/gmp-terrain-v1.md`. Obtained from `gh pr view 23 --json headRefOid,baseRefOid,files`: head `bcd261e662212c861294c89e459c412c1d7b5455`, base `76dfe4533f873648b5b335e6ac5c0b4803ae7ce6`, two-file list as above. Local `HEAD` matched the PR head.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`
- **Mode:** OpenCode implementation review; project slot records `design: none`.

## Findings

None.

## Review

Independently enumerated the original `.gmp` files in `maps/` and
`Data/Tutorial/` (43 total), scanned for each complete terrain-frame
signature, and read its little-endian f32 trailing slot at grid start plus
`(row * (cols + 1) + cols) * 4`. Each map had exactly one bounded frame.
Across 19,755 rows the distribution is exactly 18,475 `0.0` and 1,280
`330.6503601074219` (the f32 representation of `330.65036`). The only
nonzero values are on Alternate USA and Coast to Coast, rows 0–639; both
final row-640 values are zero. The other 41 maps have only zero trailing
values; no other values occur.

The test at `crates/rt3-kit/src/terrain.rs:1154-1188` checks all 43 paths,
asserts 19,755 total rows, uses the stored row stride `cols + 1`, and
addresses exactly the extra cell at column `cols`. Its `r + 1 == g.rows`
condition makes the final row zero without an off-by-one; for both named
maps all preceding rows are checked against the nonzero f32 literal, and
all other rows/maps against zero. `read_framed` extracts only the `cols`
corner-height cells, so this assertion observes the original bytes
without adding the trailing cell to `HeightGrid` or render behavior.

The documentation at `docs/evidence/gmp-terrain-v1.md:113-128` matches
the independently computed distribution and explicitly leaves semantic
purpose open. The existing all-43 frame/decode scope remains unchanged.
The PR diff adds only this test and evidence prose; no artifact or
production behavior changed.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME=<original installation> ~/.cargo/bin/cargo test --workspace` — passed: 42 `rt3-kit` tests and 5 `rt3ctl` tests.
- `RT3_HOME=<original installation> CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed; committed evidence artifacts match.
- `git diff --check` — passed.

No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
