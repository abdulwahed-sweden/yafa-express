//! Geographic primitives and the flat projection used by the map view.

use serde::{Deserialize, Serialize};

/// Mean radius of the Earth in metres.
const EARTH_RADIUS_M: f64 = 6_371_008.8;

/// Latitude of the projection origin (roughly the centre of the network).
const ORIGIN_LAT_DEG: f64 = 32.2;
/// Longitude of the projection origin.
const ORIGIN_LON_DEG: f64 = 35.0;

/// A point on the globe, in decimal degrees.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat_deg: f64,
    pub lon_deg: f64,
}

/// A point on the flat map, in kilometres east (`x`) and north (`y`) of the
/// projection origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapPoint {
    pub x_km: f32,
    pub y_km: f32,
}

impl GeoPoint {
    pub const fn new(lat_deg: f64, lon_deg: f64) -> Self {
        Self { lat_deg, lon_deg }
    }

    /// Equirectangular projection about [`ORIGIN_LAT_DEG`] / [`ORIGIN_LON_DEG`].
    ///
    /// Over a span of ~200 km the distortion is far below the thickness of a
    /// drawn rail line, so the cheap projection is the right one here.
    pub fn to_map(self) -> MapPoint {
        let lat0 = ORIGIN_LAT_DEG.to_radians();
        let x = (self.lon_deg - ORIGIN_LON_DEG).to_radians() * lat0.cos() * EARTH_RADIUS_M;
        let y = (self.lat_deg - ORIGIN_LAT_DEG).to_radians() * EARTH_RADIUS_M;
        MapPoint { x_km: (x / 1000.0) as f32, y_km: (y / 1000.0) as f32 }
    }

    /// Great-circle distance to `other`, in metres.
    pub fn haversine_m(self, other: Self) -> f64 {
        let (lat1, lat2) = (self.lat_deg.to_radians(), other.lat_deg.to_radians());
        let dlat = lat2 - lat1;
        let dlon = (other.lon_deg - self.lon_deg).to_radians();
        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        2.0 * EARTH_RADIUS_M * a.sqrt().asin()
    }
}

impl MapPoint {
    pub const fn new(x_km: f32, y_km: f32) -> Self {
        Self { x_km, y_km }
    }

    /// Linear interpolation; `t` is clamped to `0.0..=1.0`.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            x_km: self.x_km + (other.x_km - self.x_km) * t,
            y_km: self.y_km + (other.y_km - self.y_km) * t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_matches_known_distance() {
        // Jaffa to Jerusalem is about 54 km as the crow flies.
        let yafa = GeoPoint::new(32.055, 34.752);
        let quds = GeoPoint::new(31.783, 35.217);
        let km = yafa.haversine_m(quds) / 1000.0;
        assert!((50.0..58.0).contains(&km), "unexpected distance: {km}");
    }

    #[test]
    fn projection_puts_north_above_and_east_right() {
        let south = GeoPoint::new(31.5, 35.0).to_map();
        let north = GeoPoint::new(32.9, 35.0).to_map();
        let east = GeoPoint::new(32.2, 35.5).to_map();
        assert!(north.y_km > south.y_km);
        assert!(east.x_km > 0.0);
    }
}
