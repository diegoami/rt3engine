## Where I reviewed

- **Revision:** `68f8d6e41506d07cedc4804b611f7a8a3b2dfeb6` (PR #44 head).
- **Target proof:** `gh pr view 44 --json headRefOid,baseRefOid,files` reported this head and base `9e8f22952c8d413a36404ec9ecb50c2b0fbdf41a`; local `HEAD` and `origin/fix/43-evidence-totals` equal the PR head. `git merge-base main HEAD` equals the PR base.
- **File list:** obtained from `gh pr view 44 --json ...` and independently compared with `git diff --name-only $(git merge-base main HEAD)..HEAD`; both list exactly `docs/environment.md`, `docs/evidence/README.md`, and `tools/evidence.sh`. I read the full diff from that merge-base.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`).
- **Mode:** OpenCode implementation review, independent fresh-context reviewer.

## Verification

- The artifact totals are **1,177 files** and **2,032,240,149 bytes**; the two documents state **1,177 files** and **~2.03 GB**.
- `env -u RT3_HOME sh tools/evidence.sh check` passed, confirming the check runs without an installation and without unset-variable failure. In a scratch copy, changing the README count to `1,176 files` made the same command print `documented totals drift: docs/evidence/README.md does not state 1177 files` and exit 1.
- In a scratch copy, a stale README count of `11,177 files` passed with exit 0. With the count correct but its size changed to `~12.03 GB`, the check also passed with exit 0. See finding 1.
- The awk extraction returned `1177` and `2032240149`; byte conversion returned `2.03` for the artifact, `2.03` below the 2.035-billion rounding boundary, and `2.04` at it. A compact JSON spelling is rejected; the current inventory producer emits the deterministic multi-line format the check reads (`crates/rt3ctl/src/inventory.rs:40-55,99-105`).
- The install-backed `sh tools/evidence.sh check` passed with `RT3_HOME` and all three supplied save fixtures set. The installation was read-only; no emulator was booted.
- The PR body contains all four required parts: what was built, ticked done-when items, check output, and what was left out. `Closes #43` is on its own line, and GitHub lists issue #43 as a closing reference. CI check run `45621953823` is green.

## Findings

1. **Blocking — substring matching lets stale totals pass.** `tools/evidence.sh:50,54` searches for the expected text as an unanchored substring. I reproduced false greens in a scratch copy: `11,177 files` contains the searched `1177 files`, and `12.03 GB` contains `2.03 GB`; each incorrect documented total passed `env -u RT3_HOME sh tools/evidence.sh check` with exit 0. This defeats the change's required assertion that stale prose is caught. Require whole-number/whole-value matching for each documented total and add a regression check for these false-green cases.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
