//! Scripted rail company and a declared AI competitor (milestone 6, claims
//! C5 and C6).
//!
//! A company runs one-way scheduled trips over the declared line and is paid a
//! declared fare per car per completed trip; it pays declared per-tick and
//! per-distance operating costs. The v0 competitor follows a fixed scripted
//! policy: it runs the same route under the same rules, starting at a declared
//! tick and repeating at a declared interval. There is no recovered original
//! finance or AI behavior, and no interaction between the two companies.
//!
//! Every rule and divergence is recorded in `docs/evidence/rail-economy-v1.md`.

use crate::movement::Path;
use crate::schedule::{Consist, Schedule, Train};

/// Declared company/economy model version.
pub const COMPANY_VERSION: u32 = 1;

/// Declared revenue and cost rules.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompanyRules {
    /// Revenue for one completed trip, per car.
    pub fare_per_car_trip: f64,
    /// Operating cost per simulated tick (including dwell).
    pub cost_per_tick: f64,
    /// Operating cost per track-world unit travelled.
    pub cost_per_unit: f64,
    /// Cash the company starts with.
    pub initial_cash: f64,
}

impl Default for CompanyRules {
    fn default() -> CompanyRules {
        CompanyRules {
            fare_per_car_trip: 250.0,
            cost_per_tick: 0.5,
            cost_per_unit: 0.05,
            initial_cash: 0.0,
        }
    }
}

impl CompanyRules {
    fn valid(&self) -> bool {
        [
            self.fare_per_car_trip,
            self.cost_per_tick,
            self.cost_per_unit,
            self.initial_cash,
        ]
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
    }
}

/// The declared competitor v0 policy: delayed start, fixed trip interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompetitorPolicy {
    pub start_tick: u64,
    pub interval_ticks: u64,
}

impl Default for CompetitorPolicy {
    fn default() -> CompetitorPolicy {
        CompetitorPolicy {
            start_tick: 300,
            interval_ticks: 300,
        }
    }
}

/// One company's running totals.
#[derive(Debug, Clone, PartialEq)]
pub struct Company {
    pub name: String,
    pub cash: f64,
    pub revenue: f64,
    pub costs: f64,
    pub trips: u64,
    /// 1-based trip after which cash first became positive.
    pub first_profitable_trip: Option<u64>,
}

impl Company {
    fn new(name: &str, initial_cash: f64) -> Company {
        Company {
            name: name.to_string(),
            cash: initial_cash,
            revenue: 0.0,
            costs: 0.0,
            trips: 0,
            first_profitable_trip: None,
        }
    }

    fn apply(&mut self, trip: TripFigures, trip_number: u64) -> bool {
        let revenue = self.revenue + trip.revenue;
        let costs = self.costs + trip.costs;
        let cash = self.cash + trip.revenue - trip.costs;
        if !(revenue.is_finite() && costs.is_finite() && cash.is_finite()) {
            return false;
        }
        self.revenue = revenue;
        self.costs = costs;
        self.cash = cash;
        self.trips += 1;
        if self.cash > 0.0 && self.first_profitable_trip.is_none() {
            self.first_profitable_trip = Some(trip_number);
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TripFigures {
    ticks: u64,
    revenue: f64,
    costs: f64,
}

/// One joint run of the player company and the declared competitor.
#[derive(Debug, Clone, PartialEq)]
pub struct JointRun {
    pub rules: CompanyRules,
    pub policy: CompetitorPolicy,
    pub trips_per_company: u64,
    pub player: Company,
    pub competitor: Company,
    /// Last tick at which either company is still running a trip.
    pub horizon_ticks: u64,
}

impl JointRun {
    /// Runs the player and competitor over the same path and schedule.
    ///
    /// The player's trips are sequential from tick 0. The competitor's trip
    /// `n` (0-based) starts at `policy.start_tick + n * interval`; a trip lasts
    /// the same number of ticks for both because they share the consist and
    /// schedule. The companies do not interact.
    pub fn run(
        consist: &Consist,
        path: &Path,
        schedule: &Schedule,
        rules: CompanyRules,
        trips_per_company: u64,
        policy: CompetitorPolicy,
    ) -> Option<JointRun> {
        if trips_per_company == 0 || policy.interval_ticks == 0 || !rules.valid() {
            return None;
        }
        let trip = run_trip(consist, path, schedule, rules)?;

        let mut player = Company::new("player", rules.initial_cash);
        let mut player_end = 0u64;
        for trip_number in 1..=trips_per_company {
            if !player.apply(trip, trip_number) {
                return None;
            }
            player_end = player_end.checked_add(trip.ticks)?;
        }

        let mut competitor = Company::new("competitor", rules.initial_cash);
        let mut horizon = player_end;
        for trip_index in 0..trips_per_company {
            let start = trip_index
                .checked_mul(policy.interval_ticks)?
                .checked_add(policy.start_tick)?;
            horizon = horizon.max(start.checked_add(trip.ticks)?);
            if !competitor.apply(trip, trip_index + 1) {
                return None;
            }
        }

        Some(JointRun {
            rules,
            policy,
            trips_per_company,
            player,
            competitor,
            horizon_ticks: horizon,
        })
    }

    /// The canonical text hashed for the joint run.
    pub fn state_line(&self) -> String {
        let mut out = format!("version {COMPANY_VERSION}\n");
        out.push_str(&format!(
            "rules {:.6} {:.6} {:.6} {:.6}\n",
            self.rules.fare_per_car_trip,
            self.rules.cost_per_tick,
            self.rules.cost_per_unit,
            self.rules.initial_cash
        ));
        out.push_str(&format!(
            "policy {} {}\n",
            self.policy.start_tick, self.policy.interval_ticks
        ));
        out.push_str(&format!("trips {}\n", self.trips_per_company));
        for company in [&self.player, &self.competitor] {
            out.push_str(&format!(
                "company {} {:.6} {:.6} {:.6} {} {}\n",
                company.name,
                company.cash,
                company.revenue,
                company.costs,
                company.trips,
                match company.first_profitable_trip {
                    Some(trip) => trip.to_string(),
                    None => "none".to_string(),
                }
            ));
        }
        out.push_str(&format!("horizon {}\n", self.horizon_ticks));
        out
    }

    /// SHA-256 of [`JointRun::state_line`].
    pub fn state_digest(&self) -> [u8; 32] {
        rt3_kit::save::sha256(self.state_line().as_bytes())
    }
}

fn run_trip(
    consist: &Consist,
    path: &Path,
    schedule: &Schedule,
    rules: CompanyRules,
) -> Option<TripFigures> {
    let mut train = Train::new(consist.clone(), schedule.clone(), path.clone())?;
    while !train.finished() {
        train.step();
    }
    let ticks = train.movement.tick;
    let distance = train.movement.path.length();
    let revenue = rules.fare_per_car_trip * consist.car_count() as f64;
    let costs = rules.cost_per_tick * ticks as f64 + rules.cost_per_unit * distance;
    if !revenue.is_finite() || !costs.is_finite() {
        return None;
    }
    Some(TripFigures {
        ticks,
        revenue,
        costs,
    })
}
