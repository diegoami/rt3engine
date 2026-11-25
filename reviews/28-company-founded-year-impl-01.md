# PR #37 — Company founded year implementation review

- **Revision reviewed:** `9dcd021ae39849133223747e6068d5c55db334f7`
- **PR head:** `9dcd021ae39849133223747e6068d5c55db334f7`; **base:** `main` at `69b2f3d611147985f94f51cb54768c8cef2c71d8`.
- **File list:** obtained with `gh pr view 37 --json files`; it matches `git diff --name-status $(git merge-base main 9dcd021)..9dcd021`.
  - `crates/rt3-kit/src/companies.rs`
  - `crates/rt3-kit/src/save.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `crates/rt3ctl/src/save_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/controlled-save-inventory-v1.json`
  - `docs/evidence/gmc-save-v1.md`
  - `docs/evidence/gmp-actors-v1.md`
  - `docs/evidence/save-inventory-v1.json`
  - `docs/evidence/terrain-scan-v1.json`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; the project slot records `design: none`.

## Review

No blocking or non-blocking findings.

`companies::founded_year` reads a little-endian `u32` at record offset 67 and returns `None` if the record does not contain the full field (`crates/rt3-kit/src/companies.rs:15,23-32`). The all-map test checks 43 maps, 37 records, and the reported year distribution (`crates/rt3-kit/src/companies.rs:54-102`). The save fixture test asserts `[1830, 1830, 1829]` and identifies Great Republic (`crates/rt3-kit/src/save.rs:225-249`). I also inspected the private controlled-save records: Edinburgh Express (Liquidated) and London Rails (Liquidated) each contain 1830; Great Republic contains 1829.

Both JSON producers emit `founded_years` arrays, including empty arrays (`crates/rt3ctl/src/map_cmd.rs:350-363,424-428`; `crates/rt3ctl/src/save_cmd.rs:86-99`). The regenerated-output check and direct JSON checks verified 43 map arrays, 37 total map records, the specified distribution, integer bounds, and both save arrays (`[]` and `[1830, 1830, 1829]`). The documentation describes the controlled January 1829 scenario without making a current-date claim (`docs/evidence/gmp-actors-v1.md:48-54`; `docs/evidence/gmc-save-v1.md:80-83`).

## Verification

The Makefile recipes were run directly because `make` is unavailable in this environment:

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` with `RT3_HOME`, `RT3_SAVE` and `RT3_CONTROL_SAVE` set to the supplied fixtures — **59 `rt3-kit` and 5 `rt3ctl` tests passed**, including the all-map and both save-fixture tests.
- `sh tools/evidence.sh check` with those variables — passed; generated all-map and save JSON matched the committed artifacts.
- `gh pr checks 37` — GitGuardian and CI checks passed.

No emulator or native game launch was performed, and the original installation was only read.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
