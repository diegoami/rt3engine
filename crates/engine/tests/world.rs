//! Install-backed check for claim C1: a map-grounded world loads the
//! structural records and the cargo catalogue. Values are the committed
//! `docs/evidence/terrain-scan-v1.json` and `typed-tables-v1.json` figures;
//! the test fails if the loaders drift from them.

use std::path::PathBuf;

use engine::world;

struct Case {
    dir: &'static str,
    file: &'static str,
    rows: usize,
    cols: usize,
    min_height_m: f32,
    max_height_m: f32,
    cities: usize,
    buildings: usize,
}

const CASES: [Case; 3] = [
    Case {
        dir: "maps",
        file: "British Isles.gmp",
        rows: 513,
        cols: 321,
        min_height_m: 0.0,
        max_height_m: 760.5,
        cities: 68,
        buildings: 0,
    },
    Case {
        dir: "maps",
        file: "California Island.gmp",
        rows: 257,
        cols: 257,
        min_height_m: -454.0,
        max_height_m: 3051.1,
        cities: 46,
        buildings: 0,
    },
    // A current map with placed building records, so the building path is
    // exercised with a non-zero collection too.
    Case {
        dir: "maps",
        file: "Greenland Growing.gmp",
        rows: 193,
        cols: 257,
        min_height_m: -130.1,
        max_height_m: 1177.5,
        cities: 42,
        buildings: 54,
    },
];

fn home() -> Option<PathBuf> {
    std::env::var_os("RT3_HOME").map(PathBuf::from)
}

#[test]
fn loads_map_grounded_world() {
    let Some(home) = home() else {
        eprintln!("RT3_HOME unset; skipping install-backed world check");
        return;
    };
    for case in CASES {
        let path = home.join(case.dir).join(case.file);
        let world = world::load(&path, &home).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(
            world.map_name,
            case.file.trim_end_matches(".gmp"),
            "map name for {}",
            case.file
        );
        assert_eq!(
            (world.terrain.rows, world.terrain.cols),
            (case.rows, case.cols),
            "terrain dimensions for {}",
            case.file
        );
        let (min, max) = world.terrain.height_range().expect("height range");
        assert!(
            (min - case.min_height_m).abs() < 0.06,
            "{} min height {min} != {}",
            case.file,
            case.min_height_m
        );
        assert!(
            (max - case.max_height_m).abs() < 0.06,
            "{} max height {max} != {}",
            case.file,
            case.max_height_m
        );
        assert_eq!(world.city_records, case.cities, "cities for {}", case.file);
        assert_eq!(
            world.building_records, case.buildings,
            "buildings for {}",
            case.file
        );
        assert_eq!(world.cargo.len(), 41, "cargo types for {}", case.file);
        assert!(
            world.cargo.iter().any(|t| t.name == "Alcohol"),
            "named cargo type for {}",
            case.file
        );
        assert!(
            world
                .cargo
                .windows(2)
                .all(|pair| (pair[0].index, &pair[0].name) <= (pair[1].index, &pair[1].name)),
            "cargo catalogue order for {}",
            case.file
        );
    }
}

#[test]
fn rejects_legacy_map_versions() {
    let Some(home) = home() else {
        eprintln!("RT3_HOME unset; skipping install-backed version check");
        return;
    };
    let path = home.join("Data").join("Tutorial").join("Tutorial_1.gmp");
    let error = world::load_map(&path).expect_err("legacy map must be rejected");
    assert!(
        error.to_string().contains("version 9609"),
        "unexpected error: {error}"
    );
}
