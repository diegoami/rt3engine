# Implementation review — save inventory, round 2

- **Revision:** `67374e5646e5297ca5a8ac7be032c5671534397c`
- **Base:** `d2a42a76efc3eb591e3803d22fceecf2e9030481`
- **Scope proof:** `gh pr view 31 --json headRefOid,baseRefOid,headRefName,baseRefName,files` reported head `67374e5646e5297ca5a8ac7be032c5671534397c`, base `d2a42a76efc3eb591e3803d22fceecf2e9030481`, branch `m4/save-inventory`, and the 12 files listed below. Local `HEAD` and `67374e5^{commit}` resolve to that head; `git diff --name-only <base>...<head>` exactly matches the PR file list.
- **Files checked:**
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3-kit/src/save.rs`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `crates/rt3ctl/src/save_cmd.rs`
  - `docs/environment.md`
  - `docs/evidence/README.md`
  - `docs/evidence/gmc-save-v1.md`
  - `docs/evidence/install-inventory-v1.json`
  - `docs/evidence/save-inventory-v1.json`
  - `reviews/22-save-inventory-impl-01.md`
  - `tools/evidence.sh`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; project slot records `design: none` and delegated merge.

## Findings

No remaining findings. Round 1's blocking inventory-drift finding is resolved.

## Review

The live inventory producer, run against the read-only `RT3_HOME`, exactly matches `docs/evidence/install-inventory-v1.json`: **1,177 files**, **2,031,926,720** total bytes, **927,576,454** bytes under `Data`, and **13,621** `.txt` bytes. Both evidence-check modes pass; with `RT3_SAVE` unset, the optional save check reports its expected skip. The other implementation code and save artifact are unchanged from round 1. The original fixture remains ignored and untracked at `review-tmp/saves/Central west Pacific 1.gmc`, with size **8,490,851** and SHA-256 `3c11cbbdde844273c499086a8a4c23df9e1cfe9837910696fa13bf32d121d99b`.

## Verification

- PR head/base and all 12 changed paths match the local revision and diff.
- Read the complete implementation delta from round 1 and confirmed the only subsequent changes are the corrected inventory metadata, the documented file count, and the round-one review record.
- Reproduced `rt3ctl inventory --json` from the read-only installation; its parsed result exactly equals the committed inventory artifact.
- With the requested `RT3_HOME` and `RT3_SAVE`, `make evidence-check` — passed.
- With `RT3_HOME` set and `RT3_SAVE` unset, `make evidence-check` — passed and explicitly skipped the optional save-inventory check.
- With the requested `RT3_HOME` and `RT3_SAVE`, `make fmt` — passed.
- With the same environment, `make clippy` — passed with warnings denied.
- With the same environment, `make rust` — passed: **53 `rt3-kit` tests** and **5 `rt3ctl` tests**.
- The save fixture hash/size and save-inventory artifact are unchanged; no `.gmc` or `.sav` files are tracked. No emulator was used.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
