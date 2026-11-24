# Implementation review — tile sections, round 2

- **Revision:** `cc7a976e3dd2221daa4c36725d3d335f8cb0cc4c`
- **Target proof:** `gh pr view 26 --json headRefOid,baseRefOid,files` reported head `cc7a976e3dd2221daa4c36725d3d335f8cb0cc4c`, base `63a573dbdb4b4e3cec82ed9a1f095e67b0caa8a9`, and exactly the seven files listed below. Local `HEAD` and `cc7a976^{commit}` both resolved to the reported head; `git diff --name-only <base>..<head>` agrees.
- **Files checked:**
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3-kit/src/tiles.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/gmp-terrain-v1.md`
  - `docs/evidence/terrain-scan-v1.json`
  - `reviews/19-tile-sections-impl-01.md`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; project slot records `design: none`.

The round-two implementation delta from `f578b2649019fd52657ba505879278a9a4bdcab1` is confined to `crates/rt3-kit/src/tiles.rs`; the other implementation files are unchanged.

## Findings

No remaining findings. Round-one blocking findings on speculative allocation and dimension overflow are resolved. The non-blocking interleaving-fixture finding is also resolved.

## Verification

- **Allocation and bounds:** `layout` uses checked arithmetic for rows, columns, cell count, tile count and eager-reserve size, and rejects over-cap layouts before reading nested/array framing tags or allocating. Candidate offsets and tag ranges are checked; the scan bound uses saturating subtraction. Cell arrays, tile records and per-record entries use `try_reserve_exact`; dynamic-entry accumulation is checked and capped.
- **Reproduced round-one cases:** With an eight-byte `0x2f43`/`0x9471` prefix, `rows = cols = 4001` and `rows = cols = 4_294_967_297` both returned `None` without panic. A counting-allocator probe observed zero requested allocation bytes for either case.
- **Allocation budget:** On this 64-bit target, `TileRecord` is 480 bytes. Enumerating permitted dimensions gives a maximum eager array-plus-record vector payload of 35,528,688 bytes (33.88 MiB), below the 40 MiB limit. The dynamic-entry cap separately bounds entry payload to 1,000,000 × 6 = 6,000,000 bytes (5.72 MiB). Thus total requested vector payload is at most about 39.61 MiB, excluding allocator metadata. The per-record reserve occurs only after checked accumulation against the dynamic cap.
- **Fixture and truncation:** The synthetic mask now covers sparse fields across indices 0–2, and assertions check both decoded values and omitted zero values. The test walks every strict prefix of the fixture for panic freedom. `rejects_excessive_or_overflowing_dimensions` covers both reproduced dimension cases.
- **Independent all-map walk:** A direct read-only scan of all 43 installed maps, independent of `tiles::read`, rechecked the full section signatures, dimension-derived record counts, every record’s optional-field lengths and dynamic count, and the `0x2ee5` tag at each artifact end offset. Results remain **645,120 records, 4,458 dynamic entries, maximum 4 per record, across nine maps**. All artifact tile offsets/counts/end tags match; removing `tiles` from every artifact row leaves its prior fields exactly equal to the merge-base artifact.
- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- With `RT3_HOME` set to the installation specified for this review, `~/.cargo/bin/cargo test --workspace` — passed: 45 `rt3-kit` tests and 5 `rt3ctl` tests.
- With `RT3_HOME` set to the installation specified for this review, `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed.
- No emulator was used.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
