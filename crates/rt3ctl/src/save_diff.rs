//! `rt3ctl save diff` — structural comparison of two original saves.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rt3_kit::fixed::FixedBlobRecord;
use rt3_kit::save::{self, SaveGame};

const MAX_DIFFERING_OFFSETS: usize = 16;

pub struct SaveDiff {
    pub left: PathBuf,
    pub right: PathBuf,
    pub json: bool,
}

pub fn run(command: SaveDiff) -> ExitCode {
    let left = match read_save(&command.left) {
        Ok(save) => save,
        Err(message) => {
            eprintln!("rt3ctl: {message}");
            return ExitCode::from(1);
        }
    };
    let right = match read_save(&command.right) {
        Ok(save) => save,
        Err(message) => {
            eprintln!("rt3ctl: {message}");
            return ExitCode::from(1);
        }
    };
    let left_name = file_name(&command.left);
    let right_name = file_name(&command.right);
    if let Err(detail) = validate_actor_counts(&left, &right) {
        eprintln!("rt3ctl: {detail}");
        return ExitCode::from(1);
    }
    if !command.json {
        let companies = changed_records(&left.companies.records, &right.companies.records);
        let players = changed_records(&left.players.records, &right.players.records);
        println!(
            "{} -> {}: file_bytes {:+}, companies {}/{} changed, players {}/{} changed",
            left_name,
            right_name,
            delta(right.file_bytes, left.file_bytes),
            companies,
            left.companies.records.len(),
            players,
            left.players.records.len()
        );
        return ExitCode::SUCCESS;
    }
    match render_diff_json(&left, left_name, &right, right_name) {
        Ok(json) => {
            print!("{json}");
            ExitCode::SUCCESS
        }
        Err(detail) => {
            eprintln!("rt3ctl: {detail}");
            ExitCode::from(1)
        }
    }
}

fn read_save(path: &Path) -> Result<SaveGame, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    save::from_bytes(&bytes).ok_or_else(|| {
        format!(
            "{}: not a supported version-9701 save with all proven sections",
            path.display()
        )
    })
}

fn file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("save.gmc")
}

fn delta(right: usize, left: usize) -> i64 {
    right as i64 - left as i64
}

fn validate_actor_counts(left: &SaveGame, right: &SaveGame) -> Result<(), &'static str> {
    if left.companies.records.len() != right.companies.records.len()
        || left.players.records.len() != right.players.records.len()
    {
        return Err("actor record counts differ");
    }
    Ok(())
}

fn changed_records(left: &[FixedBlobRecord], right: &[FixedBlobRecord]) -> usize {
    left.iter()
        .zip(right)
        .filter(|(a, b)| a.data != b.data)
        .count()
}

fn render_diff_json(
    left: &SaveGame,
    left_name: &str,
    right: &SaveGame,
    right_name: &str,
) -> Result<String, &'static str> {
    validate_actor_counts(left, right)?;
    let company_changes = actor_changes(
        &left.companies.records,
        &right.companies.records,
        rt3_kit::companies::name,
    )?;
    let player_changes = actor_changes(
        &left.players.records,
        &right.players.records,
        rt3_kit::players::name,
    )?;
    let mut out = String::new();
    writeln!(out, "{{").unwrap();
    writeln!(
        out,
        "  \"left\": {{\"file\": {}, \"sha256\": {}}},",
        crate::map_cmd::json_escape(left_name),
        crate::map_cmd::json_escape(&left.sha256_hex())
    )
    .unwrap();
    writeln!(
        out,
        "  \"right\": {{\"file\": {}, \"sha256\": {}}},",
        crate::map_cmd::json_escape(right_name),
        crate::map_cmd::json_escape(&right.sha256_hex())
    )
    .unwrap();
    writeln!(
        out,
        "  \"file_bytes_delta\": {},",
        delta(right.file_bytes, left.file_bytes)
    )
    .unwrap();
    writeln!(
        out,
        "  \"source_map\": {{\"left\": {}, \"right\": {}, \"equal\": {}}},",
        crate::map_cmd::json_escape(&left.source_map),
        crate::map_cmd::json_escape(&right.source_map),
        left.source_map == right.source_map
    )
    .unwrap();
    writeln!(out, "  \"sections\": {{").unwrap();
    writeln!(
        out,
        "    \"terrain\": {{\"offset_delta\": {}, \"rows_delta\": {}, \"cols_delta\": {}}},",
        delta(right.terrain.offset, left.terrain.offset),
        right.terrain.rows as i64 - left.terrain.rows as i64,
        right.terrain.cols as i64 - left.terrain.cols as i64
    )
    .unwrap();
    section(&mut out, "tiles", &left.tiles, &right.tiles, false);
    section(&mut out, "cities", &left.cities, &right.cities, false);
    section(
        &mut out,
        "buildings",
        &left.buildings,
        &right.buildings,
        false,
    );
    section(
        &mut out,
        "companies",
        &left.companies,
        &right.companies,
        false,
    );
    section(&mut out, "players", &left.players, &right.players, false);
    section(&mut out, "events", &left.events, &right.events, true);
    writeln!(out, "  }},").unwrap();
    writeln!(out, "  \"company_changes\": {company_changes},").unwrap();
    writeln!(out, "  \"player_changes\": {player_changes}").unwrap();
    writeln!(out, "}}").unwrap();
    Ok(out)
}

trait CollectionSection {
    fn section_offset(&self) -> usize;
    fn end_offset(&self) -> usize;
    fn capacity(&self) -> usize;
    fn records(&self) -> usize;
}

macro_rules! section_impl {
    ($type:ty) => {
        impl CollectionSection for $type {
            fn section_offset(&self) -> usize {
                self.section_offset
            }
            fn end_offset(&self) -> usize {
                self.end_offset
            }
            fn capacity(&self) -> usize {
                self.capacity
            }
            fn records(&self) -> usize {
                self.records.len()
            }
        }
    };
}

impl CollectionSection for rt3_kit::tiles::TileSections {
    fn section_offset(&self) -> usize {
        self.section_offset
    }
    fn end_offset(&self) -> usize {
        self.end_offset
    }
    fn capacity(&self) -> usize {
        self.records.len()
    }
    fn records(&self) -> usize {
        self.records.len()
    }
}

section_impl!(rt3_kit::cities::CityCollection);
section_impl!(rt3_kit::fixed::FixedBlobCollection);
section_impl!(rt3_kit::sparse::SparseBlobCollection);

fn section(
    out: &mut String,
    name: &str,
    left: &impl CollectionSection,
    right: &impl CollectionSection,
    last: bool,
) {
    let terminator = if last { "" } else { "," };
    writeln!(
        out,
        "    \"{name}\": {{\"offset_delta\": {}, \"end_offset_delta\": {}, \"capacity_delta\": {}, \"records_delta\": {}}}{terminator}",
        delta(right.section_offset(), left.section_offset()),
        delta(right.end_offset(), left.end_offset()),
        delta(right.capacity(), left.capacity()),
        delta(right.records(), left.records())
    )
    .unwrap();
}

fn actor_changes(
    left: &[FixedBlobRecord],
    right: &[FixedBlobRecord],
    name: fn(&FixedBlobRecord) -> Option<&str>,
) -> Result<String, &'static str> {
    if left.len() != right.len() {
        return Err("actor record counts differ");
    }
    let mut identical = 0;
    let mut different = 0;
    let mut out = String::new();
    write!(out, "{{\"records\": [").unwrap();
    for (index, (a, b)) in left.iter().zip(right).enumerate() {
        if a.data.len() != b.data.len() {
            return Err("actor record lengths differ");
        }
        let (Some(left_name), Some(right_name)) = (name(a), name(b)) else {
            return Err("invalid actor name field");
        };
        let offsets = a
            .data
            .iter()
            .zip(&b.data)
            .enumerate()
            .filter_map(|(offset, (x, y))| (x != y).then_some(offset))
            .take(MAX_DIFFERING_OFFSETS)
            .collect::<Vec<_>>();
        let byte_differences = a.data.iter().zip(&b.data).filter(|(x, y)| x != y).count();
        if byte_differences == 0 {
            identical += 1;
        } else {
            different += 1;
        }
        if index != 0 {
            out.push_str(", ");
        }
        write!(
            out,
            "{{\"index\": {index}, \"left_name\": {}, \"right_name\": {}, \"identical\": {}, \"byte_differences\": {byte_differences}, \"differing_offsets\": {:?}}}",
            crate::map_cmd::json_escape(left_name),
            crate::map_cmd::json_escape(right_name),
            byte_differences == 0,
            offsets
        )
        .unwrap();
    }
    write!(
        out,
        "], \"identical_records\": {identical}, \"different_records\": {different}}}"
    )
    .unwrap();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controlled_pair_has_expected_actor_differences() {
        let (Ok(left), Ok(right)) = (
            std::env::var("RT3_CONTROL_SAVE"),
            std::env::var("RT3_CONTROL_SAVE_2"),
        ) else {
            eprintln!("controlled save pair unset; skipping differential check");
            return;
        };
        let left = read_save(Path::new(&left)).unwrap();
        let right = read_save(Path::new(&right)).unwrap();
        let json = render_diff_json(&left, "first.gms", &right, "second.gms").unwrap();
        assert!(json.contains("\"file_bytes_delta\": 30604"));
        assert!(json.contains("\"equal\": true"));
        assert!(json.contains("\"byte_differences\": 1, \"differing_offsets\": [55]"));
        assert!(json.contains("\"byte_differences\": 3, \"differing_offsets\": [485, 486, 487]"));
        assert!(json.contains("\"byte_differences\": 2, \"differing_offsets\": [486, 487]"));
        assert!(json.contains("\"identical_records\": 2, \"different_records\": 1"));
        assert!(json.contains("\"identical_records\": 1, \"different_records\": 2"));
        assert!(!json.contains("}},"));
    }

    #[test]
    fn unequal_actor_counts_are_rejected() {
        let Ok(path) = std::env::var("RT3_CONTROL_SAVE") else {
            eprintln!("RT3_CONTROL_SAVE unset; skipping unequal-count check");
            return;
        };
        let left = read_save(Path::new(&path)).unwrap();
        let mut right = read_save(Path::new(&path)).unwrap();
        right.companies.records.pop();
        assert_eq!(
            validate_actor_counts(&left, &right),
            Err("actor record counts differ")
        );
        assert!(render_diff_json(&left, "first.gms", &right, "second.gms").is_err());
    }
}
