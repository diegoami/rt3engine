# Implementation review: byte-alignment terrain scans (round 3)

- **Revision covered:** `df9139fd1174ed99e30723db1e88e8ec48374f23`
- **Files checked (full PR #15):** `crates/rt3-kit/src/terrain.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/07-pitch-width-impl-04.md`, `reviews/08-alignment-impl-01.md`, `reviews/08-alignment-impl-02.md`.
- **How obtained:** `gh pr view 15 --json headRefName,headRefOid,baseRefName,baseRefOid,files` reported branch `m3b/byte-alignment`, this head, base `main` at `1efa7a588ba40402cc905575a1ec89a79ab96a4a`, and the six-file list above. `git diff --name-only 1efa7a588ba40402cc905575a1ec89a79ab96a4a..HEAD` matched it; checked-out `HEAD` matched the reported head. Reviewed `git diff 2697ce2..df9139f` (three changed files), the full-PR file list, and the listed files at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-2 findings rechecked

1. **Resolved.** `docs/evidence/gmp-terrain-v1.md:97-102` replaces the stale five-map “validate nowhere” claim with the reconciled ladder result. The five named maps (Britain, British Isles, Coast to Coast, Germantown and South East Australia) are all pitch-anchored with grids in `docs/evidence/terrain-scan-v1.json`.
2. **Resolved.** `crates/rt3-kit/src/terrain.rs:1000-1026` adds `scan_all_geo_finds_nonzero_residue_grid`, which reads Italy, calls `scan_all_geo`, and asserts its first hit is 2 mod 4. Its input bound is the known grid offset plus the complete `geo.grid_bytes()` window and a 4096-byte margin, capped only by the file length; it does not truncate the grid needed by the scan. The assertion distinguishes the intended result from the prior 0-mod-4-only path.

## Prior resolutions rechecked

- `scan_all_geo` at `crates/rt3-kit/src/terrain.rs:550-574` iterates the ranked byte alignments and advances by four within each residue. The Italy reference still asserts residue 2, and the pitch-anchored references still include Argentina Opens Up.
- Read `docs/evidence/gmp-terrain-v1.md` in full and parsed `docs/evidence/terrain-scan-v1.json`: 40 maps total; 13 verified, 25 pitch-anchored, 1 candidate (California Island), and 1 no-result (Ireland). The 39 rows with grids comprise 38 validated results and the candidate; the five formerly named maps are pitch-anchored. No “validate nowhere” or equivalent failed-validation list remains. The narrative's classifications and counts agree with the artifact.

## Findings

None.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 17 passed, 0 failed, including the new Italy multi-hit regression.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no output or drift.
- `git diff --check 2697ce2..df9139f` — passed. No game or emulator was booted; no capture was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

Landed with merge commit `920119a` (delegated merge per decision 0002,
2026-11-23). Done-when items and evidence:

- All scan paths cover data-dependent byte alignment: `scan_geo` and
  `scan_all_geo` use the density-ranked alignment loop; the Italy
  multi-hit regression proves a first hit at residue 2.
- Install-backed references cover the header, pitch and alignment paths:
  17 terrain tests passed, including Italy and Argentina Opens Up.
- Evidence and artifact reconcile at 13 verified, 25 pitch-anchored,
  one candidate (California Island) and one no-result (Ireland);
  `tools/evidence.sh check` passed on the reviewed revision.
- The final AGREE covers `df9139f`; the verdict is posted on PR #15.

— Implementer (zai-coding-plan/glm-5.3)
