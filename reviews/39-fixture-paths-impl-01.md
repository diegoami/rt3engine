# Implementation review — PR #52 (round 1)

- **Revision:** `9c2efa27b2ee93f8d83a096ae072cf67d721ad69`
- **Pull request:** [#52](https://github.com/diegoami/rt3engine/pull/52), base `main` at `0df8573ba0fb76fbca9ed8f0cb373bc5f34e3322`
- **Files checked (PR file list):**
  - `crates/rt3-kit/src/save.rs`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/save_cmd.rs`
  - `crates/rt3ctl/src/save_diff.rs`
- **How obtained:** `gh pr view 52 --json headRefOid,baseRefOid,files` reported this head, base and four paths; local `HEAD` equalled the reported head. `git merge-base main 9c2efa27b2ee93f8d83a096ae072cf67d721ad69` equalled the reported base, and `git diff --name-only 0df8573ba0fb76fbca9ed8f0cb373bc5f34e3322..9c2efa27b2ee93f8d83a096ae072cf67d721ad69` matched the PR file list above. Read the full merge-base diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Findings

1. **blocking — a missing repository-relative fixture can silently fall back to a crate-relative file.** `crates/rt3-kit/src/save.rs:207-209` and `crates/rt3ctl/src/main.rs:244-246` return the original relative path when the repository-root candidate does not exist. I reproduced this with `RT3_SAVE='../../review-tmp/saves/Central west Pacific 1.gmc' cargo test -p rt3-kit save::tests::reads_original_save`: the repository-root candidate is absent, but the test passed by resolving the value from the crate working directory. The same passed with `RT3_CONTROL_SAVE='../../review-tmp/saves/rt3engine-bi-18290101-202.gms' cargo test -p rt3ctl save_cmd::tests::valid_json_includes_actor_names`. Thus a value missing at the documented repository root may load a different crate-relative file instead of failing. Resolve relative values strictly against the repository root; do not fall back to the original relative path.

## Verification

- Reproduced the original `NotFound` failures on `main` with the documented relative values. On the PR revision, `cargo test -p rt3-kit save` and `cargo test -p rt3ctl` passed with the documented relative values and again with absolute values; install-backed tests ran without skipping. A valid repository-relative value containing `..` passed; unset fixture variables emitted the expected skip messages; a normally missing fixture path failed.
- `$HOME/tools/bin/make fmt`, `make clippy`, and `make test` passed with the documented relative values. PR CI passed (`check` and GitGuardian Security Checks). No emulator was booted.
- The changes are confined to test code (`fixture` is in a test module; `fixture_path` is `#[cfg(test)]`). Assertions and evidence artifacts are unchanged, and both helpers use the same resolution logic.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
