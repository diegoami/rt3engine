//! Metadata-only inventory of an original installation.
//!
//! Walks the installation root and aggregates file counts and byte sizes per
//! top-level entry and per file extension. It never reads file contents and
//! never emits timestamps, so the same tree always produces the same JSON.
//! Only aggregate metadata is committed as evidence (`docs/evidence/`), never
//! original bytes or full file listings.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Aggregated counts for one bucket (top-level entry or extension).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Bucket {
    pub files: u64,
    pub bytes: u64,
}

impl Bucket {
    fn add_file(&mut self, bytes: u64) {
        self.files += 1;
        self.bytes += bytes;
    }
}

/// The full inventory report.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub root_label: String,
    pub total: Bucket,
    /// Recursive aggregates per top-level entry, sorted by name.
    pub top_level: BTreeMap<String, Bucket>,
    /// Aggregates per lowercase file extension (`""` for none), sorted.
    pub extensions: BTreeMap<String, Bucket>,
}

impl Report {
    /// Renders deterministic JSON with two-space indentation.
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(&format!("  \"root\": {},\n", json_string(&self.root_label)));
        out.push_str("  \"total\": ");
        bucket_json(&mut out, &self.total, 2);
        out.push_str(",\n");
        out.push_str("  \"top_level\": {\n");
        write_map(&mut out, &self.top_level, 4);
        out.push_str("  },\n");
        out.push_str("  \"extensions\": {\n");
        write_map(&mut out, &self.extensions, 4);
        out.push_str("  }\n}\n");
        out
    }

    /// Renders the default human-readable table.
    pub fn print(&self, out: &mut impl Write) {
        let _ = writeln!(out, "RT3 installation inventory: {}", self.root_label);
        let _ = writeln!(
            out,
            "total: {} files, {} bytes",
            self.total.files, self.total.bytes
        );
        let _ = writeln!(out, "\nper top-level entry:");
        for (name, bucket) in &self.top_level {
            let _ = writeln!(
                out,
                "  {name:<24} {:>7} files {:>13} bytes",
                bucket.files, bucket.bytes
            );
        }
        let _ = writeln!(out, "\nper extension:");
        let mut extensions: Vec<_> = self.extensions.iter().collect();
        extensions.sort_by_key(|(_, bucket)| std::cmp::Reverse(bucket.bytes));
        for (name, bucket) in extensions {
            let label = if name.is_empty() { "(none)" } else { name };
            let _ = writeln!(
                out,
                "  .{label:<24} {:>7} files {:>13} bytes",
                bucket.files, bucket.bytes
            );
        }
    }
}

fn write_map(out: &mut String, map: &BTreeMap<String, Bucket>, indent: usize) {
    let pad = " ".repeat(indent);
    for (position, (name, bucket)) in map.iter().enumerate() {
        let comma = if position + 1 == map.len() { "" } else { "," };
        out.push_str(&format!("{pad}{}", json_string(name)));
        out.push_str(": ");
        bucket_json(out, bucket, indent);
        out.push_str(comma);
        out.push('\n');
    }
}

fn bucket_json(out: &mut String, bucket: &Bucket, indent: usize) {
    let pad = " ".repeat(indent + 2);
    out.push_str("{\n");
    out.push_str(&format!("{pad}\"files\": {},\n", bucket.files));
    out.push_str(&format!("{pad}\"bytes\": {}\n", bucket.bytes));
    out.push_str(&format!("{}}}", " ".repeat(indent)));
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
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

fn extension_of(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// Walks `root` and builds the aggregate report. Fails if `root` is not a
/// readable directory.
pub fn inventory(root: &Path) -> Result<Report, String> {
    if !root.is_dir() {
        return Err(format!("not a directory: {}", root.display()));
    }
    let mut report = Report {
        root_label: root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.display().to_string()),
        ..Report::default()
    };
    let mut entries: Vec<PathBuf> = fs::read_dir(root)
        .map_err(|e| format!("read_dir {}: {e}", root.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for entry in &entries {
        let name = entry
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut bucket = Bucket::default();
        if entry.is_dir() {
            walk_dir(entry, &mut |bytes| {
                bucket.add_file(bytes);
                report.total.add_file(bytes);
            })?;
        } else {
            let bytes = file_size(entry)?;
            bucket.add_file(bytes);
            report.total.add_file(bytes);
        }
        report.top_level.insert(name, bucket);
    }
    // Extension aggregates need a second pass with per-file extension context.
    walk_paths(root, &mut |path, bytes| {
        let bucket = report.extensions.entry(extension_of(path)).or_default();
        bucket.add_file(bytes);
    })
    .map_err(|e| format!("extension pass over {}: {e}", root.display()))?;
    Ok(report)
}

/// Recursively walks `dir`, calling `sink` with each file's size.
fn walk_dir(dir: &Path, sink: &mut dyn FnMut(u64)) -> Result<(), String> {
    walk_paths(dir, &mut |path, bytes| {
        let _ = path;
        sink(bytes);
    })
}

/// Recursively walks `dir`, calling `sink` with each file path and size.
fn walk_paths(dir: &Path, sink: &mut dyn FnMut(&Path, u64)) -> Result<(), String> {
    let mut children: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    children.sort();
    for child in &children {
        if child.is_dir() {
            walk_paths(child, sink)?;
        } else {
            sink(child, file_size(child)?);
        }
    }
    Ok(())
}

fn file_size(path: &Path) -> Result<u64, String> {
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("metadata {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture_tree(base: &Path) {
        // A miniature RT3-shaped tree: names/structure only matter, sizes are
        // chosen so aggregates are easy to assert.
        fs::create_dir_all(base.join("Data/CargoTypes")).unwrap();
        fs::create_dir_all(base.join("Data/2D")).unwrap();
        fs::create_dir_all(base.join("maps")).unwrap();
        fs::write(base.join("RT3.exe"), vec![0u8; 100]).unwrap();
        fs::write(base.join("Data/CargoTypes/Grain.cty"), vec![0u8; 10]).unwrap();
        fs::write(base.join("Data/CargoTypes/Steel.cty"), vec![0u8; 20]).unwrap();
        fs::write(base.join("Data/2D/rt3_2DDS.PK4"), vec![0u8; 40]).unwrap();
        fs::write(base.join("maps/Britain.gmp"), vec![0u8; 30]).unwrap();
    }

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("rt3ctl-tests")
            .join(format!("{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn aggregates_files_bytes_and_extensions() {
        let root = temp_root("aggregate");
        fixture_tree(&root);
        let report = inventory(&root).unwrap();
        assert_eq!(report.total.files, 5);
        assert_eq!(report.total.bytes, 200);
        let data = &report.top_level["Data"];
        assert_eq!((data.files, data.bytes), (3, 70));
        let maps = &report.top_level["maps"];
        assert_eq!((maps.files, maps.bytes), (1, 30));
        let exe = &report.top_level["RT3.exe"];
        assert_eq!((exe.files, exe.bytes), (1, 100));
        assert_eq!(
            (
                report.extensions["exe"].files,
                report.extensions["exe"].bytes
            ),
            (1, 100)
        );
        assert_eq!(
            (
                report.extensions["cty"].files,
                report.extensions["cty"].bytes
            ),
            (2, 30)
        );
        assert_eq!(
            (
                report.extensions["pk4"].files,
                report.extensions["pk4"].bytes
            ),
            (1, 40)
        );
        assert_eq!(
            (
                report.extensions["gmp"].files,
                report.extensions["gmp"].bytes
            ),
            (1, 30)
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn json_is_deterministic_and_sorted() {
        let root = temp_root("json");
        fixture_tree(&root);
        let first = inventory(&root).unwrap().to_json();
        let second = inventory(&root).unwrap().to_json();
        assert_eq!(first, second);
        assert!(first.starts_with("{\n  \"root\": \""));
        // Top-level map stays name-sorted (byte order: uppercase first):
        // Data before RT3.exe before maps.
        let data = first.find("\"Data\"").unwrap();
        let exe = first.find("\"RT3.exe\"").unwrap();
        let maps = first.find("\"maps\"").unwrap();
        assert!(data < exe && exe < maps);
        assert!(first.contains("\"files\": 5"));
        assert!(first.contains("\"bytes\": 200"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_missing_root() {
        let root = temp_root("missing").join("nope");
        assert!(inventory(&root).is_err());
    }
}
