# Implementation review — issue #56

- **Revision:** `bd66c7071cceb424f96e316e9162168f89e3b6af`
- **Pull request:** [#57](https://github.com/diegoami/rt3engine/pull/57), base `main` at `e91da288513c5c3ab6d8d5491a1d8127d51f48d2`
- **Files checked (PR file list):**
  - `crates/engine/tests/world.rs`
- **How obtained:** `gh pr view 57 --json headRefOid,baseRefOid,files` reported the revision and one-file list above; local `HEAD` matched the PR head. `git merge-base main bd66c7071cceb424f96e316e9162168f89e3b6af` equals the reported base, and `git diff --name-only <base>..<head>` matches the PR file list. Read the full merge-base diff; it changes only the malformed-prelude test.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Verification

- `rejects_malformed_prelude_despite_intact_sections` captures `error.detail()` before removing the temporary file, then asserts both the parser reason and absence of that path. `WorldError::detail()` returns only its stored detail; `engine::world::load_map` stores the parser's `MapError::detail()`, while `MapError::Display` includes the path.
- Temporarily changed `engine::world` to format the parser error through `MapError::Display`. The malformed-prelude test failed at the new path-leak assertion, and its message contained the generated temporary map path and `scenario text is not NUL-terminated`. Restored `world.rs` byte-for-byte; `git status` was clean.
- `$HOME/tools/bin/make fmt`, `make clippy`, and `make test` passed. `make test` ran all four `engine` world tests with zero skipped; the workspace tests and install-backed evidence check also passed. No emulator was booted.
- PR CI: `check` and GitGuardian Security Checks passed.
- The PR body has the four required parts; `Closes #56` is on its own line. `gh pr view 57 --json closingIssuesReferences` lists issue #56.

## Findings

None.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

- `rejects_malformed_prelude_despite_intact_sections` now asserts the parser
  reason through `WorldError::detail()` and that the detail excludes the
  temporary map path; the path-leaking `Display` is no longer the assertion.
- Demonstrated: wrapping the parser error with `MapError`'s `Display` makes the
  test fail at the path-leak assertion; the production code is unchanged.
- PR #57 merged as `9dd6b4c`; issue #56 is closed. Gates green on the reviewed
  revision; the world tests ran without skips, and CI `check` run 45856662554
  succeeded.

— Implementer (zai-coding-plan/glm-5.3)
