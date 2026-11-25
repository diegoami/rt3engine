# Implementation review — M5 C5 (round 1)

- **Revision:** `8af106992c6a3e998862c7d8ffc84ae33b0ee36b`
- **Pull request:** [#49](https://github.com/diegoami/rt3engine/pull/49), base `main` at `65758edc653d41b768f5f1a37c9592da1cc923c0`
- **Files checked (PR file list):**
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/sim_cmd.rs`
  - `crates/rt3ctl/tests/sim_compare.rs`
  - `docs/evidence/economy-v1.md`
- **How obtained:** `gh pr view 49 --json headRefOid,baseRefOid,files` reported head `8af106992c6a3e998862c7d8ffc84ae33b0ee36b`, base `65758edc653d41b768f5f1a37c9592da1cc923c0`, and the four paths above. The local `HEAD` and `origin/m5/compare` equal the PR head; `git merge-base origin/main HEAD` equals the reported base; `git diff --name-only 65758edc653d41b768f5f1a37c9592da1cc923c0..HEAD` matches the PR file list. Read the full merge-base diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Verification

- Ran `rt3ctl sim compare "British Isles"` in text and JSON modes with the controlled save. Both report terrain `513×321` / `513×321`, cities `68` / `68`, buildings `0` / `520`, companies `0` / `3`, and 41 cargo types; JSON reports source map `British Isles.gmp`. The successful output uses only the save basename. Parsed the different-map JSON output with Python's JSON parser; quoted source-map text is escaped correctly.
- Independently cross-checked `controlled-save-inventory-v1.json` (source map, save terrain, city/building/company counts), `terrain-scan-v1.json` (British Isles terrain, city and building counts), and `typed-tables-v1.json` (41 cargo files). All reported figures match.
- Failure probes: missing `--save` exits 2 with a named argument error; `Cargo.toml` as the save exits 1 with a named unsupported-save error; the British Isles save against California Island exits 0 and reports source-map, terrain and city divergences; a missing map exits 2 without panic. The missing-map diagnostic includes the installation root.
- `sim_compare.rs` invokes `env!("CARGO_BIN_EXE_rt3ctl")` and asserts all comparison counts plus building and company divergences. However, with the evidence document's repo-relative `RT3_CONTROL_SAVE` value, `cargo test -p rt3ctl --test sim_compare` fails before a successful comparison because the subprocess resolves the path from the crate test working directory. Re-running with an absolute fixture path passes.
- `$HOME/tools/bin/make fmt` and `make clippy` passed. `make test` passed with the installation and an absolute path to the controlled fixture; the install-backed comparison test passed. CI `check` and GitGuardian passed for the reviewed revision. No emulator was booted.

## Findings

1. **blocking — the documented fixture path makes the C5 test fail before it can compare.** `crates/rt3ctl/tests/sim_compare.rs:14-25` reads `RT3_CONTROL_SAVE` unchanged and passes it to a child process whose working directory is the test crate. The reproduction command in `docs/evidence/economy-v1.md:92-95` sets `RT3_CONTROL_SAVE` to `review-tmp/saves/rt3engine-bi-18290101-202.gms`; with that value, `cargo test -p rt3ctl --test sim_compare` fails with “No such file or directory” before checking any counts. Resolve the documented repo-relative path before launching the binary, or otherwise make the test accept the documented value.
2. **blocking — C5 omits a declared economy rule.** `docs/evidence/economy-v1.md:70-79` lists the model rules without original-constant provenance, but omits the rule table's stock-update formula, `stock' = clamp(stock + production - demand + noise, 0, cap)` (`docs/evidence/economy-v1.md:34`). C5 requires every such declared economy rule to be listed; include the update/clamp rule explicitly.
3. **blocking — the building-count cause overstates what the structural evidence establishes.** `docs/evidence/economy-v1.md:66,81-83` attributes the 520 save records to industries “placed during play”; `crates/rt3ctl/src/sim_cmd.rs:412-416` similarly labels them runtime-placed. The committed M4 evidence establishes the `0` versus `520` record-count difference, while `gmp-collections-v1.md` leaves building types, coordinates, production and links open, and `gmc-save-v1.md` leaves save-specific field meanings open. Keep the cause structural (the records are present in the save but absent from the map), or provide evidence for the asserted placement semantics.
4. **blocking — error output leaks machine paths.** `crates/rt3ctl/src/sim_cmd.rs:364-368` forwards the map resolver's error, whose missing-map message includes the installation root (`sim_cmd.rs:234`); I reproduced the full `RT3_HOME` value in stderr. The invalid-save branches also print `compare.save.display()` (`sim_cmd.rs:378-390`). Avoid echoing absolute installation/save paths, consistent with the project slot's path-handling rule.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
