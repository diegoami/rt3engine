//! Install-backed check for claims C5/C6: the scripted company reaches
//! positive cash flow on the populated fixture line while the declared
//! competitor v0 runs the same scenario, and the joint state hash replays.

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

const JOINT_HASH: &str = "2d517847f5b1250fd9fc75cc3f8962b7605df9b92868153337894be8056f7e4c";

fn env_path(name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var_os(name)?);
    if path.is_absolute() {
        return Some(path);
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let candidate = repo.join(&path);
    Some(if candidate.exists() { candidate } else { path })
}

fn company(home: &std::path::Path, track_save: &std::path::Path, trips: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rt3ctl"))
        .args(["sim", "company", "--track-save"])
        .arg(track_save)
        .args(["--path"])
        .arg(home)
        .args(["--trips", trips, "--json"])
        .output()
        .expect("run rt3ctl sim company")
}

#[test]
fn fixture_company_is_positive_and_replays() {
    let (Some(home), Some(track_save)) = (env_path("RT3_HOME"), env_path("RT3_TRACK_SAVE")) else {
        eprintln!("RT3_HOME or RT3_TRACK_SAVE unset; skipping company run check");
        return;
    };

    let first = company(&home, &track_save, "5");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let value: Value = serde_json::from_slice(&first.stdout).expect("company JSON");
    let player_cash = value["player"]["cash"].as_f64().expect("player cash");
    assert!(player_cash > 0.0, "player cash must be positive: {value}");
    assert_eq!(value["player_positive"], true);
    assert_eq!(value["player"]["trips"], 5);
    assert_eq!(value["competitor"]["trips"], 5);
    assert_eq!(
        value["competitor"]["cash"].as_f64().expect("cash"),
        player_cash
    );
    assert_eq!(value["player"]["first_profitable_trip"], 1);
    assert_eq!(value["competitor"]["first_profitable_trip"], 1);
    assert_eq!(value["player"]["revenue"], 5000.0);
    assert_eq!(value["horizon_ticks"], 1696);
    assert_eq!(value["rules"]["fare_per_car_trip"], 250.0);
    assert_eq!(value["policy"]["start_tick"], 300);
    assert_eq!(value["state_sha256"], JOINT_HASH);

    let second = company(&home, &track_save, "5");
    assert!(second.status.success());
    assert_eq!(
        first.stdout, second.stdout,
        "the joint run must replay byte-identically"
    );

    let none = company(&home, &track_save, "0");
    assert!(!none.status.success(), "a zero-trip run must be rejected");
}
