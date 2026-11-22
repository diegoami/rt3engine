//! `rt3ctl map list` — inventory the shipped `.gmp` scenario maps.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use rt3_kit::map::GmpMap;

/// Arguments of `rt3ctl map list`.
pub struct MapList {
    pub path: Option<PathBuf>,
    pub json: bool,
}

/// Parses the arguments after `map list`.
pub fn parse<'a, I>(args: I) -> MapList
where
    I: Iterator<Item = &'a String>,
{
    let mut list = MapList {
        path: None,
        json: false,
    };
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--json" => list.json = true,
            "--path" => {
                if let Some(value) = rest.next() {
                    list.path = Some(PathBuf::from(value));
                }
            }
            other => eprintln!("rt3ctl: ignoring unknown map list argument: {other}"),
        }
    }
    list
}

struct Row {
    name: String,
    version: u32,
    status: &'static str,
    text_len: usize,
    tail_len: u64,
    file_len: u64,
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Runs `map list`.
pub fn run(list: MapList) -> ExitCode {
    let Some(root) = list
        .path
        .clone()
        .or_else(|| std::env::var_os("RT3_HOME").map(PathBuf::from))
    else {
        eprintln!("rt3ctl: map list needs --path or RT3_HOME");
        return ExitCode::from(2);
    };
    if !root.is_dir() {
        eprintln!(
            "rt3ctl: installation root is not a directory: {}",
            root.display()
        );
        return ExitCode::from(2);
    }

    let mut paths = Vec::new();
    for dir in ["maps", "Data/Tutorial"] {
        let dir = root.join(dir);
        if let Ok(rd) = fs::read_dir(&dir) {
            for entry in rd.filter_map(|e| e.ok()) {
                let p = entry.path();
                let is_gmp = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("gmp"))
                    .unwrap_or(false);
                if is_gmp {
                    paths.push(p);
                }
            }
        }
    }
    paths.sort();

    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for path in &paths {
        let name = path
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match fs::read(path) {
            Ok(bytes) => match GmpMap::peek_version(&bytes) {
                Some(v @ 9609..=9610) => rows.push(Row {
                    name,
                    version: v,
                    status: "legacy",
                    text_len: 0,
                    tail_len: 0,
                    file_len: bytes.len() as u64,
                }),
                Some(9701) => match GmpMap::from_bytes(path, &bytes) {
                    Ok(map) => rows.push(Row {
                        name,
                        version: map.version,
                        status: "current",
                        text_len: map.text_len,
                        tail_len: map.tail_len,
                        file_len: map.file_len,
                    }),
                    Err(e) => failures.push(e.to_string()),
                },
                other => failures.push(format!("{}: unexpected version {other:?}", path.display())),
            },
            Err(e) => failures.push(format!("{}: {e}", path.display())),
        }
    }
    if !failures.is_empty() {
        for f in &failures {
            eprintln!("rt3ctl: {f}");
        }
        return ExitCode::from(1);
    }

    let current = rows.iter().filter(|r| r.status == "current").count();
    let legacy = rows.len() - current;
    if list.json {
        println!("{{");
        println!("  \"maps\": [");
        for (i, r) in rows.iter().enumerate() {
            let comma = if i + 1 < rows.len() { "," } else { "" };
            println!(
                "    {{\"map\": {}, \"version\": {}, \"status\": {}, \"text_len\": {}, \"tail_bytes\": {}, \"file_bytes\": {}}}{}",
                json_escape(&r.name),
                r.version,
                json_escape(r.status),
                r.text_len,
                r.tail_len,
                r.file_len,
                comma
            );
        }
        println!("  ],");
        println!(
            "  \"total\": {{\"maps\": {}, \"current\": {}, \"legacy\": {}}}",
            rows.len(),
            current,
            legacy
        );
        println!("}}");
    } else {
        for r in &rows {
            println!(
                "{:<28} v{} {:>7} text {:>10} tail {:>10} bytes",
                r.name, r.version, r.status, r.text_len, r.tail_len
            );
        }
        println!(
            "total: {} maps ({} current, {} legacy)",
            rows.len(),
            current,
            legacy
        );
    }
    ExitCode::SUCCESS
}
