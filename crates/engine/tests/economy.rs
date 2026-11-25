//! Synthetic-scenario checks for claim C2: the declared economy's stock
//! identity, bounds, seeded determinism and stable monthly dump.

use std::fs;
use std::path::PathBuf;

use engine::economy::{Economy, ModelRules};
use engine::world::World;
use rt3_kit::save::sha256;
use rt3_kit::terrain::HeightGrid;
use rt3_kit::typed::CargoType;

fn cargo(name: &str, price: f32, factor: f32, index: i32) -> CargoType {
    CargoType {
        name: name.to_string(),
        price,
        factor,
        category: 0,
        icon: 0,
        car_type: "Boxcar".to_string(),
        index,
    }
}

fn synthetic(cities: usize, buildings: usize) -> World {
    World {
        map_name: "synthetic".to_string(),
        terrain: HeightGrid {
            offset: 0,
            rows: 2,
            cols: 2,
            heights: vec![0.0; 4],
            nan_cells: 0,
            zero_cells: 4,
        },
        city_records: cities,
        building_records: buildings,
        cargo: vec![cargo("Coal", 100.0, 0.2, 0), cargo("Grain", 50.0, 0.3, 1)],
    }
}

/// Reads a committed synthetic scenario fixture.
fn scenario(name: &str) -> World {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut cities = 0;
    let mut buildings = 0;
    let mut cargo_types = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("cities") => cities = parts.next().unwrap().parse().unwrap(),
            Some("buildings") => buildings = parts.next().unwrap().parse().unwrap(),
            Some("cargo") => {
                let name = parts.next().unwrap().to_string();
                let price = parts.next().unwrap().parse().unwrap();
                let factor = parts.next().unwrap().parse().unwrap();
                let index = parts.next().unwrap().parse().unwrap();
                cargo_types.push(CargoType {
                    name,
                    price,
                    factor,
                    category: 0,
                    icon: 0,
                    car_type: "Boxcar".to_string(),
                    index,
                });
            }
            other => panic!("bad scenario line: {other:?}"),
        }
    }
    World {
        map_name: "synthetic".to_string(),
        terrain: HeightGrid {
            offset: 0,
            rows: 2,
            cols: 2,
            heights: vec![0.0; 4],
            nan_cells: 0,
            zero_cells: 4,
        },
        city_records: cities,
        building_records: buildings,
        cargo: cargo_types,
    }
}

fn hex(bytes: &[u8]) -> String {
    sha256(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn stock_identity_holds_without_noise() {
    let world = synthetic(2, 3);
    let rules = ModelRules {
        ticks_per_month: 1,
        noise_amplitude: 0.0,
        ..ModelRules::default()
    };
    let mut economy = Economy::new(&world, rules, 1);
    let line = economy.step_month();

    // Building 0 and 2 produce Coal, building 1 produces Grain (1 unit each).
    // Each of the 2 cities consumes 0.04 of every cargo.
    let coal = &economy.cargo()[0];
    let grain = &economy.cargo()[1];
    assert!(
        (coal.stock - (10_000.0 + 2.0 - 0.08)).abs() < 1e-9,
        "coal stock identity: {}",
        coal.stock
    );
    assert!(
        (grain.stock - (10_000.0 + 1.0 - 0.08)).abs() < 1e-9,
        "grain stock identity: {}",
        grain.stock
    );
    assert!(line.contains("\"month\":1"), "dump line: {line}");
    assert!(line.contains("\"name\":\"Coal\""), "dump line: {line}");
}

#[test]
fn stocks_and_prices_stay_within_bounds() {
    let world = synthetic(70, 0);
    let rules = ModelRules::default();
    let mut economy = Economy::new(&world, rules, 7);
    for _ in 0..120 {
        economy.step_month();
        for state in economy.cargo() {
            assert!(
                (0.0..=rules.initial_stock * rules.stock_cap_multiple).contains(&state.stock),
                "{} stock out of bounds: {}",
                state.name,
                state.stock
            );
            let base = if state.name == "Coal" { 100.0 } else { 50.0 };
            assert!(
                (base * rules.min_price_factor..=base * rules.max_price_factor)
                    .contains(&state.price),
                "{} price out of bounds: {}",
                state.name,
                state.price
            );
        }
    }
}

#[test]
fn same_seed_replays_and_a_different_seed_diverges() {
    let world = scenario("economy-scenario.txt");
    let run = |seed| {
        let mut economy = Economy::new(&world, ModelRules::default(), seed);
        economy.run_months(12)
    };
    assert_eq!(run(42), run(42), "same seed must replay byte-identically");
    assert_ne!(run(42), run(43), "a different seed must change the dump");
}

#[test]
fn stock_cap_and_price_clamps_are_reached() {
    let rules = ModelRules {
        noise_amplitude: 0.0,
        ..ModelRules::default()
    };
    let cap = rules.initial_stock * rules.stock_cap_multiple;

    // 500 buildings on one cargo and no cities: stock reaches the cap and the
    // price is clamped to the floor.
    let mut economy = Economy::new(&scenario("economy-cap.txt"), rules, 1);
    for _ in 0..12 {
        economy.step_month();
    }
    assert_eq!(economy.cargo()[0].stock, cap, "stock cap reached");
    assert_eq!(economy.cargo()[0].price, 50.0, "price floor applied");

    // No buildings and extreme demand on one cargo: stock reaches zero and
    // the price is clamped to the ceiling.
    let mut economy = Economy::new(&scenario("economy-drain.txt"), rules, 1);
    for _ in 0..2 {
        economy.step_month();
    }
    assert_eq!(economy.cargo()[0].stock, 0.0, "stock floor reached");
    assert_eq!(economy.cargo()[0].price, 150.0, "price ceiling applied");
}

#[test]
fn twelve_month_dump_has_a_stable_hash() {
    let world = scenario("economy-scenario.txt");
    let mut economy = Economy::new(&world, ModelRules::default(), 42);
    let dump = economy.run_months(12);
    assert_eq!(
        hex(dump.as_bytes()),
        "9c6d4d97b1ce71e996b67ccc9403cb11bb02e1d613d73ace0d5601de300a1b8c"
    );
}
