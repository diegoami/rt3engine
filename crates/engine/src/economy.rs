//! Declared fixed-tick cargo economy (milestone 5, claim C2).
//!
//! The original's economy constants are not recovered, so this module
//! implements a **declared model** rather than a copy of the game's formulas.
//! Every rule here is listed in `docs/evidence/economy-v1.md`, and every
//! divergence from measurable original state belongs to claim C5.
//!
//! Determinism: cargo is processed in index order, no hash-map iteration is
//! involved, and the only randomness is a seeded SplitMix64 stream whose draw
//! order is fixed by that index order.

use crate::world::World;

/// SplitMix64, the declared demand-noise generator.
#[derive(Debug, Clone)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Declared model parameters; the rule table is
/// `docs/evidence/economy-v1.md`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelRules {
    /// Ticks in one simulated month.
    pub ticks_per_month: u32,
    /// Stock every cargo starts with.
    pub initial_stock: f64,
    /// Stock ceiling as a multiple of `initial_stock`.
    pub stock_cap_multiple: f64,
    /// Units one placed building produces per tick.
    pub production_per_building: f64,
    /// Units of each cargo one city consumes per tick.
    pub demand_per_city: f64,
    /// Half-width of the uniform stock noise, in units.
    pub noise_amplitude: f64,
    /// Price floor as a factor of the cargo's base price.
    pub min_price_factor: f64,
    /// Price ceiling as a factor of the cargo's base price.
    pub max_price_factor: f64,
}

impl Default for ModelRules {
    fn default() -> Self {
        ModelRules {
            ticks_per_month: 30,
            initial_stock: 1_000.0,
            stock_cap_multiple: 10.0,
            production_per_building: 10.0,
            demand_per_city: 5.0,
            noise_amplitude: 2.0,
            min_price_factor: 0.5,
            max_price_factor: 1.5,
        }
    }
}

/// One cargo's live state.
#[derive(Debug, Clone, PartialEq)]
pub struct CargoState {
    pub name: String,
    pub index: i32,
    pub stock: f64,
    pub price: f64,
    base_price: f64,
    factor: f64,
}

/// The declared economy over one loaded world.
#[derive(Debug, Clone)]
pub struct Economy {
    rules: ModelRules,
    rng: SplitMix64,
    cargo: Vec<CargoState>,
    production: Vec<f64>,
    demand: Vec<f64>,
    month: u32,
}

impl Economy {
    /// Builds the economy from a loaded world. Production is assigned to
    /// building index `j` as cargo `j mod cargo types`; every city consumes
    /// every cargo. Both are declared rules, not recovered game behaviour.
    pub fn new(world: &World, rules: ModelRules, seed: u64) -> Economy {
        let count = world.cargo.len();
        let mut production = vec![0.0; count];
        if count > 0 {
            for building in 0..world.building_records {
                production[building % count] += rules.production_per_building;
            }
        }
        let demand = vec![world.city_records as f64 * rules.demand_per_city; count];
        let cargo = world
            .cargo
            .iter()
            .map(|cargo| {
                let base = cargo.price as f64;
                CargoState {
                    name: cargo.name.clone(),
                    index: cargo.index,
                    stock: rules.initial_stock,
                    price: base,
                    base_price: base,
                    factor: (cargo.factor as f64).clamp(0.0, 1.0),
                }
            })
            .collect();
        Economy {
            rules,
            rng: SplitMix64::new(seed),
            cargo,
            production,
            demand,
            month: 0,
        }
    }

    /// Advances one simulated month and returns its JSON dump line.
    pub fn step_month(&mut self) -> String {
        let cap = self.rules.initial_stock * self.rules.stock_cap_multiple;
        for _ in 0..self.rules.ticks_per_month {
            for index in 0..self.cargo.len() {
                let noise = (self.rng.unit() * 2.0 - 1.0) * self.rules.noise_amplitude;
                let stock = (self.cargo[index].stock + self.production[index] - self.demand[index]
                    + noise)
                    .clamp(0.0, cap);
                self.cargo[index].stock = stock;
                let base = self.cargo[index].base_price;
                let factor = self.cargo[index].factor;
                let price = base * (1.0 + factor * (1.0 - stock / self.rules.initial_stock));
                self.cargo[index].price = price.clamp(
                    base * self.rules.min_price_factor,
                    base * self.rules.max_price_factor,
                );
            }
        }
        self.month += 1;
        self.dump_line()
    }

    /// Advances `months` months and returns the concatenated dump lines.
    pub fn run_months(&mut self, months: u32) -> String {
        let mut dump = String::new();
        for _ in 0..months {
            dump.push_str(&self.step_month());
            dump.push('\n');
        }
        dump
    }

    /// The live cargo states, in cargo-index order.
    pub fn cargo(&self) -> &[CargoState] {
        &self.cargo
    }

    /// Completed simulated months.
    pub fn month(&self) -> u32 {
        self.month
    }

    /// The declared rules this economy runs under.
    pub fn rules(&self) -> ModelRules {
        self.rules
    }

    fn dump_line(&self) -> String {
        let mut line = format!("{{\"month\":{},\"cargo\":[", self.month);
        for (index, state) in self.cargo.iter().enumerate() {
            if index > 0 {
                line.push(',');
            }
            line.push_str(&format!(
                "{{\"name\":{},\"stock\":{:.2},\"price\":{:.2}}}",
                json_escape(&state.name),
                state.stock,
                state.price
            ));
        }
        line.push_str("]}");
        line
    }
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
