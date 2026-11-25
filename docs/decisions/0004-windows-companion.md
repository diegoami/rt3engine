# Decision 0004 — native Windows companion for oracle automation

Date: 2026-11-24. Owner: diegoami. Status: decided.

The owner asked whether Windows-dependent work should run in a native Windows
session and whether to create a parallel Windows project for it.

The companion is [rt3engine-windows](https://github.com/diegoami/rt3engine-windows).
It owns AutoHotkey/native-window automation, controlled original save generation,
native capture and future Windows-only oracle diagnostics. It is a tooling
project, not a second engine implementation.

Rules:

1. Use a disposable RT3 copy for oracle runs; the original GOG installation is
   read-only.
2. Original assets and save bytes never enter either Git repository. A pure
   Windows session uses a private output directory named by
   `RT3_PRIVATE_OUT`; no WSL path or shared filesystem is assumed. When a
   private transfer is possible and requested, export fixtures into
   rt3engine's gitignored working area and use `RT3_SAVE`. Only small derived
   evidence may be committed after review.
3. A native Windows session may report a DirectX 8.1 hardware warning while
   remaining playable. Record and dismiss the warning with OK, then verify a
   playable game window; stop only if none follows. Never bypass or alter the
   game's device check.
4. The main engine stays headless-first. No engine architecture, milestone
   ordering or per-change review requirement changes.
