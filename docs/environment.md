# Environment

Reference machine: the owner's Linux box (Ubuntu 24.04) with the GOG
installation reachable through the Windows mount.

## Original installation

- `RT3_HOME` names the Railroad Tycoon 3 root. Default on this machine:
  the GOG install at `/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3`
  (referenced only through `RT3_HOME`; always treated read-only).
- `RT3_SAVE` optionally names one owner-supplied original `.gmc` save fixture;
  `RT3_CONTROL_SAVE` optionally names the first independently generated `.gms`
  control fixture; `RT3_CONTROL_SAVE_2` optionally names the second for the
  differential producer; `RT3_TRACK_SAVE` optionally names a save from a run
  with built track and at least one train, used only by the rail-section
  check. Both are read-only inputs to their inventory/diff
  producers and tests, normally in the gitignored working area. Save bytes
  never enter Git. CI self-skips them when unset, just as install-backed
  checks self-skip without `RT3_HOME`.
- Verified content (2026-11-21): `RT3.exe` (PE32, 2.3 MB), `Data/` with
  `2D/3D/Sound` PK4 archives, typed directories (`BuildingTypes`,
  `CargoTypes`, `EngineTypes`, `TreeTypes`), `Configuration/*.cfg` (binary),
  `Language/RT3.lng` (INI-like text), `GrayscaleMaps/*.TGA`, `maps/*.gmp`,
  mp3 music/voice, no user saves yet. Aggregate inventory:
  [evidence v1](evidence/install-inventory-v1.json) — 1,177 files, ~2.03 GB.
- Executable identity and patch level (2026-11-25): `RT3.exe` is PE32,
  2,330,624 bytes, SHA-256
  `accb099b0d8ebac8b50c539a5cc91a863bc9ae399ae70da382ff1c22301e38a2`;
  `d8tx.dll` is 73,728 bytes, SHA-256
  `2b48ca33982fe78729fec134596d062ed1595019c77d80e729569f79d4fdcc5b`.
  Patch level is recorded as **1.05 by inference**, not from an independent
  executable-build check: the bundled `History.rtf` ends at "Version 1.05
  Changes", and the 40 current maps carry the version word 9701 (`0x25E5`),
  the community-documented 1.05 map mark (1.06 maps use 9901, `0x26AD`; the
  three legacy maps carry 9609/9610). The PE version resource is not diagnostic
  (FileVersion `1`, ProductVersion `0, 2, 0, 1`), and no independent build
  identifier has been matched to the hashes above, so a 1.06 executable cannot
  be excluded from this evidence alone. Version-9901 originals are rejected by
  the current parser; that is a statement about parser scope, not about which
  patch built this executable.

## Toolchain

| Tool | Location / note |
| --- | --- |
| Rust | 1.99.1 (`cargo`/`rustc` on PATH); workspace gates: `make test fmt clippy` |
| GNU make | 4.4.1 at `~/tools/bin/make` (extracted Ubuntu package; no system make on this box). Put `~/tools/bin` on `PATH` |
| Ghidra | 12.2 at `~/tools/ghidra_12.2_PUBLIC`; run headless via the wrapper `~/tools/bin/ghidra-headless` (sets `JAVA_HOME`) |
| JDK | Ghidra 12.2 requires JDK **25**: `~/tools/jdk-25.0.5.1+2` (wrapper sets it; JDK 21 at `~/tools/jdk-21.0.13.1+2` is spare) |
| Wine | 9.0 (`wine-9.0`, Ubuntu package). The oracle runs on a copy of the installation at `C:\RT3` in the prefix `~/tools/wine-rt3`; the GOG install stays untouched. Launcher `tools/oracle/launch.sh` (DXVK x32 d3d8/d3d9 on lavapipe, binkw32 disabled, Wine virtual desktop, GLX-capable Xvfb under a renamed binary). Take the capture lock below before any oracle boot |
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

## Native Windows companion

Native AutoHotkey, controlled-save and Windows-capture work belongs to
[rt3engine-windows](https://github.com/diegoami/rt3engine-windows), created by
owner decision [0004](decisions/0004-windows-companion.md). Use a disposable
RT3 copy there; export only read-only save fixtures and derived evidence to
this repository when a private transfer is possible. Pure Windows sessions
use `RT3_PRIVATE_OUT` and no WSL/shared-filesystem assumption. A DirectX 8.1
warning may be dismissible; record it, click OK and verify the playable game
window rather than treating the warning alone as fatal.
