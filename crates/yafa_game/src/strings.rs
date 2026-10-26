//! Every word the interface says, in both languages, in one table.
//!
//! Nothing anywhere else in the game writes a user-facing string directly.
//! A phrase is asked for by name and the current [`Language`] decides which
//! column of the table answers, which is the only way a second language stays
//! honest: a phrase that is added without a translation will not compile.
//!
//! Phrases that carry a number or a place use `{}` where the value goes, and
//! are assembled with [`fill`]. `format!` needs its string at compile time and
//! these are chosen at run time, so they cannot be format strings.

use bevy::prelude::*;

/// Which language the interface is speaking.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Arabic,
}

impl Language {
    pub fn other(self) -> Self {
        match self {
            Language::English => Language::Arabic,
            Language::Arabic => Language::English,
        }
    }

    /// Whether the interface runs right to left.
    pub const fn is_rtl(self) -> bool {
        matches!(self, Language::Arabic)
    }

    /// The name of the *other* language, for the toggle, always written in
    /// that language - the point of the control is to be legible to somebody
    /// who cannot read the one currently showing.
    pub const fn toggle_label(self) -> &'static str {
        match self {
            Language::English => "العربية",
            Language::Arabic => "English",
        }
    }

    /// Looks a phrase up.
    pub fn say(self, phrase: Phrase) -> &'static str {
        let (english, arabic) = phrase.both();
        match self {
            Language::English => english,
            Language::Arabic => arabic,
        }
    }
}

/// Fills the `{}` markers of a phrase, in order.
pub fn fill(template: &str, values: &[&str]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    for value in values {
        match rest.split_once("{}") {
            Some((before, after)) => {
                out.push_str(before);
                out.push_str(value);
                rest = after;
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}

/// Everything the interface can say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phrase {
    // The ticket counter.
    TicketCounter,
    TicketCounterNote,
    From,
    To,
    Depart,
    Clear,
    SelectDeparture,
    NoLine,
    FromSelectDestination,
    TicketPair,
    NoOpenLine,
    TicketSummary,
    Shortcuts,
    // The station board.
    StationBoard,
    StationBoardNote,
    WaitingForTicket,
    CounterOpen,
    BoardingAt,
    InTransitTo,
    StandingAt,
    ArrivedAt,
    RunComplete,
    RunProgress,
    Aboard,
    WaitingOnPlatforms,
    // The city card.
    AboutTheCity,
    AboutTheCityNote,
    NotYetVisited,
    StampedNth,
    TrackOpened,
    TrackOnPaper,
    LayLine,
    Arrived,
    // The passport.
    Passport,
    PassportNote,
    VisitedCities,
    Odometer,
    StillToCallAt,
    AllStamped,
    Locked,
    Subtitle,
    // The header.
    Stamped,
    OnTheNetwork,
    Piastres,
    // Corridors.
    CoastalLine,
    MountainLine,
    JunctionLinks,
    // Seasons.
    Winter,
    Spring,
    Summer,
    Autumn,
    // The announcer.
    AnnDeparting,
    AnnRightAway,
    AnnStamped,
    AnnStampStands,
    AnnArrived,
    AnnCounterOpen,
    AnnTimetable,
    AnnNoLineTo,
    AnnYearStanding,
    AnnLineOpens,
    AnnLinesOpen,
    AnnCoal,
    AnnSetDown,
    AnnLineLaid,
    AnnCannotAfford,
    // Timetable speeds.
    Local,
    Express,
    Mail,
}

impl Phrase {
    /// Every phrase there is.
    ///
    /// Kept beside the table so the tests can sweep the lot. A new variant
    /// that is not added here is still caught by the compiler, which will
    /// reject `both` for not covering it.
    pub const ALL: [Phrase; 69] = {
        use Phrase::*;
        [
            TicketCounter,
            TicketCounterNote,
            From,
            To,
            Depart,
            Clear,
            SelectDeparture,
            NoLine,
            FromSelectDestination,
            TicketPair,
            NoOpenLine,
            TicketSummary,
            Shortcuts,
            StationBoard,
            StationBoardNote,
            WaitingForTicket,
            CounterOpen,
            BoardingAt,
            InTransitTo,
            StandingAt,
            ArrivedAt,
            RunComplete,
            RunProgress,
            Aboard,
            WaitingOnPlatforms,
            AboutTheCity,
            AboutTheCityNote,
            NotYetVisited,
            StampedNth,
            TrackOpened,
            TrackOnPaper,
            LayLine,
            Arrived,
            Passport,
            PassportNote,
            VisitedCities,
            Odometer,
            StillToCallAt,
            AllStamped,
            Locked,
            Subtitle,
            Stamped,
            OnTheNetwork,
            Piastres,
            CoastalLine,
            MountainLine,
            JunctionLinks,
            Winter,
            Spring,
            Summer,
            Autumn,
            AnnDeparting,
            AnnRightAway,
            AnnStamped,
            AnnStampStands,
            AnnArrived,
            AnnCounterOpen,
            AnnTimetable,
            AnnNoLineTo,
            AnnYearStanding,
            AnnLineOpens,
            AnnLinesOpen,
            AnnCoal,
            AnnSetDown,
            AnnLineLaid,
            AnnCannotAfford,
            Local,
            Express,
            Mail,
        ]
    };

    /// The two columns of the table. English first.
    const fn both(self) -> (&'static str, &'static str) {
        use Phrase::*;
        match self {
            TicketCounter => ("Ticket Counter", "شبّاك التذاكر"),
            TicketCounterNote => {
                ("Choose where you board and where you are bound", "اختر محطة الانطلاق ومحطة الوصول")
            }
            From => ("From", "من"),
            To => ("To", "إلى"),
            Depart => ("Depart", "انطلاق"),
            Clear => ("Clear", "مسح"),
            SelectDeparture => ("Select a departure station", "اختر محطة الانطلاق"),
            NoLine => ("no line", "لا خط"),
            FromSelectDestination => ("From {} - select a destination", "من {} - اختر محطة الوصول"),
            TicketPair => ("{} to {}", "{} إلى {}"),
            NoOpenLine => {
                ("No open line joins {} and {} in {}.", "لا يوجد خط مفتوح بين {} و{} في {}.")
            }
            TicketSummary => (
                "{} km  ·  {} calls  ·  {} min  ·  {}\ncoal {} pi  ·  up to {} pi in fares",
                "{} كم  ·  {} محطات  ·  {} دقيقة  ·  {}\nالفحم {} قرش  ·  حتى {} قرش أجرة",
            ),
            Shortcuts => (
                "Space depart · Esc clear · Tab timetable",
                "مسافة للانطلاق · خروج للمسح · تبويب للجدول",
            ),

            StationBoard => ("Station Board", "لوحة المحطة"),
            StationBoardNote => {
                ("Departures, arrivals and the state of the line", "المغادرات والوصول وحال الخط")
            }
            WaitingForTicket => ("Waiting for a ticket", "في انتظار تذكرة"),
            CounterOpen => (
                "The counter is open. Pick a departure and a destination.",
                "الشبّاك مفتوح. اختر محطة الانطلاق ومحطة الوصول.",
            ),
            BoardingAt => ("Boarding at {}", "الصعود في {}"),
            InTransitTo => ("In transit to {}", "في الطريق إلى {}"),
            StandingAt => ("Standing at {}", "متوقف في {}"),
            ArrivedAt => ("Arrived at {}", "وصل إلى {}"),
            RunComplete => (
                "{} km run complete. Depart again, or press Space, for a new ticket.",
                "اكتملت رحلة {} كم. انطلق مرة أخرى، أو اضغط مسافة، لتذكرة جديدة.",
            ),
            RunProgress => (
                "{} - {} km run, {} km to go, about {} min on the timetable.",
                "{} - قُطع {} كم، وبقي {} كم، نحو {} دقيقة على الجدول.",
            ),
            Aboard => ("{} aboard of {} seats", "{} راكبًا من {} مقعدًا"),
            WaitingOnPlatforms => ("{} waiting on the platforms", "{} في انتظار على الأرصفة"),

            AboutTheCity => ("About the City", "عن المدينة"),
            AboutTheCityNote => (
                "Click any station, on the map or in the list",
                "انقر أي محطة، على الخريطة أو في القائمة",
            ),
            NotYetVisited => (
                "Not yet visited - buy a ticket through here",
                "لم تُزر بعد - اشترِ تذكرة تمر من هنا",
            ),
            StampedNth => {
                ("Stamped - the {} station in this passport", "مختومة - المحطة {} في هذا الجواز")
            }
            TrackOpened => ("to {} - {} km, opened {}", "إلى {} - {} كم، افتُتح {}"),
            TrackOnPaper => ("to {} - {} km, on paper", "إلى {} - {} كم، على الورق"),
            LayLine => ("Lay  {} pi", "مدّ  {} قرش"),
            Arrived => ("Arrived", "وصل"),

            Passport => ("Passport", "جواز السفر"),
            PassportNote => ("One stamp for every station called at", "ختم واحد لكل محطة تقف فيها"),
            VisitedCities => ("Visited cities  {} / {}", "المدن المزارة  {} / {}"),
            Odometer => {
                ("{} journeys  ·  {} km of track behind you", "{} رحلة  ·  {} كم من السكة خلفك")
            }
            StillToCallAt => ("Still to call at: {}{}.", "بقي أن تقف في: {}{}."),
            AllStamped => {
                ("Every station on the network is stamped.", "كل محطة على الشبكة مختومة.")
            }
            Locked => ("-", "-"),

            Subtitle => ("the railways of historical Palestine", "سكك حديد فلسطين التاريخية"),
            Stamped => ("{} / {} stamped", "{} / {} مختومة"),
            OnTheNetwork => ("{} stations on the network", "{} محطة على الشبكة"),
            Piastres => ("{} pi", "{} قرش"),

            CoastalLine => ("Coastal Line", "خط الساحل"),
            MountainLine => ("Mountain Line", "خط الجبل"),
            JunctionLinks => ("Junction Links", "وصلات التقاطع"),

            Winter => ("winter", "شتاء"),
            Spring => ("spring", "ربيع"),
            Summer => ("summer", "صيف"),
            Autumn => ("autumn", "خريف"),

            AnnDeparting => (
                "Departing {} for {} - {} km via {} stop(s).",
                "المغادرة من {} إلى {} - {} كم عبر {} محطة.",
            ),
            AnnRightAway => ("Right away from {}.", "الانطلاق من {}."),
            AnnStamped => ("{} stamped - station {} of {}.", "{} مختومة - المحطة {} من {}."),
            AnnStampStands => ("{} again; the stamp already stands.", "{} مجددًا؛ الختم قائم."),
            AnnArrived => ("Arrived {}. {} km from {}.", "وصل {}. {} كم من {}."),
            AnnCounterOpen => ("Ticket counter open.", "شبّاك التذاكر مفتوح."),
            AnnTimetable => ("Timetable set to {} service.", "ضُبط الجدول على خدمة {}."),
            AnnNoLineTo => ("No line runs to {} in {}.", "لا خط يصل إلى {} في {}."),
            AnnYearStanding => {
                ("{}. {} of {} stations are on the network.", "{}. {} من {} محطة على الشبكة.")
            }
            AnnLineOpens => ("{}: the {} line opens.", "{}: افتُتح خط {}."),
            AnnLinesOpen => (
                "{}: the {} line opens, and {} more with it.",
                "{}: افتُتح خط {}، ومعه {} خطوط أخرى.",
            ),
            AnnCoal => ("Coal for the run: {} piastres.", "فحم الرحلة: {} قرشًا."),
            AnnSetDown => ("Set down {} at {} - {} piastres.", "أنزل {} في {} - {} قرشًا."),
            AnnLineLaid => {
                ("{}: the {} line is laid, for {} piastres.", "{}: مُدّ خط {}، بـ {} قرشًا.")
            }
            AnnCannotAfford => (
                "The {} line costs {} piastres; the railway has {}.",
                "خط {} يكلف {} قرشًا؛ لدى السكة {}.",
            ),

            Local => ("Local", "محلي"),
            Express => ("Express", "سريع"),
            Mail => ("Mail", "بريد"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_phrase_is_said_in_both_languages() {
        for phrase in Phrase::ALL {
            let (english, arabic) = phrase.both();
            assert!(!english.trim().is_empty(), "{phrase:?} has no English");
            assert!(!arabic.trim().is_empty(), "{phrase:?} has no Arabic");
        }
    }

    #[test]
    fn both_columns_take_the_same_number_of_values() {
        // A translation that drops a placeholder would silently swallow a
        // station name or a sum of money.
        for phrase in Phrase::ALL {
            let (english, arabic) = phrase.both();
            assert_eq!(
                english.matches("{}").count(),
                arabic.matches("{}").count(),
                "{phrase:?} takes a different number of values in each language"
            );
        }
    }

    #[test]
    fn filling_puts_the_values_where_the_markers_are() {
        assert_eq!(fill("{} of {}", &["3", "23"]), "3 of 23");
        // Too few values leaves the rest of the template alone rather than
        // producing a half-written sentence with no marker in it.
        assert_eq!(fill("{} of {}", &["3"]), "3 of {}");
        assert_eq!(fill("no markers", &["3"]), "no markers");
    }

    #[test]
    fn the_toggle_names_the_language_it_switches_to() {
        assert_eq!(Language::English.toggle_label(), "العربية");
        assert_eq!(Language::Arabic.toggle_label(), "English");
        assert_eq!(Language::English.other(), Language::Arabic);
        assert!(!Language::English.is_rtl());
        assert!(Language::Arabic.is_rtl());
    }
}
