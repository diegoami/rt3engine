# rt3engine

An open, native **stand-in engine for Railroad Tycoon 3** (PopTop, 2003) that
plays the original game from the user's own installation assets, with
headless simulation, observers, original save/map compatibility, and an MCP
interface for AI-assisted play. Structured after the sibling project
[totengine](https://github.com/diegoami/totengine).

The main game experience reads the user's original installation behind
`RT3_HOME`. The original executable is not needed at runtime; no proprietary
artwork, maps, saves, sounds or scenarios are distributed. MIT licensed.

Read [`AGENTS.md`](AGENTS.md) if you work with OpenCode, or
[`CLAUDE.md`](CLAUDE.md) if you work with Claude Code. Both read
[`PRINCIPLES.md`](PRINCIPLES.md).

## Mission

[Mission](docs/mission.md) · [Delivery plan](docs/plan.md) ·
[Milestones](docs/milestones.md) · [Backlog](BACKLOG.md) ·
[Decisions](docs/decisions/0001-founding-decisions.md).

Product targets: playable stand-in · original-format compatibility (PK4,
maps, typed data, saves) · headless campaigns · observation and recording ·
staged MCP agent interface · enhanced playing (opt-in QoL).

## Status

Milestone 0 (bootstrap), 2026-11-21: harness process adopted, Rust workspace
skeleton (`rt3-kit`, `engine`, `rt3ctl`), metadata-only installation
inventory with its [evidence artifact](docs/evidence/README.md), CI. No
format decoding, simulation or client exists yet. Next: milestone 1 — PK4
asset archives ([milestones](docs/milestones.md)).

## Development

```sh
# Gates (CI runs the same)
make test        # cargo test --workspace + install-backed evidence check
make fmt clippy

# Installation inventory (metadata only; never reads file contents)
export RT3_HOME="/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
cargo run -p rt3ctl -- inventory --json
```

Toolchain and reference setup (Ghidra, JDK, Wine oracle): 
[environment](docs/environment.md). Format findings:
[evidence](docs/evidence/README.md). Reuse policy:
[prior art](docs/prior-art.md).

## Policy

- **mode:** OpenCode mode — the implementer's tool is OpenCode (the project slot in `CLAUDE.md`).
- **merge:** `owner` — the owner merges.
- **design:** `none` — no design stage; the implementation review alone decides.

## License

[MIT](LICENSE) © 2026 diegoami. Railroad Tycoon 3 is a trademark of its
respective owners; this project is not affiliated with or endorsed by them.
