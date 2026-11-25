# Implementation review — M5 C5 (round 3)

- **Revision:** `be9bc6bac8f3f074e88fc3d3578818bd768187d2`
- **Pull request:** [#49](https://github.com/diegoami/rt3engine/pull/49), base `main` at `65758edc653d41b768f5f1a37c9592da1cc923c0`
- **Files checked (PR file list):**
  - `crates/engine/src/world.rs`
  - `crates/engine/tests/world.rs`
  - `crates/rt3-kit/src/map.rs`
  - `crates/rt3-kit/src/typed.rs`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/sim_cmd.rs`
  - `crates/rt3ctl/tests/sim_compare.rs`
  - `docs/evidence/economy-v1.md`
  - `reviews/38-sim-compare-impl-01.md`
  - `reviews/38-sim-compare-impl-02.md`
- **How obtained:** Fetched `origin/main` and `origin/m5/compare`; `gh pr view 49 --json headRefOid,baseRefOid,files` and local `HEAD` reported revision `be9bc6bac8f3f074e88fc3d3578818bd768187d2`, with base `65758edc653d41b768f5f1a37c9592da1cc923c0`. `git merge-base origin/main HEAD` equals the reported base, and the local merge-base diff names match GitHub's ten paths. The current implementation list includes the two error accessors, plus `crates/engine/tests/world.rs`; the PR also contains both earlier review records. Read the full merge-base diff and the round-2-to-round-3 changes.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Round 2 follow-up

1. **Resolved.** The path-free `MapError::detail()` is used when `engine::world` wraps parser errors. Reproduced the malformed-prelude case through both `sim inspect` and `sim compare`; stderr named the temporary map and gave the parser reason without an absolute path.

## Verification

- Generated a temporary British Isles map with the prelude bytes damaged while preserving the later sections. Both `sim inspect` and `sim compare` rejected it with a named, path-free reason. A non-map file used as the map and an invalid save likewise reported only the basename and a clear error. A malformed `.cty` in a temporary install root also produced a path-free typed-data error.
- A valid British Isles JSON comparison matched terrain `513×321`, cities `68` / `68`, buildings `0` / `520`, companies `0` / `3`, 41 cargo types and source map `British Isles.gmp`. The California Island comparison reported the expected source-map divergence. The relative-fixture `sim_compare` test passed; `make test` passed the decade/replay tests on both maps.
- `MapError::Display` and `TypedError::Display` still include their stored path; `map_cmd` and `typed_cmd` still use `to_string()`. `engine::world` uses the new `MapError::detail()` and `TypedError::detail()` accessors when it needs path-free details.
- **Test coverage note:** `rejects_non_map_files_with_a_named_error` now asserts that `error.detail()` excludes the temporary path (`crates/engine/tests/world.rs:198-208`). The malformed-prelude test still checks `error.to_string()` for the reason (`world.rs:167-191`); the path-free behavior for that case was reproduced through both CLI commands above.
- `$HOME/tools/bin/make fmt`, `make clippy`, and `make test` passed. For `make test`, `RT3_SAVE`, `RT3_CONTROL_SAVE` and `RT3_CONTROL_SAVE_2` used absolute paths to fixtures under `review-tmp/saves/`. CI `check` passed for this revision (run `45678856276`). No emulator was booted.

## Findings

1. **non-blocking — the malformed-prelude unit test does not assert path-free detail.** `crates/engine/tests/world.rs:167-191` still checks `error.to_string()`, while only the non-map test asserts that `error.detail()` excludes the temporary path (`crates/engine/tests/world.rs:198-208`). The malformed-prelude CLI behavior passes both reproductions, so this does not block the path-leak fix; consider adding the same detail assertion to that test.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

- `rt3ctl sim compare` and the C5 divergence record land: PR #49, merged as
  `0de7059`.
- On British Isles: terrain 513×321 on both sides, 68 city records on both,
  0 map building records against the save's 520, 0 companies against 3, and 41
  shared cargo types. Every difference and every declared-only rule is listed
  in `docs/evidence/economy-v1.md`.
- Reviews: `reviews/38-sim-compare-impl-01.md` (BLOCK), `-02.md` (BLOCK) and
  `-03.md` (AGREE). The non-blocking note that the malformed-prelude unit test
  asserts `Display` rather than path-free `detail()` is recorded; the CLI
  path-free behaviour is covered by the non-map test and by the malformed-map
  reproduction.
- Gates on the change: `make fmt`, `make clippy` and `make test` passed; CI
  green.

— Implementer (zai-coding-plan/glm-5.3)
