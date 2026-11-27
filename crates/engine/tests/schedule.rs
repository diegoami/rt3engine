//! Declared station/consist/schedule checks for milestone 6 claim C3: stop
//! order and dwell on a synthetic graph, schedule validation, byte-identical
//! replay, the typed catalogue and the populated fixture run.

use std::path::PathBuf;

use engine::movement::Path;
use engine::schedule::{Consist, Schedule, Station, Stop, Train};
use rt3_kit::fixed::{FixedBlobCollection, FixedBlobRecord};
use rt3_kit::rail::{stations, track, TrackGraph, TrackPiece};
use rt3_kit::save::sha256;
use rt3_kit::typed::{scan_family, CarType, Locomotive};

fn piece(x: f32, y: f32) -> TrackPiece {
    TrackPiece {
        origin: [x, y, 0.0],
    }
}

fn straight_path() -> Path {
    Path::from_graph(&TrackGraph::from_pieces(vec![
        piece(0.0, 0.0),
        piece(30.0, 0.0),
        piece(60.0, 0.0),
        piece(90.0, 0.0),
    ]))
    .expect("path")
}

fn stations_collection(count: usize) -> FixedBlobCollection {
    FixedBlobCollection {
        section_offset: 0,
        capacity: count,
        records: vec![
            FixedBlobRecord {
                data: vec![0; 1_549],
            };
            count
        ],
        end_offset: 0,
        trailer: Vec::new(),
    }
}

fn locomotive(name: &str) -> Locomotive {
    Locomotive {
        name: name.to_string(),
        floats: [0.0; 4],
        u8_field: 0,
        u32s: [0; 7],
        u8s: [0; 4],
        clamped: [0, 0],
    }
}

fn car(name: &str) -> CarType {
    CarType {
        flag: 0,
        string_id: 0,
        display_name: name.to_string(),
        lco_name: name.to_string(),
        id_name: name.to_string(),
        short_name: name.to_string(),
        tail_name: String::new(),
        tail_u32s: [0; 4],
    }
}

fn station(stations: &FixedBlobCollection, slot: usize, label: &str) -> Station {
    Station::from_record(stations, slot, label).expect("station record")
}

fn schedule(stations: &FixedBlobCollection, length: f64) -> Schedule {
    Schedule::new(
        vec![
            Stop {
                station: station(stations, 0, "A"),
                distance: 0.0,
                dwell_ticks: 2,
            },
            Stop {
                station: station(stations, 1, "B"),
                distance: 30.0,
                dwell_ticks: 2,
            },
            Stop {
                station: station(stations, 2, "C"),
                distance: length,
                dwell_ticks: 0,
            },
        ],
        length,
    )
    .expect("schedule")
}

fn consist() -> Consist {
    Consist::new(&locomotive("Test Loco"), &[car("Car A"), car("Car B")])
}

fn digest(lines: &[String]) -> String {
    let mut bytes = Vec::new();
    for line in lines {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    sha256(&bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn run(train: &mut Train, max_ticks: u64) {
    while !train.finished() && train.movement.tick < max_ticks {
        train.step();
    }
}

fn fixture() -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var("RT3_TRACK_SAVE").ok()?);
    if path.is_absolute() {
        return Some(path);
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let candidate = repo.join(&path);
    if candidate.exists() {
        Some(candidate)
    } else {
        Some(path)
    }
}

#[test]
fn synthetic_schedule_stops_at_every_station_in_order() {
    let stations = stations_collection(3);
    let path = straight_path();
    let schedule = schedule(&stations, path.length());
    let mut train = Train::new(consist(), schedule, path).expect("train");
    assert_eq!(train.consist.locomotive, "Test Loco");
    assert_eq!(train.consist.car_count(), 2);

    run(&mut train, 500);
    assert!(train.finished(), "run did not reach the final station");
    assert_eq!(train.movement.speed, 0.0);
    assert_eq!(train.movement.distance, train.movement.path.length());

    let stops: Vec<(&str, u64, Option<u64>)> = train
        .stops
        .iter()
        .map(|record| (record.station.as_str(), record.arrival, record.departure))
        .collect();
    assert_eq!(
        stops,
        vec![("A", 0, Some(2)), ("B", 8, Some(10)), ("C", 18, None)]
    );
    assert_eq!(
        train
            .stops
            .iter()
            .map(|record| record.slot)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        digest(&train.dump_lines()),
        "073f73bf46ce7b2aa6f3c192e0e5210610b1dac3a94fc30e4faa0ea4b5591dd8",
        "synthetic schedule dump changed"
    );
}

#[test]
fn schedule_requires_ordered_stops_from_start_to_path_end() {
    let stations = stations_collection(3);
    let stop = |slot: usize, distance: f64| Stop {
        station: station(&stations, slot, "S"),
        distance,
        dwell_ticks: 0,
    };

    assert!(Schedule::new(vec![stop(0, 0.0)], 90.0).is_none());
    assert!(Schedule::new(vec![stop(0, 10.0), stop(1, 90.0)], 90.0).is_none());
    assert!(Schedule::new(vec![stop(0, 0.0), stop(1, 80.0)], 90.0).is_none());
    assert!(Schedule::new(vec![stop(0, 0.0), stop(1, 0.0), stop(2, 90.0)], 90.0).is_none());
    assert!(Schedule::new(vec![stop(0, 0.0), stop(1, 90.0), stop(2, 80.0)], 90.0).is_none());
    assert!(Schedule::new(vec![stop(0, 0.0), stop(1, -5.0), stop(2, 90.0)], 90.0).is_none());
    assert!(Schedule::new(vec![stop(0, 0.0), stop(1, 100.0)], 90.0).is_none());
    assert!(Schedule::new(vec![stop(0, 0.0), stop(1, 90.0)], 0.0).is_none());

    assert!(Schedule::new(vec![stop(0, 0.0), stop(1, 90.0)], 90.0).is_some());
    assert!(Schedule::between(&stations, 0, "A", 2, "C", 90.0).is_some());
    assert!(Schedule::between(&stations, 0, "A", 3, "D", 90.0).is_none());
}

#[test]
fn zero_dwell_origin_records_its_departure() {
    let stations = stations_collection(2);
    let path = Path::from_graph(&TrackGraph::from_pieces(vec![
        piece(0.0, 0.0),
        piece(15.0, 0.0),
        piece(30.0, 0.0),
    ]))
    .expect("path");
    let schedule = Schedule::new(
        vec![
            Stop {
                station: station(&stations, 0, "A"),
                distance: 0.0,
                dwell_ticks: 0,
            },
            Stop {
                station: station(&stations, 1, "B"),
                distance: 30.0,
                dwell_ticks: 0,
            },
        ],
        path.length(),
    )
    .expect("schedule");
    let mut train = Train::new(consist(), schedule, path).expect("train");
    assert_eq!(train.stops[0].departure, Some(0));

    run(&mut train, 500);
    assert!(train.finished());
    assert_eq!(train.stops[0].departure, Some(0));
    assert!(train.stops[1].arrival > 0);
}

#[test]
fn near_equal_final_stop_is_normalized_to_the_path_end() {
    let stations = stations_collection(2);
    let path = Path::from_graph(&TrackGraph::from_pieces(vec![
        piece(0.0, 0.0),
        piece(15.0, 0.0),
        piece(30.0, 0.0),
    ]))
    .expect("path");
    let schedule = Schedule::new(
        vec![
            Stop {
                station: station(&stations, 0, "A"),
                distance: 0.0,
                dwell_ticks: 1,
            },
            Stop {
                station: station(&stations, 1, "B"),
                distance: 29.999_999_5,
                dwell_ticks: 0,
            },
        ],
        path.length(),
    )
    .expect("schedule");
    let mut train = Train::new(consist(), schedule, path).expect("train");
    run(&mut train, 500);
    assert!(train.finished());
    assert!(train.movement.stopped);
    assert_eq!(train.movement.distance, train.movement.path.length());
    assert!(train
        .dump_lines()
        .last()
        .unwrap()
        .contains("\"distance\":30.000"));
}

#[test]
fn replay_produces_the_same_stops_and_dump() {
    let stations = stations_collection(3);
    let mut first = {
        let path = straight_path();
        let schedule = schedule(&stations, path.length());
        Train::new(consist(), schedule, path).expect("train")
    };
    let mut second = {
        let path = straight_path();
        let schedule = schedule(&stations, path.length());
        Train::new(consist(), schedule, path).expect("train")
    };
    run(&mut first, 500);
    run(&mut second, 500);
    assert_eq!(first.stops, second.stops);
    assert_eq!(first.dump_lines(), second.dump_lines());
    assert_eq!(digest(&first.dump_lines()), digest(&second.dump_lines()));
}

fn catalogue_consist(root: &std::path::Path) -> Consist {
    let dir = root.join("Data/EngineTypes");
    let locos = scan_family(&dir, "lco", Locomotive::parse);
    let cars = scan_family(&dir, "car", CarType::parse);
    assert!(locos.failures.is_empty(), "unparsed locomotive records");
    assert!(cars.failures.is_empty(), "unparsed car records");
    let (_, planet) = locos
        .records
        .iter()
        .find(|(_, record)| record.name.starts_with("Planet"))
        .expect("Planet locomotive");
    let typed_cars: Vec<CarType> = cars
        .records
        .iter()
        .take(4)
        .map(|(_, record)| record.clone())
        .collect();
    assert_eq!(typed_cars.len(), 4);
    Consist::new(planet, &typed_cars)
}

#[test]
fn consist_is_built_from_the_typed_catalogue() {
    let Some(root) = std::env::var_os("RT3_HOME").map(PathBuf::from) else {
        eprintln!("RT3_HOME unset; skipping typed catalogue consist check");
        return;
    };
    let consist = catalogue_consist(&root);
    assert_eq!(consist.locomotive, "PlanetL");
    assert_eq!(consist.car_count(), 4);
    assert!(consist.cars.iter().all(|name| !name.is_empty()));
}

#[test]
fn builds_and_runs_the_populated_fixture_schedule() {
    let Some(root) = std::env::var_os("RT3_HOME").map(PathBuf::from) else {
        eprintln!("RT3_HOME unset; skipping populated schedule check");
        return;
    };
    let Some(path) = fixture() else {
        eprintln!("RT3_TRACK_SAVE unset; skipping populated schedule check");
        return;
    };
    let bytes = std::fs::read(&path).expect("track fixture");
    let track = track(&bytes).expect("track collection");
    let graph = TrackGraph::from_collection(&track).expect("track graph");
    let path = Path::from_graph(&graph).expect("ordered path");
    let stations = stations(&bytes).expect("station collection");
    assert_eq!(stations.records.len(), 3);

    let schedule = Schedule::between(&stations, 0, "Bath", 1, "Gloucester", path.length())
        .expect("fixture schedule");
    let mut train = Train::new(catalogue_consist(&root), schedule, path).expect("train");
    assert_eq!(train.consist.locomotive, "PlanetL");
    assert_eq!(train.consist.car_count(), 4);
    run(&mut train, 5_000);

    assert!(train.finished(), "run did not reach Gloucester");
    let labels: Vec<&str> = train
        .stops
        .iter()
        .map(|record| record.station.as_str())
        .collect();
    assert_eq!(labels, vec!["Bath", "Gloucester"]);
    assert_eq!(train.stops[0].arrival, 0);
    assert_eq!(
        train.stops[0].departure,
        Some(engine::schedule::DEFAULT_DWELL_TICKS)
    );
    assert!(train.stops[1].arrival > 0);
    assert_eq!(train.stops[1].arrival, 196);
    assert_eq!(train.stops[1].departure, None);
    assert_eq!(train.movement.distance, train.movement.path.length());
    assert_eq!(
        digest(&train.dump_lines()),
        "b272a9a6a88cb2fc39c5dc2c20924fde728eccda0b89d65e0157ac5833e9a419",
        "fixture schedule dump changed"
    );
}
