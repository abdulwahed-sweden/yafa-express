//! Components, messages, resources and states - the vocabulary the systems
//! speak.
//!
//! Note what is *not* here: no component holds a reference into the rail
//! network. Stations travel through the ECS as [`CityId`], which is `Copy`, so
//! a system can read the network, write the passport and move the train in the
//! same body without a borrow conflict.

use std::collections::VecDeque;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::sprite_render::ColorMaterial;
use yafa_core::{CityId, Journey, JourneyConfig, Network, Passport, TicketDesk};

// --- States ---------------------------------------------------------------

/// Where the player is in the loop: buying a ticket, riding, or arrived.
#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GamePhase {
    /// No train running. The ticket counter is open.
    #[default]
    AtTheCounter,
    /// A train is working a route.
    OnTheRails,
    /// The train has arrived and is waiting to be dismissed.
    Arrived,
}

// --- Resources ------------------------------------------------------------

/// The historical network. Built once, read everywhere, never mutated.
#[derive(Resource, Deref)]
pub struct Rail(pub Network);

/// The two boxes on the blank ticket at the counter.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct Desk(pub TicketDesk);

/// How fast the world clock runs and how quickly trains move.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct Clock(pub JourneyConfig);

/// The traveller's passport, and where it is kept on disk.
#[derive(Resource)]
pub struct PassportBook {
    pub passport: Passport,
    pub path: PathBuf,
    /// Set when a stamp lands; cleared once the file has been rewritten.
    pub unsaved: bool,
}

impl PassportBook {
    pub fn new(passport: Passport, path: PathBuf) -> Self {
        Self { passport, path, unsaved: false }
    }
}

/// The station whose history is showing in the "About the City" panel.
#[derive(Resource, Default)]
pub struct Focus(pub Option<CityId>);

/// The rolling announcements under the station board.
#[derive(Resource, Default)]
pub struct Announcer {
    lines: VecDeque<String>,
}

impl Announcer {
    /// How many announcements the board holds before the oldest scrolls off.
    ///
    /// Kept short on purpose: the board sits over the south of the map, and
    /// every line it grows is a line of coast it hides.
    const CAPACITY: usize = 3;

    /// Wipes the board.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn say(&mut self, line: impl Into<String>) {
        self.lines.push_front(line.into());
        self.lines.truncate(Self::CAPACITY);
    }

    /// Newest first, which is how the board reads.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(String::as_str)
    }
}

/// A reusable buffer for the events a journey tick produces, so the movement
/// system does not allocate every frame.
#[derive(Resource, Default)]
pub struct JourneyScratch(pub Vec<yafa_core::JourneyEvent>);

/// How map kilometres become screen pixels.
#[derive(Resource)]
pub struct MapView {
    pub px_per_km: f32,
    /// Map-space point drawn at the centre of the map area.
    pub centre: Vec2,
    /// Where that centre sits on screen, in pixels.
    pub offset: Vec2,
}

impl MapView {
    /// Projects a point from map kilometres into world pixels.
    pub fn world(&self, point: yafa_core::MapPoint) -> Vec2 {
        Vec2::new(point.x_km, point.y_km) * self.px_per_km - self.centre * self.px_per_km
            + self.offset
    }
}

// --- Messages -------------------------------------------------------------
//
// Buffered messages rather than observers: several stations can be passed in
// one frame when the clock is wound up, and every one of them has to be read
// by the stamping system in order.

/// The train drew up at a station.
#[derive(Message, Debug, Clone, Copy)]
pub struct TrainReached {
    pub city: CityId,
    pub final_stop: bool,
}

/// The train pulled out of a station.
#[derive(Message, Debug, Clone, Copy)]
pub struct TrainDeparted(pub CityId);

/// The passport clerk brought the stamp down.
#[derive(Message, Debug, Clone, Copy)]
pub struct PassportStamped {
    pub city: CityId,
    /// `false` if the traveller had already collected this station.
    pub fresh: bool,
    pub order: u32,
}

/// A whole route was completed.
#[derive(Message, Debug, Clone, Copy)]
pub struct JourneyFinished {
    pub origin: CityId,
    pub destination: CityId,
    pub track_m: u32,
}

// --- Map components -------------------------------------------------------

/// The 2D camera looking at the map.
#[derive(Component)]
pub struct MapCamera;

/// The locomotive. Always present; hidden when no journey is running.
#[derive(Component)]
pub struct Train;

/// The journey the locomotive is working. Absent when it is in the shed.
#[derive(Component, Deref, DerefMut)]
pub struct TrainRun(pub Journey);

/// A station dot on the map.
#[derive(Component, Clone, Copy)]
pub struct StationDot(pub CityId);

/// The station's name, printed beside its dot.
#[derive(Component, Clone, Copy)]
pub struct StationLabel(pub CityId);

// --- UI components --------------------------------------------------------

/// What a clickable piece of the interface does when activated.
///
/// One component and one observer handle every button in the game, so adding a
/// control never means adding a system.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    /// Swap the interface between English and Arabic.
    ToggleLanguage,
    /// Pay to lay a surveyed line, bringing it into service this year.
    OpenLine(yafa_core::SegmentId),
    /// Write this station onto the ticket.
    PickCity(CityId),
    /// Issue the ticket and start the train.
    Depart,
    /// Tear up the ticket.
    ClearTicket,
    /// Dismiss an arrived train and reopen the counter.
    BackToCounter,
    /// Step through the timetable speeds.
    CycleClock,
    /// Look up a station's history without ticketing it.
    ShowCity(CityId),
}

/// A row in the station list, so it can be restyled as the ticket changes.
#[derive(Component, Clone, Copy)]
pub struct StationRow(pub CityId);

/// The ticket's "From" / "To" boxes.
#[derive(Component, Clone, Copy)]
pub enum TicketSlot {
    Origin,
    Destination,
}

/// The line that reads out the route the ticket buys.
#[derive(Component)]
pub struct TicketRouteText;

/// The departure button, which greys out until a ticket is complete.
#[derive(Component)]
pub struct DepartButton;

/// Label on the clock button.
#[derive(Component)]
pub struct ClockLabel;

/// The big status line on the station board.
#[derive(Component)]
pub struct BoardStatus;

/// The smaller detail line under the status.
#[derive(Component)]
pub struct BoardDetail;

/// The filled part of the journey progress bar.
#[derive(Component)]
pub struct BoardProgressFill;

/// The announcement lines under the board.
#[derive(Component)]
pub struct BoardAnnouncements;

/// The sheet of the "About the City" panel, hidden while an arrival shows.
#[derive(Component)]
pub struct CityPanelSheet;

/// Pieces of the "About the City" panel.
#[derive(Component, Clone, Copy, Debug)]
pub enum CityPanelPart {
    Name,
    Subtitle,
    Fact,
    Stamped,
}

/// The "Visited Cities: X / Y" line.
#[derive(Component)]
pub struct PassportCounter;

/// The filled part of the passport completion bar.
#[derive(Component)]
pub struct PassportFill;

/// The per-corridor summary in the passport.
#[derive(Component)]
pub struct PassportLines;

/// The lifetime odometer line.
#[derive(Component)]
pub struct PassportOdometer;

/// The list of stations still waiting for a stamp.
#[derive(Component)]
pub struct PassportRemaining;

/// One stamp card in the passport grid.
#[derive(Component, Clone, Copy)]
pub struct StampCard(pub CityId);

/// The number printed on a stamp card once it is earned.
#[derive(Component, Clone, Copy)]
pub struct StampCardOrder(pub CityId);

/// Freshly stamped cards flash; this times the flash out.
#[derive(Component)]
pub struct StampFlash {
    pub remaining: f32,
}

impl StampFlash {
    pub const DURATION: f32 = 1.4;

    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for StampFlash {
    fn default() -> Self {
        Self { remaining: Self::DURATION }
    }
}

/// A control was activated - by click, by keyboard, or from the map.
///
/// Routing every control through one message means the rules for what a button
/// *does* live in a single ordinary system, not scattered across observers.
#[derive(Message, Debug, Clone, Copy)]
pub struct ActionRequested(pub UiAction);

/// The status note at the right-hand end of a station list row ("FROM", "TO",
/// or the stamp tick).
#[derive(Component, Clone, Copy)]
pub struct StationRowNote(pub CityId);

/// The telegraph code printed on a passport stamp card.
#[derive(Component, Clone, Copy)]
pub struct StampCardCode(pub CityId);

/// The scrolling container holding the station rows.
#[derive(Component)]
pub struct StationList;

/// Where the game stands in time. Wound forward by the track the train runs.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct Almanac(pub yafa_core::Calendar);

/// A body of water, so the season can move the light on it.
#[derive(Component)]
pub struct SeaFill;

/// One of the two coloured squares that make up a station marker.
#[derive(Component, Clone, Copy)]
pub struct StationFace {
    pub city: CityId,
    /// The smaller inner square, which carries the corridor's colour.
    pub inner: bool,
}

/// The year, shown in the header beside the passport count.
#[derive(Component)]
pub struct HeaderYear;

/// The language the interface currently on screen was built in.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltLanguage(pub crate::strings::Language);

/// The root of the whole interface, torn down when the language changes.
#[derive(Component)]
pub struct InterfaceRoot;

/// The label on the language toggle.
#[derive(Component)]
pub struct LanguageToggleLabel;

/// The season, shown under the year.
#[derive(Component)]
pub struct HeaderSeason;

/// The passport count, shown in the header beside the year.
#[derive(Component)]
pub struct HeaderCount;

/// A line opened this year and the board should say so.
#[derive(Message, Debug, Clone, Copy)]
pub struct LineOpened {
    pub year: u16,
    pub segment: yafa_core::SegmentId,
}

/// The money the railway has, and what it charges.
#[derive(Resource)]
pub struct Counting {
    pub treasury: yafa_core::Treasury,
    pub tariff: yafa_core::Tariff,
}

impl Default for Counting {
    fn default() -> Self {
        Self { treasury: yafa_core::Treasury::new(), tariff: yafa_core::Tariff::default() }
    }
}

/// Who is waiting on every platform.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct Queues(pub yafa_core::Platforms);

/// How often the stations draw a fresh traveller.
#[derive(Resource)]
pub struct TrafficTimer(pub Timer);

impl Default for TrafficTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.45, TimerMode::Repeating))
    }
}

/// Who is on the train. Present only while it is working a route.
#[derive(Component, Deref, DerefMut, Default)]
pub struct Riders(pub yafa_core::Manifest);

/// A fare was collected, or coal was burnt.
#[derive(Message, Debug, Clone, Copy)]
pub struct Takings {
    pub piastres: i64,
    pub at: Option<CityId>,
    pub passengers: usize,
}

/// The balance, shown in the header.
#[derive(Component)]
pub struct HeaderBalance;

/// How many are aboard, shown on the station board.
#[derive(Component)]
pub struct BoardRiders;

/// One row of the city card's track list, with its "Open line" control.
#[derive(Component, Clone, Copy)]
pub struct CityLinkRow(pub usize);

/// The text of a track-list row.
#[derive(Component, Clone, Copy)]
pub struct CityLinkText(pub usize);

/// The "Open line" control on a track-list row.
#[derive(Component, Clone, Copy)]
pub struct CityLinkBuy(pub usize);

/// The label on that control.
#[derive(Component, Clone, Copy)]
pub struct CityLinkBuyLabel(pub usize);

/// The card that announces an arrival, and slides away again.
///
/// It stands in front of the permanent city card for as long as it is on
/// screen, so the right-hand column always shows exactly one station.
#[derive(Component)]
pub struct ArrivalCard {
    /// Seconds left before it has gone entirely.
    pub remaining: f32,
    pub city: Option<CityId>,
}

impl ArrivalCard {
    /// How long the whole thing lasts, slide in and out included.
    pub const DURATION: f32 = 4.0;
    /// How long the slide at each end takes.
    pub const SLIDE: f32 = 0.35;

    /// How far off its resting place the card sits, in pixels, with 0 fully
    /// arrived and positive numbers off to the right.
    pub fn offset(&self, width: f32) -> f32 {
        let elapsed = Self::DURATION - self.remaining;
        let eased = if elapsed < Self::SLIDE {
            1.0 - elapsed / Self::SLIDE
        } else if self.remaining < Self::SLIDE {
            1.0 - self.remaining / Self::SLIDE
        } else {
            0.0
        };
        // Cubic ease, so it settles rather than stopping dead.
        width * eased * eased * eased
    }
}

impl Default for ArrivalCard {
    fn default() -> Self {
        Self { remaining: 0.0, city: None }
    }
}

/// Pieces of the arrival card.
#[derive(Component, Clone, Copy, Debug)]
pub enum ArrivalPart {
    /// The romanised name.
    Name,
    /// The same name in Arabic script.
    Arabic,
    Fact,
}

/// The dark ground the schematic diagram is drawn on.
#[derive(Component)]
pub struct SchematicGround;

/// A station on the schematic, and which shape its mark takes.
#[derive(Component, Clone, Copy)]
pub struct SchematicStation {
    pub city: CityId,
    pub capsule: bool,
}

/// A station's mark on the schematic: a dot, or a capsule if it is an
/// interchange.
#[derive(Component, Clone)]
pub struct StationMark {
    pub city: CityId,
    /// Its own material, so its state can be coloured without disturbing the
    /// twenty-two others.
    pub paint: Handle<ColorMaterial>,
}

/// The Latin name set under a station's Arabic one on the schematic.
#[derive(Component, Clone, Copy)]
pub struct StationLatin(pub CityId);
