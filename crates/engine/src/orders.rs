//! Declared player orders, order state and canonical state hashing
//! (milestone 6, claim C4).
//!
//! The commands are the headless player's minimal v0 set: build a declared
//! track between two station references, buy a train from declared catalogue
//! names, and assign a schedule over the built track. Building a track does
//! not recover original geometry — it lays a straight declared path between the
//! two station references, as recorded in `docs/evidence/orders-v1.md`.
//!
//! Applying an order is deterministic and side-effect free apart from the
//! state it returns; [`OrderState::state_line`] is the canonical text hashed
//! into checkpoints and replayed order logs.

use rt3_kit::rail::{TrackGraph, TrackPiece};

use crate::movement::Path;
use crate::schedule::{Consist, Schedule, Station, Stop};

/// Checkpoint and order-log version written by the CLI.
pub const ORDER_STATE_VERSION: u32 = 1;
/// Declared buildable track length bounds, in track-world units.
pub const MIN_TRACK_LENGTH: f64 = 2.0;
/// Declared maximum buildable track length.
pub const MAX_TRACK_LENGTH: f64 = 100_000.0;
/// Spacing used by the declared straight-track builder.
pub const BUILD_STEP_UNITS: f64 = 30.0;

/// A declared straight track between two station-DB record references.
#[derive(Debug, Clone, PartialEq)]
pub struct DeclaredTrack {
    pub from_slot: usize,
    pub to_slot: usize,
    /// Declared length; the built path may differ by f32 rounding.
    pub length: f64,
    pub path: Path,
}

impl DeclaredTrack {
    /// Builds the declared path: evenly spaced pieces along the x axis.
    ///
    /// Pieces are at most `BUILD_STEP_UNITS` apart, so the graph is one run and
    /// the ordered path exists. The length bounds keep the piece count sane.
    pub fn new(from_slot: usize, to_slot: usize, length: f64) -> Option<DeclaredTrack> {
        if from_slot == to_slot
            || !length.is_finite()
            || !(MIN_TRACK_LENGTH..=MAX_TRACK_LENGTH).contains(&length)
        {
            return None;
        }
        let segments = (length / BUILD_STEP_UNITS).ceil().max(1.0) as usize;
        let mut pieces = Vec::with_capacity(segments + 1);
        for index in 0..=segments {
            let x = (length * index as f64 / segments as f64) as f32;
            pieces.push(TrackPiece {
                origin: [x, 0.0, 0.0],
            });
        }
        let graph = TrackGraph::from_pieces(pieces);
        let path = Path::from_graph(&graph)?;
        Some(DeclaredTrack {
            from_slot,
            to_slot,
            length,
            path,
        })
    }
}

/// One declared headless player order.
#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    BuildTrack {
        from_slot: usize,
        to_slot: usize,
        length: f64,
    },
    BuyTrain {
        locomotive: String,
        cars: Vec<String>,
    },
    AssignSchedule {
        dwell_ticks: u64,
    },
}

impl Order {
    /// The order name used in the log and command output.
    pub fn name(&self) -> &'static str {
        match self {
            Order::BuildTrack { .. } => "build-track",
            Order::BuyTrain { .. } => "buy-train",
            Order::AssignSchedule { .. } => "assign-schedule",
        }
    }
}

/// The declared state built by applying orders.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderState {
    pub stations: Vec<Station>,
    pub track: Option<DeclaredTrack>,
    pub consist: Option<Consist>,
    pub schedule: Option<Schedule>,
}

impl OrderState {
    /// Creates an empty state over declared station references.
    pub fn new(stations: Vec<Station>) -> OrderState {
        OrderState {
            stations,
            track: None,
            consist: None,
            schedule: None,
        }
    }

    /// Applies one order, or leaves the state untouched and returns the reason.
    pub fn apply(&mut self, order: &Order) -> Result<(), String> {
        let mut next = self.clone();
        next.apply_one(order)?;
        next.validate()?;
        *self = next;
        Ok(())
    }

    fn apply_one(&mut self, order: &Order) -> Result<(), String> {
        match order {
            Order::BuildTrack {
                from_slot,
                to_slot,
                length,
            } => {
                if self.station(*from_slot).is_none() {
                    return Err(format!(
                        "build-track: no station record in slot {from_slot}"
                    ));
                }
                if self.station(*to_slot).is_none() {
                    return Err(format!("build-track: no station record in slot {to_slot}"));
                }
                let track = DeclaredTrack::new(*from_slot, *to_slot, *length)
                    .ok_or_else(|| "build-track: declared track rejected".to_string())?;
                self.track = Some(track);
                self.schedule = None;
            }
            Order::BuyTrain { locomotive, cars } => {
                if !clean_name(locomotive) {
                    return Err(
                        "buy-train: locomotive name is empty or has a control character".into(),
                    );
                }
                if cars.iter().any(|car| !clean_name(car)) {
                    return Err("buy-train: car name is empty or has a control character".into());
                }
                self.consist = Some(Consist {
                    locomotive: locomotive.clone(),
                    cars: cars.clone(),
                });
                self.schedule = None;
            }
            Order::AssignSchedule { dwell_ticks } => {
                let track = self
                    .track
                    .as_ref()
                    .ok_or_else(|| "assign-schedule: build a track first".to_string())?;
                let consist = self
                    .consist
                    .as_ref()
                    .ok_or_else(|| "assign-schedule: buy a train first".to_string())?;
                let from = self
                    .station(track.from_slot)
                    .ok_or_else(|| "assign-schedule: missing origin station".to_string())?;
                let to = self
                    .station(track.to_slot)
                    .ok_or_else(|| "assign-schedule: missing destination station".to_string())?;
                let schedule = Schedule::new(
                    vec![
                        Stop {
                            station: from.clone(),
                            distance: 0.0,
                            dwell_ticks: *dwell_ticks,
                        },
                        Stop {
                            station: to.clone(),
                            distance: track.path.length(),
                            dwell_ticks: 0,
                        },
                    ],
                    track.path.length(),
                )
                .ok_or_else(|| "assign-schedule: schedule rejected".to_string())?;
                crate::schedule::Train::new(consist.clone(), schedule.clone(), track.path.clone())
                    .ok_or_else(|| "assign-schedule: run rejected".to_string())?;
                self.schedule = Some(schedule);
            }
        }
        Ok(())
    }

    /// Checks the state invariants a checkpoint must satisfy: non-empty
    /// station labels, unique station slots, a consist with non-empty names, a
    /// track whose endpoints are existing distinct station slots, and a
    /// schedule that references existing slots and matches the track
    /// endpoints.
    pub fn validate(&self) -> Result<(), String> {
        if self.stations.is_empty() {
            return Err("state has no station references".into());
        }
        let mut slots = Vec::with_capacity(self.stations.len());
        for station in &self.stations {
            if !clean_name(&station.label) {
                return Err("station label is empty or has a control character".into());
            }
            if slots.contains(&station.slot) {
                return Err(format!("duplicate station slot {}", station.slot));
            }
            slots.push(station.slot);
        }
        if let Some(track) = &self.track {
            if !slots.contains(&track.from_slot) {
                return Err(format!(
                    "track origin references missing station slot {}",
                    track.from_slot
                ));
            }
            if !slots.contains(&track.to_slot) {
                return Err(format!(
                    "track destination references missing station slot {}",
                    track.to_slot
                ));
            }
            if track.from_slot == track.to_slot {
                return Err("track endpoints are the same station slot".into());
            }
            if !track.path.length().is_finite() || track.path.length() <= 0.0 {
                return Err("track path has no length".into());
            }
        }
        if let Some(consist) = &self.consist {
            if !clean_name(&consist.locomotive) {
                return Err("locomotive name is empty or has a control character".into());
            }
            if consist.cars.iter().any(|car| !clean_name(car)) {
                return Err("car name is empty or has a control character".into());
            }
        }
        if let Some(schedule) = &self.schedule {
            let track = self
                .track
                .as_ref()
                .ok_or_else(|| "schedule exists without a track".to_string())?;
            if self.consist.is_none() {
                return Err("schedule exists without a consist".into());
            }
            for stop in &schedule.stops {
                if !slots.contains(&stop.station.slot) {
                    return Err(format!(
                        "schedule references missing station slot {}",
                        stop.station.slot
                    ));
                }
            }
            let first = schedule
                .stops
                .first()
                .ok_or_else(|| "schedule has no stops".to_string())?;
            let last = schedule
                .stops
                .last()
                .ok_or_else(|| "schedule has no stops".to_string())?;
            if first.station.slot != track.from_slot || last.station.slot != track.to_slot {
                return Err("schedule endpoints do not match the track".into());
            }
        }
        Ok(())
    }

    fn station(&self, slot: usize) -> Option<&Station> {
        self.stations.iter().find(|station| station.slot == slot)
    }

    /// The canonical text hashed into checkpoints and order logs.
    pub fn state_line(&self) -> String {
        let mut out = format!("version {ORDER_STATE_VERSION}\n");
        out.push_str(&format!("stations {}\n", self.stations.len()));
        for station in &self.stations {
            out.push_str(&format!(
                "station {} {}\n",
                station.slot,
                escape(&station.label)
            ));
        }
        match &self.track {
            Some(track) => out.push_str(&format!(
                "track {} {} {:.6}\n",
                track.from_slot,
                track.to_slot,
                track.path.length()
            )),
            None => out.push_str("track none\n"),
        }
        match &self.consist {
            Some(consist) => {
                let cars: Vec<String> = consist.cars.iter().map(|car| escape(car)).collect();
                if cars.is_empty() {
                    out.push_str(&format!("consist {} 0\n", escape(&consist.locomotive)));
                } else {
                    out.push_str(&format!(
                        "consist {} {} {}\n",
                        escape(&consist.locomotive),
                        cars.len(),
                        cars.join("|")
                    ));
                }
            }
            None => out.push_str("consist none\n"),
        }
        match &self.schedule {
            Some(schedule) => {
                out.push_str(&format!("schedule {}\n", schedule.stops.len()));
                for (index, stop) in schedule.stops.iter().enumerate() {
                    out.push_str(&format!(
                        "stop {} {} {} {:.6} {}\n",
                        index,
                        stop.station.slot,
                        escape(&stop.station.label),
                        stop.distance,
                        stop.dwell_ticks
                    ));
                }
            }
            None => out.push_str("schedule none\n"),
        }
        out
    }

    /// SHA-256 of [`OrderState::state_line`].
    pub fn state_digest(&self) -> [u8; 32] {
        rt3_kit::save::sha256(self.state_line().as_bytes())
    }
}

fn clean_name(name: &str) -> bool {
    !name.is_empty() && !name.chars().any(char::is_control)
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '|' => out.push_str("\\|"),
            ' ' => out.push_str("\\s"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
