//! The train's trip along a [`Route`], as a small deterministic state machine.
//!
//! Deliberately free of any engine types: the Bevy layer ticks it with a delta
//! time and translates the returned [`JourneyEvent`]s into ECS events. That
//! keeps the interesting rules unit-testable without opening a window.

use crate::city::CityId;
use crate::geo::MapPoint;
use crate::network::{Leg, Network, Route};

/// Tuning for how the train behaves.
///
/// Travel is run on a compressed clock, but the time a train spends standing at
/// a platform is measured in real seconds - the station is where the player
/// reads the board and watches the stamp come down, so it runs at human speed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JourneyConfig {
    /// Line speed in kilometres per simulated hour.
    pub line_speed_kmh: f32,
    /// Simulated seconds per real second while running.
    pub time_scale: f32,
    /// Real seconds spent at an intermediate platform.
    pub dwell_seconds: f32,
    /// Real seconds spent at the origin before the whistle.
    pub boarding_seconds: f32,
}

impl Default for JourneyConfig {
    fn default() -> Self {
        Self {
            line_speed_kmh: 55.0,
            time_scale: Self::CLOCK_PRESETS[1].1,
            dwell_seconds: 1.8,
            boarding_seconds: 1.2,
        }
    }
}

impl JourneyConfig {
    /// Clock speeds offered on the station board, slowest first.
    pub const CLOCK_PRESETS: [(&'static str, f32); 3] =
        [("Local", 240.0), ("Express", 480.0), ("Mail", 960.0)];

    /// Index of the active preset, if the clock is sitting on one.
    ///
    /// Presets are copied verbatim into `time_scale`, so exact equality is the
    /// right test here: a tolerance would quietly report a hand-set clock as a
    /// preset and then cycle to the wrong one.
    pub fn preset_index(&self) -> Option<usize> {
        Self::CLOCK_PRESETS.iter().position(|&(_, scale)| scale == self.time_scale)
    }

    /// Advances to the next clock preset, wrapping around.
    pub fn cycle_clock(&mut self) {
        let next = self.preset_index().map_or(0, |i| (i + 1) % Self::CLOCK_PRESETS.len());
        self.time_scale = Self::CLOCK_PRESETS[next].1;
    }

    pub fn clock_label(&self) -> &'static str {
        self.preset_index().map_or("Custom", |i| Self::CLOCK_PRESETS[i].0)
    }

    /// Timetabled running time for a distance, in simulated minutes.
    pub fn running_minutes(&self, metres: u32) -> f32 {
        if self.line_speed_kmh <= 0.0 {
            return 0.0;
        }
        (metres as f32 / 1000.0) / self.line_speed_kmh * 60.0
    }
}

/// What the train is doing right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Standing at the origin, doors open.
    Boarding,
    /// On the move between two stations.
    Running,
    /// Standing at an intermediate platform.
    Dwelling,
    /// Finished; the destination has been reached.
    Arrived,
}

/// Something worth reacting to: a whistle, a platform, the end of the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JourneyEvent {
    /// Pulled out of this station.
    Departed(CityId),
    /// Drew up at this station. `final_stop` marks the destination.
    Reached { city: CityId, final_stop: bool },
    /// The whole route is complete. Always follows the last `Reached`.
    Completed,
}

/// A train working a route.
#[derive(Debug, Clone)]
pub struct Journey {
    route: Route,
    /// Index into `route.legs` of the leg being run, or about to be run.
    leg: usize,
    /// Metres covered into the current leg.
    into_leg_m: f32,
    /// Metres covered over the whole route, for the odometer.
    travelled_m: f32,
    phase: Phase,
    /// Real seconds left of the current platform stop.
    timer: f32,
}

impl Journey {
    /// Starts a train at the origin of `route`, doors open.
    pub fn board(route: Route, config: &JourneyConfig) -> Self {
        Self {
            route,
            leg: 0,
            into_leg_m: 0.0,
            travelled_m: 0.0,
            phase: Phase::Boarding,
            timer: config.boarding_seconds,
        }
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn is_finished(&self) -> bool {
        self.phase == Phase::Arrived
    }

    /// The leg currently being run, or the one about to start.
    pub fn current_leg(&self) -> Leg {
        self.route.legs[self.leg.min(self.route.legs.len() - 1)]
    }

    /// Fraction of the whole route completed, in `0.0..=1.0`.
    pub fn progress(&self) -> f32 {
        if self.route.total_m == 0 {
            return 1.0;
        }
        (self.travelled_m / self.route.total_m as f32).clamp(0.0, 1.0)
    }

    pub fn travelled_km(&self) -> f32 {
        self.travelled_m / 1000.0
    }

    pub fn remaining_km(&self) -> f32 {
        (self.route.total_m as f32 - self.travelled_m).max(0.0) / 1000.0
    }

    /// Timetabled minutes still to run, ignoring platform stops.
    pub fn eta_minutes(&self, config: &JourneyConfig) -> f32 {
        config.running_minutes((self.remaining_km() * 1000.0) as u32)
    }

    /// The station the train is standing at, if it is standing at one.
    pub fn standing_at(&self) -> Option<CityId> {
        match self.phase {
            Phase::Boarding => Some(self.route.origin()),
            Phase::Dwelling => Some(self.current_leg().from),
            Phase::Arrived => Some(self.route.destination()),
            Phase::Running => None,
        }
    }

    /// Where to draw the train, in map kilometres.
    pub fn position(&self, network: &Network) -> MapPoint {
        if let Some(city) = self.standing_at() {
            return network.position(city);
        }
        let leg = self.current_leg();
        let t = if leg.track_m == 0 { 1.0 } else { self.into_leg_m / leg.track_m as f32 };
        network.position(leg.from).lerp(network.position(leg.to), t)
    }

    /// Unit heading of the train, for rotating the sprite. Falls back to the
    /// current leg's bearing while the train is standing still.
    pub fn heading(&self, network: &Network) -> (f32, f32) {
        let leg = self.current_leg();
        let (from, to) = (network.position(leg.from), network.position(leg.to));
        let (dx, dy) = (to.x_km - from.x_km, to.y_km - from.y_km);
        let length = (dx * dx + dy * dy).sqrt();
        if length < f32::EPSILON { (1.0, 0.0) } else { (dx / length, dy / length) }
    }

    /// Advances the train by `real_dt` seconds, pushing anything that happened
    /// into `events`.
    ///
    /// The caller owns `events` so that the per-frame system can reuse one
    /// buffer instead of allocating. Several stations can be passed in a single
    /// call when the clock is wound up, and each one still reports its event.
    pub fn advance(
        &mut self,
        real_dt: f32,
        config: &JourneyConfig,
        events: &mut Vec<JourneyEvent>,
    ) {
        if self.phase == Phase::Arrived || !real_dt.is_finite() || real_dt <= 0.0 {
            return;
        }

        let metres_per_real_second = config.line_speed_kmh * 1000.0 / 3600.0 * config.time_scale;
        let mut remaining = real_dt;

        // Each turn of the loop either consumes all remaining time or completes
        // one phase, and there are finitely many legs - so this terminates.
        // The guard only covers a pathological config (zero or absurd speed).
        for _ in 0..1024 {
            if remaining <= 0.0 {
                return;
            }
            match self.phase {
                Phase::Boarding | Phase::Dwelling => {
                    let spent = remaining.min(self.timer);
                    self.timer -= spent;
                    remaining -= spent;
                    if self.timer > 0.0 {
                        return;
                    }
                    let from = self.current_leg().from;
                    self.phase = Phase::Running;
                    events.push(JourneyEvent::Departed(from));
                }
                Phase::Running => {
                    if metres_per_real_second <= 0.0 {
                        return;
                    }
                    let leg = self.current_leg();
                    let left_on_leg = leg.track_m as f32 - self.into_leg_m;
                    let reachable = remaining * metres_per_real_second;

                    if reachable < left_on_leg {
                        self.into_leg_m += reachable;
                        self.travelled_m += reachable;
                        return;
                    }

                    remaining -= left_on_leg / metres_per_real_second;
                    self.travelled_m += left_on_leg;
                    self.into_leg_m = 0.0;

                    let final_stop = self.leg + 1 == self.route.legs.len();
                    events.push(JourneyEvent::Reached { city: leg.to, final_stop });
                    if final_stop {
                        self.travelled_m = self.route.total_m as f32;
                        self.phase = Phase::Arrived;
                        events.push(JourneyEvent::Completed);
                        return;
                    }
                    self.leg += 1;
                    self.timer = config.dwell_seconds;
                    self.phase = Phase::Dwelling;
                }
                Phase::Arrived => return,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JourneyConfig {
        JourneyConfig {
            line_speed_kmh: 60.0,
            time_scale: 600.0,
            dwell_seconds: 1.0,
            boarding_seconds: 1.0,
        }
    }

    fn run_to_completion(journey: &mut Journey, config: &JourneyConfig) -> Vec<JourneyEvent> {
        let mut all = Vec::new();
        let mut buffer = Vec::new();
        for _ in 0..10_000 {
            buffer.clear();
            journey.advance(1.0 / 60.0, config, &mut buffer);
            all.extend(buffer.iter().copied());
            if journey.is_finished() {
                break;
            }
        }
        assert!(journey.is_finished(), "journey never finished");
        all
    }

    #[test]
    fn stamps_every_station_including_both_ends() {
        let net = Network::new();
        let config = test_config();
        let route = net.route(CityId::Akka, CityId::AlKhalil).unwrap();
        let expected: Vec<CityId> = route.stops().collect();
        let mut journey = Journey::board(route, &config);

        let events = run_to_completion(&mut journey, &config);
        let reached: Vec<CityId> = events
            .iter()
            .filter_map(|e| match e {
                JourneyEvent::Reached { city, .. } => Some(*city),
                _ => None,
            })
            .collect();

        // Every stop after the origin is reported, in order.
        assert_eq!(reached, expected[1..]);
        assert_eq!(events.last(), Some(&JourneyEvent::Completed));
    }

    #[test]
    fn a_single_huge_step_still_reports_every_station() {
        let net = Network::new();
        let config = test_config();
        let route = net.route(CityId::Akka, CityId::AlKhalil).unwrap();
        let stops: Vec<CityId> = route.stops().collect();
        let mut journey = Journey::board(route, &config);

        // One absurd frame: a lagged tick must not teleport past platforms.
        let mut events = Vec::new();
        journey.advance(10_000.0, &config, &mut events);

        assert!(journey.is_finished());
        let reached = events.iter().filter(|e| matches!(e, JourneyEvent::Reached { .. })).count();
        assert_eq!(reached, stops.len() - 1);
    }

    #[test]
    fn departures_and_arrivals_alternate() {
        let net = Network::new();
        let config = test_config();
        let route = net.route(CityId::Haifa, CityId::Ghazzah).unwrap();
        let mut journey = Journey::board(route, &config);
        let events = run_to_completion(&mut journey, &config);

        let mut expect_departure = true;
        for event in &events {
            match event {
                JourneyEvent::Departed(_) => {
                    assert!(expect_departure, "two departures in a row");
                    expect_departure = false;
                }
                JourneyEvent::Reached { .. } => {
                    assert!(!expect_departure, "two arrivals in a row");
                    expect_departure = true;
                }
                JourneyEvent::Completed => {}
            }
        }
    }

    #[test]
    fn progress_rises_monotonically_to_one() {
        let net = Network::new();
        let config = test_config();
        let route = net.route(CityId::Safad, CityId::Yafa).unwrap();
        let mut journey = Journey::board(route, &config);

        let mut previous = 0.0;
        let mut buffer = Vec::new();
        for _ in 0..10_000 {
            buffer.clear();
            journey.advance(1.0 / 60.0, &config, &mut buffer);
            let now = journey.progress();
            assert!(now >= previous - 1e-5, "progress went backwards: {previous} -> {now}");
            previous = now;
            if journey.is_finished() {
                break;
            }
        }
        assert!((journey.progress() - 1.0).abs() < 1e-4);
        assert!(journey.remaining_km() < 0.01);
    }

    #[test]
    fn train_sits_at_the_platform_while_boarding() {
        let net = Network::new();
        let config = test_config();
        let route = net.route(CityId::Yafa, CityId::AlQuds).unwrap();
        let journey = Journey::board(route, &config);
        assert_eq!(journey.phase(), Phase::Boarding);
        assert_eq!(journey.standing_at(), Some(CityId::Yafa));
        let at = journey.position(&net);
        let platform = net.position(CityId::Yafa);
        assert!((at.x_km - platform.x_km).abs() < 1e-6);
        assert!((at.y_km - platform.y_km).abs() < 1e-6);
    }

    #[test]
    fn a_stopped_clock_does_not_move_the_train() {
        let net = Network::new();
        let config = JourneyConfig { line_speed_kmh: 0.0, ..test_config() };
        let route = net.route(CityId::Yafa, CityId::AlQuds).unwrap();
        let mut journey = Journey::board(route, &config);
        let mut events = Vec::new();
        journey.advance(1000.0, &config, &mut events);
        assert!(!journey.is_finished());
        assert_eq!(journey.progress(), 0.0);
    }
}
