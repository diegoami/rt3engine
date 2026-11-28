//! Scripted company and competitor v0 checks for claims C5 and C6.

use engine::company::{CompanyRules, CompetitorPolicy, JointRun};
use engine::movement::Path;
use engine::schedule::{Consist, Schedule, Station, Stop, Train};
use rt3_kit::rail::{TrackGraph, TrackPiece};
use rt3_kit::save::sha256;

fn path() -> Path {
    let pieces = (0..=4)
        .map(|index| TrackPiece {
            origin: [index as f32 * 30.0, 0.0, 0.0],
        })
        .collect();
    Path::from_graph(&TrackGraph::from_pieces(pieces)).expect("path")
}

fn stations() -> Vec<Station> {
    vec![
        Station {
            slot: 0,
            label: "A".to_string(),
        },
        Station {
            slot: 1,
            label: "B".to_string(),
        },
    ]
}

fn schedule(path: &Path) -> Schedule {
    let stations = stations();
    Schedule::new(
        vec![
            Stop {
                station: stations[0].clone(),
                distance: 0.0,
                dwell_ticks: 5,
            },
            Stop {
                station: stations[1].clone(),
                distance: path.length(),
                dwell_ticks: 0,
            },
        ],
        path.length(),
    )
    .expect("schedule")
}

fn consist() -> Consist {
    Consist {
        locomotive: "Test Loco".to_string(),
        cars: vec!["Car A".to_string(), "Car B".to_string()],
    }
}

fn trip_ticks(path: &Path, schedule: &Schedule) -> u64 {
    let mut train = Train::new(consist(), schedule.clone(), path.clone()).expect("train");
    while !train.finished() {
        train.step();
    }
    train.movement.tick
}

fn run() -> JointRun {
    let path = path();
    let schedule = schedule(&path);
    JointRun::run(
        &consist(),
        &path,
        &schedule,
        CompanyRules::default(),
        3,
        CompetitorPolicy {
            start_tick: 40,
            interval_ticks: 50,
        },
    )
    .expect("joint run")
}

#[test]
fn scripted_company_reaches_positive_cash_flow() {
    let path = path();
    let schedule = schedule(&path);
    let ticks = trip_ticks(&path, &schedule);
    let run = run();

    assert_eq!(run.player.trips, 3);
    assert_eq!(run.player.first_profitable_trip, Some(1));
    assert!(run.player.cash > 0.0);
    assert_eq!(run.player.revenue, 3.0 * 250.0 * 2.0);
    let expected_costs = 3.0 * (0.5 * ticks as f64 + 0.05 * path.length());
    assert!((run.player.costs - expected_costs).abs() < 1e-9);
    assert!((run.player.cash - (run.player.revenue - run.player.costs)).abs() < 1e-9);
}

#[test]
fn competitor_policy_places_its_own_trips_on_the_same_horizon() {
    let path = path();
    let schedule = schedule(&path);
    let ticks = trip_ticks(&path, &schedule);
    let run = run();

    assert_eq!(run.competitor.trips, 3);
    assert_eq!(run.competitor.first_profitable_trip, Some(1));
    assert!(run.competitor.cash > 0.0);
    assert_eq!(run.competitor.revenue, run.player.revenue);
    assert_eq!(run.competitor.costs, run.player.costs);
    let player_end = 3 * ticks;
    let competitor_end = 40 + 2 * 50 + ticks;
    assert_eq!(run.horizon_ticks, player_end.max(competitor_end));
}

#[test]
fn joint_run_is_deterministic_with_a_golden_hash() {
    let first = run();
    let second = run();
    assert_eq!(first, second);
    assert_eq!(first.state_line(), second.state_line());
    assert_eq!(first.state_digest(), second.state_digest());
    assert_eq!(
        first.state_digest(),
        sha256(first.state_line().as_bytes()),
        "the digest must hash the canonical text once"
    );
    assert_eq!(
        first
            .state_digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "cf2fbff569f06dee8dc1948a8384d80adfb4bcc8f3395a51f3597135c663eb0e"
    );
}

#[test]
fn rejects_invalid_company_inputs() {
    let path = path();
    let schedule = schedule(&path);
    let policy = CompetitorPolicy::default();
    let rules = CompanyRules::default();

    assert!(JointRun::run(&consist(), &path, &schedule, rules, 0, policy).is_none());
    assert!(JointRun::run(
        &consist(),
        &path,
        &schedule,
        rules,
        1,
        CompetitorPolicy {
            start_tick: 0,
            interval_ticks: 0,
        }
    )
    .is_none());
    for bad in [
        CompanyRules {
            fare_per_car_trip: -1.0,
            ..rules
        },
        CompanyRules {
            cost_per_tick: f64::NAN,
            ..rules
        },
        CompanyRules {
            cost_per_unit: f64::INFINITY,
            ..rules
        },
        CompanyRules {
            initial_cash: -1.0,
            ..rules
        },
    ] {
        assert!(JointRun::run(&consist(), &path, &schedule, bad, 1, policy).is_none());
    }

    let mut wrong = schedule.clone();
    wrong.stops[1].distance = path.length() - 1.0;
    assert!(JointRun::run(&consist(), &path, &wrong, rules, 1, policy).is_none());

    assert!(JointRun::run(
        &consist(),
        &path,
        &schedule,
        CompanyRules {
            fare_per_car_trip: f64::MAX,
            ..rules
        },
        1,
        policy
    )
    .is_none());
    assert!(JointRun::run(
        &consist(),
        &path,
        &schedule,
        CompanyRules {
            cost_per_tick: f64::MAX,
            ..rules
        },
        1,
        policy
    )
    .is_none());
    assert!(JointRun::run(
        &consist(),
        &path,
        &schedule,
        rules,
        1,
        CompetitorPolicy {
            start_tick: u64::MAX,
            interval_ticks: 1,
        }
    )
    .is_none());
}
