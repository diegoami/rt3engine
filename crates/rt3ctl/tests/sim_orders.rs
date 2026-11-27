//! CLI checks for milestone 6 claim C4: declared order commands, the order log,
//! checkpoints and byte-identical replay from a checkpoint or from the start.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const FINAL_HASH: &str = "9c526411bf691fbedc5f756d9d302ff271489f6593c57caf2c7fb56f0829a23c";

fn exe() -> &'static str {
    env!("CARGO_BIN_EXE_rt3ctl")
}

fn dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("rt3engine-orders-{name}-{}", std::process::id()));
    fs::remove_dir_all(&path).ok();
    fs::create_dir_all(&path).expect("temp dir");
    path
}

fn ctl(args: &[&str]) -> Output {
    Command::new(exe()).args(args).output().expect("run rt3ctl")
}

fn path_string(path: &Path) -> String {
    path.to_str().expect("UTF-8 path").to_string()
}

fn stdout_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "bad JSON ({error}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn init(checkpoint: &Path, log: &Path, stations: &[&str]) -> Output {
    let mut args = vec![
        "sim".to_string(),
        "init".to_string(),
        "--checkpoint".to_string(),
        path_string(checkpoint),
        "--log".to_string(),
        path_string(log),
    ];
    for station in stations {
        args.push("--station".to_string());
        args.push((*station).to_string());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = ctl(&refs);
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn orders_log_checkpoints_and_replay_match() {
    let root = dir("flow");
    let checkpoint = root.join("checkpoint.json");
    let log = root.join("orders.jsonl");
    let start = root.join("cp0.json");
    let after_build = root.join("cp1.json");

    let output = init(&checkpoint, &log, &["0:Bath", "1:Gloucester"]);
    let value = stdout_json(&output);
    assert_eq!(value["seq"], 0);
    assert_eq!(value["stations"], 2);
    fs::copy(&checkpoint, &start).expect("copy start checkpoint");

    let build = ctl(&[
        "sim",
        "build-track",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--from",
        "0",
        "--to",
        "1",
        "--length",
        "120",
    ]);
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert_eq!(stdout_json(&build)["seq"], 1);
    fs::copy(&checkpoint, &after_build).expect("copy build checkpoint");

    let buy = ctl(&[
        "sim",
        "buy-train",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--locomotive",
        "PlanetL",
        "--car",
        "242 A1",
        "--car",
        "2D2",
    ]);
    assert!(
        buy.status.success(),
        "buy failed: {}",
        String::from_utf8_lossy(&buy.stderr)
    );
    assert_eq!(stdout_json(&buy)["seq"], 2);

    let assign = ctl(&[
        "sim",
        "assign-schedule",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--dwell",
        "2",
    ]);
    assert!(
        assign.status.success(),
        "assign failed: {}",
        String::from_utf8_lossy(&assign.stderr)
    );
    assert_eq!(stdout_json(&assign)["seq"], 3);

    let checkpoint_value: Value =
        serde_json::from_str(&fs::read_to_string(&checkpoint).expect("checkpoint"))
            .expect("checkpoint JSON");
    assert_eq!(checkpoint_value["version"], 1);
    assert_eq!(checkpoint_value["seq"], 3);
    assert_eq!(checkpoint_value["state"]["track"]["from"], 0);
    assert_eq!(checkpoint_value["state"]["track"]["to"], 1);
    assert_eq!(
        checkpoint_value["state"]["consist"]["locomotive"],
        "PlanetL"
    );
    assert_eq!(checkpoint_value["state"]["schedule"]["stops"][1][0], 1);

    let full = ctl(&[
        "sim",
        "replay",
        "--checkpoint",
        &path_string(&start),
        "--log",
        &path_string(&log),
        "--json",
    ]);
    assert!(
        full.status.success(),
        "full replay failed: {}",
        String::from_utf8_lossy(&full.stderr)
    );
    let full_value = stdout_json(&full);
    assert_eq!(full_value["seq"], 3);
    assert_eq!(full_value["state_sha256"], FINAL_HASH);
    assert_eq!(
        full_value["state_sha256"], checkpoint_value["state_sha256"],
        "replay must reproduce the checkpoint state"
    );
    assert_eq!(full_value["state"]["schedule"]["stops"][0][3], 2);

    let resumed = ctl(&[
        "sim",
        "replay",
        "--checkpoint",
        &path_string(&after_build),
        "--log",
        &path_string(&log),
        "--json",
    ]);
    assert!(
        resumed.status.success(),
        "resumed replay failed: {}",
        String::from_utf8_lossy(&resumed.stderr)
    );
    assert_eq!(
        resumed.stdout, full.stdout,
        "a resumed replay must end byte-identically to a full replay"
    );

    let log_text = fs::read_to_string(&log).expect("log");
    let lines: Vec<&str> = log_text.lines().collect();
    assert_eq!(lines.len(), 3);
    for (index, line) in lines.iter().enumerate() {
        let value: Value = serde_json::from_str(line).expect("log line JSON");
        assert_eq!(value["seq"], index as u64 + 1);
    }

    fs::remove_dir_all(&root).ok();
}

#[test]
fn invalid_orders_do_not_advance_the_log() {
    let root = dir("invalid");
    let checkpoint = root.join("checkpoint.json");
    let log = root.join("orders.jsonl");
    let start = root.join("cp0.json");

    init(&checkpoint, &log, &["0:Bath", "1:Gloucester"]);
    fs::copy(&checkpoint, &start).expect("copy start checkpoint");
    assert_eq!(fs::read(&log).expect("log").len(), 0);

    let assign = ctl(&[
        "sim",
        "assign-schedule",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
    ]);
    assert!(!assign.status.success(), "assign without a track must fail");
    assert_eq!(fs::read(&log).expect("log").len(), 0);

    let same_slot = ctl(&[
        "sim",
        "build-track",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--from",
        "0",
        "--to",
        "0",
        "--length",
        "120",
    ]);
    assert!(!same_slot.status.success(), "same-slot track must fail");

    let short = ctl(&[
        "sim",
        "build-track",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--from",
        "0",
        "--to",
        "1",
        "--length",
        "1",
    ]);
    assert!(!short.status.success(), "below-minimum track must fail");
    assert_eq!(fs::read(&log).expect("log").len(), 0);

    let empty_locomotive = ctl(&[
        "sim",
        "buy-train",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--locomotive",
        "",
    ]);
    assert!(
        !empty_locomotive.status.success(),
        "empty locomotive must fail"
    );

    let empty_replay = ctl(&[
        "sim",
        "replay",
        "--checkpoint",
        &path_string(&start),
        "--log",
        &path_string(&log),
        "--json",
    ]);
    assert!(empty_replay.status.success());
    assert_eq!(stdout_json(&empty_replay)["seq"], 0);

    // A valid order, then a tampered state hash: replay must reject the log.
    let build = ctl(&[
        "sim",
        "build-track",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--from",
        "0",
        "--to",
        "1",
        "--length",
        "120",
    ]);
    assert!(build.status.success());
    let log_text = fs::read_to_string(&log).expect("log");
    let first: Value = serde_json::from_str(log_text.lines().next().expect("first log line"))
        .expect("log line JSON");
    let recorded = first["state_sha256"]
        .as_str()
        .expect("recorded state hash")
        .to_string();
    let tampered = log_text.replacen(&recorded, &"0".repeat(64), 1);
    fs::write(&log, tampered).expect("tamper log");
    let replay = ctl(&[
        "sim",
        "replay",
        "--checkpoint",
        &path_string(&start),
        "--log",
        &path_string(&log),
        "--json",
    ]);
    assert!(!replay.status.success(), "tampered log must be rejected");
    assert!(
        String::from_utf8_lossy(&replay.stderr).contains("does not match the log"),
        "unexpected error: {}",
        String::from_utf8_lossy(&replay.stderr)
    );

    fs::remove_dir_all(&root).ok();
}

#[test]
fn init_rejects_duplicate_station_slots() {
    let root = dir("duplicate");
    let checkpoint = root.join("checkpoint.json");
    let log = root.join("orders.jsonl");
    let output = ctl(&[
        "sim",
        "init",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--station",
        "0:Bath",
        "--station",
        "0:Other",
    ]);
    assert!(!output.status.success());
    assert!(!checkpoint.exists());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn a_log_checkpoint_mismatch_is_rejected_without_appending() {
    let root = dir("mismatch");
    let checkpoint = root.join("checkpoint.json");
    let log = root.join("orders.jsonl");
    let after_build = root.join("cp1.json");

    init(&checkpoint, &log, &["0:Bath", "1:Gloucester"]);
    let build = ctl(&[
        "sim",
        "build-track",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--from",
        "0",
        "--to",
        "1",
        "--length",
        "120",
    ]);
    assert!(build.status.success());
    fs::copy(&checkpoint, &after_build).expect("copy checkpoint");
    let buy = ctl(&[
        "sim",
        "buy-train",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--locomotive",
        "PlanetL",
    ]);
    assert!(buy.status.success());

    let original = fs::read_to_string(&log).expect("log");
    let tampered = original.replacen("\"build-track\"", "\"buy-train\"", 1);
    assert_ne!(tampered, original);
    fs::write(&log, &tampered).expect("tamper log");
    let size = fs::metadata(&log).expect("log metadata").len();

    let assign = ctl(&[
        "sim",
        "assign-schedule",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
    ]);
    assert!(
        !assign.status.success(),
        "a log/checkpoint mismatch must reject a new order"
    );
    assert!(
        String::from_utf8_lossy(&assign.stderr).contains("does not match the checkpoint"),
        "unexpected error: {}",
        String::from_utf8_lossy(&assign.stderr)
    );
    assert_eq!(
        fs::metadata(&log).expect("log metadata").len(),
        size,
        "a rejected order must not append to the log"
    );

    let replay = ctl(&[
        "sim",
        "replay",
        "--checkpoint",
        &path_string(&after_build),
        "--log",
        &path_string(&log),
        "--json",
    ]);
    assert!(
        !replay.status.success(),
        "a changed log prefix must fail replay"
    );
    assert!(
        String::from_utf8_lossy(&replay.stderr).contains("prefix does not match"),
        "unexpected error: {}",
        String::from_utf8_lossy(&replay.stderr)
    );

    fs::remove_dir_all(&root).ok();
}

#[test]
fn stations_come_from_the_populated_fixture_when_supplied() {
    let Some(path) = std::env::var_os("RT3_TRACK_SAVE").map(PathBuf::from) else {
        eprintln!("RT3_TRACK_SAVE unset; skipping fixture-backed order check");
        return;
    };
    let path = if path.is_absolute() {
        path
    } else {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let candidate = repo.join(&path);
        if candidate.exists() {
            candidate
        } else {
            path
        }
    };

    let root = dir("fixture");
    let checkpoint = root.join("checkpoint.json");
    let log = root.join("orders.jsonl");
    let output = ctl(&[
        "sim",
        "init",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--stations-from",
        &path_string(&path),
        "--station",
        "0:Bath",
        "--station",
        "1:Gloucester",
        "--station",
        "2:Tower",
    ]);
    assert!(
        output.status.success(),
        "fixture init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout_json(&output)["stations"], 3);

    let build = ctl(&[
        "sim",
        "build-track",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--from",
        "0",
        "--to",
        "1",
        "--length",
        "2054.776",
    ]);
    assert!(
        build.status.success(),
        "fixture build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let replay = ctl(&[
        "sim",
        "replay",
        "--checkpoint",
        &path_string(&checkpoint),
        "--log",
        &path_string(&log),
        "--json",
    ]);
    assert!(
        replay.status.success(),
        "fixture replay failed: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let value = stdout_json(&replay);
    assert_eq!(value["state"]["stations"][0][1], "Bath");
    assert_eq!(value["state"]["stations"][2][1], "Tower");
    assert_eq!(value["state"]["track"]["length"], 2054.776);

    fs::remove_dir_all(&root).ok();
}
