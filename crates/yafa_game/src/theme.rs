//! The look of the game: a set of railway documents laid out on a dark desk.
//!
//! Everything visual that is not a decision about *what* to show lives here, so
//! the UI module can stay a description of layout rather than a pile of colour
//! literals.

use bevy::prelude::*;
use yafa_core::LineType;

// --- The desk -------------------------------------------------------------

/// Deep roasted brown behind everything; the desk the papers lie on.
pub const DESK: Color = Color::srgb_u8(0x17, 0x13, 0x0F);
// --- Paper ----------------------------------------------------------------

/// Aged ticket paper - the ground of every panel.
pub const PAPER: Color = Color::srgb_u8(0xEF, 0xE2, 0xC8);
/// A shade deeper, for inset rows and table stripes.
pub const PAPER_DEEP: Color = Color::srgb_u8(0xE0, 0xD0, 0xAE);
/// Deeper still, for a pressed or selected row.
pub const PAPER_PRESSED: Color = Color::srgb_u8(0xCF, 0xBC, 0x95);

// --- Ink ------------------------------------------------------------------

/// Printing ink, nearly black with a brown cast.
pub const INK: Color = Color::srgb_u8(0x2A, 0x24, 0x20);
/// Faded ink, for secondary lines and captions.
pub const INK_SOFT: Color = Color::srgb_u8(0x6B, 0x5E, 0x50);
/// Very faded, for things not yet earned.
pub const INK_FAINT: Color = Color::srgb_u8(0x9C, 0x8E, 0x7C);

// --- Accents --------------------------------------------------------------

/// Brass fittings: borders, rules, station dots.
pub const BRASS: Color = Color::srgb_u8(0xC4, 0xA4, 0x54);
/// Darker brass for rules on paper.
pub const BRASS_DEEP: Color = Color::srgb_u8(0x9A, 0x7E, 0x38);
/// The ink pad at the ticket counter.
pub const STAMP_RED: Color = Color::srgb_u8(0xA6, 0x3A, 0x33);
/// Olive, for a journey in progress.
pub const OLIVE: Color = Color::srgb_u8(0x7D, 0x8E, 0x4F);

/// Colour of a corridor, shared by the map and the UI legend.
pub fn line_color(line: LineType) -> Color {
    let [r, g, b] = line.rgb();
    Color::srgb_u8(r, g, b)
}

/// A corridor colour, dimmed - used for track the train is not on.
pub fn line_color_dim(line: LineType) -> Color {
    line_color(line).with_alpha(0.38)
}

// --- Type scale -----------------------------------------------------------

pub const SIZE_TITLE: f32 = 19.0;
pub const SIZE_HEADING: f32 = 13.0;
pub const SIZE_BODY: f32 = 14.0;
pub const SIZE_SMALL: f32 = 12.0;
pub const SIZE_TINY: f32 = 10.0;
pub const SIZE_LARGE: f32 = 21.0;
pub const SIZE_DISPLAY: f32 = 26.0;

/// Text of a given size and colour.
pub fn text_font(size: f32) -> TextFont {
    TextFont { font_size: FontSize::Px(size), ..default() }
}

// --- Building blocks ------------------------------------------------------

/// The layout of a sheet of paper. Callers extend it with `..panel_node()`
/// rather than adding a second `Node`, which would quietly replace this one.
pub fn panel_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(13.0)),
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(Val::Px(3.0)),
        row_gap: Val::Px(7.0),
        ..default()
    }
}

/// Everything about a panel except its layout: aged paper, a brass edge, and
/// an opaque surface that a click does not fall through.
pub fn panel_chrome() -> impl Bundle {
    (
        BackgroundColor(PAPER),
        BorderColor::all(BRASS_DEEP),
        Pickable { should_block_lower: true, is_hoverable: false },
    )
}

/// The heading strip at the top of a panel: a title over a brass rule.
pub fn panel_header(title: &str, subtitle: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            border: UiRect::bottom(Val::Px(1.0)),
            padding: UiRect::bottom(Val::Px(6.0)),
            row_gap: Val::Px(1.0),
            ..default()
        },
        BorderColor::all(BRASS_DEEP),
        Pickable::IGNORE,
        children![
            (
                Text::new(title.to_uppercase()),
                text_font(SIZE_HEADING),
                TextColor(INK),
                Pickable::IGNORE,
            ),
            (
                Text::new(subtitle.to_owned()),
                text_font(SIZE_TINY),
                TextColor(INK_FAINT),
                Pickable::IGNORE,
            ),
        ],
    )
}
