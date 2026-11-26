# PR #55 implementation review

- **Revision:** `5731819ec4a8c9f13bc1edfa3d8983137c2338bb`
- **Files reviewed:** obtained from `gh pr view 55 --json files` and
  cross-checked against `git diff --name-only e0f5f0978007907ddb91532c99a9fd5bd0eee61a..5731819ec4a8c9f13bc1edfa3d8983137c2338bb`, after confirming that
  `git merge-base main 5731819ec4a8c9f13bc1edfa3d8983137c2338bb` is
  `e0f5f0978007907ddb91532c99a9fd5bd0eee61a`:
  - `crates/rt3-kit/src/cities.rs`
  - `crates/rt3-kit/src/events.rs`
  - `crates/rt3-kit/src/sparse.rs`
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`
- **Mode:** OpenCode; project slot records `design: none` and delegated merge.

## Verification

- The full merge-base diff changes test code only, including a synthetic
  collection builder; no parser behavior, format claim or shipped fixture
  changed.
- Temporarily removing the sparse end-tag guard made
  `sparse::tests::rejects_truncated_and_malformed_sparse_collections` fail with
  `accepted prefix 12449`. Temporarily removing the city terminator guard made
  `cities::tests::reads_sparse_city_slots_and_raw_labels` fail with
  `accepted city prefix 14199`. Both temporary edits were reverted.
- The test-count guard issue in finding 1 was reproduced: removing
  `records.len() >= active` still let
  `sparse::tests::rejects_truncated_and_malformed_sparse_collections` pass.
- `$HOME/tools/bin/make fmt`, `make clippy` and `make test` passed with
  `RT3_HOME` and the documented fixtures under `review-tmp/saves/`. The
  workspace results match the PR body: 5 economy, 4 world, 60 `rt3-kit`, 10
  `rt3ctl`, 1 `sim_compare` and 2 `sim_run` tests passed. The install-backed
  city, building and event collection tests passed, asserting 3,726, 2,044
  and 1,169 active records respectively; none skipped.
- CI `check` and GitGuardian Security Checks passed. No emulator was booted.
- The PR body has the four required parts; `Closes #54` is on its own line and
  GitHub lists issue #54 as a closing issue. The sparse end-tag and city
  terminator demonstrations reproduced the matching outputs claimed in the
  body.

## Findings

1. **blocking** — `crates/rt3-kit/src/sparse.rs:217-222` (mask decoding at
   `crates/rt3-kit/src/sparse.rs:100-105`, final record-count check at
   `crates/rt3-kit/src/sparse.rs:118`). The mask uses set bits to
   mark absent slots, so setting `no_active[base + 136]` to `0` marks all four
   slots active, not zero. The parser rejects this case at the
   `records.len() >= active` guard when it reaches the third active slot, not
   because the mask yields fewer records than the header declares. The fixture
   contains only two record payloads; removing that guard still leaves the
   test green because reading a third payload later runs out of bytes. Thus
   this assertion does not verify the stated under-count case or provide a
   regression for the record-count guard. Use an isolated under-count fixture
   (for example, all slots absent with the end tag immediately after the mask
   while `active` remains 2), so the final `records.len() != active` guard is
   what rejects it.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
