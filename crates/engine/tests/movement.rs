//! Declared movement checks for milestone 6 claim C2: a synthetic graph walk,
//! the join-crossing path, bounded motion, stop at the path end, byte-identical
//! replay and a golden state hash on the populated fixture.

use std::fs;
use std::path::PathBuf;

use engine::movement::{Movement, Path, ACCELERATION, MAX_SPEED, TICK_SECONDS};
use rt3_kit::rail::{track, TrackGraph, TrackPiece};
use rt3_kit::save::sha256;

fn piece(x: f32, y: f32) -> TrackPiece {
    TrackPiece {
        origin: [x, y, 0.0],
    }
}

fn grid(pieces: Vec<TrackPiece>) -> TrackGraph {
    TrackGraph::from_pieces(pieces)
}

fn hex(bytes: &[u8]) -> String {
    sha256(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn field(line: &str, name: &str) -> f64 {
    line.split(&format!("\"{name}\":"))
        .nth(1)
        .and_then(|rest| rest.split([',', '}']).next())
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("missing {name} in {line}"))
}

/// Concatenated state lines with one trailing newline per line.
fn digest(lines: &[String]) -> String {
    let mut bytes = Vec::new();
    for line in lines {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    hex(&bytes)
}

fn run(path: Path, max_ticks: u64) -> (Movement, Vec<String>) {
    let mut movement = Movement::new(path);
    let mut lines = vec![movement.state_line()];
    while !movement.stopped && movement.tick < max_ticks {
        movement.step();
        lines.push(movement.state_line());
    }
    (movement, lines)
}

/// The fixture is resolved like the rail test: `RT3_TRACK_SAVE` may be
/// repository-relative.
fn track_fixture() -> Option<PathBuf> {
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
fn synthetic_path_accelerates_to_cruise_and_stops_at_the_end() {
    // Four pieces 30 units apart: a 90-unit straight path.
    let graph = grid(vec![
        piece(0.0, 0.0),
        piece(30.0, 0.0),
        piece(60.0, 0.0),
        piece(90.0, 0.0),
    ]);
    let path = Path::from_graph(&graph).expect("path");
    assert_eq!(path.pieces, vec![0, 1, 2, 3]);
    assert!((path.length() - 90.0).abs() < 1e-9);

    let (movement, lines) = run(path, 500);
    assert!(movement.stopped, "consist did not reach the path end");
    assert!((movement.distance - 90.0).abs() < 1e-9);
    assert_eq!(movement.speed, 0.0);

    // Accelerates by exactly one tick's worth until cruise, and never exceeds it.
    for line in &lines {
        assert!(line.starts_with('{') && line.ends_with('}'));
    }
    let speeds = [(1, 2.0), (2, 4.0), (3, 6.0), (4, 8.0), (5, 10.0), (6, 12.0)];
    for (tick, speed) in speeds {
        let prefix = format!("\"tick\":{tick},");
        let line = lines
            .iter()
            .find(|line| line.contains(&prefix))
            .expect("state line");
        assert!(
            line.contains(&format!("\"speed\":{speed:.6}")),
            "tick {tick}: {line}"
        );
    }
    assert!((ACCELERATION * TICK_SECONDS - 2.0).abs() < 1e-9);
    assert!(lines
        .iter()
        .all(|line| field(line, "speed") <= MAX_SPEED + 1e-9));
}

#[test]
fn synthetic_path_follows_a_join_and_visits_every_piece() {
    // Two short runs and a long middle run in record order A, C, B, so the
    // declared joins connect A's end to B's start and B's end to C's start.
    let graph = grid(vec![
        piece(0.0, 0.0),
        piece(33.0, 0.0),
        piece(132.0, 66.0),
        piece(165.0, 66.0),
        piece(33.0, 33.0),
        piece(66.0, 33.0),
        piece(99.0, 33.0),
        piece(132.0, 33.0),
    ]);
    assert_eq!(graph.runs, vec![0..2, 2..4, 4..8]);
    assert_eq!(graph.joins, vec![(1, 4), (7, 2)]);

    let path = Path::from_graph(&graph).expect("path");
    assert_eq!(path.pieces, vec![0, 1, 4, 5, 6, 7, 2, 3]);

    let (movement, lines) = run(path, 500);
    assert!(movement.stopped);
    assert_eq!(movement.distance, movement.path.length());
    assert_eq!(movement.position().0, 3, "ends on the last piece");
    assert!(lines
        .iter()
        .all(|line| { (0.0..=movement.path.length()).contains(&field(line, "distance")) }));
}

#[test]
fn replay_is_byte_identical_without_randomness() {
    let path = || {
        Path::from_graph(&grid(vec![
            piece(0.0, 0.0),
            piece(30.0, 0.0),
            piece(60.0, 0.0),
        ]))
        .expect("path")
    };
    let (_, first) = run(path(), 500);
    let (_, second) = run(path(), 500);
    assert_eq!(first, second);
    assert_eq!(digest(&first), digest(&second));
}

#[test]
fn rejects_a_graph_that_is_not_one_ordered_path() {
    // A join that lands on the middle of a run gives a piece three neighbours.
    let branched = TrackGraph {
        pieces: vec![
            piece(0.0, 0.0),
            piece(30.0, 0.0),
            piece(60.0, 0.0),
            piece(90.0, 0.0),
        ],
        runs: std::iter::once(0..4).collect(),
        joins: vec![(1, 3)],
        piece_length: 30.0,
    };
    assert!(Path::from_graph(&branched).is_none());

    // A disconnected pair of two-piece runs has four degree-1 endpoints.
    let split = grid(vec![
        piece(0.0, 0.0),
        piece(30.0, 0.0),
        piece(1000.0, 0.0),
        piece(1030.0, 0.0),
    ]);
    assert!(Path::from_graph(&split).is_none());

    assert!(Path::from_graph(&grid(Vec::new())).is_none());
}

#[test]
fn builds_and_runs_the_populated_fixture_path() {
    let Some(path) = track_fixture() else {
        eprintln!("RT3_TRACK_SAVE unset; skipping populated movement check");
        return;
    };
    let bytes = fs::read(&path).expect("track fixture");
    let collection = track(&bytes).expect("track collection");
    let graph = TrackGraph::from_collection(&collection).expect("track graph");
    let path = Path::from_graph(&graph).expect("ordered path");
    assert_eq!(path.pieces.len(), 61);
    assert_eq!(path.pieces.first(), Some(&0));
    assert_eq!(path.pieces.last(), Some(&9));
    assert!(path.length() > 1_500.0 && path.length() < 2_500.0);

    let (movement, lines) = run(path, 5_000);
    assert!(
        movement.stopped,
        "consist did not reach the fixture path end"
    );
    assert_eq!(movement.distance, movement.path.length());
    assert!((movement.distance - movement.path.length()).abs() < 1e-9);

    let (_, replay) = run(Path::from_graph(&graph).expect("ordered path"), 5_000);
    assert_eq!(lines, replay);
    assert_eq!(
        digest(&lines),
        "9bbdfd39b16df265f5650b32178d72b45fc2239d085bad20bfac6bee13176cf6",
        "fixture movement state hash changed"
    );
}
