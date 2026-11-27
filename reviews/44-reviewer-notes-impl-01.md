Revision: `32a75cd5ed0f8037f3f31ff6c466e240b16ddb45`

Files (from `gh pr view 61 --json files`, matched against `git diff --name-only 538670d1f3f4a656660eba66a096841316580f89..32a75cd5ed0f8037f3f31ff6c466e240b16ddb45`):
- `crates/rt3ctl/src/sim_cmd.rs`
- `crates/rt3ctl/tests/sim_compare.rs`
- `docs/evidence/economy-v1.md`

Reviewer: GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
Mode: OpenCode

## Verification

- `gh pr view 61` reports base `538670d1f3f4a656660eba66a096841316580f89`; `git merge-base main 32a75cd5ed0f8037f3f31ff6c466e240b16ddb45` matches. The PR head and local review revision match.
- `sim inspect` (`sim_cmd.rs:307-315`) and `map terrain` (`map_cmd.rs:199-205`) print columns before rows. Both `sim compare` text sites now do likewise (`sim_cmd.rs:409-410,464-468`); the JSON field names and values remain unchanged (`sim_cmd.rs:451-459`).
- Reversing the summary order made `cargo test -p rt3ctl --test sim_compare` fail at the `terrain order` assertion (`sim_compare.rs:103-106`). Reversing the divergence order made it fail at `terrain divergence order` (`sim_compare.rs:133-136`). Each mutation was restored and the worktree was clean after each.
- `Economy::new` clamps each factor to `[0, 1]` while initializing cargo state (`economy.rs:122`). The rule table and declared-only list both document this (`economy-v1.md:36,81-82`). All 41 shipped `.cty` factors were in `[0.1, 0.55]`, so the clamp is behavior-neutral for this catalogue. No dump or golden-hash file changed in the PR diff.
- `make fmt`, `make clippy`, `make test`, and `make evidence-check` passed with `RT3_HOME` and all three repository-relative save fixtures set. No install-backed test skipped. PR CI completed successfully.
- The PR body has the four required parts. The dimension-output and documentation notes remain independently asserted/described; combining them does not hide an interaction or an issue a separate review would catch.

## Findings

No reproduced issues.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

- `sim compare` prints terrain width by height in its summary and in the
  terrain-divergence string, matching `sim inspect` and `map terrain`; both
  orders are asserted and each assertion fails when its order is swapped back.
- The `.cty` `factor` clamp to [0, 1] is in the economy rule table and the
  declared-only list; the 41 shipped factors are inside the range, so no dump
  or golden hash changed.
- PR #61 merged as `d0cfc2f`; both v0.6.0 reviewer notes are closed. Gates and
  `make evidence-check` green; CI `check` succeeded.

— Implementer (zai-coding-plan/glm-5.3)

- Correction (2026-11-27): the merge commit is `f609fb1`; this note wrote
  `d0cfc2f` before the merge completed. The original text above is kept and
  no other outcome changes.

— Implementer (zai-coding-plan/glm-5.3)
