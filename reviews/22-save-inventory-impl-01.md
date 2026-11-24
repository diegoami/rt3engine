# Implementation review — save inventory, round 1

- **Revision:** `bfe77fe204736b99e19fe1615eb010316db5573a`
- **Base:** `d2a42a76efc3eb591e3803d22fceecf2e9030481`
- **Scope proof:** `gh pr view 31 --json headRefOid,baseRefOid,headRefName,baseRefName,files` reported head `bfe77fe204736b99e19fe1615eb010316db5573a`, base `d2a42a76efc3eb591e3803d22fceecf2e9030481`, branch `m4/save-inventory`, and the 11 files listed below. Local `HEAD` and `bfe77fe^{commit}` resolve to that head; `git diff --name-only <base>...<head>` matches the PR file list. Read all 11 changed files and the complete diff.
- **Files checked:**
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3-kit/src/save.rs`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `crates/rt3ctl/src/save_cmd.rs`
  - `docs/environment.md`
  - `docs/evidence/README.md`
  - `docs/evidence/gmc-save-v1.md`
  - `docs/evidence/install-inventory-v1.json`
  - `docs/evidence/save-inventory-v1.json`
  - `tools/evidence.sh`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; project slot records `design: none` and delegated merge.

## Findings

1. **Blocking — install inventory evidence is stale** (`docs/evidence/install-inventory-v1.json:5,14,280`). With the requested read-only `RT3_HOME`, the inventory producer reports `2,031,926,720` total bytes, `927,576,454` bytes under `Data`, and `13,621` `.txt` bytes. The committed artifact reports `2,031,930,087`, `927,579,821`, and `16,988` respectively: each byte total is 3,367 too high. Reproduced by `make evidence-check`, which fails on this artifact both with and without `RT3_SAVE`. Correct the derived inventory against the read-only installation so the required evidence gate is green.

## Review

The parser composes the existing framed terrain, tile, city, building and event readers and rejects a non-9701 version or any missing section. On the supplied read-only fixture at `review-tmp/saves/Central west Pacific 1.gmc`, an independent version read, `stat`, and system `sha256sum` confirmed version **9701**, **8,490,851** bytes and SHA-256 `3c11cbbdde844273c499086a8a4c23df9e1cfe9837910696fa13bf32d121d99b`. The existing collection parsers returned terrain **257×385**, **6,144** tiles, **59** cities, **410** buildings and **17** events. The producer's JSON exactly matches the committed save artifact, including section offsets: terrain **35,804**, tiles **2,601,345**, buildings **3,579,183**, cities **4,084,035**, events **4,244,640**, and all section end offsets.

The SHA-256 implementation follows standard block padding and compression; its `abc` known-vector test passes, and its multi-block digest for the original fixture matches the independent system hash. With `RT3_SAVE` set, `save::tests::reads_original_save` passed and asserts the fixture size, digest and counts. Wrong-version and missing-section CLI inputs were refused. The CLI reads the named file without writing it; repeated JSON output was byte-identical, a filename containing quote, backslash, tab, newline and a control character remained valid JSON, and the original fixture's modification time did not change. The evidence script runs and compares the optional save producer when `RT3_SAVE` is set and reports a clean skip when unset.

## Verification

- PR head/base and all 11 changed paths match the local revision and diff.
- With the requested `RT3_HOME` and `RT3_SAVE`, `make fmt` — passed.
- With the same environment, `make clippy` — passed with warnings denied.
- With the same environment, `make rust` — passed: **53 `rt3-kit` tests** and **5 `rt3ctl` tests**.
- `make evidence-check` with `RT3_SAVE` set — **failed** on the install-inventory drift above; the save-inventory artifact matched.
- With `RT3_SAVE` unset, `cargo test -p rt3-kit save -- --nocapture` — passed: **3 tests**, including the original-save test's explicit self-skip. Unset `make evidence-check` reported the optional save check skipped, then failed on the same install-inventory drift.
- `git ls-files` contains no `.gmc` or `.sav` files; the fixture is ignored under `review-tmp/`. Original installation and fixture were read only. No emulator was used.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
