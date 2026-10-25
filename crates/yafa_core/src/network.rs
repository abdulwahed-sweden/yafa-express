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
const TRACKS: [(CityId, CityId, LineType, u32, Opening); 12] = [
    // The coastal corridor, north to south.
    (CityId::Akka, CityId::Haifa, LineType::Coastal, 23_000, Opening::Opened(1913)),
    (CityId::Haifa, CityId::Tulkarm, LineType::Coastal, 68_000, Opening::Opened(1915)),
    (CityId::Tulkarm, CityId::AlLadd, LineType::Coastal, 52_000, Opening::Opened(1915)),
    (CityId::AlLadd, CityId::Yafa, LineType::Coastal, 19_000, Opening::Opened(1892)),
    (CityId::AlLadd, CityId::Ghazzah, LineType::Coastal, 74_000, Opening::Opened(1917)),
    // The links that tie the coast to the highlands.
    (CityId::AlLadd, CityId::AlQuds, LineType::Junction, 67_000, Opening::Opened(1892)),
    (CityId::Tulkarm, CityId::Nablus, LineType::Junction, 32_000, Opening::Opened(1915)),
    // The inland corridor, Galilee down the spine of the highlands.
    (CityId::Haifa, CityId::Beisan, LineType::Mountain, 51_000, Opening::Opened(1904)),
    (CityId::Beisan, CityId::Nablus, LineType::Mountain, 58_000, Opening::Opened(1914)),
    (CityId::Beisan, CityId::Safad, LineType::Mountain, 44_000, Opening::Projected),
    (CityId::Nablus, CityId::AlQuds, LineType::Mountain, 63_000, Opening::Projected),
    (CityId::AlQuds, CityId::AlKhalil, LineType::Mountain, 36_000, Opening::Projected),
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

    /// Shortest route by track distance, using Dijkstra over integer metres.
    ///
    /// Returns `None` only if `from == to`; the graph itself is connected.
    pub fn route(&self, from: CityId, to: CityId) -> Option<Route> {
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
    fn every_station_is_reachable_from_every_other() {
        let net = Network::new();
        for &from in &CityId::ALL {
            for &to in &CityId::ALL {
                if from == to {
                    assert!(net.route(from, to).is_none());
                    continue;
                }
                let route = net.route(from, to).unwrap_or_else(|| {
                    panic!("no route from {} to {}", net.city(from).name, net.city(to).name)
                });
                assert_eq!(route.origin(), from);
                assert_eq!(route.destination(), to);
                assert!(route.total_m > 0);
            }
        }
    }

    #[test]
    fn route_legs_form_an_unbroken_chain() {
        let net = Network::new();
        let route = net.route(CityId::Akka, CityId::AlKhalil).expect("route exists");
        for pair in route.legs.windows(2) {
            assert_eq!(pair[0].to, pair[1].from, "legs do not join up");
        }
        let summed: u32 = route.legs.iter().map(|leg| leg.track_m).sum();
        assert_eq!(summed, route.total_m);
    }

    #[test]
    fn coast_to_mountain_goes_through_a_junction() {
        let net = Network::new();
        let route = net.route(CityId::Yafa, CityId::AlQuds).expect("route exists");
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
        // Haifa to Nablus: via Beisan is 51+58=109 km, via Tulkarm is 68+32=100 km.
        let route = net.route(CityId::Haifa, CityId::Nablus).expect("route exists");
        assert_eq!(
            route.stops().collect::<Vec<_>>(),
            vec![CityId::Haifa, CityId::Tulkarm, CityId::Nablus]
        );
        assert_eq!(route.total_m, 100_000);
    }

    #[test]
    fn routes_are_symmetric_in_length() {
        let net = Network::new();
        for &from in &CityId::ALL {
            for &to in &CityId::ALL {
                if from == to {
                    continue;
                }
                let there = net.route(from, to).unwrap();
                let back = net.route(to, from).unwrap();
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
