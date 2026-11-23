# Framed terrain — implementation review, round 1

- **Revision reviewed:** `a0c272d72230527dd31a7001c18d7b4c6abc94a7`
- **PR:** #19, `m3b/framed-terrain`; base `28c8aeb139fca09275beea046ccafc2452938575`
- **Target proof and files:** `gh pr view 19 --json headRefOid,baseRefOid,files` reported the head and base above and these four files. `git diff --name-only <base>...<head>` independently returned the same list:
  - `crates/rt3-kit/src/terrain.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `docs/evidence/gmp-terrain-v1.md`
  - `docs/evidence/terrain-scan-v1.json`
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`; fresh OpenCode implementation-review context.
- **Design:** none, as recorded in the project slot.

## Findings

1. **blocking — `docs/evidence/gmp-terrain-v1.md:54-56`.** The claim that all prior scan-derived starts were 88–96 bytes before the framed starts is false for Britain. Comparing the merge-base artifact (`git show 28c8aeb139fca09275beea046ccafc2452938575:docs/evidence/terrain-scan-v1.json`) with the reviewed artifact gives Britain’s old offset as 35199 and framed offset as 35283: an 84-byte (21-cell) difference. The other 39 differences are 88, 92 or 96 bytes. Amend the stated range to 84–96 bytes (21–24 cells) so the evidence matches the artifact.

## Verification

- Read all four changed files at the reviewed HEAD and checked the full diff. The framed parser checks the full eight-word signature at every byte offset from `0x3000`, bounds dimensions before deriving geometry, validates at `section + 32`, then extracts the grid and checks its returned cells. The fixed-size bounds make its geometry arithmetic bounded. The fallback dimensions, pitch and candidate APIs and their tests are unchanged; `scan_map` alone now prefers a framed result and reports it as verified.
- Independently enumerated the 40 artifact-listed original version-9701 maps read-only. Each had exactly one matching frame; every artifact offset equalled marker + 32, dimensions matched the frame, and status was verified with no missed result. Residues were 10/18/6/6. The only old-start delta outside the documented 88–96 range was Britain at 84 bytes.
- The 27 maps with `dims: null` exactly match the evidence document’s 27-row oracle table. California’s corrected claim is consistent: marker `0x845b`, grid `0x847b`, 257×257; no current map remains a candidate.
- The synthetic frame test asserts exact start and dimensions, then refuses a changed signature constant. The install-backed California test asserts `0x847b`, 257×257 and the full returned height range.
- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME` set; `~/.cargo/bin/cargo test --workspace -- --nocapture` — passed: 36 `rt3-kit` tests (including 20 terrain tests), 3 `rt3ctl` tests, and doc tests.
- `RT3_HOME` set and `CARGO=$HOME/.cargo/bin/cargo`; `sh tools/evidence.sh check` — passed.
- `RT3_HOME` set; `~/.cargo/bin/cargo run -q --release -p rt3ctl -- map terrain 'California Island'` — passed; output reported grid `0x847b` (257×257) `[verified]`.
- No emulator was started.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
