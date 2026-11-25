## Where I reviewed

- **Revision:** `da2b05184ee72aaa293b3ccb8b79f081b20f3572` (PR #44 head).
- **Target proof:** `gh pr view 44 --json headRefOid,baseRefOid,files` reported this head and base `9e8f22952c8d413a36404ec9ecb50c2b0fbdf41a`; local `HEAD` and `origin/fix/43-evidence-totals` equal the PR head.
- **File list:** `gh pr view 44 --json ...files` and `git diff --name-only $(git merge-base main HEAD)..HEAD` both report `docs/environment.md`, `docs/evidence/README.md`, `reviews/33-evidence-totals-impl-01.md`, and `tools/evidence.sh`. The review file is the round-1 record already committed to the branch; the three implementation paths are `docs/environment.md`, `docs/evidence/README.md`, and `tools/evidence.sh`. I read the full diff from the merge-base and the round-2 matcher change in `tools/evidence.sh`.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`).
- **Mode:** OpenCode implementation review, round 2, independent reviewer.

## Verification

- **Round-1 finding re-checked:** the blocking substring-match finding in `tools/evidence.sh:50,54` is fixed for the reported values. In scratch copies, changing the README to `11,177 files` and, separately, `~12.03 GB` each made `env -u RT3_HOME sh tools/evidence.sh check` report drift and exit 1.
- The selftest rejects the correct-total line's stale variants, including `1,176 files`. In a scratch copy with the matcher loosened to substring matching, the selftest reported that it accepted the stale file count and size, then exited 1.
- A missing README inventory row reports drift and exits 1. A duplicate valid inventory table row after the correct row, with `11,177 files`, passes with exit 0; see finding 1.
- The clean check passed with `RT3_HOME` unset and with the read-only install and all three supplied save fixtures set. `make fmt`, `make clippy`, and `make test` passed; the unit tests reported 59 `rt3-kit` and 10 `rt3ctl` tests passing. No emulator was booted.
- CI check run `45623043154` is green.

## Findings

1. **Blocking — a duplicate inventory row can hide stale totals.** `tools/evidence.sh:85` uses `grep -m1`, so it validates only the first line containing `install-inventory-v1.json`. In a scratch copy I left the correct README table row first, then added a second valid table row for the same artifact stating `11,177 files`; `env -u RT3_HOME sh tools/evidence.sh check` exited 0. The check therefore accepts contradictory documented totals. Require exactly one matching row (and reject duplicates) in each document, with a regression test for a correct row followed by a stale duplicate.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
