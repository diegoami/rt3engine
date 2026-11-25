## Where I reviewed

- **Revision:** `fb1bfae85127c533a4955ee977101d68478f3dbc` (PR #44 head).
- **Target proof:** `gh pr view 44 --json headRefOid,baseRefOid,files` reported this head and base `9e8f22952c8d413a36404ec9ecb50c2b0fbdf41a`; local `HEAD` and `origin/fix/43-evidence-totals` equal the PR head.
- **File list:** `gh pr view 44 --json ...files` and `git diff --name-only $(git merge-base main HEAD)..HEAD` report `docs/environment.md`, `docs/evidence/README.md`, `reviews/33-evidence-totals-impl-01.md`, `reviews/33-evidence-totals-impl-02.md`, `reviews/33-evidence-totals-impl-03.md`, and `tools/evidence.sh`. The review files are prior-round records; the implementation paths remain the three stated docs/tool files. I read the full merge-base diff and the round-4 change in `tools/evidence.sh`.
- **Owner authorization:** confirmed in PR comment https://github.com/diegoami/rt3engine/pull/44#issuecomment-7167179121.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`).
- **Mode:** OpenCode implementation review, owner-authorized round 4.

## Verification

- **Round-3 finding re-checked:** the README near-miss `install-inventory-v1xjson`, with correct totals, now makes `env -u RT3_HOME sh tools/evidence.sh check` report `no install-inventory-v1.json row` and exit 1.
- In a scratch tree with `docs` symlinked, reverting the selector from `grep -F` to the regex form made the selftest report `accepted a near-miss artifact name` and exit 1. Restoring the current script made the selftest and no-install check pass.
- The clean check passed with `RT3_HOME` unset and with the read-only installation and all three supplied save fixtures set. No emulator was booted.
- `make fmt`, `make clippy`, and `make test` passed; 59 `rt3-kit` and 10 `rt3ctl` tests passed. With an otherwise empty environment, `/bin/sh` (dash) and `bash --posix` both passed the no-install check; I reproduced no `set -u` or shell-mode failure.
- CI check run `45647581345` is green.

## Findings

1. **Blocking — a longer filename containing the artifact name is accepted as the artifact row.** `tools/evidence.sh:60` uses a fixed-string substring search, not an exact filename match. In a scratch copy I changed the README's only artifact row to `install-inventory-v1.json.bak`, leaving the totals correct. `env -u RT3_HOME sh tools/evidence.sh check` exited 0 even though the README had no row for the exact artifact `install-inventory-v1.json`. Require an exact filename boundary and add a regression case for a suffixed lookalike.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
