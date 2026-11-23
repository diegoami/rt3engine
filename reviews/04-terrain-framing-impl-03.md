# Implementation review: terrain framing/raw grids (round 3)

- **Revision covered:** `5f7fef3e46ddee8d04a13fde8a4b45c4be89645d`
- **Files checked:** `.gitignore`, `crates/rt3-kit/src/lib.rs`, `crates/rt3-kit/src/map.rs`, `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/main.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-format-v1.md`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/install-inventory-v1.json`, `docs/evidence/terrain-scan-v1.json`, `reviews/04-terrain-framing-impl-01.md`, `reviews/04-terrain-framing-impl-02.md`, `tools/evidence.sh`, `tools/oracle/__pycache__/capture_lock.cpython-312.pyc` (deleted), `tools/oracle/__pycache__/drive.cpython-312.pyc` (deleted), `tools/oracle/drive.py`, `tools/oracle/look.py`.
- **How obtained:** Full pull-request file list from the local diff at merge-base `c1fa04bff0cba8d2441165fe084a3a3ec0305238` (`git merge-base main HEAD`, then `git diff --name-only <merge-base>..HEAD`). Round-3 delta inspected with `git diff 28d30f8..HEAD`; checked-out `HEAD` matched the covered revision. Read the delta's touched text files at `HEAD`; the other listed text files were unchanged from the prior checked revision. The binary cache deletions were identified in the diff.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-2 finding resolution

1. **Resolved.** `run_terrain` returns exit 2 for a non-directory root, and `run_terrain_all` returns exit 2 when the root has no `.gmp` maps. The nonexistent-root and existing-empty-directory reproductions both returned exit 2.
2. **Resolved.** The parser rejects a second positional argument. `map terrain a b` returned exit 2 with “unexpected extra argument b”.
3. **Resolved.** The evidence doc scopes the `(lattice+1)²` layout to the verified Crossing the Alps map and marks it unverified elsewhere; it now says the window and three-map density checks do not establish how the 28 window-less maps store terrain.

## Findings

No new findings.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain` — 8 passed, 0 failed; includes the install-backed `scans_reference_map`.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — completed with no output or drift.
- The real release `map terrain --all --json` invocation printed the expected JSON prefix (first three lines).
- No game or emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
