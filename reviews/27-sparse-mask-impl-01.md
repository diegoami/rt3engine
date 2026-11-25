# Implementation review — sparse mask exact-multiple edge

- **Revision:** `4f28092a35fed324c2fb8eac642d944ccef567df`
- **Base:** `5c8e4955ab989642c37685c1aba6d38b5458ec32`
- **Scope proof:** `gh pr view 36 --json headRefOid,baseRefOid,files`; PR #36 was open with the reported head equal to local `HEAD`, and its file list matched `git diff --name-only <base>..<head>`:
  - `crates/rt3-kit/src/sparse.rs`
  - `docs/evidence/gmp-collections-v1.md`
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`
- **Mode:** OpenCode; project slot records `design: none` and delegated merge.

## Findings

No blocking or non-blocking findings.

## Review

`crates/rt3-kit/src/sparse.rs:70-73` rejects capacity above 1,024 before the changed sizing at lines 94-97. Thus `checked_add(8)` is bounded, and shifting right by three gives the original allocation count, including the following byte when capacity is an exact multiple of eight. The slot loop only indexes bits `0..capacity`, so that extra byte advances the record cursor without changing slot-mask semantics. The documentation at `docs/evidence/gmp-collections-v1.md:26-30` now describes that layout.

The synthetic capacity-eight test at `crates/rt3-kit/src/sparse.rs:150-184` uses a two-byte mask, checks the active slot and record payload, and requires every strict prefix to be rejected. I reproduced the regression check from this file: it passes with the changed formula and fails at the positive parse assertion when changed back to `(capacity + 7) / 8`, so the fixture distinguishes the old behavior. Its prefix loop also rejects truncation at every byte boundary, including a missing terminator or incomplete record.

The committed terrain artifact has no city, building or event capacity divisible by eight across its 43 maps. The save inventory artifacts likewise have none (city/building/event capacities are 70/430/30 and 70/530/50). No evidence JSON changed in this PR. Install-backed evidence checking passed with `RT3_HOME` and both optional save fixtures set; the artifacts showed no drift. No emulator or game launch was used, and the original installation and save inputs were only read.

## Verification

- `make -s fmt clippy test` with `RT3_HOME` set — passed; 59 `rt3-kit` tests and 5 `rt3ctl` tests passed, including install-backed map checks.
- `make -s evidence-check` with `RT3_HOME`, `RT3_SAVE` and `RT3_CONTROL_SAVE` set to the available fixtures — passed with no artifact drift.
- The capacity-eight regression test was separately run against both formulas: passed with the new formula; failed with the prior ceil formula as expected.
- `gh pr checks 36` on the reviewed head — GitGuardian and CI passed.
- No emulator, game launch or original-file writes.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
