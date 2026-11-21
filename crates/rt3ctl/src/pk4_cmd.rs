//! `rt3ctl pk4 list` — parse PK4 asset archives and report them.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rt3_kit::pk4::Pk4Archive;

/// Arguments of `rt3ctl pk4 list`.
pub struct Pk4List {
    pub path: Option<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub json: bool,
}

/// Parses the arguments after `pk4 list`. Unknown arguments are ignored with
/// the usage message owned by `main`.
pub fn parse<'a, I>(args: I) -> Pk4List
where
    I: Iterator<Item = &'a String>,
{
    let mut list = Pk4List {
        path: None,
        archives: Vec::new(),
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
            "--archive" => {
                if let Some(value) = rest.next() {
                    list.archives.push(PathBuf::from(value));
                }
            }
            other => {
                eprintln!("rt3ctl: ignoring unknown pk4 list argument: {other}");
            }
        }
    }
    list
}

/// Aggregate summary for one archive (metadata only; no entry names).
struct ArchiveSummary {
    archive: String,
    version: u32,
    entries: usize,
    payload_bytes: u64,
    table_bytes: u64,
}

impl ArchiveSummary {
    fn json(&self) -> String {
        format!(
            "    {{\n      \"archive\": {},\n      \"version\": {},\n      \"entries\": {},\n      \"payload_bytes\": {},\n      \"table_bytes\": {}\n    }}",
            json_string(&self.archive),
            self.version,
            self.entries,
            self.payload_bytes,
            self.table_bytes
        )
    }
}

fn json_string(value: &str) -> String {
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

fn shipped_archives(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in ["Data/2D", "Data/3D", "Data/Sound"] {
        let dir = root.join(dir);
        if let Ok(read_dir) = fs::read_dir(&dir) {
            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                let is_pk4 = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("pk4"))
                    .unwrap_or(false);
                if is_pk4 {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

/// Runs `pk4 list`.
pub fn run(list: Pk4List) -> ExitCode {
    let listing_mode = !list.archives.is_empty();
    let archives: Vec<PathBuf> = if listing_mode {
        list.archives.clone()
    } else {
        match list
            .path
            .clone()
            .or_else(|| std::env::var_os("RT3_HOME").map(PathBuf::from))
        {
            Some(root) => shipped_archives(&root),
            None => {
                eprintln!("rt3ctl: pk4 list needs --archive, --path or RT3_HOME");
                return ExitCode::from(2);
            }
        }
    };

    let mut summaries = Vec::new();
    let mut failures = Vec::new();
    let mut opened: Vec<(String, Pk4Archive)> = Vec::new();
    for path in &archives {
        match Pk4Archive::open(path) {
            Ok(archive) => {
                let payload: u64 = archive.entries().iter().map(|e| e.size as u64).sum();
                let name = archive
                    .path()
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                summaries.push(ArchiveSummary {
                    archive: name.clone(),
                    version: archive.version(),
                    entries: archive.entries().len(),
                    payload_bytes: payload,
                    table_bytes: 8 + 74 * archive.entries().len() as u64,
                });
                opened.push((name, archive));
            }
            Err(error) => failures.push(error.to_string()),
        }
    }

    for failure in &failures {
        eprintln!("rt3ctl: {failure}");
    }
    if !failures.is_empty() {
        return ExitCode::from(1);
    }

    let total_entries: usize = summaries.iter().map(|s| s.entries).sum();
    let total_payload: u64 = summaries.iter().map(|s| s.payload_bytes).sum();
    let mut out = std::io::stdout();

    if listing_mode {
        // Human listing of the requested archives' entries.
        for (name, archive) in &opened {
            let _ = writeln!(out, "{name}: {} entries", archive.entries().len());
            for entry in archive.entries() {
                let _ = writeln!(out, "  {:<40} {:>10} bytes", entry.name, entry.size);
            }
        }
        return ExitCode::SUCCESS;
    }

    if list.json {
        let chunks: Vec<String> = summaries
            .iter()
            .map(|s| {
                let comma = ",";
                s.json() + comma
            })
            .collect();
        let mut joined = chunks.join("\n");
        // drop the trailing comma of the last element
        if let Some(stripped) = joined.strip_suffix(',') {
            joined = stripped.to_string();
        }
        print!(
            "{{\n  \"archives\": [\n{}\n  ],\n  \"total\": {{\n    \"archives\": {},\n    \"entries\": {},\n    \"payload_bytes\": {}\n  }}\n}}\n",
            joined,
            summaries.len(),
            total_entries,
            total_payload
        );
    } else {
        for summary in &summaries {
            let _ = writeln!(
                out,
                "{:<28} v{} {:>5} entries {:>12} payload bytes",
                summary.archive, summary.version, summary.entries, summary.payload_bytes
            );
        }
        let _ = writeln!(
            out,
            "total: {} archives, {} entries, {} payload bytes",
            summaries.len(),
            total_entries,
            total_payload
        );
    }
    ExitCode::SUCCESS
}
