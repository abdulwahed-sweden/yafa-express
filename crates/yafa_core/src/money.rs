//! Fares, coal and the treasury, in piastres.
//!
//! Money is whole piastres. The tariff is quoted in fractions - five and
//! fifteen hundredths of a piastre to the kilometre - but every sum that
//! reaches the treasury is rounded first, so a balance is always a number the
//! player can read off the header and check.

use serde::{Deserialize, Serialize};

/// What the railway charges and what it costs to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tariff {
    /// Paid by every passenger, whatever the distance.
    pub booking_fee: f32,
    /// Paid on top, per kilometre carried.
    pub per_km: f32,
    /// Coal burnt, in piastres per kilometre.
    pub coal_per_km: f32,
    /// What it costs to lay a kilometre of surveyed line.
    pub track_per_km: f32,
}

impl Default for Tariff {
    fn default() -> Self {
        Self { booking_fee: 5.0, per_km: 0.15, coal_per_km: 0.5, track_per_km: 10.0 }
    }
}

impl Tariff {
    /// What one passenger pays for a journey of `metres`.
    pub fn fare(&self, metres: u32) -> i64 {
        round_piastres(self.booking_fee + self.per_km * (metres as f32 / 1000.0))
    }

    /// What the coal for a run of `metres` costs.
    pub fn coal(&self, metres: u32) -> i64 {
        round_piastres(self.coal_per_km * (metres as f32 / 1000.0))
    }

    /// What it costs to lay `metres` of surveyed line.
    pub fn track(&self, metres: u32) -> i64 {
        round_piastres(self.track_per_km * (metres as f32 / 1000.0))
    }
}

fn round_piastres(amount: f32) -> i64 {
    amount.round().max(0.0) as i64
}

/// The money the railway has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Treasury {
    piastres: i64,
}

impl Default for Treasury {
    fn default() -> Self {
        Self::new()
    }
}

impl Treasury {
    /// The float the railway starts with.
    pub const OPENING_BALANCE: i64 = 500;

    pub const fn new() -> Self {
        Self { piastres: Self::OPENING_BALANCE }
    }

    pub const fn with(piastres: i64) -> Self {
        Self { piastres }
    }

    pub const fn balance(&self) -> i64 {
        self.piastres
    }

    pub const fn can_afford(&self, cost: i64) -> bool {
        self.piastres >= cost
    }

    pub const fn credit(&mut self, piastres: i64) {
        self.piastres += piastres;
    }

    /// Takes `cost` out of the balance if it is there, and says whether it
    /// was. The railway is never allowed to spend money it has not got.
    pub const fn debit(&mut self, cost: i64) -> bool {
        if self.piastres < cost {
            return false;
        }
        self.piastres -= cost;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_railway_opens_with_five_hundred() {
        assert_eq!(Treasury::new().balance(), 500);
    }

    #[test]
    fn a_fare_is_the_booking_fee_plus_the_distance() {
        let tariff = Tariff::default();
        // 86 km from Yafa to Al-Quds: 5 + 12.9, to the nearest piastre.
        assert_eq!(tariff.fare(86_000), 18);
        assert_eq!(tariff.fare(0), 5);
    }

    #[test]
    fn coal_is_a_piastre_every_two_kilometres() {
        let tariff = Tariff::default();
        assert_eq!(tariff.coal(86_000), 43);
        assert_eq!(tariff.coal(2_000), 1);
    }

    #[test]
    fn track_costs_ten_a_kilometre() {
        assert_eq!(Tariff::default().track(63_000), 630);
    }

    #[test]
    fn the_railway_cannot_spend_what_it_has_not_got() {
        let mut treasury = Treasury::new();
        assert!(!treasury.debit(501));
        assert_eq!(treasury.balance(), 500, "a refused debit must not move the balance");
        assert!(treasury.debit(500));
        assert_eq!(treasury.balance(), 0);
    }
}
