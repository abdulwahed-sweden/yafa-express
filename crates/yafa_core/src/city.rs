//! The stations of the network and the historical lines they belong to.
//!
//! Cities are addressed through the [`CityId`] enum rather than by reference.
//! Every system in the game therefore stores a plain `Copy` identifier, and the
//! single owning [`crate::Network`] can be borrowed immutably wherever the data
//! is actually needed - which is what keeps the borrow checker out of the way.

use serde::{Deserialize, Serialize};

use crate::geo::GeoPoint;

/// One of the three historical corridors of the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LineType {
    /// The Mediterranean corridor, Akka down to Ghazzah.
    Coastal,
    /// The inland corridor through Galilee and the central highlands.
    Mountain,
    /// The connecting stations that tie the two corridors together.
    Junction,
}

impl LineType {
    pub const ALL: [LineType; 3] = [LineType::Coastal, LineType::Mountain, LineType::Junction];

    /// Display name used on tickets, the station board and the map legend.
    pub const fn label(self) -> &'static str {
        match self {
            LineType::Coastal => "Coastal Line",
            LineType::Mountain => "Mountain Line",
            LineType::Junction => "Junction Links",
        }
    }

    /// Short code printed on the ticket and the passport stamp.
    pub const fn code(self) -> &'static str {
        match self {
            LineType::Coastal => "CL",
            LineType::Mountain => "ML",
            LineType::Junction => "JX",
        }
    }

    /// Line colour as linear-sRGB bytes, so that the rendering layer does not
    /// have to own the palette decisions of the map.
    pub const fn rgb(self) -> [u8; 3] {
        match self {
            LineType::Coastal => [58, 141, 160],  // sea blue-green
            LineType::Mountain => [173, 118, 62], // terraced hillside ochre
            LineType::Junction => [196, 164, 84], // brass junction plate
        }
    }
}

/// Every station on the network.
///
/// The discriminants are stable: they are the index into
/// [`Network::cities`](crate::Network::cities) and the on-disk passport format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum CityId {
    Akka = 0,
    Haifa = 1,
    Safad = 2,
    Beisan = 3,
    Tulkarm = 4,
    Nablus = 5,
    Yafa = 6,
    AlLadd = 7,
    AlQuds = 8,
    AlKhalil = 9,
    Ghazzah = 10,
}

/// Number of stations on the network.
pub const CITY_COUNT: usize = 11;

impl CityId {
    /// All stations, in the order they are stored in the network.
    pub const ALL: [CityId; CITY_COUNT] = [
        CityId::Akka,
        CityId::Haifa,
        CityId::Safad,
        CityId::Beisan,
        CityId::Tulkarm,
        CityId::Nablus,
        CityId::Yafa,
        CityId::AlLadd,
        CityId::AlQuds,
        CityId::AlKhalil,
        CityId::Ghazzah,
    ];

    /// Index into the city table. Infallible by construction.
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Inverse of [`CityId::index`].
    pub const fn from_index(index: usize) -> Option<Self> {
        if index < CITY_COUNT { Some(Self::ALL[index]) } else { None }
    }

    /// Three-letter telegraph code, as stations carried on timetables and as
    /// the passport prints on each stamp.
    pub const fn code(self) -> &'static str {
        match self {
            CityId::Akka => "AKA",
            CityId::Haifa => "HFA",
            CityId::Safad => "SFD",
            CityId::Beisan => "BSN",
            CityId::Tulkarm => "TLK",
            CityId::Nablus => "NBS",
            CityId::Yafa => "YFA",
            CityId::AlLadd => "LDD",
            CityId::AlQuds => "QDS",
            CityId::AlKhalil => "KHL",
            CityId::Ghazzah => "GZA",
        }
    }

    /// Stable lowercase identifier, used for save files and asset lookups.
    pub const fn slug(self) -> &'static str {
        match self {
            CityId::Akka => "akka",
            CityId::Haifa => "haifa",
            CityId::Safad => "safad",
            CityId::Beisan => "beisan",
            CityId::Tulkarm => "tulkarm",
            CityId::Nablus => "nablus",
            CityId::Yafa => "yafa",
            CityId::AlLadd => "al-ladd",
            CityId::AlQuds => "al-quds",
            CityId::AlKhalil => "al-khalil",
            CityId::Ghazzah => "ghazzah",
        }
    }
}

/// A station: who it is, where it is, and why it matters.
#[derive(Debug, Clone)]
pub struct City {
    pub id: CityId,
    /// Romanised Arabic name, as it is printed on the ticket.
    pub name: &'static str,
    /// The name the station carried on European timetables, where it differs.
    pub exonym: Option<&'static str>,
    /// Real-world position, projected for the map view.
    pub coordinates: GeoPoint,
    /// The corridor this station belongs to.
    pub line_type: LineType,
    /// One sentence of context, shown in the "About the City" panel.
    pub historical_fact: &'static str,
}

/// The station table. Index `i` always holds `CityId::ALL[i]`.
pub(crate) const CITIES: [City; CITY_COUNT] = [
    City {
        id: CityId::Akka,
        name: "Akka",
        exonym: Some("Acre"),
        coordinates: GeoPoint::new(32.928, 35.082),
        line_type: LineType::Coastal,
        historical_fact: "A walled Mediterranean port whose sea ramparts turned back Napoleon's \
                          siege of 1799. A branch line reached it from Haifa in 1913.",
    },
    City {
        id: CityId::Haifa,
        name: "Haifa",
        exonym: None,
        coordinates: GeoPoint::new(32.818, 34.989),
        line_type: LineType::Coastal,
        historical_fact: "Western terminus of the Jezreel Valley Railway, opened in 1905 to carry \
                          the Mediterranean trade inland to the Hejaz main line at Dera'a.",
    },
    City {
        id: CityId::Safad,
        name: "Safad",
        exonym: Some("Safed"),
        coordinates: GeoPoint::new(32.965, 35.497),
        line_type: LineType::Mountain,
        historical_fact: "A hill town of Upper Galilee some 900 metres above the sea, long a \
                          centre of scholarship, printing and the copying of manuscripts.",
    },
    City {
        id: CityId::Beisan,
        name: "Beisan",
        exonym: Some("Beth Shean"),
        coordinates: GeoPoint::new(32.497, 35.500),
        line_type: LineType::Mountain,
        historical_fact: "Set in the Jordan Valley well below sea level; its stop on the Jezreel \
                          Valley line was among the lowest-lying railway stations on earth.",
    },
    City {
        id: CityId::Tulkarm,
        name: "Tulkarm",
        exonym: None,
        coordinates: GeoPoint::new(32.310, 35.028),
        line_type: LineType::Junction,
        historical_fact: "Where the coastal main line met the branch that climbed east into the \
                          highlands towards Nablus, laid during the First World War.",
    },
    City {
        id: CityId::Nablus,
        name: "Nablus",
        exonym: None,
        coordinates: GeoPoint::new(32.221, 35.254),
        line_type: LineType::Mountain,
        historical_fact: "Seated between Mount Ebal and Mount Gerizim, famed for its olive-oil \
                          soap, its kunafa, and the spring markets of the central highlands.",
    },
    City {
        id: CityId::Yafa,
        name: "Yafa",
        exonym: Some("Jaffa"),
        coordinates: GeoPoint::new(32.054, 34.752),
        line_type: LineType::Coastal,
        historical_fact: "The Bride of the Sea. Its Shamouti oranges sailed from these quays, and \
                          in 1892 the country's first railway ran from here up to Al-Quds.",
    },
    City {
        id: CityId::AlLadd,
        name: "Al-Ladd",
        exonym: Some("Lydda"),
        coordinates: GeoPoint::new(31.951, 34.896),
        line_type: LineType::Junction,
        historical_fact: "The great junction of the network, where the Yafa-Al-Quds line crossed \
                          the main line running north to Haifa and south to Ghazzah.",
    },
    City {
        id: CityId::AlQuds,
        name: "Al-Quds",
        exonym: Some("Jerusalem"),
        coordinates: GeoPoint::new(31.783, 35.217),
        line_type: LineType::Mountain,
        historical_fact: "Terminus of the 1892 line from the coast. Its station stood in the \
                          Baq'a quarter, south-west of the old walled city.",
    },
    City {
        id: CityId::AlKhalil,
        name: "Al-Khalil",
        exonym: Some("Hebron"),
        coordinates: GeoPoint::new(31.529, 35.098),
        line_type: LineType::Mountain,
        historical_fact: "A hill town of vineyards and glassblowers. Wartime construction pushed \
                          rails south from Al-Quds through its district in 1915.",
    },
    City {
        id: CityId::Ghazzah,
        name: "Ghazzah",
        exonym: Some("Gaza"),
        coordinates: GeoPoint::new(31.504, 34.466),
        line_type: LineType::Coastal,
        historical_fact: "A caravan city on the Via Maris between Egypt and Syria; the military \
                          railway of 1917 tied it back into the coastal line.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_table_is_indexed_by_id() {
        for (index, city) in CITIES.iter().enumerate() {
            assert_eq!(city.id.index(), index, "{} is out of place", city.name);
            assert_eq!(CityId::from_index(index), Some(city.id));
        }
        assert_eq!(CityId::from_index(CITY_COUNT), None);
    }

    #[test]
    fn slugs_and_codes_are_unique() {
        for mut names in [
            CityId::ALL.iter().map(|c| c.slug()).collect::<Vec<_>>(),
            CityId::ALL.iter().map(|c| c.code()).collect::<Vec<_>>(),
        ] {
            names.sort_unstable();
            let before = names.len();
            names.dedup();
            assert_eq!(names.len(), before, "duplicate identifier");
        }
    }

    #[test]
    fn codes_are_three_letters() {
        for city in CityId::ALL {
            assert_eq!(city.code().len(), 3, "{} has an odd code", city.slug());
        }
    }
}
