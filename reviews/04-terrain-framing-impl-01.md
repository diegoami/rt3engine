# Implementation review: terrain framing/raw grids

- **Revision covered:** `d27a6245833f17db0de7fd8d6d67c9c65d1e940c`
- **Files checked:** `.gitignore`, `crates/rt3-kit/src/lib.rs`, `crates/rt3-kit/src/map.rs`, `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/main.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-format-v1.md`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/install-inventory-v1.json`, `docs/evidence/terrain-scan-v1.json`, `tools/evidence.sh`, `tools/oracle/__pycache__/capture_lock.cpython-312.pyc` (deleted), `tools/oracle/__pycache__/drive.cpython-312.pyc` (deleted), `tools/oracle/drive.py`, `tools/oracle/look.py`.
- **How obtained:** `git diff main...HEAD`, `git diff --stat main...HEAD`, and `git diff --name-only main...HEAD`; the diff is from merge-base `c1fa04bff0cba8d2441165fe084a3a3ec0305238`. Read each changed text file at `HEAD`; the two binary cache deletions were identified in the diff. Checked-out `HEAD` matched the covered revision.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Findings

1. **blocking** — `crates/rt3-kit/src/terrain.rs:280-285,321-340`. `Probe::Water` is treated as proof that `validate` cannot pass, but the probe checks only 64 positions from the validator's 4,225 sampled cells. I reproduced a counterexample with the unchanged scanner: 2×2 sampled patches of 100 m cells in each 32-cell tile give 256/4,225 (6.1%) land, a 100 m range, and mean neighbor difference below the 8 m threshold. `validate` returns true at the true start, `quick_probe` says Water, and `scan` returns `None`. The optimization therefore skips a window accepted by the scanner's own full validator. Make the shortcut sound (or avoid this skip) and retain a regression test.

2. **blocking** — `crates/rt3-kit/src/terrain.rs:23-26,142-152,257-269`; `docs/evidence/gmp-terrain-v1.md:47-57`. `ROW_BYTES` is fixed at 1,032 bytes (258 floats) throughout probe, validation, and extraction, and the scan never uses `TerrainDims`. Yet the evidence says, for example, Germany's lattice is 512 and that the scanner validates 257-wide windows inside larger grids; its artifact records a grid for Germany. A 257-wide subwindow in a wider row-major grid must still advance by the parent grid's row pitch, not 258 floats. The current extraction therefore cannot establish the documented rectangular subwindow from a larger grid. Derive/use the actual row pitch or leave these larger-map hits as unverified candidates until their layout is demonstrated.

3. **blocking** — `docs/evidence/gmp-terrain-v1.md:79-80`. The sentence says the 27 maps without a `0x2ee2` header “still carry raw grids,” but `terrain-scan-v1.json` has 27 rows with `dims: null` and only four of those rows have a non-null `grid` (Central Pacific, Chicago to New York, Dutchlantis, Tex-Mex). The other 23 are marked “no validated grid.” Correct the count and distinguish maps without a dims header from those with a detected grid.

4. **blocking** — `docs/evidence/gmp-terrain-v1.md:51-62`. The “Proven” section asserts that all 12 scan hits are raw terrain height grids, while `terrain::scan` searches arbitrary offsets using range/land/smoothness heuristics without tying a hit to a terrain tag or dims header. The JSON retains only the hit offset and summary statistics; the install-backed test checks only Crossing the Alps, and the stated in-game visual match is only for that map. Those outputs re-derive plausible candidate windows, not the semantic identity of all 12 as terrain. Mark uncorroborated hits as candidates/open or retain evidence that ties each claimed grid to terrain.

5. **non-blocking** — `crates/rt3-kit/src/terrain.rs:7-11`. The module comment says the dims header reads `64,64,64,64,256,256,1024,1024,0` “on every map checked so far,” but the committed artifact shows varying values (for example Germany has 128/512/2048 groupings) and only 13 of 40 maps have a detected dims header. Update this stale claim.

6. **non-blocking** — `crates/rt3ctl/src/main.rs:85-97`. Terrain argument parsing silently accepts malformed combinations: `map terrain --all --bogus` treats `--bogus` as a name and then runs `--all`, ignoring it; `map terrain --all --path` silently leaves the path unset and falls back to `RT3_HOME`. Reject unknown options, extra names with `--all`, and a missing `--path` value.

7. **non-blocking** — `crates/rt3-kit/src/terrain.rs:452-461`. `rejects_ff_filler` starts with `0xFF` but changes every seventh byte to zero, so it does not test the all-`0xFF` filler named by the test or claimed in the module comment. Add a pure-`0xFF` case or rename the test to describe the mixed fixture.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain` — 6 passed, 0 failed; includes `scans_reference_map`.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no drift or producer errors.
- `docs/evidence/terrain-scan-v1.json` parses as JSON: 40 maps, 12 grid hits, 13 dims headers, and 4 grid hits without dims headers.
- The scanner counterexample was run from a temporary harness outside the repository. No game or emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
