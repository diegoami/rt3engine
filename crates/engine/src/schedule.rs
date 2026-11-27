//! Declared stations, consists and schedules (milestone 6, claim C3).
//!
//! The original's station naming, train composition and timetable data are not
//! recovered, so this module composes the types that are decoded — typed
//! locomotive/car descriptors and station-DB record slots — under a **declared
//! schedule model**. Every rule and divergence is recorded in
//! `docs/evidence/schedule-v1.md`.
//!
//! A consist is built from a parsed [`Locomotive`] and parsed [`CarType`]
//! records. A station is a station-DB record slot plus a declared label; the
//! record's own name and position are still open, so the label and the path
//! distance are declared by the caller. A run stops at every scheduled stop in
//! order, dwells a declared number of ticks, and then departs under the
//! movement model in [`crate::movement`].

use rt3_kit::fixed::FixedBlobCollection;
use rt3_kit::typed::{CarType, Locomotive};

use crate::movement::{Movement, Path};

/// Declared dwell at the origin and at intermediate stops.
pub const DEFAULT_DWELL_TICKS: u64 = 5;

/// A consist assembled from typed catalogue records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consist {
    /// Locomotive descriptor name.
    pub locomotive: String,
    /// Car descriptor display names, in train order.
    pub cars: Vec<String>,
}

impl Consist {
    /// Builds a consist from a parsed locomotive and parsed car records.
    pub fn new(locomotive: &Locomotive, cars: &[CarType]) -> Consist {
        Consist {
            locomotive: locomotive.name.clone(),
            cars: cars.iter().map(|car| car.display_name.clone()).collect(),
        }
    }

    /// Number of cars in the consist.
    pub fn car_count(&self) -> usize {
        self.cars.len()
    }
}

/// A station-DB record slot with a declared label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Station {
    pub slot: usize,
    pub label: String,
}

impl Station {
    /// References an existing station-DB record by slot.
    ///
    /// The label is declared: the record's own name is not decoded.
    pub fn from_record(
        stations: &FixedBlobCollection,
        slot: usize,
        label: impl Into<String>,
    ) -> Option<Station> {
        stations.records.get(slot)?;
        Some(Station {
            slot,
            label: label.into(),
        })
    }
}

/// One scheduled stop at an absolute distance along the path.
#[derive(Debug, Clone, PartialEq)]
pub struct Stop {
    pub station: Station,
    /// Distance from the path start, in track-world units.
    pub distance: f64,
    /// Ticks spent at rest before departure; ignored at the final stop.
    pub dwell_ticks: u64,
}

/// An ordered schedule for one path.
#[derive(Debug, Clone, PartialEq)]
pub struct Schedule {
    pub stops: Vec<Stop>,
}

impl Schedule {
    /// Validates a schedule: at least two strictly increasing stops, the first
    /// at the path start and the last at the path end, all within the path.
    pub fn new(stops: Vec<Stop>, path_length: f64) -> Option<Schedule> {
        if stops.len() < 2 || !path_length.is_finite() || path_length <= 0.0 {
            return None;
        }
        let mut previous = None;
        for stop in &stops {
            if !stop.distance.is_finite() || stop.distance < 0.0 || stop.distance > path_length {
                return None;
            }
            if previous.is_some_and(|value| stop.distance <= value) {
                return None;
            }
            previous = Some(stop.distance);
        }
        if stops[0].distance != 0.0 || (stops.last()?.distance - path_length).abs() > 1e-6 {
            return None;
        }
        Some(Schedule { stops })
    }

    /// A two-stop schedule between two station-DB records.
    ///
    /// The stops sit at the declared path start and end; the station labels
    /// and those distances are declared, not decoded from the records.
    pub fn between(
        stations: &FixedBlobCollection,
        from_slot: usize,
        from_label: &str,
        to_slot: usize,
        to_label: &str,
        path_length: f64,
    ) -> Option<Schedule> {
        let from = Station::from_record(stations, from_slot, from_label)?;
        let to = Station::from_record(stations, to_slot, to_label)?;
        Schedule::new(
            vec![
                Stop {
                    station: from,
                    distance: 0.0,
                    dwell_ticks: DEFAULT_DWELL_TICKS,
                },
                Stop {
                    station: to,
                    distance: path_length,
                    dwell_ticks: 0,
                },
            ],
            path_length,
        )
    }
}

/// One completed stop in a run.
#[derive(Debug, Clone, PartialEq)]
pub struct StopRecord {
    pub stop: usize,
    pub slot: usize,
    pub station: String,
    pub distance: f64,
    pub arrival: u64,
    pub departure: Option<u64>,
}

/// A consist executing a schedule over one path.
#[derive(Debug, Clone, PartialEq)]
pub struct Train {
    pub consist: Consist,
    pub schedule: Schedule,
    pub movement: Movement,
    next_stop: usize,
    dwell_remaining: u64,
    /// Stops reached so far, in order.
    pub stops: Vec<StopRecord>,
}

impl Train {
    /// Places the consist at the first station on `path`.
    pub fn new(consist: Consist, schedule: Schedule, path: Path) -> Option<Train> {
        let mut schedule = Schedule::new(schedule.stops.clone(), path.length())?;
        // The endpoint tolerance accepts a final stop within a millionth of
        // the path end; normalize it so the run reaches the exact path end.
        schedule.stops.last_mut()?.distance = path.length();
        let first = &schedule.stops[0];
        let mut stops = vec![StopRecord {
            stop: 0,
            slot: first.station.slot,
            station: first.station.label.clone(),
            distance: first.distance,
            arrival: 0,
            departure: None,
        }];
        if first.dwell_ticks == 0 {
            stops[0].departure = Some(0);
        }
        Some(Train {
            consist,
            movement: Movement::new(path),
            dwell_remaining: first.dwell_ticks,
            next_stop: 1,
            stops,
            schedule,
        })
    }

    /// True once the consist has reached the final station.
    pub fn finished(&self) -> bool {
        self.next_stop >= self.schedule.stops.len()
    }

    /// Advances the run by one tick: dwell, travel, arrival and departure.
    pub fn step(&mut self) {
        if self.finished() {
            return;
        }
        if self.dwell_remaining > 0 {
            self.dwell_remaining -= 1;
            self.movement.wait();
            if self.dwell_remaining == 0 {
                if let Some(record) = self.stops.last_mut() {
                    record.departure = Some(self.movement.tick);
                }
            }
            return;
        }

        let target = self.schedule.stops[self.next_stop].distance;
        self.movement.step_until(target);
        if self.movement.distance >= target - 1e-9 && self.movement.speed == 0.0 {
            let stop = &self.schedule.stops[self.next_stop];
            self.stops.push(StopRecord {
                stop: self.next_stop,
                slot: stop.station.slot,
                station: stop.station.label.clone(),
                distance: stop.distance,
                arrival: self.movement.tick,
                departure: None,
            });
            self.next_stop += 1;
            if self.next_stop < self.schedule.stops.len() {
                if stop.dwell_ticks == 0 {
                    if let Some(record) = self.stops.last_mut() {
                        record.departure = Some(self.movement.tick);
                    }
                } else {
                    self.dwell_remaining = stop.dwell_ticks;
                }
            }
        }
    }

    /// One fixed-order JSON line per reached stop.
    pub fn dump_lines(&self) -> Vec<String> {
        self.stops
            .iter()
            .map(|record| {
                format!(
                    "{{\"stop\":{},\"slot\":{},\"station\":\"{}\",\"distance\":{:.3},\
                     \"arrival\":{},\"departure\":{}}}",
                    record.stop,
                    record.slot,
                    json_escape(&record.station),
                    record.distance,
                    record.arrival,
                    match record.departure {
                        Some(tick) => tick.to_string(),
                        None => "null".to_string(),
                    }
                )
            })
            .collect()
    }
}

fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control.is_control() => out.push(' '),
            other => out.push(other),
        }
    }
    out
}
