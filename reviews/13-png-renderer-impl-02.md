# PR #20 implementation review 02

- **Revision reviewed:** `9689e1f5208b11f6bd8ed80defbb37ccfebfa561`
- **Target verification:** `gh pr view 20 --json headRefOid,baseRefOid,files` reported this exact head, base `e08390fc049e43830018c37415a5366ac88858b2`, and the six files listed below. `git rev-parse HEAD` and `git rev-parse 9689e1f^{commit}` both matched the PR head. `git diff --name-only <base>..<head>` matched the PR file list. I read each file at HEAD and the full `c53d12869b973497d986ddd2ba0307c6d6386f96..9689e1f5208b11f6bd8ed80defbb37ccfebfa561` delta.
- **Files:** `crates/rt3-kit/src/lib.rs`; `crates/rt3-kit/src/render.rs`; `crates/rt3ctl/src/main.rs`; `crates/rt3ctl/src/map_cmd.rs`; `docs/evidence/gmp-terrain-v1.md`; `reviews/13-png-renderer-impl-01.md`.
- **Delta since round 1:** `crates/rt3-kit/src/render.rs`; `crates/rt3ctl/src/main.rs`; `crates/rt3ctl/src/map_cmd.rs`; `docs/evidence/gmp-terrain-v1.md`; `reviews/13-png-renderer-impl-01.md`.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode, implementation re-review; `design: none`.

## Round 1 findings

1. **Blocking — dangling output symlink bypassed the installation write guard: resolved.** `crates/rt3ctl/src/map_cmd.rs:83-88` now uses `symlink_metadata` and rejects a final output symlink before the `exists()`/canonicalization branch. The test at lines 573-578 creates a dangling link into its disposable install and asserts the target stays absent. I reproduced this through the CLI with a separate disposable fake installation: it printed `rt3ctl: refusing to write a PNG through a symbolic link`, and the target remained absent. Direct installation-root output was also refused. The original installation was only read for a map copy; no original path was written.
2. **Non-blocking — `--png` consumed a following option as its filename: resolved.** `crates/rt3ctl/src/main.rs:97-108` rejects a next token beginning `--`. The parser test at lines 207-227 rejects `--png --all` and accepts the explicit `./--all` filename. CLI checks confirmed the missing-value and `--all --png out.png` errors; a successful fake-install CLI run with `./--all` created that explicitly named output.
3. **Non-blocking — PNG test decoder did not independently check the zlib checksum: resolved for Adler-32 and the reported checks.** `crates/rt3-kit/src/render.rs:121-180` uses a `u64` reference Adler-32 helper, verifies the exact trailer position and value, checks IHDR fields and stored-block BTYPE, and requires IEND plus complete chunk consumption. The all-map test now exercises those assertions. See the remaining non-blocking decoder note below.
4. **Non-blocking — zero-sized grids emitted invalid PNG dimensions: resolved.** `crates/rt3-kit/src/render.rs:72-76` asserts both dimensions are nonzero, and lines 211-223 add the expected-panic regression.

## Current finding

1. **Non-blocking — the test decoder does not require IEND to be the final chunk and still self-checks chunk CRCs.** `crates/rt3-kit/src/render.rs:137-158` records that it saw IEND, but continues parsing chunks and only checks that parsing eventually reaches the end of the buffer. In a temporary copy of this decoder, appending a correctly CRC'd `tEXt` chunk after IEND still passed decoding. At line 142, chunk CRCs are also checked with the encoder's `crc32` helper, so a shared regression in that function remains self-consistent. The encoder currently writes IEND last, and an independent Python check of a newly generated fake-install PNG verified its chunk CRCs and decompression. For stronger regression coverage, require IEND to end the file and use an independent CRC check or a standard PNG decoder.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME=<installation> ~/.cargo/bin/cargo test --workspace -- --nocapture` — passed: 39 `rt3-kit` tests and 5 `rt3ctl` tests; all 40 current version-9701 maps were encoded. The zero-sized-grid panic was expected and its test passed.
- `RT3_HOME=<installation> CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed.
- The fake-install dangling-symlink, direct-install refusal, `--png --all`, `--all --png out.png`, trailing `--png`, and explicit `./--all` CLI cases behaved as described above. No original installation writes and no emulator boot occurred.
- The round-1 implementation files and three inspected images are otherwise unchanged; render outputs remain under the gitignored working area.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

Landed with merge commit `b3eb4a9` (delegated merge per decision 0002,
2026-11-23). Done-when items and evidence:

- Dependency-free north-up RGB PNG rendering works for all 40 current
  maps, with water/NaN handling, elevation colors and CRC/Adler
  checksums. Pixel tests validate orientation and full buffers.
- `map terrain <name> --png <file>` requires an existing parent, rejects
  every output symlink (including dangling), refuses installation-root
  output, rejects option-looking values and rejects `--all` with `--png`.
- Zero-sized public grids are rejected. Tests independently verify the
  Adler-32 trailer and PNG fields; path, parser, zero-dimension and
  checksum regressions were observed failing before their fixes.
- California, Argentina and British Isles PNGs were independently decoded
  and visually inspected as recognizable north-up geography. Outputs
  remain gitignored and no original installation writes occurred.
- Formatting, clippy, 39 `rt3-kit` tests, five `rt3ctl` tests and the
  evidence check passed; CI and GitGuardian succeeded on the reviewed
  head. Final AGREE covers implementation revision
  `9689e1f5208b11f6bd8ed80defbb37ccfebfa561` and is posted on PR #20.

— Implementer (zai-coding-plan/glm-5.3)
