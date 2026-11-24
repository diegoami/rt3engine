# Implementation review — controlled save inventory

## Target

- **Revision:** `4207c539c9d8fc89587d07a6463537627a7f9ef6`
- **Pull request:** [#33](https://github.com/diegoami/rt3engine/pull/33), `m4/controlled-save-inventory` → `main`
- **Base:** `04941ad44fad41212a0a6d12ee5adbc2b02930c7`
- **Files checked:** `crates/rt3-kit/src/save.rs`, `crates/rt3ctl/src/save_cmd.rs`, `docs/environment.md`, `docs/evidence/README.md`, `docs/evidence/controlled-save-inventory-v1.json`, `docs/evidence/gmc-save-v1.md`, `docs/evidence/install-inventory-v1.json`, `tools/evidence.sh`.
- **How obtained:** `gh pr view 33 --json headRefOid,baseRefOid,files` and `gh pr diff 33 --name-only`, cross-checked against `git rev-parse` and `git diff --name-only <base>...<head>`. The PR file list and local diff list matched.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`.
- **Mode:** OpenCode implementation review; project `design: none`; ordinary merge delegated to the implementer on AGREE.

## Findings

None.

## Verification

- Independently `stat`ed and hashed the private `.gms`: 10,285,120 bytes and SHA-256 `f892bc28ea1fb3bb1ff60f3ed443e82de7818d3fd7a7c26a93e974cfb69ac6bc`. `git check-ignore` confirmed it is ignored by `/review-tmp/`, and `git ls-files` confirmed it is not tracked.
- Ran `rt3ctl save inspect --json` on that fixture. Its version, hash, terrain dimensions and range, section offsets and end offsets, capacities, and record counts match `controlled-save-inventory-v1.json` exactly.
- Ran the save tests with both fixture variables unset: 4 passed, with explicit skip messages for both optional fixtures. Ran them with both absolute fixture paths set: 4 passed.
- Ran `make fmt`, `make clippy`, and `make test` (54 `rt3-kit` tests and 5 `rt3ctl` tests passed). Also ran `make evidence-check` with `RT3_HOME`, `RT3_SAVE`, and `RT3_CONTROL_SAVE` set: passed. This regenerated neither artifacts nor fixture bytes; it verified committed evidence against the live installation and both fixtures.
- `git diff --check` passed. Reviewed the complete PR diff and changed files. No emulator or native game launch was performed.

The documented GOG `SAVEGAMEFOLDER` write hazard and the requirement to redirect and verify it for future native runs are in scope and recorded. Original-game read-back and a second controlled save remain explicitly open.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
