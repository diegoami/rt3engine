# Implementation review: terrain framing/raw grids (round 2)

- **Revision covered:** `28d30f83fa77efda7bfe313ba20a9d027e20a710`
- **Files checked:** `.gitignore`, `crates/rt3-kit/src/lib.rs`, `crates/rt3-kit/src/map.rs`, `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/main.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-format-v1.md`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/install-inventory-v1.json`, `docs/evidence/terrain-scan-v1.json`, `reviews/04-terrain-framing-impl-01.md`, `tools/evidence.sh`, `tools/oracle/__pycache__/capture_lock.cpython-312.pyc` (deleted), `tools/oracle/__pycache__/drive.cpython-312.pyc` (deleted), `tools/oracle/drive.py`, `tools/oracle/look.py`.
- **How obtained:** Full pull-request file list from the local diff at merge-base `c1fa04bff0cba8d2441165fe084a3a3ec0305238` (`git merge-base main HEAD`, then `git diff --name-only <merge-base>..HEAD`). Round-2 delta inspected with `git log --oneline main..HEAD` and `git diff d27a624..HEAD`; checked-out `HEAD` matched the covered revision. Read the changed text files at `HEAD`; the two binary cache deletions were identified in the diff.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-1 finding resolution

1. **Resolved.** The probe samples `(r,c)` at multiples of 32, a subset of the validator's multiples-of-4 lattice under the same origin and 258-float row pitch. It now rejects only on a shared out-of-range value. `finds_probe_invisible_land` passes on this revision and the same fixture returns no hit with the round-1 scanner.
2. **Resolved.** The scanner geometry remains fixed, but wider-lattice hits are now labeled `candidate`; the evidence doc explicitly says they do not demonstrate a rectilinear subwindow.
3. **Resolved.** The doc now says four of the 27 maps without a dims header produce candidate windows.
4. **Resolved.** The doc separates one verified decode (Crossing the Alps) from 11 candidates; the artifact carries matching `status` values, and the install-backed test asserts the Alps dims header lattice.
5. **Resolved.** The module comment scopes the fixed dimensions to Crossing the Alps and says wider-lattice hits are candidates.
6. **Resolved for the cited cases.** Direct CLI checks return exit 2 for `--all --bogus`, `--all <name>`, and a trailing `--path` with no value. A separate repeated-positional edge remains below.
7. **Resolved.** A pure-`0xFF` test was added and the mixed fixture was renamed.

## Findings

1. **blocking** — `crates/rt3ctl/src/map_cmd.rs:196-203,235-303`. `run_terrain_all` does not validate that the supplied installation root exists or contains the expected map directories. Reproduction with `env -u RT3_HOME cargo run -q -p rt3ctl -- map terrain --all --json --path /tmp/opencode/no-such-root` printed an empty `{"maps":[]}` and exited successfully. That makes a missing/incorrect installation look like a successful scan; in `evidence-regen` it can replace the committed artifact with an empty result. Fail on an invalid root or missing required map directories instead.

2. **non-blocking** — `crates/rt3ctl/src/main.rs:99`. Each positional argument overwrites `name`; `map terrain 'not a map' 'Crossing the Alps'` silently runs Crossing the Alps rather than rejecting the extra name. I reproduced this against the installation. Reject a second positional name.

3. **non-blocking** — `docs/evidence/gmp-terrain-v1.md:48-50,84-88`. The artifact establishes that 28 maps have no validated window, but that does not by itself establish that all 28 store heights another way; the “no dense f32 spans” check is specifically cited for only Britain, East Coast, and Ireland. Likewise, the general `(lattice+1)²` coarse-grid statement goes beyond the one verified decode while the other hits are expressly candidates. Keep these as unknown/inferred claims or scope them to the observed map.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 8 passed, 0 failed; includes the install-backed `scans_reference_map`.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no output or drift.
- The artifact parses as JSON and has 40 maps: 1 `verified`, 11 `candidate`, and 28 null statuses, matching the evidence doc.
- The three requested malformed CLI cases returned exit 2. The duplicate positional name and invalid-root cases above were also exercised. No game or emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
