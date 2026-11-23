# Implementation review: byte-alignment terrain scans

- **Revision covered:** `e107f4553029fd29f879e5a8c47799e1c3139f33`
- **Files checked (full PR #15):** `crates/rt3-kit/src/terrain.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/07-pitch-width-impl-04.md`.
- **How obtained:** `gh pr view 15 --json headRefName,headRefOid,baseRefName,baseRefOid,files` reported branch `m3b/byte-alignment`, this head, base `main` at `1efa7a588ba40402cc905575a1ec89a79ab96a4a`, and the four-file list above. `git diff --name-only main...HEAD` matched that list; checked-out `HEAD` matched the reported revision. Inspected `git diff main...HEAD` and read every changed file at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Findings

1. **blocking** — `docs/evidence/gmp-terrain-v1.md:3-11,107-110,136-171`. The evidence narrative is not reconciled with the artifact. The JSON has 40 current maps: 13 verified, 25 pitch-anchored, one candidate (California Island), and one current map with no grid (Ireland). Thus there are 38 validated grids plus a candidate window, not 39 validated decodes plus a legacy map skipped from the 40. The document still calls eight maps verified; says 30 maps have no decode and 29 have no window; identifies Central Pacific as the candidate; lists five header-bearing misses (including Italy); and says Britain and East Coast have no window. Those claims predate the artifact. The “second storage form” discussion and the three-map density-scan list rely on those stale misses as well. Reconcile the counts and classifications throughout, distinguish the candidate from a decode, identify Ireland as the current no-result map, and narrow or remove the unsupported second-storage-form inference.

2. **blocking** — `crates/rt3-kit/src/terrain.rs:550-570,659-661`. The new alignment loop is in `scan_geo`, but the public multi-hit path `scan_all_geo` still starts at `from & !3` and advances by four; `scan_all` delegates to it. It therefore still examines only 0-mod-4 starts and misses the odd-aligned grids this change is intended to cover. Make that scan path alignment-aware too and retain a regression for a nonzero-residue grid.

3. **blocking** — `crates/rt3-kit/src/terrain.rs:942-957,989-1002`. The install-backed reference sets do not include the two requested alignment cases: `scans_reference_maps` has Alps, Germany and Alternate USA, but not Italy; `scans_pitch_anchored_reference_maps` has Tex-Mex and Chicago to New York, but not Argentina Opens Up (640×384). The 16 passing terrain tests retain the earlier synthetic-grid, filler, probe-invisible-land and ambiguous-orientation invariants, but do not lock these new results. Add Italy with an assertion for its 2-mod-4 start and Argentina to the pitch-anchored references.

## Verification

- `alignment_rank` produces absolute byte residues; `scan_geo` starts each pass at the corresponding residue. Probe and prefilter steps are multiples of four, so their sample coordinates remain on the validator's every-fourth-cell lattice relative to each alignment. `refine_start` subtracts `row_cells * 4`, preserving that residue.
- `scan_with_dims`, `scan_headerless`, and `scan` all reach `scan_geo`; pitch measurement uses the top-ranked residue. The ranked outer loop exhausts one residue before the next, so a lower-ranked residue cannot preempt a higher-ranked hit. The first-hit policy can still let a false hit on an earlier-ranked residue precede a true hit on a later one; I did not reproduce that on the installed corpus. Italy's reported start is 2 mod 4.
- `is_land_like`'s `&&`/`||` precedence is semantically sound: the negative bounded range itself excludes NaN and infinities, as does the positive bounded range. No precedence defect found.
- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME`-backed `~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 16 passed, 0 failed; the two reference cases above are absent.
- `RT3_HOME`-backed `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no output or drift.
- Parsed `terrain-scan-v1.json`: 40 rows; 13 verified, 25 pitch-anchored, 1 candidate, 1 null; all 39 hits have rows/cols, and every hit with a dims header matches `(lattice_h + 1, lattice_w + 1)`. The null row is Ireland.
- Release CLI checks: Italy reports `0x878e` (2 mod 4), 385×449, verified; Argentina Opens Up reports 641×385 (640×384 lattice), pitch-anchored.
- `git diff --check main...HEAD` — passed. No game or emulator was booted; no capture was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
