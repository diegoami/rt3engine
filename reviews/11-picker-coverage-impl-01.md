# Implementation review: picker coverage

- **Revision:** `8d00b888de9d68ec74647d20aa49f2d975df43bc`
- **Pull request:** #18, `m3b/picker-complete`
- **Base:** `f71969802fd6075d1951586ce1b2671db0c60b91`
- **Files checked:** `docs/evidence/gmp-terrain-v1.md` (obtained from `gh pr view 18 --json headRefOid,baseRefOid,files`; the PR head matched this revision and the PR file list matched the local diff)
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`
- **Mode:** OpenCode implementation review; project slot is `design: none`

## Findings

None.

## Verification

- Read the changed evidence document and full PR diff. The updated table contains 27 unique headerless maps: all 26 pitch-anchored maps plus California Island. California remains a candidate for its height window, East Coast, USA remains a separate header-bearing confirmation, and row-order corroboration remains limited to British Isles, Italy, and Ireland (`docs/evidence/gmp-terrain-v1.md:81-149, 151-178, 185-191`).
- Parsed `terrain-scan-v1.json`: 40 maps comprise 13 verified, 26 pitch-anchored, and one candidate (California Island). Programmatic reconciliation confirmed each table map and decoded lattice against the JSON grid dimensions (`cols - 1`, `rows - 1`). All 27 table dimensions match the independently read visual dimensions, the capture manifest's expected lattice, and the scan. The prior three-map row-orientation evidence is unchanged.
- Inspected all three picker contact sheets and read all 27 Size labels. The large-map crops (indices 07, 09, 17, 21, and 23) retain legible original labels; no label is cut off or obscured in a way that could alter the reading. The visual readings agree with `picker-all-visual.json` and each manifest entry. OCR was not used as proof.
- Inspected the capture driver and log. The driver selects all 27 pitch-anchored/candidate maps from the scan, hashes each source and checks the writable Argentina substitution against that hash before its capture, records one screenshot per map, then exits the game and restores the backup in its `finally` cleanup. The log shows the full 27-map sequence and successful game exit. A byte-for-byte comparison of the restored Argentina map with its backup passed; no RT3/drive process remains. The capture log records that the capture lock was acquired. No screenshot or original map bytes are in the PR.
- Gates on this revision:
  - `~/.cargo/bin/cargo fmt --all -- --check` — passed.
  - `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
  - `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test --workspace` — passed (34 `rt3-kit` tests and 3 `rt3ctl` tests).
  - `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed.
- No emulator was booted during this review.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

Landed with merge commit `00202e2` (delegated merge per decision 0002,
2026-11-23). Done-when items and evidence:

- Original-picker size labels were captured for all 26 pitch-anchored
  maps plus California Island; 27/27 visual reads match the decoded
  lattice artifact with no mismatches.
- Every substitution was SHA-256 checked against its source. Argentina
  was restored by an exit trap and `cmp`-verified; no oracle process
  remains and the GOG installation was untouched.
- California remains a candidate: its 256×256 geometry is confirmed,
  but its candidate height window is not established as terrain.
- Orientation evidence remains scoped to British Isles, Italy and
  Ireland. Decoder behavior and generated artifacts are unchanged.
- Formatting, clippy, 34 `rt3-kit` tests, three `rt3ctl` tests and the
  evidence check passed; CI and GitGuardian succeeded on the reviewed
  head. Final AGREE covers implementation revision
  `8d00b888de9d68ec74647d20aa49f2d975df43bc` and is posted on PR #18.

— Implementer (zai-coding-plan/glm-5.3)
