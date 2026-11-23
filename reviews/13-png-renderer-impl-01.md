# PR #20 implementation review 01

- **Revision reviewed:** `c53d12869b973497d986ddd2ba0307c6d6386f96`
- **Target verification:** `gh pr view 20 --json headRefOid,baseRefOid,files` reported this exact head, base `e08390fc049e43830018c37415a5366ac88858b2`, and the five files listed below. `git rev-parse HEAD` and `git rev-parse c53d128^{commit}` both matched the PR head. I read the full diff from base to head and each changed file at that revision.
- **Files:** `crates/rt3-kit/src/lib.rs`; `crates/rt3-kit/src/render.rs`; `crates/rt3ctl/src/main.rs`; `crates/rt3ctl/src/map_cmd.rs`; `docs/evidence/gmp-terrain-v1.md`.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode, fresh-context implementation review; `design: none`.

## Findings

1. **Blocking — a dangling output symlink bypasses the installation write guard.** `crates/rt3ctl/src/map_cmd.rs:92-97` uses `out.exists()` to decide whether to canonicalize the final path. `exists()` follows symlinks and returns false for a dangling symlink, so the fallback `parent.join(file)` keeps the symlink path; the subsequent `fs::write` follows it. Reproduced through the CLI with a disposable fake installation: direct output inside the root was refused, as was an existing symlink to a file there, but an output symlink whose not-yet-created target was inside that same fake root caused the target file to be created there. No original installation was written. Reject dangling output symlinks (or resolve their targets safely) and add a regression test asserting the target is not created.

2. **Non-blocking — an option following `--png` is silently consumed as its filename.** `crates/rt3ctl/src/main.rs:97-102` accepts any next token as the value. The trailing `--png` case correctly errors, but `map terrain "California Island" --path <fake-install> --png --all` treated `--all` as a filename and wrote that file in the disposable output directory. Treat an option-looking token as a missing value, with a parser regression test; users can still explicitly pass such a filename as `./--all`.

3. **Non-blocking — the PNG test decoder does not check the zlib checksum independently.** `crates/rt3-kit/src/render.rs:117-155` validates chunk CRCs using the encoder's own `crc32`, parses stored blocks, and stops at BFINAL without checking the Adler-32 trailer or all PNG/IHDR invariants. Thus an encoder regression in `adler32` would not fail these tests, including the all-map test. Prefer a standard independent decoder in tests or independently assert the zlib trailer and relevant PNG fields. This review separately decompressed the produced images with Python's standard-library zlib.

4. **Non-blocking — zero-sized public grids produce invalid PNG dimensions.** `crates/rt3-kit/src/render.rs:72-74,87-108` accepts zero rows or columns and writes those dimensions to IHDR. Reproduced with a directly constructed public `HeightGrid`: the encoder emitted a PNG identified as `1 x 0`. The current terrain reader rejects such geometry, so the CLI does not reach this case, but the public encoder should reject invalid dimensions (or document/enforce its input precondition).

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME=<installation> ~/.cargo/bin/cargo test --workspace -- --nocapture` — passed: 38 `rt3-kit` tests and 4 `rt3ctl` tests; the render test selected exactly 40 version-9701 maps and encoded each.
- `RT3_HOME=<installation> CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed.
- CLI parsing checks confirmed a trailing `--png` errors, `--all --png` errors, and a map name with `--all` errors. The option-as-value case is finding 2.
- On the three ignored images `review-tmp/render/{california-island,argentina-opens-up,british-isles}.png`, `file` reported RGB PNGs of 257×257, 641×385, and 321×513. Python independently verified chunk CRCs, decompressed each IDAT with `zlib.decompress`, and checked each decoded buffer length and row filter bytes. Visual inspection showed California Island's diagonal island chain, Argentina's western Andes and eastern plains, and the north-up British Isles.
- `crates/rt3-kit/Cargo.toml` has no dependencies; no dependency manifest changed. The 40-map test and CLI retain the exact current-format filter `Some(9701)`. The evidence change describes three visual maps and three orientation comparisons and explicitly disclaims original-game rendering. Generated images remain gitignored.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
