# Implementation review — city collections, round 1

- **Revision:** `b35cdce6687d070333988dc7f6125727d9131ba5`
- **Target proof:** `gh pr view 27 --json headRefOid,baseRefOid,files` reported head `b35cdce6687d070333988dc7f6125727d9131ba5`, base `ec37e4817e3204afa99d6c2b14303da656ca457d`, and the seven files listed below. Local `HEAD` and `b35cdce^{commit}` resolved to that head; `git diff --name-only <base>..<head>` matches the PR file list.
- **Files checked:**
  - `crates/rt3-kit/src/cities.rs`
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/gmp-cities-v1.md`
  - `docs/evidence/gmp-format-v1.md`
  - `docs/evidence/terrain-scan-v1.json`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; project slot records `design: none`.

## Findings

1. **Non-blocking — the allocation-fallibility claim is too broad** (`crates/rt3-kit/src/cities.rs:78-94`; `docs/evidence/gmp-cities-v1.md:38-40`). `mask.to_vec()` is infallible, and `records.try_reserve_exact(active)` does not prevent `records.push()` from growing the vector if a malformed mask marks more than `active` slots. `MAX_CITY_SLOTS` bounds this growth, so it is not unbounded and the malformed-input probe below did not panic. For precision, borrow the mask and reject before pushing beyond `active`, or describe allocations as bounded rather than all fallible.
2. **Non-blocking — add direct malformed/truncation regressions** (`crates/rt3-kit/src/cities.rs:135-170,173-225`). The synthetic fixture covers sparse slot order and a bad terminator, but not strict-prefix truncations, mismatched active counts or bad lengths. The all-map test checks 43 files, the record total and the number of control-prefixed labels, but not the aggregate capacity or the identities of those three labels. The evidence gate compares per-map summaries, and the independent checks below verified the current values; direct assertions would make these format claims less dependent on the install-backed gate.

## Verification

- **Target and diff:** The PR head, base and seven-file list match the local revision and diff. Read all seven changed files and the complete diff.
- **Reader provenance:** `/tmp/opencode/disasm_00443a50.txt` shows the map loader seeking `0x332d` at `0x0044483d` and calling `0x00421510` at `0x00444887`. `/tmp/opencode/disasm_00421510.txt` shows the reader seeking `0x5209`, calling `0x00518680`, seeking `0x520a`, iterating active entries through a virtual call, and finally seeking `0x520b`. `/tmp/opencode/disasm_00518680.txt` shows the generic collection header, level data, absence mask and slot reads. Static inspection of `RT3.exe` under `RT3_HOME` confirmed the exact `gpdCityDB` string and a code reference at `0x00445782` passing its address to `0x00442660`; this is original executable label evidence, not a name inferred from city text. `0x0041f5c0` contains another generic collection read; meanings of per-entry virtual reads after `0x520a` correctly remain open.
- **Independent map walk:** A separate read-only Python walk of all 43 original maps verified each `0x332d`/`0x5209` header, fixed values and level triples, sparse mask, active count, six-byte prefix, `u16` length `905`, 905-byte blob, and `0x520a` end tag. Totals are **4,130 slots and 3,726 active records**; Ireland is 90/76 and British Isles 70/68. Every raw label is 48 bytes at blob offset 854; Dublin and Manchester occur in their expected maps. The three leading-control labels are Louisville in Germantown, and South Bend and Louisville in Mississippi Valley. Every committed per-map `cities` summary matches this walk.
- **Artifact compatibility:** Parsed the merge-base artifact and current artifact as JSON, removed `cities` from every current map row, and compared them: all previous fields are exactly equal.
- **Parser safety:** Offsets in `u32_at`/`take` use checked arithmetic and slice bounds; capacity is capped at 1,024; active count is bounded by capacity; the mask size follows capacity; blob lengths and the exact terminator are checked; `label_bytes()` is fallible. A standalone no-panic probe exercised all **13,291** strict prefixes of a synthetic collection, malformed header/mask/length mutations, and 2,048 deterministic random inputs without panic. No emulator was used.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- With `RT3_HOME` set, `cargo test --workspace` — passed: **47 `rt3-kit` tests and 5 `rt3ctl` tests**.
- With `RT3_HOME` set, `CARGO=cargo sh tools/evidence.sh check` — passed.
- `git diff --check` and JSON syntax validation — passed. The `make` frontend is unavailable in this environment; the direct commands above are the corresponding Makefile recipes.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
