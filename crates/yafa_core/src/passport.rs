//! The traveller's passport: which stations have been called at, and the
//! stamps collected for them.
//!
//! The passport is the game's progression, so it survives the process: it is a
//! small JSON document that is rewritten whenever a new stamp lands.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::calendar::Calendar;
use crate::city::{CITY_COUNT, CityId, LineType};
use crate::network::Network;

/// On-disk format version, so an old save can be recognised rather than
/// misread.
const FORMAT_VERSION: u32 = 1;

/// A single stamp in the passport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamp {
    pub city: CityId,
    /// 1 for the first station ever stamped, 2 for the next, and so on.
    pub order: u32,
    /// Which journey put it there.
    pub journey: u32,
}

/// What happened when a station was presented to the passport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampOutcome {
    /// A new page was filled in.
    Fresh(Stamp),
    /// The traveller has been here before; the stamp stands unchanged.
    AlreadyHeld(Stamp),
}

impl StampOutcome {
    pub const fn is_fresh(self) -> bool {
        matches!(self, StampOutcome::Fresh(_))
    }

    pub const fn stamp(self) -> Stamp {
        match self {
            StampOutcome::Fresh(s) | StampOutcome::AlreadyHeld(s) => s,
        }
    }
}

/// How much of one corridor has been seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineProgress {
    pub line: LineType,
    pub stamped: usize,
    pub total: usize,
}

impl LineProgress {
    pub const fn is_complete(&self) -> bool {
        self.total > 0 && self.stamped == self.total
    }
}

/// The traveller's record. Stations are held by [`CityId`], never by
/// reference, so the passport can be cloned, saved and shared freely.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passport {
    #[serde(default = "default_version")]
    version: u32,
    pub traveller: String,
    stamps: Vec<Stamp>,
    journeys_completed: u32,
    /// Lifetime odometer, in metres.
    total_track_m: u64,
    /// Where the game stands in time. Defaulted so a passport written before
    /// there was a calendar still reads.
    #[serde(default)]
    pub calendar: Calendar,
}

const fn default_version() -> u32 {
    FORMAT_VERSION
}

impl Default for Passport {
    fn default() -> Self {
        Self::new("Traveller")
    }
}

impl Passport {
    pub fn new(traveller: impl Into<String>) -> Self {
        Self {
            version: FORMAT_VERSION,
            traveller: traveller.into(),
            stamps: Vec::with_capacity(CITY_COUNT),
            journeys_completed: 0,
            total_track_m: 0,
            calendar: Calendar::new(),
        }
    }

    /// Presents a station to the clerk. Re-visiting a station is not an error -
    /// it simply does not fill a new page.
    pub fn stamp(&mut self, city: CityId) -> StampOutcome {
        if let Some(existing) = self.stamp_for(city) {
            return StampOutcome::AlreadyHeld(existing);
        }
        let stamp = Stamp {
            city,
            order: self.stamps.len() as u32 + 1,
            journey: self.journeys_completed + 1,
        };
        self.stamps.push(stamp);
        StampOutcome::Fresh(stamp)
    }

    pub fn stamp_for(&self, city: CityId) -> Option<Stamp> {
        self.stamps.iter().copied().find(|s| s.city == city)
    }

    pub fn is_stamped(&self, city: CityId) -> bool {
        self.stamps.iter().any(|s| s.city == city)
    }

    /// Stamps in the order they were collected.
    pub fn stamps(&self) -> &[Stamp] {
        &self.stamps
    }

    pub fn visited_count(&self) -> usize {
        self.stamps.len()
    }

    pub const fn total_cities(&self) -> usize {
        CITY_COUNT
    }

    /// The "Visited Cities: X / Y" line on the passport cover.
    pub fn counter(&self) -> String {
        format!("Visited Cities: {} / {}", self.visited_count(), CITY_COUNT)
    }

    pub fn completion(&self) -> f32 {
        self.visited_count() as f32 / CITY_COUNT as f32
    }

    pub fn is_complete(&self) -> bool {
        self.visited_count() == CITY_COUNT
    }

    pub const fn journeys_completed(&self) -> u32 {
        self.journeys_completed
    }

    pub fn total_track_km(&self) -> f32 {
        self.total_track_m as f32 / 1000.0
    }

    /// Books a finished trip: one more journey, and its mileage on the
    /// odometer.
    pub fn record_journey(&mut self, track_m: u32) {
        self.journeys_completed += 1;
        self.total_track_m += u64::from(track_m);
    }

    /// Per-corridor progress, for the passport's line summary.
    pub fn line_progress(&self, network: &Network) -> Vec<LineProgress> {
        LineType::ALL
            .iter()
            .map(|&line| {
                let cities: Vec<CityId> = network.cities_on(line).map(|c| c.id).collect();
                LineProgress {
                    line,
                    stamped: cities.iter().filter(|&&c| self.is_stamped(c)).count(),
                    total: cities.len(),
                }
            })
            .collect()
    }

    /// Stations still missing, in table order - the "where next?" list.
    pub fn unstamped(&self) -> Vec<CityId> {
        CityId::ALL.iter().copied().filter(|&c| !self.is_stamped(c)).collect()
    }

    /// Default save location: `$HOME/.yafa-express/passport.json`, falling back
    /// to the working directory if there is no home.
    pub fn default_path() -> PathBuf {
        match std::env::var_os("HOME") {
            Some(home) if !home.is_empty() => {
                Path::new(&home).join(".yafa-express").join("passport.json")
            }
            _ => PathBuf::from("yafa-passport.json"),
        }
    }

    /// Reads a passport from disk. A missing file yields a fresh passport; a
    /// corrupt one is an error, so a save is never silently discarded.
    pub fn load(path: &Path) -> io::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let passport: Passport = serde_json::from_str(&text)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                if passport.version != FORMAT_VERSION {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "passport format version {} is not supported (expected {FORMAT_VERSION})",
                            passport.version
                        ),
                    ));
                }
                Ok(passport)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    /// Writes the passport, creating the directory if need be.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_station_fills_one_page_only() {
        let mut passport = Passport::new("Tester");
        let first = passport.stamp(CityId::Yafa);
        assert!(first.is_fresh());
        let second = passport.stamp(CityId::Yafa);
        assert!(!second.is_fresh());
        assert_eq!(first.stamp(), second.stamp());
        assert_eq!(passport.visited_count(), 1);
    }

    #[test]
    fn stamps_are_numbered_in_the_order_collected() {
        let mut passport = Passport::new("Tester");
        for city in [CityId::Yafa, CityId::AlLadd, CityId::AlQuds] {
            passport.stamp(city);
        }
        let orders: Vec<u32> = passport.stamps().iter().map(|s| s.order).collect();
        assert_eq!(orders, vec![1, 2, 3]);
        assert_eq!(passport.stamp_for(CityId::AlQuds).unwrap().order, 3);
    }

    #[test]
    fn counter_and_completion_track_the_whole_network() {
        let mut passport = Passport::new("Tester");
        assert_eq!(passport.counter(), format!("Visited Cities: 0 / {CITY_COUNT}"));
        assert!(!passport.is_complete());
        for &city in &CityId::ALL {
            passport.stamp(city);
        }
        assert_eq!(passport.counter(), format!("Visited Cities: {CITY_COUNT} / {CITY_COUNT}"));
        assert!(passport.is_complete());
        assert!((passport.completion() - 1.0).abs() < f32::EPSILON);
        assert!(passport.unstamped().is_empty());
    }

    #[test]
    fn line_progress_reports_each_corridor() {
        let net = Network::new();
        let mut passport = Passport::new("Tester");
        for city in net.cities_on(LineType::Coastal).map(|c| c.id).collect::<Vec<_>>() {
            passport.stamp(city);
        }
        let progress = passport.line_progress(&net);
        let coastal = progress.iter().find(|p| p.line == LineType::Coastal).unwrap();
        assert!(coastal.is_complete());
        let mountain = progress.iter().find(|p| p.line == LineType::Mountain).unwrap();
        assert_eq!(mountain.stamped, 0);
        // Counted from the table rather than written down, so adding a station
        // to a corridor does not break a test about progress.
        assert_eq!(mountain.total, net.cities_on(LineType::Mountain).count());
        assert!(mountain.total > 0);
    }

    #[test]
    fn odometer_accumulates_across_journeys() {
        let mut passport = Passport::new("Tester");
        passport.record_journey(19_000);
        passport.record_journey(67_000);
        assert_eq!(passport.journeys_completed(), 2);
        assert!((passport.total_track_km() - 86.0).abs() < 1e-3);
    }

    #[test]
    fn round_trips_through_json() {
        let mut passport = Passport::new("Umm Yafa");
        passport.stamp(CityId::Haifa);
        passport.stamp(CityId::Beisan);
        passport.record_journey(51_000);

        let dir = std::env::temp_dir().join("yafa-express-tests");
        let path = dir.join("passport-roundtrip.json");
        let _ = std::fs::remove_file(&path);
        passport.save(&path).expect("save");

        let loaded = Passport::load(&path).expect("load");
        assert_eq!(loaded.traveller, "Umm Yafa");
        assert_eq!(loaded.visited_count(), 2);
        assert!(loaded.is_stamped(CityId::Beisan));
        assert_eq!(loaded.journeys_completed(), 1);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_missing_file_is_a_blank_passport_not_an_error() {
        let path = std::env::temp_dir().join("yafa-express-tests").join("definitely-absent.json");
        let _ = std::fs::remove_file(&path);
        let passport = Passport::load(&path).expect("missing file is fine");
        assert_eq!(passport.visited_count(), 0);
    }

    #[test]
    fn a_corrupt_file_is_reported_rather_than_swallowed() {
        let dir = std::env::temp_dir().join("yafa-express-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("corrupt.json");
        std::fs::write(&path, "{ not json at all").unwrap();
        assert!(Passport::load(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
