# Implementation review — tile sections

- **Revision:** `1614502c2a11946baeb0eb67aea92a805c0bf70d`
- **Target proof:** `gh pr view 26 --json headRefOid,baseRefOid,files` reported head `1614502c2a11946baeb0eb67aea92a805c0bf70d`, base `63a573dbdb4b4e3cec82ed9a1f095e67b0caa8a9`, and exactly the six files listed below. Local `HEAD` and `1614502^{commit}` both resolved to the reported head.
- **Files checked:**
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3-kit/src/tiles.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/gmp-terrain-v1.md`
  - `docs/evidence/terrain-scan-v1.json`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; project slot records `design: none`.

## Findings

1. **Blocking — record allocation is not bounded to a safe memory budget** (`crates/rt3-kit/src/tiles.rs:82-92`). The one-million-record cap still permits a very large eager allocation, and it is checked only after copying all four cell arrays. A grid with `rows = cols = 4001` passes the divisibility checks and produces exactly 1,000,000 records. `TileRecord` is 480 bytes on this target, so `Vec::with_capacity(tile_count)` reserves about 458 MiB before the first record is read. A candidate with valid array framing but a truncated record therefore reserves that memory and then returns `None`; candidates above the cap still incur the four array copies before rejection. Bound the total parse allocation before copying/reserving (and make allocation failure return a parse error rather than relying on infallible reservation); reject oversized candidates before allocating.

2. **Blocking — public dimensions can overflow and panic before bounds checks** (`crates/rt3-kit/src/tiles.rs:71, 73-78, 86, 146`). `HeightGrid` has public fields and `tiles::read` documents no requirement that its dimensions came from a validated terrain parser. With `rows = cols = 4_294_967_297` and an eight-byte `0x2f43`/`0x9471` prefix, the dimensions pass the minimum/divisibility tests, then `(rows + 1) * (cols + 1)` overflows and panics in a debug build before the byte bounds checks. Use checked arithmetic for dimensions, tile count and offset-plus-length calculations, including `u32_at` and the scan bound, so invalid metadata returns `None` rather than panicking.

3. **Non-blocking — the synthetic fixture does not distinguish interleaved from grouped fields** (`crates/rt3-kit/src/tiles.rs:182-188`). Its only set bits are `0b111` for the first value triplet. A parser that reads all indices of one type before the next would consume those three values in the same order, so this fixture does not exercise the claimed interleaving across indices or sparse presence. Add a fixture with present fields at multiple indices and omissions between them; the disassembly and shipped-map walks independently support the current implementation.

## Verification

- **Disassembly:** Read `/tmp/opencode/disasm_00448f60.txt`, `/tmp/opencode/disasm_0042ae50.txt` and `/tmp/opencode/disasm_0044cfb0.txt` in full. `0x00448f60` seeks `0x9471`, computes `(rows+1)*(cols+1)`, reads that byte length four times, then seeks `0x9472`. The higher-level `0x0044cfb0` path enters at `0x2f43`, calls that reader, enters `0x2f44`, loops over the two record dimensions calling `0x0042ae50`, and later seeks `0x2ee5`. In `0x0042ae50`, the loop advances the field index by three until `0xa1`, giving 53 iterations; bit tests use `1 << (bit % 8)` in byte `bit / 8`, and each iteration reads optional `u16`, `u16`, `f32` in that order. The following fixed reads are 1, 2, 2 and 7 bytes; the count is sign-extended from a 16-bit word, and each positive entry is read as six bytes. These paths prove the framing and serialized field widths/order, not gameplay meanings; the field semantics remain open.
- **Independent map walk:** A separate read-only script walked the installed 43 map files directly, using the framed grid dimensions but not `tiles::read`. It verified the full `0x2f43`/`0x9471`/four-array/`0x9472`/`0x2f44` signatures, each dimension-derived record count, every record's optional fields/fixed bytes/signed count/six-byte entries, and `0x2ee5` at each reported `end_offset`. All 43 rows matched the artifact offsets, record totals, dynamic-entry totals and end-tag offsets. Totals: **645,120 records**, **4,458 dynamic entries**, **maximum 4 entries per record**, with dynamic entries on exactly nine maps: East Coast, USA (195), Eastern Canada (25), Greenland Growing (995), Japan Trembles (575), Poland (265), Rhodes Unfinished (25), Southern Pacific (701), Tutorial_2 (452) and War Effort (1,225).
- The early `0x2f43` candidates in Argentina Opens Up (`0x3fb184`), Tex-Mex (`0x51ae5a`) and Texas Tea (`0x51b489`) fail the nested `0x9471` signature; each later real candidate parses through its terminator.
- Parsed the merge-base artifact and compared each row after removing the new `tiles` key: all 43 maps' previous fields are exactly equal. The artifact's 43 tile summaries also match the independent walk.
- **Malformed-input probe:** 2,048 truncated/random byte buffers with a small valid grid returned without panicking. The oversized constructed-grid case in finding 2 reproduced the debug overflow panic. No large-allocation/OOM case was executed.
- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME` set to the installation specified for this review; `~/.cargo/bin/cargo test --workspace` — passed: 44 `rt3-kit` tests and 5 `rt3ctl` tests.
- `RT3_HOME` set to the installation specified for this review; `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed.
- No emulator was used.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
