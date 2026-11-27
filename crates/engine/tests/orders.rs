//! State-encoding and checkpoint-invariant checks for claim C4.

use engine::orders::{DeclaredTrack, Order, OrderState};
use engine::schedule::{Schedule, Station, Stop};
use rt3_kit::save::sha256;

fn stations(count: usize) -> Vec<Station> {
    (0..count)
        .map(|slot| Station {
            slot,
            label: format!("Station {slot}"),
        })
        .collect()
}

fn train(locomotive: &str, cars: &[&str]) -> OrderState {
    let mut state = OrderState::new(stations(2));
    state
        .apply(&Order::BuyTrain {
            locomotive: locomotive.to_string(),
            cars: cars.iter().map(|car| (*car).to_string()).collect(),
        })
        .expect("buy train");
    state
}

#[test]
fn names_with_spaces_do_not_collide_in_the_state_text() {
    let spaced_locomotive = train("A 1", &[]);
    let spaced_car = train("A", &["0"]);
    assert_ne!(spaced_locomotive.state_line(), spaced_car.state_line());
    assert_ne!(spaced_locomotive.state_digest(), spaced_car.state_digest());
}

#[test]
fn state_digest_is_sha256_of_the_canonical_text() {
    let state = train("PlanetL", &["242 A1", "2D2"]);
    assert_eq!(
        state.state_digest(),
        sha256(state.state_line().as_bytes()),
        "the digest must not hash the canonical text twice"
    );
}

#[test]
fn validate_rejects_a_track_with_a_missing_endpoint() {
    let mut state = OrderState::new(stations(2));
    state.track = Some(DeclaredTrack::new(0, 99, 120.0).expect("declared track"));
    assert!(state.validate().is_err());
}

#[test]
fn validate_rejects_schedule_endpoints_that_do_not_match_the_track() {
    let mut state = OrderState::new(stations(3));
    state
        .apply(&Order::BuildTrack {
            from_slot: 0,
            to_slot: 1,
            length: 120.0,
        })
        .expect("build track");
    state
        .apply(&Order::BuyTrain {
            locomotive: "PlanetL".to_string(),
            cars: Vec::new(),
        })
        .expect("buy train");
    let path_length = state.track.as_ref().expect("track").path.length();
    let schedule = Schedule::new(
        vec![
            Stop {
                station: state.stations[0].clone(),
                distance: 0.0,
                dwell_ticks: 5,
            },
            Stop {
                station: state.stations[2].clone(),
                distance: path_length,
                dwell_ticks: 0,
            },
        ],
        path_length,
    )
    .expect("schedule");
    state.schedule = Some(schedule);
    assert!(state.validate().is_err());
}

#[test]
fn validate_rejects_a_consist_with_empty_names() {
    let mut state = OrderState::new(stations(2));
    state.consist = Some(engine::schedule::Consist {
        locomotive: String::new(),
        cars: Vec::new(),
    });
    assert!(state.validate().is_err());

    state.consist = Some(engine::schedule::Consist {
        locomotive: "PlanetL".to_string(),
        cars: vec!["Car".to_string(), String::new()],
    });
    assert!(state.validate().is_err());
}

#[test]
fn validate_rejects_duplicate_station_slots() {
    let state = OrderState::new(vec![
        Station {
            slot: 0,
            label: "Bath".to_string(),
        },
        Station {
            slot: 0,
            label: "Other".to_string(),
        },
    ]);
    assert!(state.validate().is_err());
}

#[test]
fn a_rejected_order_leaves_the_state_untouched() {
    let mut state = OrderState::new(stations(2));
    let before = state.clone();
    assert!(state
        .apply(&Order::BuildTrack {
            from_slot: 0,
            to_slot: 9,
            length: 120.0,
        })
        .is_err());
    assert_eq!(state, before);
    assert!(state
        .apply(&Order::AssignSchedule { dwell_ticks: 5 })
        .is_err());
    assert_eq!(state, before);
}
