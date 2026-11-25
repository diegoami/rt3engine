//! The native Railroad Tycoon 3 simulation kernel.
//!
//! One authoritative engine backs every surface: the future clients, the
//! observer, headless campaigns and the MCP interface. It is a fixed-tick
//! simulation of the original's real-time game (decision 0001): determinism
//! comes from seeded RNG and a declared tick rate, and the fidelity baseline
//! is *comparable outcomes* against oracle evidence — bit-exact original
//! reproduction is a separate program, not this crate's bar.
//!
//! Planned modules (milestones 5–10): `economy` (cargo supply/demand and
//! prices), `world` (loaded `.gmp` maps), `track`, `trains`, `companies`
//! (player and AI competitors), `market` (stocks and bonds), `campaign`
//! (objectives), `player` (observation/order interfaces), `campaign_runner`
//! (headless autoplay), `checkpoint` (versioned native persistence).
//!
//! Status: milestone 5 — `world` (map-grounded structural world: terrain,
//! city and building records, cargo catalogue). The economy tick lands with
//! the rest of milestone 5.

pub mod world;
