//! Install-backed check for claims C3/C4: `rt3ctl sim run` completes a decade
//! on two shipped maps, emits one JSON line per month with an explicit stop
//! reason, replays byte-identically for the same seed and diverges for a
//! different seed. Every line is parsed as JSON before its schema is checked.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rt3_kit::save::sha256;

fn home() -> Option<PathBuf> {
    std::env::var_os("RT3_HOME").map(PathBuf::from)
}

fn exe() -> &'static str {
    env!("CARGO_BIN_EXE_rt3ctl")
}

fn run(home: &Path, map: &str, seed: u64, dump: &Path) -> Output {
    Command::new(exe())
        .args(["sim", "run", map, "--months", "120", "--seed"])
        .arg(seed.to_string())
        .arg("--dump")
        .arg(dump)
        .arg("--path")
        .arg(home)
        .output()
        .expect("run rt3ctl")
}

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("rt3engine-{name}-{}", std::process::id()))
}

fn hex(bytes: &[u8]) -> String {
    sha256(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

type CargoEntry = (String, f64, f64);
type MonthDump = (u64, Vec<CargoEntry>);

/// Parses one monthly dump line as JSON, then checks its schema: an integer
/// month and a cargo array whose entries carry a string name and numeric
/// stock and price. Malformed JSON or a wrong shape must fail.
fn parse_dump_line(line: &str) -> MonthDump {
    let value: serde_json::Value = serde_json::from_str(line)
        .unwrap_or_else(|error| panic!("invalid JSON line ({error}): {line}"));
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("line is not an object: {line}"));
    let month = object
        .get("month")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| panic!("month is not an unsigned integer: {line}"));
    let cargo = object
        .get("cargo")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("cargo is not an array: {line}"));
    let entries = cargo
        .iter()
        .map(|entry| {
            let entry = entry
                .as_object()
                .unwrap_or_else(|| panic!("cargo entry is not an object: {line}"));
            let name = entry
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("cargo name is not a string: {line}"))
                .to_string();
            let stock = entry
                .get("stock")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_else(|| panic!("cargo stock is not a number: {line}"));
            let price = entry
                .get("price")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_else(|| panic!("cargo price is not a number: {line}"));
            (name, stock, price)
        })
        .collect();
    (month, entries)
}

fn parse_dump(dump: &str) -> Vec<MonthDump> {
    dump.lines().map(parse_dump_line).collect()
}

#[test]
fn decade_runs_on_two_maps_replay_and_diverge() {
    let Some(home) = home() else {
        eprintln!("RT3_HOME unset; skipping install-backed sim run check");
        return;
    };

    for map in ["British Isles", "California Island"] {
        let slug = map.replace(' ', "-").to_lowercase();
        let first = temp(&format!("{slug}-a.jsonl"));
        let second = temp(&format!("{slug}-b.jsonl"));
        let third = temp(&format!("{slug}-c.jsonl"));

        let run_a = run(&home, map, 42, &first);
        assert!(
            run_a.status.success(),
            "{map}: {}",
            String::from_utf8_lossy(&run_a.stderr)
        );
        let summary = String::from_utf8(run_a.stdout).expect("summary is UTF-8");
        assert!(
            summary.contains("\"stop_reason\":\"completed\""),
            "{map} summary: {summary}"
        );
        assert!(
            summary.contains("\"months\":120"),
            "{map} summary: {summary}"
        );

        let run_b = run(&home, map, 42, &second);
        assert!(run_b.status.success(), "{map} replay failed");
        let run_c = run(&home, map, 43, &third);
        assert!(run_c.status.success(), "{map} changed-seed run failed");

        let a = fs::read(&first).unwrap();
        let b = fs::read(&second).unwrap();
        let c = fs::read(&third).unwrap();
        assert_eq!(a, b, "{map}: same seed must replay byte-identically");
        assert_ne!(a, c, "{map}: a different seed must change the dump");

        let text = String::from_utf8(a.clone()).expect("dump is UTF-8");
        let months = parse_dump(&text);
        assert_eq!(months.len(), 120, "{map}: one line per month");
        for (index, (month, entries)) in months.iter().enumerate() {
            assert_eq!(*month, index as u64 + 1, "{map}: month sequence");
            assert_eq!(entries.len(), 41, "{map}: cargo entries in month {month}");
            let mut names = std::collections::HashSet::new();
            for (name, stock, price) in entries {
                assert!(!name.is_empty(), "{map}: empty cargo name");
                assert!(names.insert(name), "{map}: duplicate cargo {name}");
                assert!(
                    stock.is_finite() && *stock >= 0.0,
                    "{map}: bad stock {stock}"
                );
                assert!(
                    price.is_finite() && *price >= 0.0,
                    "{map}: bad price {price}"
                );
            }
            assert_eq!(names.len(), 41, "{map}: 41 distinct cargo names");
        }

        let digest = hex(&a);
        assert!(
            summary.contains(&format!("\"dump_sha256\":\"{digest}\"")),
            "{map}: summary digest must match the dump file"
        );

        for path in [first, second, third] {
            fs::remove_file(path).ok();
        }
    }
}

#[test]
fn malformed_dump_lines_are_rejected() {
    let raw_control = format!(
        "{{\"month\":1,\"cargo\":[{{\"name\":\"Coal{}\",\"stock\":1,\"price\":2.0}}]}}",
        '\u{1}'
    );
    for line in [
        // JSON syntax: trailing junk, trailing commas, invalid numbers.
        r#"{"month":1,"cargo":[]} trailing"#,
        r#"{"month":1,}"#,
        r#"{"month":1,"cargo":[{"name":"Coal","stock":1,"price":2.0},]}"#,
        r#"{"month":1,"cargo":[{"name":"Coal","stock":1.,"price":2.0}]}"#,
        // Schema: wrong types and missing fields.
        r#"{"month":"1","cargo":[]}"#,
        r#"{"month":1,"cargo":[{"name":"Coal","stock":"1","price":2.0}]}"#,
        r#"{"month":1,"cargo":[{"name":"Coal","stock":1}]}"#,
        &raw_control,
    ] {
        assert!(
            std::panic::catch_unwind(|| parse_dump_line(line)).is_err(),
            "malformed line accepted: {line}"
        );
    }
    let (month, entries) =
        parse_dump_line(r#"{"month":3,"cargo":[{"name":"Coal","stock":1.50,"price":2.0}]}"#);
    assert_eq!(month, 3);
    assert_eq!(entries, vec![("Coal".to_string(), 1.5, 2.0)]);
}
