//! `rt3ctl sim` order commands and replay (milestone 6, claim C4).
//!
//! - `sim init` creates a checkpoint over declared station references, either
//!   from `--station <slot>:<label>` arguments or from a save's station-DB
//!   records (`--stations-from`).
//! - `sim build-track`, `sim buy-train` and `sim assign-schedule` apply one
//!   declared order each, append it to the order log and advance the
//!   checkpoint.
//! - `sim replay` re-applies the log from a checkpoint and checks every
//!   recorded state hash, so a resumed run and a full run end byte-identically.
//!
//! The checkpoint and log are JSON; the hashed state text is produced by
//! `engine::orders`. The declared rules and divergences are in
//! `docs/evidence/orders-v1.md`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use engine::orders::{DeclaredTrack, Order, OrderState, ORDER_STATE_VERSION};
use engine::schedule::{Consist, Schedule, Station, Stop};
use rt3_kit::save::sha256;
use serde_json::{json, Map, Value};

/// Arguments of `rt3ctl sim init`.
pub struct Init {
    pub checkpoint: PathBuf,
    pub log: PathBuf,
    pub stations: Vec<(usize, String)>,
    pub stations_from: Option<PathBuf>,
}

/// Arguments of the three order commands.
pub struct OrderCommand {
    pub checkpoint: PathBuf,
    pub log: PathBuf,
    pub order: Order,
}

/// Arguments of `rt3ctl sim replay`.
pub struct Replay {
    pub checkpoint: PathBuf,
    pub log: PathBuf,
    pub json: bool,
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

fn path_value<'a, I>(rest: &mut I, command: &str, option: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = &'a String>,
{
    Ok(PathBuf::from(take(rest, command, option)?))
}

fn number<'a, T: std::str::FromStr, I>(
    rest: &mut I,
    command: &str,
    option: &str,
) -> Result<T, String>
where
    I: Iterator<Item = &'a String>,
{
    let value = take(rest, command, option)?;
    value
        .parse()
        .map_err(|_| format!("{command}: bad {option} {value:?}"))
}

/// Parses the arguments after `sim init`.
pub fn parse_init<'a, I>(args: I) -> Result<Init, String>
where
    I: Iterator<Item = &'a String>,
{
    let command = "sim init";
    let mut checkpoint = None;
    let mut log = None;
    let mut stations = Vec::new();
    let mut stations_from = None;
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--checkpoint" => checkpoint = Some(path_value(&mut rest, command, "--checkpoint")?),
            "--log" => log = Some(path_value(&mut rest, command, "--log")?),
            "--station" => {
                let value = take(&mut rest, command, "--station")?;
                let (slot, label) = value
                    .split_once(':')
                    .ok_or_else(|| format!("{command}: --station wants <slot>:<label>"))?;
                let slot = slot
                    .parse()
                    .map_err(|_| format!("{command}: bad station slot {slot:?}"))?;
                if label.is_empty() {
                    return Err(format!("{command}: station label is empty"));
                }
                stations.push((slot, label.to_string()));
            }
            "--stations-from" => {
                stations_from = Some(path_value(&mut rest, command, "--stations-from")?)
            }
            other if other.starts_with("--") => {
                return Err(format!("{command}: unknown option {other}"));
            }
            other => return Err(format!("{command}: unexpected argument {other}")),
        }
    }
    Ok(Init {
        checkpoint: checkpoint.ok_or_else(|| format!("{command}: --checkpoint is required"))?,
        log: log.ok_or_else(|| format!("{command}: --log is required"))?,
        stations,
        stations_from,
    })
}

/// Parses the arguments after `sim build-track`.
pub fn parse_build_track<'a, I>(args: I) -> Result<OrderCommand, String>
where
    I: Iterator<Item = &'a String>,
{
    let command = "sim build-track";
    let mut checkpoint = None;
    let mut log = None;
    let mut from_slot = None;
    let mut to_slot = None;
    let mut length = None;
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--checkpoint" => checkpoint = Some(path_value(&mut rest, command, "--checkpoint")?),
            "--log" => log = Some(path_value(&mut rest, command, "--log")?),
            "--from" => from_slot = Some(number(&mut rest, command, "--from")?),
            "--to" => to_slot = Some(number(&mut rest, command, "--to")?),
            "--length" => length = Some(number(&mut rest, command, "--length")?),
            other if other.starts_with("--") => {
                return Err(format!("{command}: unknown option {other}"));
            }
            other => return Err(format!("{command}: unexpected argument {other}")),
        }
    }
    Ok(OrderCommand {
        checkpoint: checkpoint.ok_or_else(|| format!("{command}: --checkpoint is required"))?,
        log: log.ok_or_else(|| format!("{command}: --log is required"))?,
        order: Order::BuildTrack {
            from_slot: from_slot.ok_or_else(|| format!("{command}: --from is required"))?,
            to_slot: to_slot.ok_or_else(|| format!("{command}: --to is required"))?,
            length: length.ok_or_else(|| format!("{command}: --length is required"))?,
        },
    })
}

/// Parses the arguments after `sim buy-train`.
pub fn parse_buy_train<'a, I>(args: I) -> Result<OrderCommand, String>
where
    I: Iterator<Item = &'a String>,
{
    let command = "sim buy-train";
    let mut checkpoint = None;
    let mut log = None;
    let mut locomotive = None;
    let mut cars = Vec::new();
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--checkpoint" => checkpoint = Some(path_value(&mut rest, command, "--checkpoint")?),
            "--log" => log = Some(path_value(&mut rest, command, "--log")?),
            "--locomotive" => {
                locomotive = Some(take(&mut rest, command, "--locomotive")?.to_string())
            }
            "--car" => cars.push(take(&mut rest, command, "--car")?.to_string()),
            other if other.starts_with("--") => {
                return Err(format!("{command}: unknown option {other}"));
            }
            other => return Err(format!("{command}: unexpected argument {other}")),
        }
    }
    Ok(OrderCommand {
        checkpoint: checkpoint.ok_or_else(|| format!("{command}: --checkpoint is required"))?,
        log: log.ok_or_else(|| format!("{command}: --log is required"))?,
        order: Order::BuyTrain {
            locomotive: locomotive.ok_or_else(|| format!("{command}: --locomotive is required"))?,
            cars,
        },
    })
}

/// Parses the arguments after `sim assign-schedule`.
pub fn parse_assign_schedule<'a, I>(args: I) -> Result<OrderCommand, String>
where
    I: Iterator<Item = &'a String>,
{
    let command = "sim assign-schedule";
    let mut checkpoint = None;
    let mut log = None;
    let mut dwell_ticks = engine::schedule::DEFAULT_DWELL_TICKS;
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--checkpoint" => checkpoint = Some(path_value(&mut rest, command, "--checkpoint")?),
            "--log" => log = Some(path_value(&mut rest, command, "--log")?),
            "--dwell" => dwell_ticks = number(&mut rest, command, "--dwell")?,
            other if other.starts_with("--") => {
                return Err(format!("{command}: unknown option {other}"));
            }
            other => return Err(format!("{command}: unexpected argument {other}")),
        }
    }
    Ok(OrderCommand {
        checkpoint: checkpoint.ok_or_else(|| format!("{command}: --checkpoint is required"))?,
        log: log.ok_or_else(|| format!("{command}: --log is required"))?,
        order: Order::AssignSchedule { dwell_ticks },
    })
}

/// Parses the arguments after `sim replay`.
pub fn parse_replay<'a, I>(args: I) -> Result<Replay, String>
where
    I: Iterator<Item = &'a String>,
{
    let command = "sim replay";
    let mut checkpoint = None;
    let mut log = None;
    let mut json = false;
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--checkpoint" => checkpoint = Some(path_value(&mut rest, command, "--checkpoint")?),
            "--log" => log = Some(path_value(&mut rest, command, "--log")?),
            "--json" => json = true,
            other if other.starts_with("--") => {
                return Err(format!("{command}: unknown option {other}"));
            }
            other => return Err(format!("{command}: unexpected argument {other}")),
        }
    }
    Ok(Replay {
        checkpoint: checkpoint.ok_or_else(|| format!("{command}: --checkpoint is required"))?,
        log: log.ok_or_else(|| format!("{command}: --log is required"))?,
        json,
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest_hex(bytes: &[u8]) -> String {
    hex(&sha256(bytes))
}

/// Runs `rt3ctl sim init`.
pub fn run_init(init: Init) -> ExitCode {
    match init_checkpoint(init) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("rt3ctl: {message}");
            ExitCode::from(1)
        }
    }
}

fn init_checkpoint(init: Init) -> Result<(), String> {
    let mut stations: Vec<Station> = Vec::new();
    if let Some(path) = &init.stations_from {
        let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let collection = rt3_kit::rail::stations(&bytes)
            .ok_or_else(|| format!("{}: no station collection", path.display()))?;
        for slot in 0..collection.records.len() {
            stations.push(Station {
                slot,
                label: format!("station-{slot}"),
            });
        }
    }

    let mut explicit = Vec::new();
    for (slot, label) in &init.stations {
        if explicit.contains(slot) {
            return Err(format!("init: duplicate station slot {slot}"));
        }
        explicit.push(*slot);
        if let Some(station) = stations.iter_mut().find(|station| station.slot == *slot) {
            station.label = label.clone();
        } else if init.stations_from.is_some() {
            return Err(format!("init: slot {slot} is not a station record"));
        } else {
            stations.push(Station {
                slot: *slot,
                label: label.clone(),
            });
        }
    }
    if stations.is_empty() {
        return Err("init: no station references; pass --station or --stations-from".into());
    }
    if stations
        .iter()
        .any(|station| station.label.is_empty() || station.label.chars().any(char::is_control))
    {
        return Err("init: a station label is empty or has a control character".into());
    }
    stations.sort_by_key(|station| station.slot);

    let state = OrderState::new(stations);
    state.validate().map_err(|error| format!("init: {error}"))?;
    fs::write(&init.log, b"").map_err(|error| format!("{}: {error}", init.log.display()))?;
    write_checkpoint(&init.checkpoint, 0, &[], &state)
        .map_err(|error| format!("{}: {error}", init.checkpoint.display()))?;
    println!(
        "{}",
        json!({
            "seq": 0,
            "stations": state.stations.len(),
            "state_sha256": hex(&state.state_digest()),
        })
    );
    Ok(())
}

/// Runs one of the three order commands.
pub fn run_order(command: OrderCommand) -> ExitCode {
    match apply_order(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("rt3ctl: {message}");
            ExitCode::from(1)
        }
    }
}

fn apply_order(command: OrderCommand) -> Result<(), String> {
    let (mut state, seq, log_sha, _) = load_checkpoint(&command.checkpoint)?;
    let log = match fs::read(&command.log) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && seq == 0 => Vec::new(),
        Err(error) => return Err(format!("{}: {error}", command.log.display())),
    };
    if digest_hex(&log) != log_sha {
        return Err("the order log does not match the checkpoint".into());
    }
    state.apply(&command.order)?;
    let digest = hex(&state.state_digest());
    let line = json!({
        "seq": seq + 1,
        "order": order_json(&command.order),
        "state_sha256": digest,
    });
    let mut new_log = log;
    if !new_log.is_empty() && !new_log.ends_with(b"\n") {
        new_log.push(b'\n');
    }
    new_log.extend_from_slice(serde_json::to_string(&line).unwrap().as_bytes());
    new_log.push(b'\n');
    fs::write(&command.log, &new_log)
        .map_err(|error| format!("{}: {error}", command.log.display()))?;
    write_checkpoint(&command.checkpoint, seq + 1, &new_log, &state)
        .map_err(|error| format!("{}: {error}", command.checkpoint.display()))?;
    println!(
        "{}",
        json!({
            "seq": seq + 1,
            "order": command.order.name(),
            "state_sha256": digest,
        })
    );
    Ok(())
}

/// Runs `rt3ctl sim replay`.
pub fn run_replay(replay: Replay) -> ExitCode {
    match replay_orders(&replay) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("rt3ctl: {message}");
            ExitCode::from(1)
        }
    }
}

fn replay_orders(replay: &Replay) -> Result<(), String> {
    let (mut state, mut seq, log_sha, _) = load_checkpoint(&replay.checkpoint)?;
    let bytes =
        fs::read(&replay.log).map_err(|error| format!("{}: {error}", replay.log.display()))?;
    let text = String::from_utf8(bytes).map_err(|_| "the order log is not UTF-8".to_string())?;
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < seq as usize {
        return Err(format!(
            "the order log has {} lines but the checkpoint is at seq {seq}",
            lines.len()
        ));
    }
    let prefix: String = lines[..seq as usize]
        .iter()
        .map(|line| format!("{line}\n"))
        .collect();
    if digest_hex(prefix.as_bytes()) != log_sha {
        return Err("the order log prefix does not match the checkpoint".into());
    }

    for line in &lines[seq as usize..] {
        let value: Value = serde_json::from_str(line)
            .map_err(|error| format!("bad order line ({error}): {line}"))?;
        let line_seq = value
            .get("seq")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("order line has no seq: {line}"))?;
        if line_seq != seq + 1 {
            return Err(format!(
                "order line seq {line_seq} does not follow checkpoint seq {seq}"
            ));
        }
        let order = order_from_json(
            value
                .get("order")
                .ok_or_else(|| format!("order line has no order: {line}"))?,
        )?;
        state.apply(&order)?;
        let digest = hex(&state.state_digest());
        let recorded = value
            .get("state_sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("order line has no state_sha256: {line}"))?;
        if recorded != digest {
            return Err(format!(
                "replayed state at seq {} does not match the log: {digest} != {recorded}",
                seq + 1
            ));
        }
        seq += 1;
    }

    let state_sha = hex(&state.state_digest());
    if replay.json {
        println!(
            "{}",
            json!({
                "seq": seq,
                "state_sha256": state_sha,
                "state": state_json(&state),
            })
        );
    } else {
        print!("{}", state.state_line());
        println!("state_sha256 {state_sha}");
    }
    Ok(())
}

fn write_checkpoint(path: &Path, seq: u64, log: &[u8], state: &OrderState) -> Result<(), String> {
    let value = json!({
        "version": ORDER_STATE_VERSION,
        "seq": seq,
        "log_sha256": digest_hex(log),
        "state_sha256": hex(&state.state_digest()),
        "state": state_json(state),
    });
    let text = serde_json::to_string_pretty(&value).unwrap();
    fs::write(path, format!("{text}\n")).map_err(|error| error.to_string())
}

fn load_checkpoint(path: &Path) -> Result<(OrderState, u64, String, String), String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|error| format!("bad checkpoint ({error})"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "checkpoint is not an object".to_string())?;
    let version = object
        .get("version")
        .and_then(Value::as_u64)
        .ok_or_else(|| "checkpoint has no version".to_string())?;
    if version != u64::from(ORDER_STATE_VERSION) {
        return Err(format!("unsupported checkpoint version {version}"));
    }
    let seq = object
        .get("seq")
        .and_then(Value::as_u64)
        .ok_or_else(|| "checkpoint has no seq".to_string())?;
    let log_sha = object
        .get("log_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "checkpoint has no log_sha256".to_string())?
        .to_string();
    let state_sha = object
        .get("state_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| "checkpoint has no state_sha256".to_string())?
        .to_string();
    let state = state_from_json(
        object
            .get("state")
            .ok_or_else(|| "checkpoint has no state".to_string())?,
    )?;
    if hex(&state.state_digest()) != state_sha {
        return Err("checkpoint state does not match its state_sha256".into());
    }
    Ok((state, seq, log_sha, state_sha))
}

fn state_json(state: &OrderState) -> Value {
    json!({
        "stations": state
            .stations
            .iter()
            .map(|station| json!([station.slot, station.label]))
            .collect::<Vec<_>>(),
        "track": state.track.as_ref().map(|track| {
            json!({"from": track.from_slot, "to": track.to_slot, "length": track.length})
        }),
        "consist": state.consist.as_ref().map(|consist| {
            json!({"locomotive": consist.locomotive, "cars": consist.cars})
        }),
        "schedule": state.schedule.as_ref().map(|schedule| {
            json!({
                "stops": schedule
                    .stops
                    .iter()
                    .map(|stop| {
                        json!([
                            stop.station.slot,
                            stop.station.label,
                            stop.distance,
                            stop.dwell_ticks
                        ])
                    })
                    .collect::<Vec<_>>(),
            })
        }),
    })
}

fn state_from_json(value: &Value) -> Result<OrderState, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "checkpoint state is not an object".to_string())?;
    let stations_value = object
        .get("stations")
        .and_then(Value::as_array)
        .ok_or_else(|| "checkpoint state has no stations".to_string())?;
    let mut stations = Vec::new();
    for entry in stations_value {
        let pair = entry
            .as_array()
            .ok_or_else(|| "a station entry is not an array".to_string())?;
        if pair.len() != 2 {
            return Err("a station entry does not have slot and label".into());
        }
        let slot = pair[0]
            .as_u64()
            .ok_or_else(|| "a station slot is not an integer".to_string())?;
        let label = pair[1]
            .as_str()
            .ok_or_else(|| "a station label is not a string".to_string())?;
        stations.push(Station {
            slot: usize::try_from(slot).map_err(|_| "station slot is too large")?,
            label: label.to_string(),
        });
    }
    let mut state = OrderState::new(stations);

    if let Some(track) = object.get("track").filter(|value| !value.is_null()) {
        let track = track
            .as_object()
            .ok_or_else(|| "checkpoint track is not an object".to_string())?;
        let from = usize_field(track, "from")?;
        let to = usize_field(track, "to")?;
        let length = number_field(track, "length")?;
        state.track = Some(
            DeclaredTrack::new(from, to, length)
                .ok_or_else(|| "checkpoint track is invalid".to_string())?,
        );
    }

    if let Some(consist) = object.get("consist").filter(|value| !value.is_null()) {
        let consist = consist
            .as_object()
            .ok_or_else(|| "checkpoint consist is not an object".to_string())?;
        let locomotive = consist
            .get("locomotive")
            .and_then(Value::as_str)
            .ok_or_else(|| "checkpoint consist has no locomotive".to_string())?
            .to_string();
        let cars = consist
            .get("cars")
            .and_then(Value::as_array)
            .ok_or_else(|| "checkpoint consist has no cars".to_string())?
            .iter()
            .map(|car| {
                car.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "a consist car is not a string".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        state.consist = Some(Consist { locomotive, cars });
    }

    if let Some(schedule) = object.get("schedule").filter(|value| !value.is_null()) {
        let schedule = schedule
            .as_object()
            .ok_or_else(|| "checkpoint schedule is not an object".to_string())?;
        let track = state
            .track
            .as_ref()
            .ok_or_else(|| "checkpoint schedule has no track".to_string())?;
        let stops = schedule
            .get("stops")
            .and_then(Value::as_array)
            .ok_or_else(|| "checkpoint schedule has no stops".to_string())?;
        let mut parsed = Vec::new();
        for entry in stops {
            let fields = entry
                .as_array()
                .ok_or_else(|| "a schedule stop is not an array".to_string())?;
            if fields.len() != 4 {
                return Err("a schedule stop does not have slot, label, distance and dwell".into());
            }
            let slot = fields[0]
                .as_u64()
                .ok_or_else(|| "a schedule slot is not an integer".to_string())?;
            let label = fields[1]
                .as_str()
                .ok_or_else(|| "a schedule label is not a string".to_string())?;
            let distance = fields[2]
                .as_f64()
                .ok_or_else(|| "a schedule distance is not a number".to_string())?;
            let dwell_ticks = fields[3]
                .as_u64()
                .ok_or_else(|| "a schedule dwell is not an integer".to_string())?;
            parsed.push(Stop {
                station: Station {
                    slot: usize::try_from(slot).map_err(|_| "schedule slot is too large")?,
                    label: label.to_string(),
                },
                distance,
                dwell_ticks,
            });
        }
        state.schedule = Some(
            Schedule::new(parsed, track.path.length())
                .ok_or_else(|| "checkpoint schedule is invalid".to_string())?,
        );
    }
    state.validate()?;
    Ok(state)
}

fn order_json(order: &Order) -> Value {
    match order {
        Order::BuildTrack {
            from_slot,
            to_slot,
            length,
        } => json!({
            "build-track": {"from": from_slot, "to": to_slot, "length": length}
        }),
        Order::BuyTrain { locomotive, cars } => json!({
            "buy-train": {"locomotive": locomotive, "cars": cars}
        }),
        Order::AssignSchedule { dwell_ticks } => json!({
            "assign-schedule": {"dwell_ticks": dwell_ticks}
        }),
    }
}

fn order_from_json(value: &Value) -> Result<Order, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "order is not an object".to_string())?;
    if let Some(payload) = object.get("build-track") {
        let payload = payload
            .as_object()
            .ok_or_else(|| "build-track order is not an object".to_string())?;
        return Ok(Order::BuildTrack {
            from_slot: usize_field(payload, "from")?,
            to_slot: usize_field(payload, "to")?,
            length: number_field(payload, "length")?,
        });
    }
    if let Some(payload) = object.get("buy-train") {
        let payload = payload
            .as_object()
            .ok_or_else(|| "buy-train order is not an object".to_string())?;
        let locomotive = payload
            .get("locomotive")
            .and_then(Value::as_str)
            .ok_or_else(|| "buy-train order has no locomotive".to_string())?
            .to_string();
        let cars = payload
            .get("cars")
            .and_then(Value::as_array)
            .ok_or_else(|| "buy-train order has no cars".to_string())?
            .iter()
            .map(|car| {
                car.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "a buy-train car is not a string".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Order::BuyTrain { locomotive, cars });
    }
    if let Some(payload) = object.get("assign-schedule") {
        let payload = payload
            .as_object()
            .ok_or_else(|| "assign-schedule order is not an object".to_string())?;
        return Ok(Order::AssignSchedule {
            dwell_ticks: payload
                .get("dwell_ticks")
                .and_then(Value::as_u64)
                .ok_or_else(|| "assign-schedule order has no dwell_ticks".to_string())?,
        });
    }
    Err("unknown order".into())
}

fn usize_field(object: &Map<String, Value>, name: &str) -> Result<usize, String> {
    let value = object
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing integer field {name}"))?;
    usize::try_from(value).map_err(|_| format!("field {name} is too large"))
}

fn number_field(object: &Map<String, Value>, name: &str) -> Result<f64, String> {
    object
        .get(name)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("missing numeric field {name}"))
}
