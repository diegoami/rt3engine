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

fn months_in(dump: &str) -> Vec<u32> {
    dump.lines()
        .map(|line| {
            assert!(
                line.starts_with("{\"month\":")
                    && line.contains("\"cargo\":[")
                    && line.ends_with("]}"),
                "unexpected dump line: {line}"
            );
            let rest = &line["{\"month\":".len()..];
            let end = rest.find(',').expect("month terminator");
            rest[..end].parse().expect("month number")
        })
        .collect()
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
        let months = months_in(&text);
        assert_eq!(months.len(), 120, "{map}: one line per month");
        assert!(months.iter().copied().eq(1..=120), "{map}: month sequence");

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
