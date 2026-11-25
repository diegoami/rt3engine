//! `rt3ctl save inspect` — read-only original `.gmc`/`.gms` inventory.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use rt3_kit::save::{self, SaveGame};

pub struct SaveInspect {
    pub path: PathBuf,
    pub json: bool,
}

pub fn run(command: SaveInspect) -> ExitCode {
    let bytes = match fs::read(&command.path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("rt3ctl: cannot read {}: {e}", command.path.display());
            return ExitCode::from(1);
        }
    };
    let Some(save) = save::from_bytes(&bytes) else {
        eprintln!(
            "rt3ctl: {}: not a supported version-9701 save with all proven sections",
            command.path.display()
        );
        return ExitCode::from(1);
    };
    if !command.json {
        println!(
            "{}: {} bytes, source {}, terrain {}x{}, tiles {}, cities {}, buildings {}, companies {}, players {}, events {}",
            command.path.display(),
            save.file_bytes,
            save.source_map,
            save.terrain.cols,
            save.terrain.rows,
            save.tiles.records.len(),
            save.cities.records.len(),
            save.buildings.records.len(),
            save.companies.records.len(),
            save.players.records.len(),
            save.events.records.len()
        );
        return ExitCode::SUCCESS;
    }
    let name = command
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("save.gmc");
    match render_json(&save, name) {
        Ok(json) => {
            print!("{json}");
            ExitCode::SUCCESS
        }
        Err(detail) => {
            eprintln!("rt3ctl: {}: {detail}", command.path.display());
            ExitCode::from(1)
        }
    }
}

fn render_json(save: &SaveGame, name: &str) -> Result<String, &'static str> {
    let (Some(company_names), Some(player_names)) = (
        save.companies
            .records
            .iter()
            .map(rt3_kit::companies::name)
            .collect::<Option<Vec<_>>>(),
        save.players
            .records
            .iter()
            .map(rt3_kit::players::name)
            .collect::<Option<Vec<_>>>(),
    ) else {
        return Err("invalid company/player name field");
    };
    let founded_years = save
        .companies
        .records
        .iter()
        .map(|record| rt3_kit::companies::founded_year(record).unwrap())
        .collect::<Vec<_>>();
    let (lo, hi) = save.terrain.height_range().unwrap_or((0.0, 0.0));
    let mut out = String::new();
    writeln!(out, "{{").unwrap();
    writeln!(out, "  \"file\": {},", crate::map_cmd::json_escape(name)).unwrap();
    writeln!(out, "  \"version\": 9701,").unwrap();
    writeln!(
        out,
        "  \"source_map\": {},",
        crate::map_cmd::json_escape(&save.source_map)
    )
    .unwrap();
    writeln!(out, "  \"file_bytes\": {},", save.file_bytes).unwrap();
    writeln!(
        out,
        "  \"sha256\": {},",
        crate::map_cmd::json_escape(&save.sha256_hex())
    )
    .unwrap();
    writeln!(
        out,
        "  \"terrain\": {{\"grid_offset\": {}, \"rows\": {}, \"cols\": {}, \"min_height_m\": {lo:.1}, \"max_height_m\": {hi:.1}}},",
        save.terrain.offset, save.terrain.rows, save.terrain.cols
    )
    .unwrap();
    writeln!(
        out,
        "  \"tiles\": {{\"section_offset\": {}, \"records\": {}, \"dynamic_entries\": {}, \"end_offset\": {}}},",
        save.tiles.section_offset,
        save.tiles.records.len(),
        save.tiles.dynamic_entries(),
        save.tiles.end_offset
    )
    .unwrap();
    writeln!(
        out,
        "  \"cities\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}},",
        save.cities.section_offset, save.cities.capacity, save.cities.records.len(), save.cities.end_offset
    )
    .unwrap();
    writeln!(
        out,
        "  \"buildings\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}},",
        save.buildings.section_offset,
        save.buildings.capacity,
        save.buildings.records.len(),
        save.buildings.end_offset
    )
    .unwrap();
    writeln!(
        out,
        "  \"companies\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}, \"founded_years\": {:?}, \"names\": {}}},",
        save.companies.section_offset,
        save.companies.capacity,
        save.companies.records.len(),
        save.companies.end_offset,
        founded_years,
        json_strings(&company_names)
    )
    .unwrap();
    writeln!(
        out,
        "  \"players\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}, \"names\": {}}},",
        save.players.section_offset,
        save.players.capacity,
        save.players.records.len(),
        save.players.end_offset,
        json_strings(&player_names)
    )
    .unwrap();
    writeln!(
        out,
        "  \"events\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}}",
        save.events.section_offset, save.events.capacity, save.events.records.len(), save.events.end_offset
    )
    .unwrap();
    writeln!(out, "}}").unwrap();
    Ok(out)
}

fn json_strings(values: &[&str]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| crate::map_cmd::json_escape(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_json_includes_actor_names() {
        let Some(path) = crate::fixture_path("RT3_CONTROL_SAVE") else {
            eprintln!("RT3_CONTROL_SAVE unset; skipping JSON actor check");
            return;
        };
        let bytes = fs::read(path).unwrap();
        let save = save::from_bytes(&bytes).unwrap();
        let json = render_json(&save, "control.gms").unwrap();
        assert!(json.contains("Great Republic"));
        assert!(json.contains("George Hudson"));
        assert!(json.ends_with("}\n"));
    }

    #[test]
    fn invalid_actor_name_is_an_error_not_json() {
        let Some(path) = crate::fixture_path("RT3_CONTROL_SAVE") else {
            eprintln!("RT3_CONTROL_SAVE unset; skipping invalid-name JSON check");
            return;
        };
        let bytes = fs::read(path).unwrap();
        let mut save = save::from_bytes(&bytes).unwrap();
        save.companies.records[2].data[4] = 0xff;
        assert_eq!(
            render_json(&save, "control.gms"),
            Err("invalid company/player name field")
        );
    }
}
