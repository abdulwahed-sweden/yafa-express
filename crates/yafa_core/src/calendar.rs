//! The game's year, which the train itself winds forward.
//!
//! There is no wall clock here. Time passes because track passes under the
//! wheels: every [`METRES_PER_YEAR`] run moves the calendar on by one, and the
//! lines of the network open underneath the player as it does.

use serde::{Deserialize, Serialize};

/// The year the first line opened, and the year the game begins: Yafa to
/// Al-Quds, 1892.
pub const FIRST_YEAR: u16 = 1892;

/// Track that has to be run for the calendar to turn over a year.
pub const METRES_PER_YEAR: u32 = 120_000;

/// Where the game stands in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    year: u16,
    /// Track run since this year began.
    metres_into_year: u32,
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

impl Calendar {
    pub const fn new() -> Self {
        Self { year: FIRST_YEAR, metres_into_year: 0 }
    }

    /// Starts the calendar at a given year. For tests, and for a save that
    /// carries one.
    pub const fn starting_at(year: u16) -> Self {
        Self { year, metres_into_year: 0 }
    }

    pub const fn year(&self) -> u16 {
        self.year
    }

    /// How far through the current year the odometer stands, in `0.0..1.0`.
    pub fn through_year(&self) -> f32 {
        self.metres_into_year as f32 / METRES_PER_YEAR as f32
    }

    /// Books a run of track and returns every year the calendar entered as a
    /// result, in order.
    ///
    /// A long enough journey can cross more than one year boundary, and each
    /// one may open a line, so the caller gets the whole list rather than just
    /// the year it ended in.
    pub fn advance(&mut self, track_m: u32) -> Vec<u16> {
        let total = u64::from(self.metres_into_year) + u64::from(track_m);
        let years = total / u64::from(METRES_PER_YEAR);
        self.metres_into_year = (total % u64::from(METRES_PER_YEAR)) as u32;

        let first_new = self.year.saturating_add(1);
        self.year = self.year.saturating_add(years.min(u64::from(u16::MAX)) as u16);
        (first_new..=self.year).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_game_starts_in_1892() {
        assert_eq!(Calendar::new().year(), FIRST_YEAR);
        assert_eq!(Calendar::new().through_year(), 0.0);
    }

    #[test]
    fn short_runs_do_not_turn_the_year() {
        let mut calendar = Calendar::new();
        assert!(calendar.advance(86_000).is_empty());
        assert_eq!(calendar.year(), 1892);
        assert!((calendar.through_year() - 86.0 / 120.0).abs() < 1e-5);
    }

    #[test]
    fn track_run_carries_over_into_the_next_year() {
        let mut calendar = Calendar::new();
        calendar.advance(100_000);
        // 20 km short of the year; another 30 km should tip it over and leave
        // 10 km standing in 1893.
        assert_eq!(calendar.advance(30_000), vec![1893]);
        assert_eq!(calendar.year(), 1893);
        assert!((calendar.through_year() - 10.0 / 120.0).abs() < 1e-5);
    }

    #[test]
    fn a_long_run_reports_every_year_it_crosses() {
        let mut calendar = Calendar::new();
        let opened = calendar.advance(METRES_PER_YEAR * 3);
        assert_eq!(opened, vec![1893, 1894, 1895]);
        assert_eq!(calendar.year(), 1895);
    }
}
