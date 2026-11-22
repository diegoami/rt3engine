# Implementation review: M2 typed game-data formats

- **Revision covered:** `6767a1348699db7149f2d49bd6783230a72e817d`
- **Files checked:** `crates/rt3-kit/src/lib.rs`, `crates/rt3-kit/src/typed.rs`, `crates/rt3ctl/src/main.rs`, `crates/rt3ctl/src/typed_cmd.rs`, `docs/evidence/README.md`, `docs/evidence/typed-formats-v1.md`, `docs/evidence/typed-tables-v1.json`, `tools/evidence.sh`.
- **How obtained:** local diff from merge-base `c5226a62fdce54d052e1890322a61b9e53ae4a38` (`git merge-base main m2/typed-data`, then `git diff --name-only <merge-base>..m2/typed-data`); checked-out `HEAD` matched the covered revision.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`
- **Mode:** OpenCode

## Findings

1. **blocking** — `crates/rt3-kit/src/typed.rs:353-358`; format evidence: `docs/evidence/typed-formats-v1.md:70-72`. The documented `.car` tail begins at byte 252 and lays out `u32; u8; u8; u32; u8; u8; u32; u32`, so the second `u32` starts at byte 258. `CarType::parse` instead reads it at 260. An independent Python spot-check of a shipped 272-byte, version-1002 `.car` decoded different tail tuples at the documented offsets `[252, 258, 264, 268]` and the parser offsets `[252, 260, 264, 268]`. This returns an incorrect field value for the documented layout; read the second `u32` at 258 and cover the offset with distinct fixture values.

2. **blocking** — `crates/rt3-kit/src/typed.rs:143-155,217-229`. Both `BuildingType::parse` and `BuildingBehavior::parse` call `i32_at(d, 0)` before checking the input length. Inputs shorter than four bytes therefore panic instead of returning `TypedError`; reproduced by calling both public parsers with an empty slice in an external probe. Since `scan_family` expects parser errors to become per-file failures, a truncated `.bty` or `.bca` can abort the typed CLI rather than be reported. Check the minimum version-field length before reading it and exercise these truncated cases.

## Verification

- `make test` passed with `PATH="$HOME/tools/bin:$PATH"` and `RT3_HOME` set to the requested installation. The output showed `typed::tests::parses_every_shipped_typed_file ... ok`, so the install-backed test ran rather than self-skipped; the evidence check also passed.
- `make fmt` passed.
- `make clippy` passed with warnings denied.
- Independent Python spot-checks against `$RT3_HOME` confirmed a `.cty` at 62 bytes/version 1002, an `.lco` at 91 bytes/version 2005, and a `.car` at 272 bytes/version 1002; the `.car` check exposed finding 1.
- The changed evidence JSON contains aggregate counts/config metadata only; the diff adds no original game bytes. The format note and parser keep the claim at **parse**, with field semantics explicitly left open.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK

### Response to round 1

— Implementer (zai-coding-plan/glm-5.3)

1. **blocking (.car tail offsets): fixed.** `tail_u32s` now reads the u32s
   at 252/258/264/268 (was 252/260/264/268, reading a misaligned field).
   The corrected offsets decode the real tail meaningfully: u32@258 and
   u32@264 are intro/end years (242 A1: 1946/1960). A regression test pins
   all four tail u32s against a fixture with known years and float.
2. **blocking (short-input panics): fixed.** `BuildingType::parse` and
   `BuildingBehavior::parse` now length-check before the first read and
   return `TypedError` for inputs under 4 bytes; new
   `short_inputs_error_instead_of_panicking` test covers lengths 0/1/3.

Gates after the fixes: `make test` (13 rt3-kit tests incl. install-backed),
`make fmt`, `make clippy` all pass.

## Round 2 verdict

- **Revision covered:** `02837caf89c63e93036f16d2afa5432ed30b057e`
- **Files checked:** `crates/rt3-kit/src/lib.rs`, `crates/rt3-kit/src/typed.rs`, `crates/rt3ctl/src/main.rs`, `crates/rt3ctl/src/typed_cmd.rs`, `docs/evidence/README.md`, `docs/evidence/typed-formats-v1.md`, `docs/evidence/typed-tables-v1.json`, `reviews/02-typed-impl-01.md`, `tools/evidence.sh`.
- **How obtained:** local diff from merge-base `c5226a62fdce54d052e1890322a61b9e53ae4a38` (`git merge-base main HEAD`, then `git diff --name-only <merge-base>..HEAD`); `HEAD` matched the covered revision. Inspected `git show 02837ca`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`
- **Mode:** OpenCode

### Round-1 finding resolution

1. **round-1 blocking finding — resolved.** `crates/rt3-kit/src/typed.rs:365-370` now reads the four `.car` tail `u32`s at offsets 252/258/264/268. The `car_and_bca_shape` regression fixture assigns distinct values at those offsets and asserts the decoded array, including the intro/end year values. This addresses the misaligned read at 260.
2. **round-1 blocking finding — resolved.** `crates/rt3-kit/src/typed.rs:143-150,223-230` checks for the four-byte version field before reading it. `short_inputs_error_instead_of_panicking` at line 655 exercises lengths 0, 1, and 3 for both parsers and asserts errors.

No blocking findings remain.

### Verification

- `make test fmt clippy` passed with the requested `PATH` and `RT3_HOME` set.
- The test output showed `typed::tests::parses_every_shipped_typed_file ... ok`; the install-backed typed test ran, not skipped. All 13 `rt3-kit` tests passed, and the evidence check, formatting check, and warning-denied Clippy check passed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

Landed with merge commit (owner-delegated merge per decision 0002,
2026-11-22). Done-when items and their evidence:

- Issue #5 (format RE): `docs/evidence/typed-formats-v1.md` documents all
  typed families with Ghidra loader addresses and exact-length validation
  over every shipped file; open semantics recorded.
- Issue #6 (readers): `rt3-kit::typed` parsers + `scan_family`;
  install-backed test parses all shipped typed files; `rt3ctl typed list`
  CLI; aggregate evidence `typed-tables-v1.json` in `make evidence-check`.
- Claim level achieved: **parse**. `.car` tail years and short-input error
  paths carry regression tests from the review rounds.

— Implementer (zai-coding-plan/glm-5.3)
