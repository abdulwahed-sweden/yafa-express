//! Passengers: who is waiting where, who is aboard, and where they want to go.
//!
//! All of it plain data. The platforms are filled by a seeded generator rather
//! than the system clock, so a test can lay out exactly the traffic it wants
//! and get the same answer every run.

use serde::{Deserialize, Serialize};

use crate::city::{CITY_COUNT, CityId};
use crate::network::{Network, Route};

/// How many may stand on one platform before the station turns people away.
pub const PLATFORM_LIMIT: usize = 24;

/// How many a train will carry.
pub const TRAIN_CAPACITY: usize = 40;

/// A small deterministic generator, so traffic is repeatable.
///
/// Good enough to pick a destination; nothing here needs a real distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        Self::seeded(0x5EED_1892)
    }
}

impl Rng {
    pub const fn seeded(seed: u64) -> Self {
        // Zero is a fixed point of xorshift, so it is never a valid state.
        Self(if seed == 0 { 1 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// A number in `0..limit`. Returns 0 when `limit` is 0.
    pub fn below(&mut self, limit: u32) -> u32 {
        if limit == 0 {
            return 0;
        }
        (self.next_u64() % u64::from(limit)) as u32
    }

    /// True with probability `numerator / denominator`.
    pub fn chance(&mut self, numerator: u32, denominator: u32) -> bool {
        denominator > 0 && self.below(denominator) < numerator
    }
}

/// Somebody waiting for a train.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Passenger {
    pub destination: CityId,
}

/// Somebody on the train.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aboard {
    pub destination: CityId,
    pub boarded_at: CityId,
}

/// The queue at one station.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Platform {
    waiting: Vec<Passenger>,
}

impl Platform {
    pub fn waiting(&self) -> &[Passenger] {
        &self.waiting
    }

    pub fn len(&self) -> usize {
        self.waiting.len()
    }

    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }

    /// Adds somebody to the queue, unless the platform is full.
    pub fn join(&mut self, passenger: Passenger) -> bool {
        if self.waiting.len() >= PLATFORM_LIMIT {
            return false;
        }
        self.waiting.push(passenger);
        true
    }
}

/// Every platform on the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platforms {
    #[serde(with = "serde_platforms")]
    by_city: [Platform; CITY_COUNT],
    rng: Rng,
}

impl Default for Platforms {
    fn default() -> Self {
        Self::new(Rng::default())
    }
}

impl Platforms {
    pub fn new(rng: Rng) -> Self {
        Self { by_city: std::array::from_fn(|_| Platform::default()), rng }
    }

    pub fn at(&self, city: CityId) -> &Platform {
        &self.by_city[city.index()]
    }

    pub fn at_mut(&mut self, city: CityId) -> &mut Platform {
        &mut self.by_city[city.index()]
    }

    pub fn total_waiting(&self) -> usize {
        self.by_city.iter().map(Platform::len).sum()
    }

    /// Puts one more traveller on each open platform that draws custom this
    /// tick, bound for somewhere else on the network.
    ///
    /// A station's own importance decides how often it produces a traveller;
    /// every other open station's importance decides how likely it is to be
    /// where they are going. Returns how many joined a queue.
    pub fn tick(&mut self, network: &Network, year: u16) -> usize {
        let open: Vec<CityId> = network.open_stations(year).collect();
        if open.len() < 2 {
            return 0;
        }
        let total_pull: u32 = open.iter().map(|&c| u32::from(network.city(c).importance)).sum();

        let mut joined = 0;
        for &from in &open {
            let importance = u32::from(network.city(from).importance);
            // A place of importance 5 produces a traveller on every tick; a
            // halt of importance 1, on one tick in five.
            if !self.rng.chance(importance, 5) {
                continue;
            }
            let Some(destination) = self.weighted_pick(network, &open, from, total_pull) else {
                continue;
            };
            if self.at_mut(from).join(Passenger { destination }) {
                joined += 1;
            }
        }
        joined
    }

    /// Picks somewhere to go, by importance, excluding where they are.
    fn weighted_pick(
        &mut self,
        network: &Network,
        open: &[CityId],
        from: CityId,
        total_pull: u32,
    ) -> Option<CityId> {
        let here = u32::from(network.city(from).importance);
        let pull = total_pull.checked_sub(here)?;
        if pull == 0 {
            return None;
        }
        let mut choice = self.rng.below(pull);
        for &city in open {
            if city == from {
                continue;
            }
            let weight = u32::from(network.city(city).importance);
            if choice < weight {
                return Some(city);
            }
            choice -= weight;
        }
        None
    }
}

// `[Platform; 23]` is past the size serde derives for arrays, so it goes
// through a Vec on the way to and from disk.
mod serde_platforms {
    use super::{CITY_COUNT, Platform};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        platforms: &[Platform; CITY_COUNT],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        platforms.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<[Platform; CITY_COUNT], D::Error> {
        let mut loaded: Vec<Platform> = Vec::deserialize(deserializer)?;
        loaded.resize(CITY_COUNT, Platform::default());
        Ok(std::array::from_fn(|i| std::mem::take(&mut loaded[i])))
    }
}

/// Who is on the train, and where they got on.
#[derive(Debug, Clone, Default)]
pub struct Manifest {
    aboard: Vec<Aboard>,
}

impl Manifest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn aboard(&self) -> &[Aboard] {
        &self.aboard
    }

    pub fn len(&self) -> usize {
        self.aboard.len()
    }

    pub fn is_empty(&self) -> bool {
        self.aboard.is_empty()
    }

    pub fn free_seats(&self) -> usize {
        TRAIN_CAPACITY.saturating_sub(self.aboard.len())
    }

    /// Takes off everyone bound for `city`.
    pub fn alight_at(&mut self, city: CityId) -> Vec<Aboard> {
        let (leaving, staying) = self.aboard.iter().partition(|rider| rider.destination == city);
        self.aboard = staying;
        leaving
    }

    /// Takes on everyone waiting at `city` whose destination is still ahead on
    /// `route`, in the order they queued, up to the train's capacity.
    ///
    /// Returns how many got on.
    pub fn board_at(&mut self, platforms: &mut Platforms, city: CityId, route: &Route) -> usize {
        let ahead: Vec<CityId> = stops_after(route, city);
        if ahead.is_empty() {
            return 0;
        }

        let platform = platforms.at_mut(city);
        let mut taken = 0;
        let mut left_behind = Vec::with_capacity(platform.waiting.len());

        for passenger in std::mem::take(&mut platform.waiting) {
            if self.free_seats() > 0 && ahead.contains(&passenger.destination) {
                self.aboard.push(Aboard { destination: passenger.destination, boarded_at: city });
                taken += 1;
            } else {
                left_behind.push(passenger);
            }
        }
        platform.waiting = left_behind;
        taken
    }
}

/// The stations a route still has to call at after `city`.
fn stops_after(route: &Route, city: CityId) -> Vec<CityId> {
    let stops: Vec<CityId> = route.stops().collect();
    match stops.iter().position(|&stop| stop == city) {
        Some(index) => stops[index + 1..].to_vec(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_BUILT: u16 = 1918;

    #[test]
    fn a_platform_turns_people_away_when_it_is_full() {
        let mut platform = Platform::default();
        for _ in 0..PLATFORM_LIMIT {
            assert!(platform.join(Passenger { destination: CityId::Yafa }));
        }
        assert!(!platform.join(Passenger { destination: CityId::Yafa }));
        assert_eq!(platform.len(), PLATFORM_LIMIT);
    }

    #[test]
    fn ticking_fills_platforms_only_where_there_is_a_line() {
        let net = Network::new();
        let mut platforms = Platforms::new(Rng::seeded(7));
        for _ in 0..40 {
            platforms.tick(&net, 1892);
        }
        // In 1892 only three stations are on the network.
        assert!(platforms.total_waiting() > 0);
        for city in CityId::ALL {
            if net.station_is_open(city, 1892) {
                continue;
            }
            assert!(platforms.at(city).is_empty(), "{city:?} is not on the network");
        }
    }

    #[test]
    fn nobody_is_sent_somewhere_off_the_network() {
        let net = Network::new();
        let mut platforms = Platforms::new(Rng::seeded(11));
        for _ in 0..60 {
            platforms.tick(&net, 1892);
        }
        for city in CityId::ALL {
            for passenger in platforms.at(city).waiting() {
                assert!(net.station_is_open(passenger.destination, 1892));
                assert_ne!(passenger.destination, city, "nobody travels to where they are");
            }
        }
    }

    #[test]
    fn a_train_takes_only_those_going_its_way() {
        let net = Network::new();
        let route = net.route(CityId::Yafa, CityId::AlQuds, ALL_BUILT).unwrap();
        let mut platforms = Platforms::new(Rng::seeded(1));

        platforms.at_mut(CityId::Yafa).join(Passenger { destination: CityId::AlQuds });
        platforms.at_mut(CityId::Yafa).join(Passenger { destination: CityId::AlLadd });
        // Haifa is not on this route, so this one waits.
        platforms.at_mut(CityId::Yafa).join(Passenger { destination: CityId::Haifa });

        let mut manifest = Manifest::new();
        assert_eq!(manifest.board_at(&mut platforms, CityId::Yafa, &route), 2);
        assert_eq!(platforms.at(CityId::Yafa).len(), 1);
        assert_eq!(platforms.at(CityId::Yafa).waiting()[0].destination, CityId::Haifa);
    }

    #[test]
    fn a_train_fills_up_and_leaves_the_rest() {
        let net = Network::new();
        let route = net.route(CityId::Yafa, CityId::AlQuds, ALL_BUILT).unwrap();
        let mut platforms = Platforms::new(Rng::seeded(1));
        // More than will fit, and the platform caps the queue besides.
        for _ in 0..PLATFORM_LIMIT {
            platforms.at_mut(CityId::Yafa).join(Passenger { destination: CityId::AlQuds });
        }
        let mut manifest = Manifest::new();
        let taken = manifest.board_at(&mut platforms, CityId::Yafa, &route);
        assert_eq!(taken, PLATFORM_LIMIT.min(TRAIN_CAPACITY));
        assert!(manifest.len() <= TRAIN_CAPACITY);
    }

    #[test]
    fn everyone_gets_off_where_they_meant_to() {
        let net = Network::new();
        let route = net.route(CityId::Yafa, CityId::AlQuds, ALL_BUILT).unwrap();
        let mut platforms = Platforms::new(Rng::seeded(1));
        platforms.at_mut(CityId::Yafa).join(Passenger { destination: CityId::AlLadd });
        platforms.at_mut(CityId::Yafa).join(Passenger { destination: CityId::AlQuds });

        let mut manifest = Manifest::new();
        manifest.board_at(&mut platforms, CityId::Yafa, &route);

        let at_ladd = manifest.alight_at(CityId::AlLadd);
        assert_eq!(at_ladd.len(), 1);
        assert_eq!(at_ladd[0].boarded_at, CityId::Yafa);
        assert_eq!(manifest.len(), 1);

        assert_eq!(manifest.alight_at(CityId::AlQuds).len(), 1);
        assert!(manifest.is_empty());
    }

    #[test]
    fn nobody_boards_at_the_end_of_the_line() {
        let net = Network::new();
        let route = net.route(CityId::Yafa, CityId::AlQuds, ALL_BUILT).unwrap();
        let mut platforms = Platforms::new(Rng::seeded(1));
        platforms.at_mut(CityId::AlQuds).join(Passenger { destination: CityId::Yafa });

        let mut manifest = Manifest::new();
        assert_eq!(manifest.board_at(&mut platforms, CityId::AlQuds, &route), 0);
    }

    #[test]
    fn the_same_seed_lays_out_the_same_traffic() {
        let net = Network::new();
        let run = |seed: u64| {
            let mut platforms = Platforms::new(Rng::seeded(seed));
            for _ in 0..25 {
                platforms.tick(&net, ALL_BUILT);
            }
            platforms.total_waiting()
        };
        assert_eq!(run(99), run(99));
    }
}
