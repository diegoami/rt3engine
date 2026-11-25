## Where I reviewed

- **Revision:** `3f4dd884128901e7d3e261b817684b57d94730aa` (PR #44 head).
- **Target proof:** `gh pr view 44 --json headRefOid,baseRefOid,files` reported this head and base `9e8f22952c8d413a36404ec9ecb50c2b0fbdf41a`; local `HEAD` and `origin/fix/43-evidence-totals` equal the PR head.
- **File list:** `gh pr view 44 --json ...files` and `git diff --name-only $(git merge-base main HEAD)..HEAD` report `docs/environment.md`, `docs/evidence/README.md`, `reviews/33-evidence-totals-impl-01.md`, `reviews/33-evidence-totals-impl-02.md`, and `tools/evidence.sh`. The two review files are prior-round records; the three implementation paths remain `docs/environment.md`, `docs/evidence/README.md`, and `tools/evidence.sh`. I read the full merge-base diff and the round-3 change in `tools/evidence.sh`.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`).
- **Mode:** OpenCode implementation review, round 3, independent reviewer.

## Verification

- **Round-2 finding re-checked:** the duplicate-row false green is fixed. In a scratch copy, a correct README row followed by a duplicate row stating `11,177 files` made `env -u RT3_HOME sh tools/evidence.sh check` report drift and exit 1.
- The selftest passes on the current code. In a scratch tree with `docs` symlinked, I changed `check_doc_totals_file` to accept only the first matching row; the selftest reported `accepted a stale duplicate row` and exited 1. Its missing-row and two-correct-row cases also passed in the normal check.
- The clean check passed with `RT3_HOME` unset and with the read-only installation and all three supplied save fixtures set. No emulator was booted.
- `make fmt`, `make clippy`, and `make test` passed; 59 `rt3-kit` and 10 `rt3ctl` tests passed. With an otherwise empty environment, `/bin/sh` (dash) and `bash --posix` both passed the no-install check, so I reproduced no `set -u` or shell-mode failure.
- CI check run `45624110250` is green.

## Findings

1. **Blocking — a near-miss artifact name is accepted as the exact row.** `tools/evidence.sh:60` searches with the regular expression `install-inventory-v1.json`; the unescaped `.` matches any character. In a scratch copy I changed the README's artifact name to `install-inventory-v1xjson`, leaving its totals correct. `env -u RT3_HOME sh tools/evidence.sh check` exited 0 even though the README had no row naming `install-inventory-v1.json`. This bypasses the zero-matching-rows check. Match the artifact name literally and add a regression case for a near-miss name.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
