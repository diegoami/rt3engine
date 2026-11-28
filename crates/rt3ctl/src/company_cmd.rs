//! `rt3ctl sim company` — the scripted company and competitor v0 run
//! (milestone 6, claims C5 and C6).
//!
//! Loads the populated fixture's track and station records, assembles the
//! typed `PlanetL` consist, and runs the declared one-way schedule for the
//! player company and the fixed-policy competitor. Outputs the cash-flow
//! results and the joint state hash; the rules are in
//! `docs/evidence/rail-economy-v1.md`.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use engine::company::{CompanyRules, CompetitorPolicy, JointRun};
use engine::movement::Path;
use engine::schedule::{Consist, Schedule, Station, Stop};
use rt3_kit::rail::{self, TrackGraph};
use rt3_kit::typed::{scan_family, CarType, Locomotive};
use serde_json::json;

/// Arguments of `rt3ctl sim company`.
pub struct CompanyRun {
    pub track_save: PathBuf,
    pub path: Option<PathBuf>,
    pub trips: u64,
    pub dwell: u64,
    pub competitor_start: u64,
    pub competitor_interval: u64,
    pub json: bool,
}

/// Parses the arguments after `sim company`.
pub fn parse<'a, I>(args: I) -> Result<CompanyRun, String>
where
    I: Iterator<Item = &'a String>,
{
    let command = "sim company";
    let mut track_save = None;
    let mut path = None;
    let mut trips = 5u64;
    let mut dwell = engine::schedule::DEFAULT_DWELL_TICKS;
    let mut competitor_start = CompetitorPolicy::default().start_tick;
    let mut competitor_interval = CompetitorPolicy::default().interval_ticks;
    let mut json = false;
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--track-save" => {
                track_save = Some(PathBuf::from(take(&mut rest, command, "--track-save")?))
            }
            "--path" => path = Some(PathBuf::from(take(&mut rest, command, "--path")?)),
            "--trips" => {
                trips = take(&mut rest, command, "--trips")?
                    .parse()
                    .map_err(|_| format!("{command}: bad --trips"))?
            }
            "--dwell" => {
                dwell = take(&mut rest, command, "--dwell")?
                    .parse()
                    .map_err(|_| format!("{command}: bad --dwell"))?
            }
            "--competitor-start" => {
                competitor_start = take(&mut rest, command, "--competitor-start")?
                    .parse()
                    .map_err(|_| format!("{command}: bad --competitor-start"))?
            }
            "--competitor-interval" => {
                competitor_interval = take(&mut rest, command, "--competitor-interval")?
                    .parse()
                    .map_err(|_| format!("{command}: bad --competitor-interval"))?
            }
            "--json" => json = true,
            other if other.starts_with("--") => {
                return Err(format!("{command}: unknown option {other}"));
            }
            other => return Err(format!("{command}: unexpected argument {other}")),
        }
    }
    Ok(CompanyRun {
        track_save: track_save.ok_or_else(|| format!("{command}: --track-save is required"))?,
        path,
        trips,
        dwell,
        competitor_start,
        competitor_interval,
        json,
    })
}

fn take<'a, I>(rest: &mut I, command: &str, option: &str) -> Result<&'a str, String>
where
    I: Iterator<Item = &'a String>,
{
    rest.next()
        .map(String::as_str)
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("{command}: {option} needs a value"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Runs `rt3ctl sim company`.
pub fn run(run: CompanyRun) -> ExitCode {
    match run_company(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("rt3ctl: {message}");
            ExitCode::from(1)
        }
    }
}

fn run_company(run: CompanyRun) -> Result<(), String> {
    let Some(root) = crate::install_root(run.path.clone()) else {
        return Err("no installation root: pass --path or set RT3_HOME".into());
    };
    let bytes = fs::read(&run.track_save)
        .map_err(|error| format!("{}: {error}", run.track_save.display()))?;
    let track = rail::track(&bytes).ok_or_else(|| "no track collection in the save".to_string())?;
    let graph = TrackGraph::from_collection(&track)
        .ok_or_else(|| "the track collection has no declared graph".to_string())?;
    let path = Path::from_graph(&graph)
        .ok_or_else(|| "the declared graph is not one ordered path".to_string())?;
    let stations =
        rail::stations(&bytes).ok_or_else(|| "no station collection in the save".to_string())?;
    if stations.records.len() < 2 {
        return Err("the save has fewer than two station records".into());
    }
    let from = Station::from_record(&stations, 0, "Bath")
        .ok_or_else(|| "missing origin station record".to_string())?;
    let to = Station::from_record(&stations, 1, "Gloucester")
        .ok_or_else(|| "missing destination station record".to_string())?;
    let schedule = Schedule::new(
        vec![
            Stop {
                station: from,
                distance: 0.0,
                dwell_ticks: run.dwell,
            },
            Stop {
                station: to,
                distance: path.length(),
                dwell_ticks: 0,
            },
        ],
        path.length(),
    )
    .ok_or_else(|| "the declared schedule was rejected".to_string())?;

    let consist = catalogue_consist(&root)?;
    let rules = CompanyRules::default();
    let policy = CompetitorPolicy {
        start_tick: run.competitor_start,
        interval_ticks: run.competitor_interval,
    };
    let joint = JointRun::run(&consist, &path, &schedule, rules, run.trips, policy)
        .ok_or_else(|| "the joint company run was rejected".to_string())?;
    let digest = hex(&joint.state_digest());

    if run.json {
        println!(
            "{}",
            json!({
                "player": company_json(&joint.player),
                "competitor": company_json(&joint.competitor),
                "consist": {
                    "locomotive": &consist.locomotive,
                    "cars": &consist.cars,
                },
                "rules": {
                    "fare_per_car_trip": joint.rules.fare_per_car_trip,
                    "cost_per_tick": joint.rules.cost_per_tick,
                    "cost_per_unit": joint.rules.cost_per_unit,
                    "initial_cash": joint.rules.initial_cash,
                },
                "policy": {
                    "start_tick": joint.policy.start_tick,
                    "interval_ticks": joint.policy.interval_ticks,
                },
                "trips_per_company": joint.trips_per_company,
                "horizon_ticks": joint.horizon_ticks,
                "player_positive": joint.player.cash > 0.0,
                "state_sha256": digest,
            })
        );
    } else {
        println!(
            "consist {} ({} cars)",
            consist.locomotive,
            consist.car_count()
        );
        println!(
            "player cash {:.2} (revenue {:.2}, costs {:.2}, {} trips), competitor cash {:.2}, \
             horizon {} ticks",
            joint.player.cash,
            joint.player.revenue,
            joint.player.costs,
            joint.player.trips,
            joint.competitor.cash,
            joint.horizon_ticks
        );
        println!("state_sha256 {digest}");
    }
    Ok(())
}

fn company_json(company: &engine::company::Company) -> serde_json::Value {
    json!({
        "cash": company.cash,
        "revenue": company.revenue,
        "costs": company.costs,
        "trips": company.trips,
        "first_profitable_trip": company.first_profitable_trip,
        "positive": company.cash > 0.0,
    })
}

fn catalogue_consist(root: &std::path::Path) -> Result<Consist, String> {
    let data = root.join("Data/EngineTypes");
    let locomotives = scan_family(&data, "lco", Locomotive::parse);
    let cars = scan_family(&data, "car", CarType::parse);
    if !locomotives.failures.is_empty() || !cars.failures.is_empty() {
        return Err("the typed EngineTypes catalogue did not parse".into());
    }
    let (_, planet) = locomotives
        .records
        .iter()
        .find(|(_, record)| record.name.starts_with("Planet"))
        .ok_or_else(|| "no Planet locomotive in the typed catalogue".to_string())?;
    let typed_cars: Vec<CarType> = cars
        .records
        .iter()
        .take(4)
        .map(|(_, record)| record.clone())
        .collect();
    if typed_cars.len() != 4 {
        return Err("the typed catalogue has fewer than four car records".into());
    }
    Ok(Consist::new(planet, &typed_cars))
}
