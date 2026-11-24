# Environment

Reference machine: the owner's Linux box (Ubuntu 24.04) with the GOG
installation reachable through the Windows mount.

## Original installation

- `RT3_HOME` names the Railroad Tycoon 3 root. Default on this machine:
  the GOG install at `/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3`
  (referenced only through `RT3_HOME`; always treated read-only).
- `RT3_SAVE` optionally names one owner-supplied original `.gmc` save fixture;
  `RT3_CONTROL_SAVE` optionally names the independently generated `.gms`
  control fixture. Both are read-only inputs to their inventory producers and
  tests, normally in the gitignored working area. Save bytes never enter Git.
  CI self-skips them when unset, just as install-backed checks self-skip
  without `RT3_HOME`.
- Verified content (2026-11-21): `RT3.exe` (PE32, 2.3 MB), `Data/` with
  `2D/3D/Sound` PK4 archives, typed directories (`BuildingTypes`,
  `CargoTypes`, `EngineTypes`, `TreeTypes`), `Configuration/*.cfg` (binary),
  `Language/RT3.lng` (INI-like text), `GrayscaleMaps/*.TGA`, `maps/*.gmp`,
  mp3 music/voice, no user saves yet. Aggregate inventory:
  [evidence v1](evidence/install-inventory-v1.json) — 1,177 files, ~2.02 GB.

## Toolchain

| Tool | Location / note |
| --- | --- |
| Rust | 1.99.1 (`cargo`/`rustc` on PATH); workspace gates: `make test fmt clippy` |
| GNU make | 4.4.1 at `~/tools/bin/make` (extracted Ubuntu package; no system make on this box). Put `~/tools/bin` on `PATH` |
| Ghidra | 12.2 at `~/tools/ghidra_12.2_PUBLIC`; run headless via the wrapper `~/tools/bin/ghidra-headless` (sets `JAVA_HOME`) |
| JDK | Ghidra 12.2 requires JDK **25**: `~/tools/jdk-25.0.5.1+2` (wrapper sets it; JDK 21 at `~/tools/jdk-21.0.13.1+2` is spare) |
| Wine | **Not installed.** Needed from milestone 4 (oracle). System install requires owner `sudo`; no passwordless sudo on this box |
| Python | 3.12.3 (helper scripts only; stdlib) |

Installations live in `~/tools` (user-space; `sudo apt` requires the owner).
Recommended shell setup:

```sh
export PATH="$HOME/tools/bin:$PATH"
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
```

## Conventions

- **Capture-lock coordination (2026-11-23):** before booting the
  emulator/display pipeline (any oracle run or its reproduction,
  including by reviewers), take the exclusive advisory lock per
  `tools/oracle/capture_lock.py`
  (`~/projects/onenil-archaeology/.cache/capture.lock`, held for the whole
  capture, auto-released on exit; never delete it to force entry). Plain
  gates without emulator boot (cargo test/fmt/clippy, evidence checks)
  need no lock. Review prompts for oracle-touching changes must state
  this.
- Never write into `RT3_HOME`; oracle saves and captures go to a working
  directory outside it (e.g. `review-tmp/`, gitignored).
- Evidence regeneration: `make evidence-regen` / `make evidence-check`
  (self-skip without `RT3_HOME`).
