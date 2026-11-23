# Implementation review: byte-alignment terrain scans (round 2)

- **Revision covered:** `2697ce22d2871a92209931b0a0df796d62cf1a58`
- **Files checked (full PR #15):** `crates/rt3-kit/src/terrain.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/07-pitch-width-impl-04.md`, `reviews/08-alignment-impl-01.md`.
- **How obtained:** `gh pr view 15 --json headRefName,headRefOid,baseRefName,baseRefOid,files` reported branch `m3b/byte-alignment`, this head, base `main` at `1efa7a588ba40402cc905575a1ec89a79ab96a4a`, and the five files above. `git diff --name-only 1efa7a588ba40402cc905575a1ec89a79ab96a4a..HEAD` matched that list; checked-out `HEAD` matched the reported head. Inspected the full PR diff and read every listed file at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-1 findings rechecked

1. **Not fully resolved.** The headline, map-status counts, candidate/no-result names, residue counts and most withdrawn claims now reconcile with the artifact. One stale named list remains; see finding 1.
2. **Code resolved; regression requirement remains open.** `scan_all_geo` now uses the ranked four-alignment loop, but the added nonzero-residue Italy test exercises `scan_with_dims`/`scan_geo`, not the multi-hit path. See finding 2.
3. **Resolved.** Italy is in `scans_reference_maps` with an asserted 2-mod-4 offset, Argentina Opens Up is in `scans_pitch_anchored_reference_maps`, and both passed in the 16-test run.

## Findings

1. **blocking** — `docs/evidence/gmp-terrain-v1.md:97-101`. The document says five maps measured a pitch but validate nowhere, listing Britain, British Isles, Coast to Coast, Germantown and South East Australia. The current artifact instead classifies all five as `pitch-anchored` with non-null grids (`docs/evidence/terrain-scan-v1.json:5-6,10,18,34`). This is a remaining stale claim that contradicts the headline count and classification; remove or correct the failed-validation list.

2. **blocking** — `crates/rt3-kit/src/terrain.rs:550-574,763-775,945-998`. The multi-hit implementation now iterates ranked alignments, but no regression exercises `scan_all_geo` on a nonzero-residue grid. The Italy reference invokes `scan_with_dims` (which reaches `scan_geo`), while the only existing `scan_all_geo` test uses unsupported geometry and expects an empty result. Those tests would still pass if the old 0-mod-4-only multi-hit scan returned. Add a nonzero-residue assertion through `scan_all_geo`, as requested in round 1.

## Verification

- `scan_all_geo` and `scan_geo` both start each pass at the requested absolute residue and advance by four. `step_of` and `Geometry::cell_off` produce multiples of four, so probe/prefilter sample origins remain on the validator's every-fourth-cell lattice; `refine_start` subtracts whole rows of `row_cells * 4` bytes and preserves the residue.
- Parsed `terrain-scan-v1.json`: 40 rows; 13 verified, 25 pitch-anchored, 1 candidate, 1 no-result; residues 10/17/6/6 across 0/1/2/3 mod 4. The sole no-result is Ireland, the candidate is California Island, and all dims-bearing grid dimensions match their anchored lattice dimensions.
- The old 8-verified and 30/29 no-grid/no-window claims, Central Pacific candidate, five header-bearing misses, Britain/East-Coast no-window claims, and second-storage-form inference are absent. The failed-pitch five-map list in finding 1 remains.
- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 16 passed, 0 failed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no output or drift.
- `git diff --check 1efa7a588ba40402cc905575a1ec89a79ab96a4a..HEAD` — passed. No game or emulator was booted; no capture was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
