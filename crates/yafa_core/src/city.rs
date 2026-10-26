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
    // Appended, never reordered: the discriminants are the on-disk passport
    // format, and a saved game must keep meaning what it meant.
    Rafah = 11,
    KhanYunis = 12,
    AlMajdal = 13,
    Isdud = 14,
    Yibna = 15,
    Qaisariya = 16,
    BaytLahm = 17,
    Ramallah = 18,
    Jenin = 19,
    AlNasira = 20,
    Tabariya = 21,
    BirAlSabi = 22,
}

/// Number of stations on the network.
pub const CITY_COUNT: usize = 23;

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
        CityId::Rafah,
        CityId::KhanYunis,
        CityId::AlMajdal,
        CityId::Isdud,
        CityId::Yibna,
        CityId::Qaisariya,
        CityId::BaytLahm,
        CityId::Ramallah,
        CityId::Jenin,
        CityId::AlNasira,
        CityId::Tabariya,
        CityId::BirAlSabi,
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
            CityId::Rafah => "RFH",
            CityId::KhanYunis => "KYS",
            CityId::AlMajdal => "MJD",
            CityId::Isdud => "ISD",
            CityId::Yibna => "YBN",
            CityId::Qaisariya => "QSR",
            CityId::BaytLahm => "BLM",
            CityId::Ramallah => "RAM",
            CityId::Jenin => "JEN",
            CityId::AlNasira => "NSR",
            CityId::Tabariya => "TBR",
            CityId::BirAlSabi => "BSB",
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
            CityId::Rafah => "rafah",
            CityId::KhanYunis => "khan-yunis",
            CityId::AlMajdal => "al-majdal",
            CityId::Isdud => "isdud",
            CityId::Yibna => "yibna",
            CityId::Qaisariya => "qaisariya",
            CityId::BaytLahm => "bayt-lahm",
            CityId::Ramallah => "ramallah",
            CityId::Jenin => "jenin",
            CityId::AlNasira => "al-nasira",
            CityId::Tabariya => "tabariya",
            CityId::BirAlSabi => "bir-al-sabi",
        }
    }
}

/// Which way a station's name is set from its marker on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelSide {
    Left,
    Right,
    Above,
    Below,
}

/// Where a station's name sits on the map.
///
/// Placement is data rather than a rule derived from the corridor, because at
/// the drawn scale a handful of stations stand too close for any rule to
/// separate: Al-Quds and Bayt Lahm are nine kilometres apart, Rafah and Khan
/// Yunis under nine. Nudging a label is the honest fix; nudging a coordinate
/// would be a lie about where the place is.
#[derive(Debug, Clone, Copy)]
pub struct Label {
    pub side: LabelSide,
    /// Extra pixels applied after the side offset, for stations that would
    /// otherwise collide.
    pub nudge: (f32, f32),
}

impl Label {
    pub const fn new(side: LabelSide) -> Self {
        Self { side, nudge: (0.0, 0.0) }
    }

    pub const fn nudged(side: LabelSide, x: f32, y: f32) -> Self {
        Self { side, nudge: (x, y) }
    }
}

/// A station: who it is, where it is, and why it matters.
#[derive(Debug, Clone)]
pub struct City {
    pub id: CityId,
    /// Romanised Arabic name, as it is printed on the ticket.
    pub name: &'static str,
    /// The name in Arabic script.
    pub arabic: &'static str,
    /// The name the station carried on European timetables, where it differs.
    pub exonym: Option<&'static str>,
    /// Real-world position, projected for the map view.
    pub coordinates: GeoPoint,
    /// The corridor this station belongs to.
    pub line_type: LineType,
    /// One sentence of context, shown in the "About the City" panel.
    pub historical_fact: &'static str,
    /// Where the name is set from the marker.
    pub label: Label,
    /// How much traffic the place draws, 1 to 5.
    ///
    /// It decides both how often a station produces a traveller and how often
    /// it is where one is going, so the great ports and Al-Quds carry the
    /// network and the small halts on the southern line barely register.
    pub importance: u8,
}

/// The station table. Index `i` always holds `CityId::ALL[i]`.
pub(crate) const CITIES: [City; CITY_COUNT] = [
    City {
        id: CityId::Akka,
        name: "Akka",
        arabic: "عكا",
        exonym: Some("Acre"),
        coordinates: GeoPoint::new(32.928, 35.082),
        line_type: LineType::Coastal,
        historical_fact: "A walled Mediterranean port whose sea ramparts turned back Napoleon's \
                          siege of 1799. A branch line reached it from Haifa in 1913.",
        label: Label::new(LabelSide::Left),
        importance: 3,
    },
    City {
        id: CityId::Haifa,
        name: "Haifa",
        arabic: "حيفا",
        exonym: None,
        coordinates: GeoPoint::new(32.818, 34.989),
        line_type: LineType::Coastal,
        historical_fact: "Western terminus of the Jezreel Valley Railway, opened in 1905 to carry \
                          the Mediterranean trade inland to the Hejaz main line at Dera'a.",
        label: Label::new(LabelSide::Left),
        importance: 5,
    },
    City {
        id: CityId::Safad,
        name: "Safad",
        arabic: "صفد",
        exonym: Some("Safed"),
        coordinates: GeoPoint::new(32.965, 35.497),
        line_type: LineType::Mountain,
        historical_fact: "A hill town of Upper Galilee some 900 metres above the sea, long a \
                          centre of scholarship, printing and the copying of manuscripts.",
        label: Label::new(LabelSide::Right),
        importance: 2,
    },
    City {
        id: CityId::Beisan,
        name: "Beisan",
        arabic: "بيسان",
        exonym: Some("Beth Shean"),
        coordinates: GeoPoint::new(32.497, 35.500),
        line_type: LineType::Mountain,
        historical_fact: "Set in the Jordan Valley well below sea level; its stop on the Jezreel \
                          Valley line was among the lowest-lying railway stations on earth.",
        label: Label::new(LabelSide::Right),
        importance: 2,
    },
    City {
        id: CityId::Tulkarm,
        name: "Tulkarm",
        arabic: "طولكرم",
        exonym: None,
        coordinates: GeoPoint::new(32.310, 35.028),
        line_type: LineType::Junction,
        historical_fact: "Where the coastal main line met the branch that climbed east into the \
                          highlands towards Nablus, laid during the First World War.",
        label: Label::new(LabelSide::Right),
        importance: 2,
    },
    City {
        id: CityId::Nablus,
        name: "Nablus",
        arabic: "نابلس",
        exonym: None,
        coordinates: GeoPoint::new(32.221, 35.254),
        line_type: LineType::Mountain,
        historical_fact: "Seated between Mount Ebal and Mount Gerizim, famed for its olive-oil \
                          soap, its kunafa, and the spring markets of the central highlands.",
        label: Label::new(LabelSide::Right),
        importance: 4,
    },
    City {
        id: CityId::Yafa,
        name: "Yafa",
        arabic: "يافا",
        exonym: Some("Jaffa"),
        coordinates: GeoPoint::new(32.054, 34.752),
        line_type: LineType::Coastal,
        historical_fact: "The Bride of the Sea. Its Shamouti oranges sailed from these quays, and \
                          in 1892 the country's first railway ran from here up to Al-Quds.",
        label: Label::new(LabelSide::Left),
        importance: 5,
    },
    City {
        id: CityId::AlLadd,
        name: "Al-Ladd",
        arabic: "اللد",
        exonym: Some("Lydda"),
        coordinates: GeoPoint::new(31.951, 34.896),
        line_type: LineType::Junction,
        historical_fact: "The great junction of the network, where the Yafa-Al-Quds line crossed \
                          the main line running north to Haifa and south to Ghazzah.",
        label: Label::new(LabelSide::Below),
        importance: 4,
    },
    City {
        id: CityId::AlQuds,
        name: "Al-Quds",
        arabic: "القدس",
        exonym: Some("Jerusalem"),
        coordinates: GeoPoint::new(31.783, 35.217),
        line_type: LineType::Mountain,
        historical_fact: "Terminus of the 1892 line from the coast. Its station stood in the \
                          Baq'a quarter, south-west of the old walled city.",
        label: Label::new(LabelSide::Right),
        importance: 5,
    },
    City {
        id: CityId::AlKhalil,
        name: "Al-Khalil",
        arabic: "الخليل",
        exonym: Some("Hebron"),
        coordinates: GeoPoint::new(31.529, 35.098),
        line_type: LineType::Mountain,
        historical_fact: "A hill town of vineyards and glassblowers. Wartime construction pushed \
                          rails south from Al-Quds through its district in 1915.",
        label: Label::new(LabelSide::Left),
        importance: 3,
    },
    City {
        id: CityId::Ghazzah,
        name: "Ghazzah",
        arabic: "غزة",
        exonym: Some("Gaza"),
        coordinates: GeoPoint::new(31.504, 34.466),
        line_type: LineType::Coastal,
        historical_fact: "A caravan city on the Via Maris between Egypt and Syria; the military \
                          railway of 1917 tied it back into the coastal line.",
        label: Label::new(LabelSide::Left),
        importance: 4,
    },
    City {
        id: CityId::Rafah,
        name: "Rafah",
        arabic: "رفح",
        exonym: None,
        coordinates: GeoPoint::new(31.29, 34.25),
        line_type: LineType::Coastal,
        historical_fact: "The last station before Sinai, where the military railway pushed up \
                          from Egypt in 1917 crossed into the country.",
        label: Label::nudged(LabelSide::Left, 0.0, -9.0),
        importance: 1,
    },
    City {
        id: CityId::KhanYunis,
        name: "Khan Yunis",
        arabic: "خان يونس",
        exonym: None,
        coordinates: GeoPoint::new(31.35, 34.31),
        line_type: LineType::Coastal,
        historical_fact: "Named for the inn that stood on the caravan road, the last halt of any \
                          size between Ghazzah and the desert crossing.",
        label: Label::nudged(LabelSide::Left, 0.0, 9.0),
        importance: 2,
    },
    City {
        id: CityId::AlMajdal,
        name: "Al-Majdal",
        arabic: "المجدل",
        exonym: None,
        coordinates: GeoPoint::new(31.67, 34.57),
        line_type: LineType::Coastal,
        historical_fact: "A weaving town whose striped cottons were sold the length of the \
                          coast, set on the main line running south from Al-Ladd.",
        label: Label::new(LabelSide::Left),
        importance: 2,
    },
    City {
        id: CityId::Isdud,
        name: "Isdud",
        arabic: "إسدود",
        exonym: Some("Ashdod"),
        coordinates: GeoPoint::new(31.76, 34.65),
        line_type: LineType::Coastal,
        historical_fact: "One of the five old Philistine cities, standing back from its own \
                          harbour among the dunes of the southern plain.",
        label: Label::new(LabelSide::Left),
        importance: 2,
    },
    City {
        id: CityId::Yibna,
        name: "Yibna",
        arabic: "يبنا",
        exonym: Some("Yavne"),
        coordinates: GeoPoint::new(31.87, 34.74),
        line_type: LineType::Coastal,
        historical_fact: "A town of the coastal plain that gave the Crusaders their fief of \
                          Ibelin, and the southern line a station below Al-Ladd.",
        label: Label::new(LabelSide::Left),
        importance: 1,
    },
    City {
        id: CityId::Qaisariya,
        name: "Qaisariya",
        arabic: "قيسارية",
        exonym: Some("Caesarea"),
        coordinates: GeoPoint::new(32.50, 34.89),
        line_type: LineType::Coastal,
        historical_fact: "Herod's harbour city, its breakwaters long since drowned. No rail \
                          ever reached it, though every survey of a coastal line drew one.",
        label: Label::new(LabelSide::Left),
        importance: 1,
    },
    City {
        id: CityId::BaytLahm,
        name: "Bayt Lahm",
        arabic: "بيت لحم",
        exonym: Some("Bethlehem"),
        coordinates: GeoPoint::new(31.70, 35.20),
        line_type: LineType::Mountain,
        historical_fact: "A hill town of olive wood and mother-of-pearl, reached from Al-Quds \
                          by a line laid in haste in 1915.",
        label: Label::nudged(LabelSide::Right, 0.0, -10.0),
        importance: 3,
    },
    City {
        id: CityId::Ramallah,
        name: "Ramallah",
        arabic: "رام الله",
        exonym: None,
        coordinates: GeoPoint::new(31.90, 35.20),
        line_type: LineType::Mountain,
        historical_fact: "A highland town of orchards and cool summer air, 900 metres up, \
                          which the railway only ever approached on paper.",
        label: Label::nudged(LabelSide::Right, 0.0, 10.0),
        importance: 2,
    },
    City {
        id: CityId::Jenin,
        name: "Jenin",
        arabic: "جنين",
        exonym: None,
        coordinates: GeoPoint::new(32.46, 35.30),
        line_type: LineType::Mountain,
        historical_fact: "At the mouth of the Jezreel valley, where the branch from the north \
                          turned south for Nablus in 1913.",
        label: Label::new(LabelSide::Right),
        importance: 2,
    },
    City {
        id: CityId::AlNasira,
        name: "Al-Nasira",
        arabic: "الناصرة",
        exonym: Some("Nazareth"),
        coordinates: GeoPoint::new(32.70, 35.30),
        line_type: LineType::Mountain,
        historical_fact: "Set in the hills above the Jezreel plain; the valley railway ran \
                          below it from 1904 and the station carried its name.",
        label: Label::new(LabelSide::Right),
        importance: 3,
    },
    City {
        id: CityId::Tabariya,
        name: "Tabariya",
        arabic: "طبريا",
        exonym: Some("Tiberias"),
        coordinates: GeoPoint::new(32.79, 35.53),
        line_type: LineType::Mountain,
        historical_fact: "On the shore of the lake, 200 metres below the sea. The rails \
                          stopped a few miles short at Samakh and never reached the town.",
        label: Label::new(LabelSide::Right),
        importance: 3,
    },
    City {
        id: CityId::BirAlSabi,
        name: "Bir al-Sabi'",
        arabic: "بئر السبع",
        exonym: Some("Beersheba"),
        coordinates: GeoPoint::new(31.25, 34.79),
        line_type: LineType::Mountain,
        historical_fact: "Seven wells at the edge of the Naqab, joined to Ghazzah by a \
                          military line in 1915 and the market of the southern tribes.",
        label: Label::new(LabelSide::Below),
        importance: 3,
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
