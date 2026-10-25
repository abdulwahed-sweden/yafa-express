//! The look of the game: a set of railway documents laid out on a dark desk.
//!
//! Three voices, three typefaces. Oswald is the signage - condensed, upper
//! case, the lettering of a station nameboard. IBM Plex Serif is the printed
//! page, for anything written in sentences. IBM Plex Mono is the machine: codes,
//! distances, times, and the clatter of the announcer.
//!
//! Everything visual that is not a decision about *what* to show lives here, so
//! the UI module can stay a description of layout rather than a pile of colour
//! literals.

use bevy::prelude::*;
use bevy::text::LetterSpacing;
use yafa_core::LineType;

// --- The desk -------------------------------------------------------------

/// Deep roasted brown behind everything; the desk the papers lie on.
pub const DESK: Color = Color::srgb_u8(0x15, 0x11, 0x0D);
/// The shadow a sheet of paper throws onto it.
pub const SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);

// --- The map --------------------------------------------------------------

/// Ink for the shoreline and the water lines drawn out from it.
///
/// Deliberately colder and darker than the Coastal Line's own blue-green. The
/// two run alongside each other for the whole length of the map, and if the
/// water is not clearly behind the rails the eye cannot tell which is which.
pub const SEA_INK: Color = Color::srgb_u8(0x2C, 0x5B, 0x6E);
/// The faint grid of latitude and longitude.
pub const GRATICULE: Color = Color::srgba(0.82, 0.74, 0.58, 0.07);
/// Lettering on the map itself.
pub const MAP_INK: Color = Color::srgb_u8(0xE8, 0xDC, 0xC4);

// --- Paper ----------------------------------------------------------------

/// Aged ticket paper - the ground of every panel.
pub const PAPER: Color = Color::srgb_u8(0xF0, 0xE4, 0xCC);
/// A shade deeper, for inset rows and table stripes.
pub const PAPER_DEEP: Color = Color::srgb_u8(0xE2, 0xD3, 0xB2);
/// Deeper still, for a pressed or selected row.
pub const PAPER_PRESSED: Color = Color::srgb_u8(0xCF, 0xBC, 0x95);

// --- Ink ------------------------------------------------------------------

/// Printing ink, nearly black with a brown cast.
pub const INK: Color = Color::srgb_u8(0x24, 0x1E, 0x19);
/// Faded ink, for secondary lines and captions.
pub const INK_SOFT: Color = Color::srgb_u8(0x6A, 0x5C, 0x4E);
/// Very faded, for things not yet earned.
pub const INK_FAINT: Color = Color::srgb_u8(0x9D, 0x8E, 0x7B);

// --- Accents --------------------------------------------------------------

/// Brass fittings: rules, station dots, the title.
pub const BRASS: Color = Color::srgb_u8(0xC9, 0xA9, 0x4E);
/// Darker brass, for rules printed on paper.
pub const BRASS_DEEP: Color = Color::srgb_u8(0x94, 0x77, 0x2F);
/// The ink pad at the ticket counter.
pub const STAMP_RED: Color = Color::srgb_u8(0x9E, 0x32, 0x2B);
/// Olive, for a journey in progress.
pub const OLIVE: Color = Color::srgb_u8(0x6F, 0x80, 0x44);

/// Colour of a corridor, shared by the map and the UI legend.
pub fn line_color(line: LineType) -> Color {
    let [r, g, b] = line.rgb();
    Color::srgb_u8(r, g, b)
}

/// A corridor colour, dimmed - used for track the train is not on.
pub fn line_color_dim(line: LineType) -> Color {
    line_color(line).with_alpha(0.34)
}

// --- Type scale -----------------------------------------------------------

pub const CAPTION: f32 = 10.0;
pub const SMALL: f32 = 12.0;
pub const BODY: f32 = 13.0;
/// Station names in the list, and on the map.
pub const SIGN: f32 = 15.0;
pub const LARGE: f32 = 21.0;
pub const DISPLAY: f32 = 27.0;
pub const TITLE: f32 = 30.0;

// --- Typefaces ------------------------------------------------------------

/// The three faces, loaded once at startup.
///
/// Holds [`FontSource`]s rather than handles so that a headless test - which
/// has no asset server - can fall back to Bevy's built-in font and still
/// build the whole interface.
#[derive(Resource, Clone)]
pub struct Fonts {
    /// Oswald: condensed, for signage and headings.
    sign: FontSource,
    /// IBM Plex Serif: for anything written in sentences.
    prose: FontSource,
    prose_strong: FontSource,
    /// IBM Plex Mono: for codes, distances and times.
    data: FontSource,
    data_strong: FontSource,
}

impl Fonts {
    /// Builds the faces from the bytes compiled into the binary.
    ///
    /// Embedding rather than loading from disk keeps the executable
    /// self-contained: it runs from any working directory and can be copied
    /// anywhere without losing its lettering. Five faces come to under a
    /// megabyte, which is a fair price for never shipping a broken path.
    pub fn embedded(assets: &mut Assets<Font>) -> Self {
        let mut face =
            |bytes: &'static [u8]| FontSource::Handle(assets.add(Font::from_bytes(bytes.to_vec())));
        Self {
            sign: face(include_bytes!("../assets/fonts/Oswald-Variable.ttf")),
            prose: face(include_bytes!("../assets/fonts/IBMPlexSerif-Regular.ttf")),
            prose_strong: face(include_bytes!("../assets/fonts/IBMPlexSerif-SemiBold.ttf")),
            data: face(include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf")),
            data_strong: face(include_bytes!("../assets/fonts/IBMPlexMono-SemiBold.ttf")),
        }
    }

    /// Bevy's built-in face for every voice. Only for headless tests, where
    /// nothing is rasterised anyway.
    pub fn builtin() -> Self {
        Self {
            sign: FontSource::default(),
            prose: FontSource::default(),
            prose_strong: FontSource::default(),
            data: FontSource::default(),
            data_strong: FontSource::default(),
        }
    }

    /// Signage: station names, headings, the status on the board.
    pub fn sign(&self, size: f32) -> TextFont {
        TextFont {
            font: self.sign.clone(),
            font_size: FontSize::Px(size),
            weight: FontWeight::MEDIUM,
            ..default()
        }
    }

    /// Signage at its heaviest, for the few things that shout.
    pub fn sign_bold(&self, size: f32) -> TextFont {
        TextFont { weight: FontWeight::SEMIBOLD, ..self.sign(size) }
    }

    /// The printed page: history, explanations, captions in sentences.
    pub fn prose(&self, size: f32) -> TextFont {
        TextFont { font: self.prose.clone(), font_size: FontSize::Px(size), ..default() }
    }

    pub fn prose_strong(&self, size: f32) -> TextFont {
        TextFont { font: self.prose_strong.clone(), font_size: FontSize::Px(size), ..default() }
    }

    /// The machine: codes, distances, times, announcements.
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

/// Letter spacing for the small upper-case labels that run the interface.
pub const TRACKED: LetterSpacing = LetterSpacing::Px(1.1);
/// Wider still, for the title and the board's headline.
pub const TRACKED_WIDE: LetterSpacing = LetterSpacing::Px(2.2);

// --- Building blocks ------------------------------------------------------

/// The layout of a sheet of paper. Callers extend it with `..panel_node()`
/// rather than adding a second `Node`, which Bevy rejects as a duplicate.
pub fn panel_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::axes(Val::Px(15.0), Val::Px(13.0)),
        border: UiRect::all(Val::Px(1.0)),
        row_gap: Val::Px(7.0),
        ..default()
    }
}

/// Everything about a panel except its layout: aged paper, a hairline edge, a
/// shadow onto the desk, and an opaque surface a click cannot fall through.
pub fn panel_chrome() -> impl Bundle {
    (
        BackgroundColor(PAPER),
        BorderColor::all(INK.with_alpha(0.45)),
        BoxShadow(vec![ShadowStyle {
            color: SHADOW,
            x_offset: Val::Px(3.0),
            y_offset: Val::Px(5.0),
            spread_radius: Val::Px(-2.0),
            blur_radius: Val::Px(14.0),
        }]),
        Pickable { should_block_lower: true, is_hoverable: false },
    )
}

/// The heading of a panel: a tracked upper-case title over a thick-then-thin
/// rule, the way a printed form separates its sections.
pub fn panel_header(fonts: &Fonts, title: &str, subtitle: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            margin: UiRect::bottom(Val::Px(3.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                Text::new(title.to_uppercase()),
                fonts.sign_bold(SMALL),
                TextColor(INK),
                TRACKED,
                Pickable::IGNORE,
            ),
            (
                Text::new(subtitle.to_owned()),
                fonts.prose(CAPTION),
                TextColor(INK_FAINT),
                Pickable::IGNORE,
            ),
            double_rule(),
        ],
    )
}

/// A thick rule with a hairline under it.
pub fn double_rule() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            row_gap: Val::Px(2.0),
            margin: UiRect::top(Val::Px(5.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                Node { width: Val::Percent(100.0), height: Val::Px(2.0), ..default() },
                BackgroundColor(BRASS_DEEP),
                Pickable::IGNORE,
            ),
            (
                Node { width: Val::Percent(100.0), height: Val::Px(1.0), ..default() },
                BackgroundColor(BRASS_DEEP.with_alpha(0.45)),
                Pickable::IGNORE,
            ),
        ],
    )
}

/// A tear-off perforation: a run of punched holes across the paper.
pub fn perforation(holes: usize) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Px(5.0),
            overflow: Overflow::clip(),
            ..default()
        },
        Pickable::IGNORE,
        Children::spawn(SpawnIter((0..holes).map(|_| {
            (
                Node {
                    width: Val::Px(3.0),
                    height: Val::Px(3.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(INK.with_alpha(0.3)),
                Pickable::IGNORE,
            )
        }))),
    )
}
