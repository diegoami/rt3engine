//! Declared consist movement on the track graph (milestone 6, claim C2).
//!
//! The original's train performance and physics constants are not recovered,
//! so this module implements a **declared model** rather than a copy of the
//! game's motion. The rule table, the parameters and every divergence from
//! measurable original state are recorded in `docs/evidence/movement-v1.md`.
//!
//! The model: one tick is one second; a consist accelerates at a constant
//! rate toward a cruise speed, brakes when the remaining path is inside the
//! stopping distance, and stops at the end of the path. Distances are decoded
//! track-world units. There is **no randomness** in the model — replaying the
//! same path for the same number of ticks produces byte-identical state — so
//! a seed does not exist to change.

use rt3_kit::rail::TrackGraph;

/// Simulated seconds in one tick.
pub const TICK_SECONDS: f64 = 1.0;
/// Declared cruise speed, in track-world units per second.
pub const MAX_SPEED: f64 = 12.0;
/// Declared acceleration and braking rate, in units per second squared.
pub const ACCELERATION: f64 = 2.0;

/// An ordered path through the declared track graph.
///
/// The path is the unique walk between the graph's two degree-1 endpoints;
/// building it fails unless the graph has exactly two such endpoints and the
/// walk visits every piece. Travel starts at the endpoint with the lower piece
/// index. It follows the graph's runs and joins; the piece origin field and
/// the connectivity rule are the declared model in
/// `docs/evidence/rail-v1.md`.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    /// Graph piece indices in travel order.
    pub pieces: Vec<usize>,
    /// Piece origins in travel order.
    pub points: Vec<[f64; 3]>,
    /// Cumulative distance to every point, starting at zero.
    pub cumulative: Vec<f64>,
}

impl Path {
    /// Derives the ordered path from a graph's pieces and edges.
    pub fn from_graph(graph: &TrackGraph) -> Option<Path> {
        let count = graph.pieces.len();
        if count < 2 {
            return None;
        }

        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); count];
        let mut seen = Vec::new();
        for (a, b) in graph.edges() {
            if a >= count || b >= count || a == b {
                return None;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.contains(&key) {
                return None;
            }
            seen.push(key);
            adjacency[a].push(b);
            adjacency[b].push(a);
        }

        if adjacency.iter().any(|neighbours| neighbours.len() > 2) {
            return None;
        }
        let mut ends: Vec<usize> = (0..count)
            .filter(|&index| adjacency[index].len() == 1)
            .collect();
        if ends.len() != 2 {
            return None;
        }
        ends.sort_unstable();
        let start = ends[0];

        let mut order = Vec::with_capacity(count);
        let mut previous = usize::MAX;
        let mut current = start;
        loop {
            order.push(current);
            if order.len() > count {
                return None;
            }
            match adjacency[current]
                .iter()
                .copied()
                .find(|&next| next != previous)
            {
                Some(next) => {
                    previous = current;
                    current = next;
                }
                None => break,
            }
        }
        if order.len() != count {
            return None;
        }

        let mut points = Vec::with_capacity(count);
        for &piece in &order {
            let origin = graph.pieces[piece].origin;
            points.push([
                f64::from(origin[0]),
                f64::from(origin[1]),
                f64::from(origin[2]),
            ]);
        }
        let mut cumulative = Vec::with_capacity(count);
        cumulative.push(0.0);
        for pair in points.windows(2) {
            cumulative.push(cumulative[cumulative.len() - 1] + distance(pair[0], pair[1]));
        }

        Some(Path {
            pieces: order,
            points,
            cumulative,
        })
    }

    /// Total path length in track-world units.
    pub fn length(&self) -> f64 {
        *self.cumulative.last().unwrap_or(&0.0)
    }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// A consist moving along one ordered path.
#[derive(Debug, Clone, PartialEq)]
pub struct Movement {
    pub path: Path,
    /// Completed ticks.
    pub tick: u64,
    /// Distance travelled from the path start.
    pub distance: f64,
    /// Current speed in units per second.
    pub speed: f64,
    /// True once the consist has reached the path end.
    pub stopped: bool,
}

impl Movement {
    /// Places a stopped consist at the start of `path`.
    pub fn new(path: Path) -> Movement {
        Movement {
            path,
            tick: 0,
            distance: 0.0,
            speed: 0.0,
            stopped: false,
        }
    }

    /// Advances the declared model by one tick.
    pub fn step(&mut self) {
        if self.stopped {
            return;
        }
        let remaining = self.path.length() - self.distance;
        let braking = self.speed * self.speed / (2.0 * ACCELERATION);
        let target = if remaining <= braking { 0.0 } else { MAX_SPEED };
        self.speed = if target > self.speed {
            (self.speed + ACCELERATION * TICK_SECONDS).min(target)
        } else {
            (self.speed - ACCELERATION * TICK_SECONDS).max(0.0)
        };
        self.distance += self.speed * TICK_SECONDS;
        if self.distance >= self.path.length() {
            self.distance = self.path.length();
            self.speed = 0.0;
            self.stopped = true;
        }
        self.tick += 1;
    }

    /// The piece index and interpolated position at the current distance.
    pub fn position(&self) -> (usize, [f64; 3]) {
        let total = self.path.length();
        let distance = self.distance.min(total);
        if distance >= total {
            let last = self.path.pieces.len() - 1;
            return (self.path.pieces[last], self.path.points[last]);
        }
        let mut segment = self.path.cumulative.len() - 2;
        for index in 0..self.path.cumulative.len() - 1 {
            if distance <= self.path.cumulative[index + 1] {
                segment = index;
                break;
            }
        }
        let start = self.path.cumulative[segment];
        let end = self.path.cumulative[segment + 1];
        let fraction = if end > start {
            (distance - start) / (end - start)
        } else {
            0.0
        };
        let a = self.path.points[segment];
        let b = self.path.points[segment + 1];
        (
            self.path.pieces[segment],
            [
                a[0] + (b[0] - a[0]) * fraction,
                a[1] + (b[1] - a[1]) * fraction,
                a[2] + (b[2] - a[2]) * fraction,
            ],
        )
    }

    /// One fixed-order JSON state line for this tick.
    pub fn state_line(&self) -> String {
        let (piece, position) = self.position();
        format!(
            "{{\"tick\":{},\"piece\":{},\"distance\":{:.6},\"speed\":{:.6},\
             \"x\":{:.6},\"y\":{:.6},\"z\":{:.6},\"stopped\":{}}}",
            self.tick,
            piece,
            self.distance,
            self.speed,
            position[0],
            position[1],
            position[2],
            self.stopped
        )
    }
}
