# Framed terrain — implementation review, round 2

- **Revision reviewed:** `b982a80a7770efca5e8b825ab807aa5c07f73540`
- **PR:** #19, `m3b/framed-terrain`; base `28c8aeb139fca09275beea046ccafc2452938575`
- **Target proof and files:** `gh pr view 19 --json headRefOid,baseRefOid,files` reported the revision and base above and these five files. `git diff --name-only <base>...<head>` independently returned the same list:
  - `crates/rt3-kit/src/terrain.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `docs/evidence/gmp-terrain-v1.md`
  - `docs/evidence/terrain-scan-v1.json`
  - `reviews/12-framed-terrain-impl-01.md`
- **Delta from round 1:** `git diff a0c272d72230527dd31a7001c18d7b4c6abc94a7..HEAD` shows the evidence range correction; the round-1 review record is also present in the cumulative PR diff. `terrain.rs`, `map_cmd.rs`, and `terrain-scan-v1.json` are unchanged from the reviewed implementation.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`; OpenCode implementation review, round 2.
- **Design:** none, as recorded in the project slot.

## Round 1 finding resolution

The round-1 blocking finding identified Britain’s 84-byte difference from its merge-base artifact start. `docs/evidence/gmp-terrain-v1.md:54-56` now states **84–96 bytes (21–24 f32 cells)**. Re-comparison of every row in the merge-base and current artifacts confirms deltas `{84, 88, 92, 96}`; Britain is the only 84-byte case. The PR body also now states 84–96 bytes.

## Findings

No new findings. The round-1 blocking finding is resolved.

## Verification

- Independently enumerated all 40 original current version-9701 maps read-only. Each has exactly one complete matching frame; every artifact offset is marker + 32, dimensions match the header, status is `verified`, and `missed` is null. Delta counts are 84: 1, 88: 12, 92: 14, 96: 13; residues are 10/18/6/6. The artifact has 40 verified maps and no candidate.
- The 27 `dims: null` maps remain consistent with the evidence document’s 27-row oracle table. California remains framed at marker `0x845b`, grid `0x847b`, 257×257. The parser, CLI preference and artifact are unchanged from round 1.
- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME` set; `~/.cargo/bin/cargo test --workspace -- --nocapture` — passed: 36 `rt3-kit` tests, 3 `rt3ctl` tests, and doc tests.
- `RT3_HOME` set and `CARGO=$HOME/.cargo/bin/cargo`; `sh tools/evidence.sh check` — passed.
- `RT3_HOME` set; `~/.cargo/bin/cargo run -q --release -p rt3ctl -- map terrain 'California Island'` — passed; output reported `0x847b` (257×257) `[verified]`.
- No emulator was started.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

Landed with merge commit `91332ca` (delegated merge per decision 0002,
2026-11-23). Done-when items and evidence:

- `read_framed` parses the complete eight-word terrain frame at any byte
  alignment, bounds geometry, validates at `section + 32` and extracts
  every cell; the CLI prefers it while fallback APIs remain intact.
- All 40 current maps reconcile against their structural frames and are
  verified in the regenerated artifact. Previous starts were 84–96
  bytes early; round-1 review caught Britain's 84-byte case and the
  corrected range was verified in round 2.
- California Island decodes at `0x847b` from its `0x845b` section and no
  current candidate remains. Synthetic signature/refusal and exact
  California regressions were observed failing before implementation.
- Formatting, clippy, 36 `rt3-kit` tests, three `rt3ctl` tests and the
  install-backed evidence check passed; CI and GitGuardian succeeded on
  the reviewed head.
- Final AGREE covers implementation revision
  `b982a80a7770efca5e8b825ab807aa5c07f73540` and is posted on PR #19.

— Implementer (zai-coding-plan/glm-5.3)
