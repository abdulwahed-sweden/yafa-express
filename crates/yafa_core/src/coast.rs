//! The water: the Mediterranean shore, the lake in the north, the Dead Sea in
//! the south, and the river between them.
//!
//! These outlines are schematic - a few dozen points traced at the scale the
//! map is drawn at, not survey data. They exist so the network reads as a
//! place rather than a graph: Haifa sits in the crook of its bay, Yafa and
//! Ghazzah are ports, and the inland line runs down the length of the valley.

use crate::geo::GeoPoint;

/// The Mediterranean shore, north to south, from Ras an-Naqoura down to Rafah.
///
/// The inflection between Akka and Haifa is the bay: the shore runs south from
/// Akka, turns west at the mouth of the Kishon and ends against the headland
/// of Carmel. Both ports sit on it, which is why the line between them is the
/// oldest stretch of coastal track on the map.
pub const SHORE: [GeoPoint; 27] = [
    // The first and last points carry the shore off the edge of the drawn
    // sheet. Without them the water lines - which are this polyline shifted
    // seaward - would end in mid-air at the top and bottom of the view.
    GeoPoint::new(33.45, 35.17),
    GeoPoint::new(33.10, 35.11), // Ras an-Naqoura
    GeoPoint::new(33.00, 35.10),
    GeoPoint::new(32.92, 35.07), // Akka
    GeoPoint::new(32.86, 35.07),
    GeoPoint::new(32.82, 35.03), // the mouth of the Kishon
    GeoPoint::new(32.83, 34.98), // under Carmel, at Haifa
    GeoPoint::new(32.76, 34.95),
    GeoPoint::new(32.70, 34.94), // Atlit
    GeoPoint::new(32.60, 34.91),
    GeoPoint::new(32.50, 34.89), // Qaysariyya
    GeoPoint::new(32.40, 34.86),
    GeoPoint::new(32.33, 34.84),
    GeoPoint::new(32.20, 34.81),
    GeoPoint::new(32.10, 34.77),
    GeoPoint::new(32.05, 34.75), // Yafa
    GeoPoint::new(31.95, 34.70),
    GeoPoint::new(31.85, 34.66),
    GeoPoint::new(31.80, 34.63), // Isdud
    GeoPoint::new(31.72, 34.58),
    GeoPoint::new(31.67, 34.55), // Al-Majdal
    GeoPoint::new(31.58, 34.48),
    GeoPoint::new(31.52, 34.42), // Ghazzah
    GeoPoint::new(31.42, 34.35),
    GeoPoint::new(31.32, 34.28),
    GeoPoint::new(31.25, 34.23), // Rafah
    GeoPoint::new(30.95, 34.05),
];

/// The lake in the Galilee, as a closed ring.
pub const TABARIYYA: [GeoPoint; 8] = [
    GeoPoint::new(32.88, 35.56),
    GeoPoint::new(32.84, 35.62),
    GeoPoint::new(32.78, 35.65),
    GeoPoint::new(32.73, 35.62),
    GeoPoint::new(32.70, 35.57),
    GeoPoint::new(32.72, 35.52),
    GeoPoint::new(32.78, 35.50),
    GeoPoint::new(32.84, 35.52),
];

/// The Dead Sea, as a closed ring. Its southern end runs off the bottom of the
/// drawn map, which is where the land does too.
pub const BAHR_LUT: [GeoPoint; 15] = [
    GeoPoint::new(31.77, 35.47),
    GeoPoint::new(31.74, 35.54),
    GeoPoint::new(31.65, 35.58),
    GeoPoint::new(31.55, 35.55),
    GeoPoint::new(31.48, 35.52),
    GeoPoint::new(31.42, 35.50),
    GeoPoint::new(31.33, 35.48),
    GeoPoint::new(31.24, 35.44),
    GeoPoint::new(31.22, 35.39),
    GeoPoint::new(31.30, 35.40),
    GeoPoint::new(31.38, 35.42),
    GeoPoint::new(31.46, 35.42),
    GeoPoint::new(31.55, 35.42),
    GeoPoint::new(31.65, 35.43),
    GeoPoint::new(31.74, 35.43),
];

/// The Jordan, running from the lake down to the Dead Sea.
pub const JORDAN: [GeoPoint; 9] = [
    GeoPoint::new(32.70, 35.57),
    GeoPoint::new(32.60, 35.57),
    GeoPoint::new(32.50, 35.55),
    GeoPoint::new(32.40, 35.54),
    GeoPoint::new(32.30, 35.55),
    GeoPoint::new(32.10, 35.54),
    GeoPoint::new(31.95, 35.53),
    GeoPoint::new(31.85, 35.52),
    GeoPoint::new(31.78, 35.50),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shore_runs_south_except_where_the_bay_turns_it() {
        // Haifa Bay is the one place the coast heads back north: the shore
        // leaves the Kishon and runs up to the headland of Carmel. Anything
        // more than that small step would be a typo in the table.
        let mut northward_steps = 0;
        for pair in SHORE.windows(2) {
            let step = pair[1].lat_deg - pair[0].lat_deg;
            if step > 0.0 {
                northward_steps += 1;
                assert!(step < 0.05, "the shore doubles back at {:?}", pair[1]);
            }
        }
        assert_eq!(northward_steps, 1, "the bay should be the only inflection");
    }

    #[test]
    fn the_shore_passes_the_ports_it_should() {
        // Each coastal station should have a point of shore within a few km.
        for (name, port) in [
            ("Akka", GeoPoint::new(32.928, 35.082)),
            ("Haifa", GeoPoint::new(32.818, 34.989)),
            ("Yafa", GeoPoint::new(32.054, 34.752)),
            ("Ghazzah", GeoPoint::new(31.504, 34.466)),
        ] {
            let nearest = SHORE.iter().map(|p| p.haversine_m(port)).fold(f64::MAX, f64::min);
            assert!(nearest < 6_000.0, "{name} is {nearest:.0} m from the drawn shore");
        }
    }

    #[test]
    fn the_jordan_joins_the_lake_to_the_dead_sea() {
        let source = JORDAN[0];
        let mouth = JORDAN[JORDAN.len() - 1];
        let to_lake = TABARIYYA.iter().map(|p| p.haversine_m(source)).fold(f64::MAX, f64::min);
        let to_sea = BAHR_LUT.iter().map(|p| p.haversine_m(mouth)).fold(f64::MAX, f64::min);
        assert!(to_lake < 1_000.0, "the river does not start at the lake");
        assert!(to_sea < 6_000.0, "the river does not reach the Dead Sea");
    }
}
