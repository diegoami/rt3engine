# PR #34 implementation review — save source-map basename

- **Revision:** `cf605b0df19dfa3a5c1090396e805353287d625c`
- **Target proof:** PR #34 head `cf605b0df19dfa3a5c1090396e805353287d625c`,
  base `29bc9c9a0ceda244eb460f440853c495576c4777`. The file list was obtained
  with `gh pr view 34 --json number,title,state,baseRefName,baseRefOid,headRefName,headRefOid,files,url`
  and cross-checked against `git diff --name-status origin/main...HEAD`.
- **Files checked:**
  - `crates/rt3-kit/src/save.rs`
  - `crates/rt3ctl/src/save_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/controlled-save-inventory-v1.json`
  - `docs/evidence/gmc-save-v1.md`
  - `docs/evidence/save-inventory-v1.json`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`), fresh OpenCode
  subagent context.
- **Mode:** OpenCode; project slot is `design: none`. Merge is delegated on
  AGREE; this review does not merge.

## Findings

None.

## Verification

- Independently checked both private save fixtures: the NUL-terminated
  basenames at `0x73d0` are `Go West!.gmp` and `British Isles.gmp`; the
  51-byte field in the shipped British Isles map starts with NUL.
- `cargo test --workspace`, with both fixture variables set: **55 `rt3-kit`
  tests and 5 `rt3ctl` tests passed**, including both source-map fixture
  assertions.
- `cargo test -p rt3-kit save`, with fixture variables unset: passed; optional
  fixture reads self-skip.
- Ran `rt3ctl save inspect` in text and JSON modes for each fixture; both
  outputs expose the expected basename.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `sh tools/evidence.sh check`, with `RT3_HOME`, `RT3_SAVE`, and
  `RT3_CONTROL_SAVE` set: passed with no artifact drift.
- `gh pr checks 34`: all reported checks passed.
- No RT3 launch, native writes, registry changes, or original-game
  modification performed during review.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
