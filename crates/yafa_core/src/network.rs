//! The rail network as a graph: stations are nodes, track segments are edges.
//!
//! The graph is immutable once built, and everything outside this module refers
//! to it with [`CityId`] / [`SegmentId`] handles. Nothing stores a reference
//! into the network, so no system ever fights the borrow checker over it.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use serde::{Deserialize, Serialize};

use crate::city::{CITIES, CITY_COUNT, City, CityId, LineType};
use crate::geo::MapPoint;

/// Handle to a track segment in [`Network::segments`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SegmentId(pub usize);

/// Whether a stretch of track was actually laid, or only ever surveyed.
///
/// The map draws the two differently: the network as it was, and the network as
/// it was imagined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opening {
    /// Carried traffic, from the given year.
    Opened(u16),
    /// Surveyed or begun, but never completed. The train runs it anyway.
    Projected,
}

impl Opening {
    pub fn label(self) -> String {
        match self {
            Opening::Opened(year) => format!("opened {year}"),
            Opening::Projected => "projected, never completed".to_owned(),
        }
    }

    pub const fn was_built(self) -> bool {
        matches!(self, Opening::Opened(_))
    }

    /// Whether trains may run over this track in the given year.
    ///
    /// Projected track is never open on its own account. It becomes
    /// `Opened(current year)` only when the player pays to lay it.
    pub const fn is_open(self, year: u16) -> bool {
        match self {
            Opening::Opened(opened) => opened <= year,
            Opening::Projected => false,
        }
    }

    /// The year the line opened, if it ever did.
    pub const fn year(self) -> Option<u16> {
        match self {
            Opening::Opened(year) => Some(year),
            Opening::Projected => None,
        }
    }
}

/// A bidirectional stretch of track between two stations.
#[derive(Debug, Clone)]
pub struct Segment {
    pub id: SegmentId,
    pub from: CityId,
    pub to: CityId,
    pub line: LineType,
    /// Length of track in metres - the real route, not the crow-flies distance.
    pub track_m: u32,
    pub opening: Opening,
}

impl Segment {
    pub fn track_km(&self) -> f32 {
        self.track_m as f32 / 1000.0
    }

    /// The station at the far end of this segment, seen from `from`.
    pub fn other_end(&self, from: CityId) -> Option<CityId> {
        if from == self.from {
            Some(self.to)
        } else if from == self.to {
            Some(self.from)
        } else {
            None
        }
    }
}

/// One outgoing edge from a station.
#[derive(Debug, Clone, Copy)]
pub struct Link {
    pub to: CityId,
    pub segment: SegmentId,
}

/// The track table. Order is stable; `SegmentId` is the index into it.
///
/// Where a real line ran through a town, the town is a station on it rather
/// than a detour around it: the Jezreel valley railway is Haifa to Al-Nasira
/// to Beisan, not Haifa to Beisan, because that is the ground it covered.
const TRACKS: [(CityId, CityId, LineType, u32, Opening); 26] = [
    // The coastal corridor, north to south.
    (CityId::Akka, CityId::Haifa, LineType::Coastal, 23_000, Opening::Opened(1913)),
    (CityId::Haifa, CityId::Tulkarm, LineType::Coastal, 68_000, Opening::Opened(1915)),
    (CityId::Tulkarm, CityId::AlLadd, LineType::Coastal, 52_000, Opening::Opened(1915)),
    (CityId::AlLadd, CityId::Yafa, LineType::Coastal, 19_000, Opening::Opened(1892)),
    // The main line south, which the 1917 military railway pushed to the
    // Egyptian frontier.
    (CityId::AlLadd, CityId::Yibna, LineType::Coastal, 18_000, Opening::Opened(1917)),
    (CityId::Yibna, CityId::Isdud, LineType::Coastal, 16_000, Opening::Opened(1917)),
    (CityId::Isdud, CityId::AlMajdal, LineType::Coastal, 14_000, Opening::Opened(1917)),
    (CityId::AlMajdal, CityId::Ghazzah, LineType::Coastal, 23_000, Opening::Opened(1917)),
    (CityId::Ghazzah, CityId::KhanYunis, LineType::Coastal, 24_000, Opening::Opened(1917)),
    (CityId::KhanYunis, CityId::Rafah, LineType::Coastal, 10_000, Opening::Opened(1917)),
    // The coastal line proper, surveyed again and again and never laid.
    (CityId::Haifa, CityId::Qaisariya, LineType::Coastal, 40_000, Opening::Projected),
    (CityId::Qaisariya, CityId::Yafa, LineType::Coastal, 55_000, Opening::Projected),
    // The links that tie the coast to the interior.
    (CityId::AlLadd, CityId::AlQuds, LineType::Junction, 67_000, Opening::Opened(1892)),
    (CityId::Tulkarm, CityId::Nablus, LineType::Junction, 32_000, Opening::Opened(1915)),
    (CityId::Ghazzah, CityId::BirAlSabi, LineType::Junction, 48_000, Opening::Opened(1915)),
    // The Jezreel valley railway, and the branch it threw south.
    (CityId::Haifa, CityId::AlNasira, LineType::Mountain, 33_000, Opening::Opened(1904)),
    (CityId::AlNasira, CityId::Beisan, LineType::Mountain, 28_000, Opening::Opened(1904)),
    (CityId::AlNasira, CityId::Jenin, LineType::Mountain, 28_000, Opening::Opened(1913)),
    (CityId::Jenin, CityId::Nablus, LineType::Mountain, 29_000, Opening::Opened(1914)),
    // Up the spine of the highlands - mostly drawn, barely built.
    (CityId::Beisan, CityId::Tabariya, LineType::Mountain, 35_000, Opening::Projected),
    (CityId::Tabariya, CityId::Safad, LineType::Mountain, 24_000, Opening::Projected),
    (CityId::Nablus, CityId::Ramallah, LineType::Mountain, 40_000, Opening::Projected),
    (CityId::Ramallah, CityId::AlQuds, LineType::Mountain, 15_000, Opening::Projected),
    (CityId::AlQuds, CityId::BaytLahm, LineType::Mountain, 11_000, Opening::Opened(1915)),
    (CityId::BaytLahm, CityId::AlKhalil, LineType::Mountain, 24_000, Opening::Projected),
    (CityId::BirAlSabi, CityId::AlKhalil, LineType::Mountain, 46_000, Opening::Projected),
];

/// The whole map: stations, track, and the adjacency used for routing.
#[derive(Debug, Clone)]
pub struct Network {
    cities: [City; CITY_COUNT],
    segments: Vec<Segment>,
    adjacency: [Vec<Link>; CITY_COUNT],
    /// Cached projected positions, so the render layer never re-projects.
    positions: [MapPoint; CITY_COUNT],
}

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}

impl Network {
    /// Builds the historical network. Cheap enough to call once at startup.
    pub fn new() -> Self {
        let segments: Vec<Segment> = TRACKS
            .iter()
            .enumerate()
            .map(|(index, &(from, to, line, track_m, opening))| Segment {
                id: SegmentId(index),
                from,
                to,
                line,
                track_m,
                opening,
            })
            .collect();

        let mut adjacency: [Vec<Link>; CITY_COUNT] = std::array::from_fn(|_| Vec::new());
        for segment in &segments {
            adjacency[segment.from.index()].push(Link { to: segment.to, segment: segment.id });
            adjacency[segment.to.index()].push(Link { to: segment.from, segment: segment.id });
        }

        let positions = std::array::from_fn(|i| CITIES[i].coordinates.to_map());

        Self { cities: CITIES, segments, adjacency, positions }
    }

    pub fn city(&self, id: CityId) -> &City {
        &self.cities[id.index()]
    }

    pub fn cities(&self) -> &[City; CITY_COUNT] {
        &self.cities
    }

    pub fn segment(&self, id: SegmentId) -> &Segment {
        &self.segments[id.0]
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Projected map position of a station, in kilometres from the map origin.
    pub fn position(&self, id: CityId) -> MapPoint {
        self.positions[id.index()]
    }

    pub fn links(&self, from: CityId) -> &[Link] {
        &self.adjacency[from.index()]
    }

    /// Stations served by a corridor, in table order.
    pub fn cities_on(&self, line: LineType) -> impl Iterator<Item = &City> {
        self.cities.iter().filter(move |c| c.line_type == line)
    }

    /// Lays a surveyed line, bringing it into service this year.
    ///
    /// Returns false if the segment was already built - buying a line twice is
    /// a bug in the caller, not a transaction.
    pub fn open_line(&mut self, id: SegmentId, year: u16) -> bool {
        let segment = &mut self.segments[id.0];
        if segment.opening != Opening::Projected {
            return false;
        }
        segment.opening = Opening::Opened(year);
        true
    }

    /// Every stretch still only drawn on paper.
    pub fn projected(&self) -> impl Iterator<Item = &Segment> {
        self.segments.iter().filter(|s| s.opening == Opening::Projected)
    }

    /// Whether a stretch of track carries traffic in the given year.
    pub fn is_open(&self, segment: SegmentId, year: u16) -> bool {
        self.segment(segment).opening.is_open(year)
    }

    /// Whether a station has any track at all in the given year.
    ///
    /// A station with none is on the map but not on the network: it cannot be
    /// ticketed, and it is drawn washed out.
    pub fn station_is_open(&self, city: CityId, year: u16) -> bool {
        self.links(city).iter().any(|link| self.is_open(link.segment, year))
    }

    /// Every station the network reaches in the given year.
    pub fn open_stations(&self, year: u16) -> impl Iterator<Item = CityId> + '_ {
        CityId::ALL.into_iter().filter(move |&city| self.station_is_open(city, year))
    }

    /// The lines that opened in exactly this year, for the announcer.
    pub fn segments_opening_in(&self, year: u16) -> impl Iterator<Item = &Segment> {
        self.segments.iter().filter(move |s| s.opening.year() == Some(year))
    }

    /// Shortest route by track distance in the given year, using Dijkstra over
    /// integer metres.
    ///
    /// Returns `None` if `from == to`, or if no run of open track joins them.
    /// The second case is ordinary: in 1892 only the line from Yafa up to
    /// Al-Quds exists, and most of the map is unreachable.
    pub fn route(&self, from: CityId, to: CityId, year: u16) -> Option<Route> {
        if from == to {
            return None;
        }

        const UNREACHED: u32 = u32::MAX;
        let mut best = [UNREACHED; CITY_COUNT];
        // How we arrived at each station: (previous station, segment taken).
        let mut came_from: [Option<(CityId, SegmentId)>; CITY_COUNT] = [None; CITY_COUNT];
        let mut queue = BinaryHeap::new();

        best[from.index()] = 0;
        queue.push(Reverse((0u32, from.index())));

        while let Some(Reverse((cost, index))) = queue.pop() {
            // Stale entry: we already found a cheaper way here.
            if cost > best[index] {
                continue;
            }
            let here = CityId::ALL[index];
            if here == to {
                break;
            }
            for link in self.links(here) {
                if !self.is_open(link.segment, year) {
                    continue;
                }
                let next_cost = cost + self.segment(link.segment).track_m;
                let next_index = link.to.index();
                if next_cost < best[next_index] {
                    best[next_index] = next_cost;
                    came_from[next_index] = Some((here, link.segment));
                    queue.push(Reverse((next_cost, next_index)));
                }
            }
        }

        if best[to.index()] == UNREACHED {
            return None;
        }

        // Walk the predecessor chain back to the origin, then flip it.
        let mut legs = Vec::new();
        let mut cursor = to;
        while let Some((previous, segment_id)) = came_from[cursor.index()] {
            let segment = self.segment(segment_id);
            legs.push(Leg {
                from: previous,
                to: cursor,
                segment: segment_id,
                track_m: segment.track_m,
                line: segment.line,
            });
            cursor = previous;
        }
        legs.reverse();

        Some(Route { total_m: best[to.index()], legs })
    }
}

/// One station-to-station hop of a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leg {
    pub from: CityId,
    pub to: CityId,
    pub segment: SegmentId,
    pub track_m: u32,
    pub line: LineType,
}

impl Leg {
    pub fn track_km(&self) -> f32 {
        self.track_m as f32 / 1000.0
    }
}

/// A planned journey from one station to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub legs: Vec<Leg>,
    pub total_m: u32,
}

impl Route {
    pub fn origin(&self) -> CityId {
        self.legs[0].from
    }

    pub fn destination(&self) -> CityId {
        self.legs[self.legs.len() - 1].to
    }

    /// Every station called at, origin first, destination last.
    pub fn stops(&self) -> impl Iterator<Item = CityId> + '_ {
        std::iter::once(self.origin()).chain(self.legs.iter().map(|leg| leg.to))
    }

    pub fn total_km(&self) -> f32 {
        self.total_m as f32 / 1000.0
    }

    /// Intermediate stations - the ones a traveller might not have expected.
    pub fn intermediate_stops(&self) -> impl Iterator<Item = CityId> + '_ {
        self.legs[..self.legs.len() - 1].iter().map(|leg| leg.to)
    }

    /// Track between two stops of this route, in metres.
    ///
    /// A passenger pays for the distance they were carried along the train's
    /// own line, which is not always the shortest way between the two: the
    /// train goes where its ticket sends it.
    pub fn distance_between(&self, from: CityId, to: CityId) -> Option<u32> {
        let stops: Vec<CityId> = self.stops().collect();
        let start = stops.iter().position(|&stop| stop == from)?;
        let end = stops.iter().position(|&stop| stop == to)?;
        if end <= start {
            return None;
        }
        Some(self.legs[start..end].iter().map(|leg| leg.track_m).sum())
    }

    /// Corridors this route touches, in the order it first enters them.
    pub fn lines_used(&self) -> Vec<LineType> {
        let mut used = Vec::new();
        for leg in &self.legs {
            if !used.contains(&leg.line) {
                used.push(leg.line);
            }
        }
        used
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_surveyed_line_can_be_laid_once() {
        let mut net = Network::new();
        let id = net.projected().next().expect("some line is still on paper").id;
        assert!(!net.is_open(id, 1948));

        assert!(net.open_line(id, 1920));
        assert!(net.is_open(id, 1920));
        assert!(!net.is_open(id, 1919), "it opened in 1920, not before");

        // And it cannot be bought again.
        assert!(!net.open_line(id, 1921));
    }

    #[test]
    fn laying_a_line_puts_its_stations_on_the_network() {
        let mut net = Network::new();
        assert!(!net.station_is_open(CityId::Safad, 1948));
        let to_safad = net
            .projected()
            .find(|s| s.from == CityId::Tabariya && s.to == CityId::Safad)
            .expect("Tabariya to Safad is surveyed")
            .id;
        net.open_line(to_safad, 1920);
        assert!(net.station_is_open(CityId::Safad, 1920));
    }

    /// By 1918 every line that was ever built is open. Projected track still
    /// is not, and never is until it is paid for.
    const ALL_BUILT: u16 = 1918;

    #[test]
    fn every_station_on_open_track_is_reachable_from_every_other() {
        let net = Network::new();
        let open: Vec<CityId> = net.open_stations(ALL_BUILT).collect();
        assert!(open.len() > 15, "most of the network should be built by {ALL_BUILT}");

        for &from in &open {
            for &to in &open {
                if from == to {
                    assert!(net.route(from, to, ALL_BUILT).is_none());
                    continue;
                }
                let route = net.route(from, to, ALL_BUILT).unwrap_or_else(|| {
                    panic!("no route from {} to {}", net.city(from).name, net.city(to).name)
                });
                assert_eq!(route.origin(), from);
                assert_eq!(route.destination(), to);
                assert!(route.total_m > 0);
            }
        }
    }

    #[test]
    fn stations_reached_only_by_projected_track_are_off_the_network() {
        let net = Network::new();
        // Nothing but surveyed line runs to these, so no year opens them.
        for city in [CityId::Safad, CityId::Tabariya, CityId::Ramallah, CityId::Qaisariya] {
            assert!(!net.station_is_open(city, ALL_BUILT), "{city:?} should be shut");
            assert!(net.route(CityId::Yafa, city, ALL_BUILT).is_none());
        }
    }

    #[test]
    fn the_first_line_is_the_only_line_in_1892() {
        let net = Network::new();
        let route = net.route(CityId::Yafa, CityId::AlQuds, 1892).expect("the 1892 line");
        assert_eq!(
            route.stops().collect::<Vec<_>>(),
            vec![CityId::Yafa, CityId::AlLadd, CityId::AlQuds]
        );
        // And it really is the only one: three stations on the whole map.
        assert_eq!(net.open_stations(1892).count(), 3);
    }

    #[test]
    fn akka_waits_for_its_branch() {
        let net = Network::new();
        // The branch up from Haifa opened in 1913, and not a year before.
        assert!(!net.station_is_open(CityId::Akka, 1900));
        assert!(net.route(CityId::Haifa, CityId::Akka, 1900).is_none());

        assert!(net.station_is_open(CityId::Akka, 1913));
        let route = net.route(CityId::Haifa, CityId::Akka, 1913).expect("open by 1913");
        assert_eq!(route.total_m, 23_000);
    }

    #[test]
    fn a_line_announces_itself_in_the_year_it_opens() {
        let net = Network::new();
        let opened: Vec<_> = net.segments_opening_in(1913).collect();
        assert!(opened.iter().any(|s| { (s.from, s.to) == (CityId::Akka, CityId::Haifa) }));
        assert!(net.segments_opening_in(1891).next().is_none());
    }

    #[test]
    fn route_legs_form_an_unbroken_chain() {
        let net = Network::new();
        let route = net.route(CityId::Akka, CityId::Rafah, ALL_BUILT).expect("route exists");
        for pair in route.legs.windows(2) {
            assert_eq!(pair[0].to, pair[1].from, "legs do not join up");
        }
        let summed: u32 = route.legs.iter().map(|leg| leg.track_m).sum();
        assert_eq!(summed, route.total_m);
    }

    #[test]
    fn coast_to_mountain_goes_through_a_junction() {
        let net = Network::new();
        let route = net.route(CityId::Yafa, CityId::AlQuds, ALL_BUILT).expect("route exists");
        // Yafa hangs off Al-Ladd, so the junction must appear in the middle.
        assert_eq!(
            route.stops().collect::<Vec<_>>(),
            vec![CityId::Yafa, CityId::AlLadd, CityId::AlQuds]
        );
        assert!(route.lines_used().contains(&LineType::Junction));
    }

    #[test]
    fn dijkstra_picks_the_shorter_of_two_corridors() {
        let net = Network::new();
        // Haifa to Nablus: down the valley line and its Jenin branch is
        // 33+28+29 = 90 km; round by the coast through Tulkarm is 68+32 = 100.
        // The branch wins, which is the way the trains actually went.
        let route = net.route(CityId::Haifa, CityId::Nablus, ALL_BUILT).expect("route exists");
        assert_eq!(
            route.stops().collect::<Vec<_>>(),
            vec![CityId::Haifa, CityId::AlNasira, CityId::Jenin, CityId::Nablus]
        );
        assert_eq!(route.total_m, 90_000);
    }

    #[test]
    fn a_fare_is_measured_along_the_train_s_own_line() {
        let net = Network::new();
        let route = net.route(CityId::AlLadd, CityId::Rafah, ALL_BUILT).unwrap();
        // Al-Ladd to Al-Majdal, taken in order: 18 + 16 + 14.
        assert_eq!(route.distance_between(CityId::AlLadd, CityId::AlMajdal), Some(48_000));
        // Backwards along the route is not a fare.
        assert_eq!(route.distance_between(CityId::AlMajdal, CityId::AlLadd), None);
        // Nor is a station the train does not call at.
        assert_eq!(route.distance_between(CityId::AlLadd, CityId::Haifa), None);
    }

    #[test]
    fn a_town_on_a_line_is_a_stop_on_it() {
        let net = Network::new();
        // The 1917 line south ran through these towns rather than past them.
        let route = net.route(CityId::AlLadd, CityId::Rafah, ALL_BUILT).expect("route exists");
        assert_eq!(
            route.stops().collect::<Vec<_>>(),
            vec![
                CityId::AlLadd,
                CityId::Yibna,
                CityId::Isdud,
                CityId::AlMajdal,
                CityId::Ghazzah,
                CityId::KhanYunis,
                CityId::Rafah,
            ]
        );
    }

    #[test]
    fn routes_are_symmetric_in_length() {
        let net = Network::new();
        let open: Vec<CityId> = net.open_stations(ALL_BUILT).collect();
        for &from in &open {
            for &to in &open {
                if from == to {
                    continue;
                }
                let there = net.route(from, to, ALL_BUILT).unwrap();
                let back = net.route(to, from, ALL_BUILT).unwrap();
                assert_eq!(there.total_m, back.total_m);
            }
        }
    }

    #[test]
    fn adjacency_is_bidirectional() {
        let net = Network::new();
        for segment in net.segments() {
            assert!(net.links(segment.from).iter().any(|l| l.to == segment.to));
            assert!(net.links(segment.to).iter().any(|l| l.to == segment.from));
            assert_eq!(segment.other_end(segment.from), Some(segment.to));
            assert_eq!(segment.other_end(segment.to), Some(segment.from));
        }
    }
}
