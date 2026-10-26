//! The look of the game: one light page, one type family, and flat colour.
//!
//! The palette is a single light theme - an off-white page, white cards, a
//! hairline border, navy text - and the only colour that carries meaning is the
//! colour of a line or the state of a station.
//!
//! One family does all the lettering: IBM Plex Sans, with IBM Plex Sans Arabic
//! behind it in every fallback chain so Arabic script renders in a face that
//! belongs to the same design. IBM Plex Mono is kept for figures - distances,
//! times and station codes - where a fixed advance keeps columns aligned.
//!
//! Two accents carry all the state the game shows: [`VISITED`] for what has
//! happened, [`PENDING`] for what is still ahead.

use bevy::prelude::*;
use yafa_core::LineType;

// --- The page -------------------------------------------------------------

/// The page, and the ground the map is drawn on.
pub const DESK: Color = Color::srgb_u8(0xF7, 0xF5, 0xEF);

/// Hairline that bounds every card and rule.
pub const BORDER: Color = Color::srgb_u8(0xE3, 0xDE, 0xD2);

// --- The map --------------------------------------------------------------

/// Stroke for the shoreline and the water drawn around it.
pub const SEA_INK: Color = Color::srgb_u8(0x8F, 0xCB, 0xC6);
/// Flat fill for the sea and the closed bodies of water.
pub const SEA_FILL: Color = Color::srgb_u8(0xD8, 0xEF, 0xED);
/// Lettering set on the water. The shoreline stroke is far too pale to read
/// against the fill it bounds, so names take a darker tone of the same teal.
pub const SEA_LABEL: Color = Color::srgb_u8(0x2F, 0x7D, 0x78);
/// The faint grid of latitude and longitude.
pub const GRATICULE: Color = Color::srgba(0.086, 0.141, 0.227, 0.07);
/// Lettering on the map itself.
pub const MAP_INK: Color = Color::srgb_u8(0x16, 0x24, 0x3A);

// --- Surfaces -------------------------------------------------------------

/// A card.
pub const PAPER: Color = Color::srgb_u8(0xFF, 0xFF, 0xFF);
/// An inset row, or a card under the pointer.
pub const PAPER_DEEP: Color = Color::srgb_u8(0xF7, 0xF5, 0xEF);
/// A control being pressed.
pub const PAPER_PRESSED: Color = Color::srgb_u8(0xEB, 0xE7, 0xDC);

// --- Text -----------------------------------------------------------------

/// Headings.
pub const INK_HEADER: Color = Color::srgb_u8(0x16, 0x24, 0x3A);
/// Body and anything primary.
pub const INK: Color = Color::srgb_u8(0x14, 0x1D, 0x29);
/// Secondary text.
pub const INK_SOFT: Color = Color::srgb_u8(0x4B, 0x55, 0x63);
/// A station not yet visited, and anything else not yet earned.
pub const INK_FAINT: Color = Color::srgb_u8(0x9A, 0xA4, 0xB3);

// --- State ----------------------------------------------------------------

/// A station that has been visited, a journey under way, a control ready to
/// be used. Everything that has happened or is happening.
pub const VISITED: Color = Color::srgb_u8(0x1F, 0x8A, 0x4C);

/// A station the ticket is bound for but has not reached. Everything still
/// ahead.
pub const PENDING: Color = Color::srgb_u8(0xB8, 0x65, 0x1F);

/// The quiet structural grey: outlines on controls and bars, and the small
/// monospace text that sits beside them.
///
/// It cannot be the hairline. `ui.rs` uses this colour for text as well as
/// for borders, and at `#e3ded2` that text drops to about 1.3:1 on white and
/// vanishes. This reads at roughly 5:1 and still sits quietly as a border.
pub const SLATE: Color = Color::srgb_u8(0x6B, 0x74, 0x82);

/// Colour of a corridor, shared by the map and the UI legend.
///
/// The three line colours are defined here rather than read from
/// `LineType::rgb`, so that re-theming the game never means editing the
/// domain crate.
pub fn line_color(line: LineType) -> Color {
    match line {
        LineType::Coastal => Color::srgb_u8(0x1C, 0x8A, 0x84),
        LineType::Mountain => Color::srgb_u8(0xB8, 0x65, 0x1F),
        // The SL concept names this one: #8b94a3, a shade lighter than the
        // slate used for text, so the links read on the dark diagram.
        LineType::Junction => Color::srgb_u8(0x8B, 0x94, 0xA3),
    }
}

/// A corridor colour, dimmed - used for track the train is not on.
///
/// Lightened rather than faded: on a light page, alpha alone turns every line
/// the same shade of grey.
pub fn line_color_dim(line: LineType) -> Color {
    line_color(line).mix(&DESK, 0.62)
}

// --- The departure board --------------------------------------------------
//
// The board's own four colours, which appear nowhere else in the game: a navy
// band under a yellow stripe, amber dot-matrix type on near-black.

pub const BOARD_BAND: Color = Color::srgb_u8(0x1D, 0x2A, 0x6B);
pub const BOARD_STRIPE: Color = Color::srgb_u8(0xF2, 0xC2, 0x00);
pub const BOARD_GROUND: Color = Color::srgb_u8(0x0B, 0x0F, 0x1C);
pub const BOARD_AMBER: Color = Color::srgb_u8(0xFF, 0xB0, 0x00);
/// The second departure in each direction, held back from the first.
pub const BOARD_AMBER_DIM: Color = Color::srgb_u8(0xC9, 0x8C, 0x00);

// --- The ticket gates -----------------------------------------------------

/// An open gate's signal, and the flash when a ticket is accepted. The same
/// green as a visited station: this has happened.
pub const GATE_OPEN: Color = VISITED;

/// A gate that stays shut, and the flash when a ticket is refused. The only
/// red in the game, and it only ever means a real error.
pub const GATE_SHUT: Color = Color::srgb_u8(0xB4, 0x23, 0x18);

/// The lit strip across an open gate.
pub const GATE_OPEN_GLOW: Color = Color::srgb_u8(0xEA, 0xF6, 0xEE);

/// The same across a shut one.
pub const GATE_SHUT_GLOW: Color = Color::srgb_u8(0xFD, 0xEC, 0xEB);

// --- The season -----------------------------------------------------------

/// Where the year stands, as the map is lit.
///
/// The calendar turns over every 120 km run, so a single journey is weeks of
/// a year and the light moves with it: a cool, low winter, a green spring, a
/// bleached summer, an amber autumn. The wash is deliberately slight. It is
/// meant to be noticed over a few journeys, not between one frame and the
/// next.
pub struct Season;

/// The four corners of the year, as (page, water, name).
const SEASONS: [(Color, Color, &str); 4] = [
    // Winter: the light goes blue and the page darkens.
    (Color::srgb_u8(0xE8, 0xE9, 0xEC), Color::srgb_u8(0xC9, 0xE2, 0xE8), "winter"),
    // Spring.
    (Color::srgb_u8(0xF3, 0xF6, 0xEA), Color::srgb_u8(0xD5, 0xEF, 0xE4), "spring"),
    // High summer: bleached and warm.
    (Color::srgb_u8(0xFA, 0xF6, 0xE6), Color::srgb_u8(0xDC, 0xF0, 0xEE), "summer"),
    // Autumn.
    (Color::srgb_u8(0xF5, 0xF0, 0xE2), Color::srgb_u8(0xD6, 0xE9, 0xE4), "autumn"),
];

impl Season {
    /// The ground the map is drawn on.
    pub fn page(through_year: f32) -> Color {
        let (from, to, blend) = Self::between(through_year);
        SEASONS[from].0.mix(&SEASONS[to].0, blend)
    }

    /// The water, which takes the same light.
    pub fn water(through_year: f32) -> Color {
        let (from, to, blend) = Self::between(through_year);
        SEASONS[from].1.mix(&SEASONS[to].1, blend)
    }

    /// What to call it, for the header.
    pub fn name(through_year: f32) -> &'static str {
        let (from, to, blend) = Self::between(through_year);
        if blend < 0.5 { SEASONS[from].2 } else { SEASONS[to].2 }
    }

    /// The same, as a phrase the strings table can translate.
    pub fn phrase(through_year: f32) -> crate::strings::Phrase {
        use crate::strings::Phrase;
        match Self::name(through_year) {
            "winter" => Phrase::Winter,
            "spring" => Phrase::Spring,
            "summer" => Phrase::Summer,
            _ => Phrase::Autumn,
        }
    }

    /// The pair of seasons the year stands between, and how far across.
    ///
    /// The year starts in winter, so the position is taken straight from the
    /// calendar with no offset.
    fn between(through_year: f32) -> (usize, usize, f32) {
        let position = through_year.rem_euclid(1.0) * SEASONS.len() as f32;
        let from = position.floor() as usize % SEASONS.len();
        (from, (from + 1) % SEASONS.len(), position.fract())
    }
}

// --- Type scale -----------------------------------------------------------

pub const CAPTION: f32 = 14.0;
pub const BODY: f32 = 17.0;
/// Station names, in the list and on the map.
pub const SIGN: f32 = 18.0;
pub const HEADING: f32 = 20.0;
pub const DISPLAY: f32 = 28.0;

// --- Typefaces ------------------------------------------------------------

/// The faces, built once at startup.
///
/// Holds [`FontSource`]s rather than handles so that a headless test - which
/// has no asset store - can fall back to Bevy's built-in font and still build
/// the whole interface.
#[derive(Resource, Clone)]
pub struct Fonts {
    /// IBM Plex Sans Regular, over Arabic.
    sign: FontSource,
    /// IBM Plex Sans Medium, over Arabic.
    sign_medium: FontSource,
    /// The same Regular face: one family does all the lettering now.
    prose: FontSource,
    prose_strong: FontSource,
    /// IBM Plex Mono, for figures.
    data: FontSource,
    data_strong: FontSource,
}

impl Fonts {
    /// Builds the faces from the bytes compiled into the binary.
    ///
    /// Embedding rather than loading from disk keeps the executable
    /// self-contained: it runs from any working directory and can be copied
    /// anywhere without losing its lettering.
    pub fn embedded(assets: &mut Assets<Font>) -> Self {
        let mut face =
            |bytes: &'static [u8]| FontSource::Handle(assets.add(Font::from_bytes(bytes.to_vec())));

        let latin = face(include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf"));
        let latin_medium = face(include_bytes!("../assets/fonts/IBMPlexSans-Medium.ttf"));
        let arabic = face(include_bytes!("../assets/fonts/IBMPlexSansArabic-Regular.ttf"));
        let arabic_medium = face(include_bytes!("../assets/fonts/IBMPlexSansArabic-Medium.ttf"));
        let mono = face(include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf"));
        let mono_strong = face(include_bytes!("../assets/fonts/IBMPlexMono-SemiBold.ttf"));

        // Arabic sits behind Latin in every chain, so a string in either
        // script is set in a face from the same family rather than in
        // whatever the system happens to offer.
        let regular = FontSource::List(vec![latin, arabic.clone()]);
        let medium = FontSource::List(vec![latin_medium, arabic_medium]);

        Self {
            sign: regular.clone(),
            sign_medium: medium.clone(),
            prose: regular,
            prose_strong: medium,
            data: FontSource::List(vec![mono, arabic.clone()]),
            // Arabic behind this one too. Without it the few lines set in the
            // heavier mono - the stamped notice, for one - come out as tofu.
            data_strong: FontSource::List(vec![mono_strong, arabic]),
        }
    }

    /// Bevy's built-in face for every voice. Only for headless tests, where
    /// nothing is rasterised anyway.
    pub fn builtin() -> Self {
        Self {
            sign: FontSource::default(),
            sign_medium: FontSource::default(),
            prose: FontSource::default(),
            prose_strong: FontSource::default(),
            data: FontSource::default(),
            data_strong: FontSource::default(),
        }
    }

    /// Station names, headings, the status on the board.
    pub fn sign(&self, size: f32) -> TextFont {
        TextFont { font: self.sign.clone(), font_size: FontSize::Px(size), ..default() }
    }

    /// The same family one step heavier, for the few things that lead.
    pub fn sign_bold(&self, size: f32) -> TextFont {
        TextFont { font: self.sign_medium.clone(), font_size: FontSize::Px(size), ..default() }
    }

    /// Running text: history, explanations, captions in sentences.
    pub fn prose(&self, size: f32) -> TextFont {
        TextFont { font: self.prose.clone(), font_size: FontSize::Px(size), ..default() }
    }

    pub fn prose_strong(&self, size: f32) -> TextFont {
        TextFont { font: self.prose_strong.clone(), font_size: FontSize::Px(size), ..default() }
    }

    /// Figures: distances, times and station codes.
    pub fn data(&self, size: f32) -> TextFont {
        TextFont { font: self.data.clone(), font_size: FontSize::Px(size), ..default() }
    }

    pub fn data_strong(&self, size: f32) -> TextFont {
        TextFont { font: self.data_strong.clone(), font_size: FontSize::Px(size), ..default() }
    }
}

impl Default for Fonts {
    fn default() -> Self {
        Self::builtin()
    }
}

// --- Building blocks ------------------------------------------------------

/// The layout of a card. Callers extend it with `..panel_node()` rather than
/// adding a second `Node`, which Bevy rejects as a duplicate.
pub fn panel_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(16.0)),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        row_gap: Val::Px(8.0),
        ..default()
    }
}

/// Everything about a card except its layout: white, a hairline edge, and an
/// opaque surface a click cannot fall through.
pub fn panel_chrome() -> impl Bundle {
    (
        BackgroundColor(PAPER),
        BorderColor::all(BORDER),
        Pickable { should_block_lower: true, is_hoverable: false },
    )
}

/// The heading of a card: a title in sentence case over its subtitle.
pub fn panel_header(fonts: &Fonts, title: &str, subtitle: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            margin: UiRect::bottom(Val::Px(2.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                Text::new(title.to_owned()),
                fonts.sign_bold(HEADING),
                TextColor(INK_HEADER),
                Pickable::IGNORE,
            ),
            (
                Text::new(subtitle.to_owned()),
                fonts.prose(CAPTION),
                TextColor(INK_SOFT),
                Pickable::IGNORE,
            ),
        ],
    )
}

/// A single hairline divider.
///
/// Keeps its old name because it is still the one rule in the design; the
/// thick-over-thin pair it used to draw is gone.
pub fn double_rule() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(BORDER),
        Pickable::IGNORE,
    )
}
