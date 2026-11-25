//! Install-backed check for claims C3/C4: `rt3ctl sim run` completes a decade
//! on two shipped maps, emits one JSON line per month with an explicit stop
//! reason, replays byte-identically for the same seed and diverges for a
//! different seed.

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

fn parse_name(input: &str) -> (String, &str) {
    let mut out = String::new();
    let mut escaped = false;
    for (index, c) in input.char_indices().skip(1) {
        if escaped {
            out.push(match c {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return (out, &input[index + 1..]);
        } else {
            out.push(c);
        }
    }
    panic!("unterminated cargo name in {input:?}");
}

fn parse_number(input: &str) -> (f64, &str) {
    let end = input
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')))
        .unwrap_or(input.len());
    let value = input[..end]
        .parse()
        .unwrap_or_else(|_| panic!("bad number in {input:?}"));
    (value, &input[end..])
}

type CargoEntry = (String, f64, f64);
type MonthDump = (u32, Vec<CargoEntry>);

/// Strictly parses one monthly dump line: shape, month, then every cargo
/// entry's name, stock and price. Malformed or partial JSON must fail.
fn parse_dump_line(line: &str) -> MonthDump {
    let rest = line
        .strip_prefix("{\"month\":")
        .unwrap_or_else(|| panic!("no month prefix: {line}"));
    let (month, rest) = rest
        .split_once(",\"cargo\":[")
        .unwrap_or_else(|| panic!("no cargo array: {line}"));
    let month: u32 = month
        .parse()
        .unwrap_or_else(|_| panic!("bad month: {line}"));
    let body = rest
        .strip_suffix("]}")
        .unwrap_or_else(|| panic!("no line suffix: {line}"));
    let mut entries = Vec::new();
    let mut rest = body;
    while !rest.is_empty() {
        let after = rest
            .strip_prefix("{\"name\":")
            .unwrap_or_else(|| panic!("no name key: {line}"));
        let (name, after) = parse_name(after);
        let after = after
            .strip_prefix(",\"stock\":")
            .unwrap_or_else(|| panic!("no stock key: {line}"));
        let (stock, after) = parse_number(after);
        let after = after
            .strip_prefix(",\"price\":")
            .unwrap_or_else(|| panic!("no price key: {line}"));
        let (price, after) = parse_number(after);
        let after = after
            .strip_prefix('}')
            .unwrap_or_else(|| panic!("no entry close: {line}"));
        entries.push((name, stock, price));
        if after.is_empty() {
            rest = after;
        } else {
            rest = after
                .strip_prefix(',')
                .unwrap_or_else(|| panic!("no entry separator: {line}"));
        }
    }
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
            assert_eq!(*month, index as u32 + 1, "{map}: month sequence");
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
    for line in [
        r#"{"month":1,"cargo":[not-json-seed-42]}"#,
        r#"{"month":1,"cargo":[{"name":"Coal","stock":1}]}"#,
        r#"{"month":x,"cargo":[]}"#,
        r#"{"month":1,"cargo":[{"name":"Coal","stock":1,"price":2}]} trailing"#,
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
