//! `rt3ctl save inspect` — read-only original `.gmc`/`.gms` inventory.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use rt3_kit::save;

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
    let (lo, hi) = save.terrain.height_range().unwrap_or((0.0, 0.0));
    println!("{{");
    println!("  \"file\": {},", crate::map_cmd::json_escape(name));
    println!("  \"version\": 9701,");
    println!(
        "  \"source_map\": {},",
        crate::map_cmd::json_escape(&save.source_map)
    );
    println!("  \"file_bytes\": {},", save.file_bytes);
    println!(
        "  \"sha256\": {},",
        crate::map_cmd::json_escape(&save.sha256_hex())
    );
    println!(
        "  \"terrain\": {{\"grid_offset\": {}, \"rows\": {}, \"cols\": {}, \"min_height_m\": {lo:.1}, \"max_height_m\": {hi:.1}}},",
        save.terrain.offset, save.terrain.rows, save.terrain.cols
    );
    println!(
        "  \"tiles\": {{\"section_offset\": {}, \"records\": {}, \"dynamic_entries\": {}, \"end_offset\": {}}},",
        save.tiles.section_offset,
        save.tiles.records.len(),
        save.tiles.dynamic_entries(),
        save.tiles.end_offset
    );
    println!(
        "  \"cities\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}},",
        save.cities.section_offset, save.cities.capacity, save.cities.records.len(), save.cities.end_offset
    );
    println!(
        "  \"buildings\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}},",
        save.buildings.section_offset,
        save.buildings.capacity,
        save.buildings.records.len(),
        save.buildings.end_offset
    );
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
        eprintln!(
            "rt3ctl: {}: invalid company/player name field",
            command.path.display()
        );
        return ExitCode::from(1);
    };
    let founded_years = save
        .companies
        .records
        .iter()
        .map(|record| rt3_kit::companies::founded_year(record).unwrap())
        .collect::<Vec<_>>();
    println!(
        "  \"companies\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}, \"founded_years\": {:?}, \"names\": {}}},",
        save.companies.section_offset,
        save.companies.capacity,
        save.companies.records.len(),
        save.companies.end_offset,
        founded_years,
        json_strings(&company_names)
    );
    println!(
        "  \"players\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}, \"names\": {}}},",
        save.players.section_offset,
        save.players.capacity,
        save.players.records.len(),
        save.players.end_offset,
        json_strings(&player_names)
    );
    println!(
        "  \"events\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}}",
        save.events.section_offset, save.events.capacity, save.events.records.len(), save.events.end_offset
    );
    println!("}}");
    ExitCode::SUCCESS
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
