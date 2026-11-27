//! Install-backed check for claim C5: `rt3ctl sim compare` compares the
//! kernel's British Isles start state with the controlled original save and
//! names each divergence. A second comparison against a different map
//! exercises the branches that a same-map pair cannot reach.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn home() -> Option<PathBuf> {
    std::env::var_os("RT3_HOME").map(PathBuf::from)
}

fn save() -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var_os("RT3_CONTROL_SAVE")?);
    if path.is_absolute() {
        return Some(path);
    }
    // The documented value is repository-relative; the child process runs
    // from the crate directory, so resolve it against the repository root.
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let candidate = repo.join(&path);
    Some(if candidate.exists() { candidate } else { path })
}

fn compare(name: &str, home: &Path, save: &Path, json: bool) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rt3ctl"));
    command
        .args(["sim", "compare", name, "--save"])
        .arg(save)
        .arg("--path")
        .arg(home);
    if json {
        command.arg("--json");
    }
    command.output().expect("run rt3ctl sim compare")
}

#[test]
fn compares_the_controlled_save_and_names_divergences() {
    let (Some(home), Some(save)) = (home(), save()) else {
        eprintln!("RT3_HOME or RT3_CONTROL_SAVE unset; skipping install-backed compare check");
        return;
    };

    let output = compare("British Isles", &home, &save, true);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("compare JSON");
    assert_eq!(value["map"], "British Isles");
    assert_eq!(value["source_map"], "British Isles.gmp");
    assert_eq!(value["terrain"]["map_rows"], 513);
    assert_eq!(value["terrain"]["save_rows"], 513);
    assert_eq!(value["terrain"]["map_cols"], 321);
    assert_eq!(value["terrain"]["save_cols"], 321);
    assert_eq!(value["cities"]["map"], 68);
    assert_eq!(value["cities"]["save"], 68);
    assert_eq!(value["buildings"]["map"], 0);
    assert_eq!(value["buildings"]["save"], 520);
    assert_eq!(value["companies"]["kernel"], 0);
    assert_eq!(value["companies"]["save"], 3);
    assert_eq!(value["cargo_types"], 41);

    let divergences: Vec<&str> = value["divergences"]
        .as_array()
        .expect("divergence list")
        .iter()
        .map(|item| item.as_str().expect("divergence is a string"))
        .collect();
    assert!(
        divergences
            .iter()
            .any(|item| item.contains("placed building records differ")),
        "missing building divergence: {divergences:?}"
    );
    assert!(
        divergences
            .iter()
            .any(|item| item.contains("models no companies")),
        "missing company divergence: {divergences:?}"
    );
    assert!(
        divergences
            .iter()
            .any(|item| item.contains("city and building semantics are not modelled")),
        "missing semantics note: {divergences:?}"
    );
    assert!(
        divergences
            .iter()
            .any(|item| item.contains("cargo catalogue (41 types) is shared typed data")),
        "missing cargo note: {divergences:?}"
    );

    let text = compare("British Isles", &home, &save, false);
    assert!(text.status.success(), "text compare failed");
    let text = String::from_utf8(text.stdout).expect("text is UTF-8");
    assert!(text.contains("divergences:"), "text output: {text}");

    // A cross-map pair reaches the terrain, city-count and source-map
    // branches that the same-map pair above never fires, and all seven
    // divergences are present.
    let other = compare("California Island", &home, &save, true);
    assert!(
        other.status.success(),
        "{}",
        String::from_utf8_lossy(&other.stderr)
    );
    let other: Value = serde_json::from_slice(&other.stdout).expect("cross-map compare JSON");
    assert_eq!(other["map"], "California Island");
    assert_eq!(other["source_map"], "British Isles.gmp");
    assert_eq!(other["terrain"]["map_rows"], 257);
    assert_eq!(other["terrain"]["map_cols"], 257);
    assert_eq!(other["terrain"]["save_rows"], 513);
    assert_eq!(other["terrain"]["save_cols"], 321);
    assert_eq!(other["cities"]["map"], 46);
    assert_eq!(other["cities"]["save"], 68);
    let divergences: Vec<&str> = other["divergences"]
        .as_array()
        .expect("cross-map divergence list")
        .iter()
        .map(|item| item.as_str().expect("divergence is a string"))
        .collect();
    assert_eq!(divergences.len(), 7, "{divergences:?}");
    for needle in [
        "terrain dimensions differ",
        "city record counts differ",
        "placed building records differ",
        "models no companies",
        "does not match map",
        "city and building semantics are not modelled",
        "cargo catalogue",
    ] {
        assert!(
            divergences.iter().any(|item| item.contains(needle)),
            "missing {needle}: {divergences:?}"
        );
    }
}
