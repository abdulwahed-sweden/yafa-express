//! The interface: the ticket counter, the station board, the city panel and
//! the passport.
//!
//! The layout is built once at startup; after that a handful of refresh
//! systems copy state onto the widgets. Each widget is found by a marker
//! component carrying the [`CityId`] it belongs to, so nothing has to keep an
//! index into a list that might move.
//!
//! Typography does the organising. Station names are set in the condensed
//! signage face, anything written in sentences in the serif, and every number,
//! code and time in the monospace - so the eye can tell a place from a
//! measurement before it reads either.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::LineHeight;
use bevy::ui::Pressed;
use bevy::ui_widgets::Button;
use yafa_core::{CityId, LineType, Network, Phase};

use crate::components::*;
use crate::strings::{Language, Phrase, fill};
use crate::theme::{self, Fonts};

const LEFT_WIDTH: f32 = 318.0;
const RIGHT_WIDTH: f32 = 348.0;
const BOARD_WIDTH: f32 = 560.0;
/// No station on the map has more than four lines leaving it.
const MAX_LINKS: usize = 4;
/// Stamp cards, four to a row inside the passport.
const CARD_WIDTH: f32 = 74.0;
const CARD_HEIGHT: f32 = 48.0;

// --- Construction ---------------------------------------------------------

/// Lays out the whole interface.
pub fn build_ui(mut commands: Commands, rail: Res<Rail>, fonts: Res<Fonts>, lang: Res<Language>) {
    let lang = *lang;
    // Recorded here rather than inferred later: this is the language the
    // interface standing on screen is actually written in.
    commands.insert_resource(BuiltLanguage(lang));
    commands
        .spawn((
            InterfaceRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                // Arabic reads right to left, so the whole column order turns
                // round with it and the ticket counter ends up where the eye
                // starts. `Rtl` alone does that - it reverses the main axis
                // the way CSS does, and pairing it with `RowReverse` reverses
                // the reversal and leaves the layout exactly as it was.
                direction: if lang.is_rtl() { InlineDirection::Rtl } else { InlineDirection::Ltr },
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(14.0)),
                column_gap: Val::Px(14.0),
                ..default()
            },
            // The map lies underneath: only the panels themselves swallow
            // clicks, so stations stay reachable everywhere else.
            Pickable::IGNORE,
            Name::new("Interface"),
        ))
        .with_children(|root| {
            ticket_counter(root, &fonts, lang);
            centre_column(root, &fonts, lang);
            right_column(root, &rail, &fonts, lang);
        });
}

// --- Ticket counter -------------------------------------------------------

fn ticket_counter(root: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    root.spawn((
        Node { width: Val::Px(LEFT_WIDTH), flex_direction: FlexDirection::Column, ..default() },
        Pickable::IGNORE,
    ))
    .with_children(|column| {
        column
            .spawn((Node { flex_grow: 1.0, ..theme::panel_node() }, theme::panel_chrome()))
            .with_children(|panel| {
                panel.spawn(theme::panel_header(
                    fonts,
                    lang.say(Phrase::TicketCounter),
                    lang.say(Phrase::TicketCounterNote),
                ));
                ticket_stub(panel, fonts, lang);
                panel.spawn((
                    TicketRouteText,
                    Text::new(lang.say(Phrase::SelectDeparture)),
                    fonts.data(theme::CAPTION),
                    TextColor(theme::INK_SOFT),
                    LineHeight::RelativeToFont(1.45),
                    Pickable::IGNORE,
                ));
                departure_board(panel, fonts, lang);
                control_row(panel, fonts, lang);
                keyboard_hints(panel, fonts, lang);
            });
    });
}

/// The two boxes on the ticket itself.
fn ticket_stub(panel: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    panel
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                width: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            slot_box(row, fonts, lang.say(Phrase::From), TicketSlot::Origin, theme::VISITED);
            slot_box(row, fonts, lang.say(Phrase::To), TicketSlot::Destination, theme::PENDING);
        });
}

fn slot_box(
    row: &mut ChildSpawnerCommands,
    fonts: &Fonts,
    caption: &str,
    slot: TicketSlot,
    accent: Color,
) {
    row.spawn((
        Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(9.0), Val::Px(7.0)),
            border: UiRect::left(Val::Px(3.0)),
            row_gap: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(theme::PAPER_DEEP),
        BorderColor::all(accent),
        Pickable::IGNORE,
        children![
            (
                Text::new(caption.to_owned()),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_FAINT),
                Pickable::IGNORE,
            ),
            (
                slot,
                Text::new("-"),
                fonts.sign(theme::SIGN),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ),
        ],
    ));
}

/// The platform board, shown while a departure station is written on the
/// ticket.
///
/// Two directions, two departures each. In the concept they stand side by
/// side with a clock between them; this column is 318 px wide, so they are
/// stacked in two pairs and the clock is dropped, which is what the demo's
/// own narrow layout does.
fn departure_board(panel: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    panel
        .spawn((
            DepartureBoardPanel,
            Node {
                flex_direction: FlexDirection::Column,
                display: Display::None,
                row_gap: Val::Px(4.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(9.0)),
                border: UiRect::bottom(Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                width: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(theme::BOARD_BAND),
            BorderColor::all(theme::BOARD_STRIPE),
            Pickable::IGNORE,
            children![(
                Text::new(lang.say(Phrase::Departures)),
                fonts.data(theme::CAPTION - 2.0),
                TextColor(theme::BOARD_STRIPE),
                Pickable::IGNORE,
            )],
        ))
        .with_children(|board| {
            for row in 0..DepartureBoard::ROWS {
                // The first departure in a direction leads; the one behind it
                // is the same line a step fainter, as on the board itself.
                let leading = row % 2 == 0;
                let amber = if leading { theme::BOARD_AMBER } else { theme::BOARD_AMBER_DIM };
                board.spawn((
                    DepartureRow(row),
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(theme::BOARD_GROUND),
                    Pickable::IGNORE,
                    children![
                        (
                            DepartureLine(row),
                            Node {
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(0.0)),
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(theme::line_color(LineType::Coastal)),
                            Pickable::IGNORE,
                            children![(
                                Text::new(""),
                                fonts.data_strong(theme::CAPTION - 3.0),
                                TextColor(Color::WHITE),
                                Pickable::IGNORE,
                            )],
                        ),
                        (
                            DepartureDest(row),
                            Text::new(""),
                            if leading {
                                fonts.data_strong(theme::CAPTION)
                            } else {
                                fonts.data(theme::CAPTION)
                            },
                            TextColor(amber),
                            Node { flex_grow: 1.0, ..default() },
                            Pickable::IGNORE,
                        ),
                        (
                            DepartureTime(row),
                            Text::new(""),
                            fonts.data(theme::CAPTION),
                            TextColor(amber),
                            Pickable::IGNORE,
                        ),
                    ],
                ));
            }
        });
}

fn control_row(panel: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    panel
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                width: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn((
                control_chrome(UiAction::Depart),
                DepartButton,
                Node { flex_grow: 1.0, ..control_node() },
                children![(
                    Text::new(lang.say(Phrase::Depart)),
                    fonts.sign_bold(theme::BODY),
                    TextColor(theme::INK),
                    Pickable::IGNORE,
                )],
            ));
            row.spawn((
                control_chrome(UiAction::ClearTicket),
                control_node(),
                children![(
                    Text::new(lang.say(Phrase::Clear)),
                    fonts.sign(theme::BODY),
                    TextColor(theme::INK_SOFT),
                    Pickable::IGNORE,
                )],
            ));
            row.spawn((
                control_chrome(UiAction::CycleClock),
                control_node(),
                children![(
                    ClockLabel,
                    Text::new(lang.say(Phrase::Express)),
                    fonts.sign(theme::BODY),
                    TextColor(theme::INK_SOFT),
                    Pickable::IGNORE,
                )],
            ));
        });
}

/// The shortcuts, set at the foot of the counter where there is room for them.
fn keyboard_hints(panel: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    panel.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            margin: UiRect::top(Val::Px(14.0)),
            width: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
        children![
            theme::double_rule(),
            (
                Text::new(lang.say(Phrase::Shortcuts)),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_FAINT),
                Pickable::IGNORE,
            ),
        ],
    ));
}

// --- Centre: the title and the station board ------------------------------

fn centre_column(root: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    root.spawn((
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|centre| {
        header_bar(centre, fonts, lang);

        station_board(centre, fonts, lang);
    });
}

/// The bar over the map: what the game is called, what year it is, and how
/// much of the country has been seen.
fn header_bar(centre: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    centre
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(20.0),
                width: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(1.0), ..default() },
                Pickable::IGNORE,
                children![
                    (
                        Text::new("Yafa Express"),
                        fonts.sign_bold(theme::DISPLAY),
                        TextColor(theme::INK_HEADER),
                        Pickable::IGNORE,
                    ),
                    (
                        Text::new(lang.say(Phrase::Subtitle)),
                        fonts.prose(theme::CAPTION),
                        TextColor(theme::INK_FAINT),
                        Pickable::IGNORE,
                    ),
                ],
            ));

            bar.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(16.0),
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexEnd,
                            ..default()
                        },
                        Pickable::IGNORE,
                        children![
                            (
                                HeaderYear,
                                Text::new("1892"),
                                fonts.sign_bold(theme::DISPLAY),
                                TextColor(theme::VISITED),
                                Pickable::IGNORE,
                            ),
                            (
                                HeaderSeason,
                                Text::new(""),
                                fonts.data(theme::CAPTION),
                                TextColor(theme::INK_FAINT),
                                Pickable::IGNORE,
                            ),
                        ],
                    ),
                    (
                        HeaderBalance,
                        Text::new(""),
                        fonts.sign_bold(theme::HEADING),
                        TextColor(theme::INK_HEADER),
                        Pickable::IGNORE,
                    ),
                    (
                        HeaderCount,
                        Text::new(""),
                        fonts.data(theme::CAPTION),
                        TextColor(theme::INK_SOFT),
                        Pickable::IGNORE,
                    ),
                    (
                        control_chrome(UiAction::ToggleLanguage),
                        control_node(),
                        children![(
                            LanguageToggleLabel,
                            // Written in the language it switches to: the
                            // control has to be legible to somebody who
                            // cannot read the one currently showing.
                            Text::new(lang.toggle_label()),
                            fonts.sign(theme::BODY),
                            TextColor(theme::INK_HEADER),
                            Pickable::IGNORE,
                        )],
                    ),
                ],
            ));
        });
}

fn station_board(centre: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    centre
        .spawn((Node { width: Val::Px(BOARD_WIDTH), ..theme::panel_node() }, theme::panel_chrome()))
        .with_children(|panel| {
            // No heading on this one: every pixel it takes is a pixel of the
            // map's southern end that disappears behind it.
            panel.spawn((
                BoardStatus,
                Text::new(lang.say(Phrase::WaitingForTicket)),
                fonts.sign(theme::HEADING),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ));
            panel.spawn((
                BoardDetail,
                Text::new(lang.say(Phrase::CounterOpen)),
                fonts.prose(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                LineHeight::RelativeToFont(1.4),
                Pickable::IGNORE,
            ));
            panel.spawn((
                BoardRiders,
                Text::new(""),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ));
            progress_bar(panel, BoardProgressFill, theme::VISITED);
            panel.spawn((
                BoardAnnouncements,
                Text::new(""),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                LineHeight::RelativeToFont(1.55),
                Pickable::IGNORE,
            ));
        });
}

/// A thin filled bar. `fill` marks the inner node so a refresh system can set
/// its width.
fn progress_bar(panel: &mut ChildSpawnerCommands, fill: impl Component, color: Color) {
    panel
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::PAPER_DEEP),
            BorderColor::all(theme::SLATE),
            Pickable::IGNORE,
        ))
        .with_child((
            fill,
            Node { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() },
            BackgroundColor(color),
            Pickable::IGNORE,
        ));
}

// --- Right: the city panel and the passport -------------------------------

fn right_column(root: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts, lang: Language) {
    root.spawn((
        Node {
            width: Val::Px(RIGHT_WIDTH),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(14.0),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|column| {
        // The arrival card and the city card share a slot: whichever is
        // showing, the column names exactly one station.
        column
            .spawn((Node { width: Val::Percent(100.0), ..default() }, Pickable::IGNORE))
            .with_children(|slot| {
                city_panel(slot, fonts, lang);
                arrival_card(slot, fonts, lang);
            });
        passport_panel(column, rail, fonts, lang);
    });
}

fn city_panel(column: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    column.spawn((
        CityPanelSheet,
        theme::panel_node(),
        theme::panel_chrome(),
        children![
            theme::panel_header(
                fonts,
                lang.say(Phrase::AboutTheCity),
                lang.say(Phrase::AboutTheCityNote)
            ),
            (
                CityPanelPart::Name,
                Text::new("Yafa"),
                fonts.sign(theme::DISPLAY),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ),
            (
                CityPanelPart::Subtitle,
                Text::new(""),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_FAINT),
                Pickable::IGNORE,
            ),
            (
                CityPanelPart::Fact,
                Text::new(""),
                fonts.prose(theme::BODY),
                TextColor(theme::INK_SOFT),
                LineHeight::RelativeToFont(1.5),
                Pickable::IGNORE,
            ),
            track_list(fonts),
            (
                CityPanelPart::Stamped,
                Text::new(""),
                fonts.data_strong(theme::CAPTION),
                // Overwritten by `refresh_city_panel` on the first frame, which
                // picks visited or muted; this is only what it starts as.
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ),
        ],
    ));
}

/// The track running out of a station, a row apiece.
///
/// Four rows are spawned once and shown or hidden as the focus moves, because
/// no station on the map has more than four lines leaving it. A row whose line
/// is still only surveyed carries the control that pays to lay it.
fn track_list(fonts: &Fonts) -> impl Bundle {
    // The spawner outlives the borrow, so it gets its own copies of the faces.
    let (caption, label) = (fonts.data(theme::CAPTION), fonts.data(theme::CAPTION));
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            width: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
        Children::spawn(SpawnIter((0..MAX_LINKS).map(move |index| {
            (
                CityLinkRow(index),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(8.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    (
                        CityLinkText(index),
                        Text::new(""),
                        caption.clone(),
                        TextColor(theme::SLATE),
                        Node { flex_grow: 1.0, ..default() },
                        Pickable::IGNORE,
                    ),
                    (
                        CityLinkBuy(index),
                        Button,
                        // Replaced as the focus moves; never fires as spawned.
                        UiAction::ShowCity(CityId::Yafa),
                        Hovered::default(),
                        Pickable::default(),
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            // Fixed, so the label never squeezes to three
                            // lines when the row's text runs long.
                            min_width: Val::Px(96.0),
                            flex_shrink: 0.0,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(theme::PAPER_DEEP),
                        BorderColor::all(theme::BORDER),
                        children![(
                            CityLinkBuyLabel(index),
                            Text::new(""),
                            label.clone(),
                            TextColor(theme::INK_SOFT),
                            Pickable::IGNORE,
                        )],
                    ),
                ],
            )
        }))),
    )
}

/// The card that slides in when the train draws up somewhere.
///
/// Laid over the city card rather than beside it, so it covers the panel it
/// stands in for instead of pushing the passport down the column.
fn arrival_card(slot: &mut ChildSpawnerCommands, fonts: &Fonts, lang: Language) {
    slot.spawn((
        ArrivalCard::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            display: Display::None,
            ..theme::panel_node()
        },
        // Spelled out rather than `panel_chrome()`, which carries a border
        // colour of its own: this card wants the arrival green.
        BackgroundColor(theme::PAPER),
        BorderColor::all(theme::VISITED),
        Pickable { should_block_lower: true, is_hoverable: false },
        children![
            (
                Text::new(lang.say(Phrase::Arrived)),
                fonts.data(theme::CAPTION),
                TextColor(theme::VISITED),
                Pickable::IGNORE,
            ),
            (
                ArrivalPart::Name,
                Text::new(""),
                fonts.sign(theme::DISPLAY),
                TextColor(theme::INK_HEADER),
                Pickable::IGNORE,
            ),
            (
                ArrivalPart::Arabic,
                Text::new(""),
                fonts.sign(theme::HEADING),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ),
            (
                ArrivalPart::Fact,
                Text::new(""),
                fonts.prose(theme::BODY),
                TextColor(theme::INK_SOFT),
                LineHeight::RelativeToFont(1.5),
                Pickable::IGNORE,
            ),
        ],
    ));
}

fn passport_panel(
    column: &mut ChildSpawnerCommands,
    rail: &Network,
    fonts: &Fonts,
    lang: Language,
) {
    column
        .spawn((Node { flex_grow: 1.0, ..theme::panel_node() }, theme::panel_chrome()))
        .with_children(|panel| {
            panel.spawn(theme::panel_header(
                fonts,
                lang.say(Phrase::Passport),
                lang.say(Phrase::PassportNote),
            ));
            panel.spawn((
                PassportCounter,
                Text::new(lang.say(Phrase::VisitedCities)),
                fonts.sign(theme::HEADING),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ));
            progress_bar(panel, PassportFill, theme::VISITED);
            panel.spawn((
                PassportLines,
                Text::new(""),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ));
            panel.spawn((
                PassportOdometer,
                Text::new(""),
                fonts.prose(theme::CAPTION),
                TextColor(theme::INK_FAINT),
                Pickable::IGNORE,
            ));
            stamp_grid(panel, rail, fonts, lang);
            panel.spawn((
                PassportRemaining,
                Text::new(""),
                fonts.prose(theme::CAPTION),
                TextColor(theme::INK_FAINT),
                LineHeight::RelativeToFont(1.5),
                Node { margin: UiRect::top(Val::Px(6.0)), ..default() },
                Pickable::IGNORE,
            ));
        });
}

/// The grid of stamp cards, one per station, locked until it is earned.
fn stamp_grid(panel: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts, lang: Language) {
    panel
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(7.0),
                margin: UiRect::top(Val::Px(7.0)),
                width: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|grid| {
            for city in CityId::ALL {
                let data = rail.city(city);
                grid.spawn((
                    Button,
                    UiAction::ShowCity(city),
                    StampCard(city),
                    Hovered::default(),
                    Pickable::default(),
                    // A hand-held stamp never lands square. Each card gets its
                    // own small, fixed tilt so the page looks worked in.
                    UiTransform::from_rotation(Rot2::degrees(card_tilt(city))),
                    Node {
                        width: Val::Px(CARD_WIDTH),
                        height: Val::Px(CARD_HEIGHT),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(0.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(theme::PAPER_DEEP),
                    BorderColor::all(theme::INK_FAINT),
                    // Code and number share a line: at the new type sizes three
                    // stacked lines no longer fit a card this size, and the
                    // number belongs with the code anyway.
                    children![
                        (
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Baseline,
                                column_gap: Val::Px(4.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                            children![
                                (
                                    StampCardCode(city),
                                    Text::new(city.code()),
                                    fonts.sign_bold(theme::SIGN),
                                    TextColor(theme::INK_FAINT),
                                    Pickable::IGNORE,
                                ),
                                (
                                    StampCardOrder(city),
                                    Text::new(""),
                                    fonts.data(theme::CAPTION),
                                    TextColor(theme::INK_FAINT),
                                    Pickable::IGNORE,
                                ),
                            ],
                        ),
                        (
                            Text::new(station_name(lang, data)),
                            fonts.prose(theme::CAPTION),
                            TextColor(theme::INK_SOFT),
                            Pickable::IGNORE,
                        ),
                    ],
                ));
            }
        });
}

/// A small fixed tilt per card, from its own identifier - repeatable, and
/// never the same two cards running.
fn card_tilt(city: CityId) -> f32 {
    const TILTS: [f32; 5] = [-3.4, 2.1, -1.3, 3.0, -2.4];
    TILTS[city.index() % TILTS.len()]
}

/// The layout of a clickable control. Extend it with `..control_node()`
/// rather than adding a second `Node`, which Bevy rejects as a duplicate.
fn control_node() -> Node {
    Node {
        padding: UiRect::axes(Val::Px(11.0), Val::Px(6.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(2.0)),
        ..default()
    }
}

/// Everything a clickable control needs except its layout.
fn control_chrome(action: UiAction) -> impl Bundle {
    (
        Button,
        action,
        Hovered::default(),
        Pickable::default(),
        BackgroundColor(theme::PAPER_DEEP),
        BorderColor::all(theme::SLATE),
    )
}

// --- Refresh --------------------------------------------------------------

/// Highlights plain controls under the pointer.
pub fn style_controls(
    mut controls: Query<
        (&Hovered, Has<Pressed>, &mut BackgroundColor),
        (With<UiAction>, Without<StationRow>, Without<StampCard>),
    >,
) {
    for (hovered, pressed, mut background) in &mut controls {
        background.0 = match (pressed, hovered.get()) {
            (true, _) => theme::PAPER_PRESSED,
            (false, true) => theme::PAPER,
            (false, false) => theme::PAPER_DEEP,
        };
    }
}

/// Writes the ticket, and keeps the station list in step with it.
pub fn refresh_ticket(
    rail: Res<Rail>,
    lang: Res<Language>,
    desk: Res<Desk>,
    clock: Res<Clock>,
    almanac: Res<Almanac>,
    counting: Res<Counting>,
    phase: Res<State<GamePhase>>,
    mut slots: Query<(&TicketSlot, &mut Text), (Without<TicketRouteText>, Without<ClockLabel>)>,
    mut route_text: Query<&mut Text, (With<TicketRouteText>, Without<ClockLabel>)>,
    mut clock_label: Query<&mut Text, With<ClockLabel>>,
    mut depart: Query<&mut BorderColor, (With<DepartButton>, Without<StationRow>)>,
) {
    for (slot, mut text) in &mut slots {
        let city = match slot {
            TicketSlot::Origin => desk.origin,
            TicketSlot::Destination => desk.destination,
        };
        text.0 =
            city.map_or_else(|| "-".to_owned(), |c| station_name(*lang, rail.city(c)).to_owned());
    }

    if let Ok(mut text) = route_text.single_mut() {
        text.0 = match desk.issue(&rail, almanac.year()) {
            Some(route) => {
                // Corridor codes, not full names: the long form wrapped to
                // three lines and pushed the station list down the panel.
                let lines: Vec<&str> = route.lines_used().iter().map(|l| l.code()).collect();
                // What the run is worth if the train fills, against what the
                // coal will cost whether it does or not.
                let seats = yafa_core::traffic::TRAIN_CAPACITY as i64;
                let best = counting.tariff.fare(route.total_m) * seats;
                format!(
                    "{:.0} km  ·  {} calls  ·  {:.0} min  ·  {}\ncoal {} pi  ·  up to {} pi in fares",
                    route.total_km(),
                    route.legs.len(),
                    clock.running_minutes(route.total_m),
                    lines.join(" + "),
                    counting.tariff.coal(route.total_m),
                    best,
                )
            }
            None if desk.is_ready() => format!(
                "No open line joins {} and {} in {}.",
                rail.city(desk.origin.unwrap()).name,
                rail.city(desk.destination.unwrap()).name,
                almanac.year(),
            ),
            // The desk's own summary is written in English; the counter has
            // to say the same thing in whichever language is showing.
            None => match (desk.origin, desk.destination) {
                (None, _) => lang.say(Phrase::SelectDeparture).to_owned(),
                (Some(origin), None) => fill(
                    lang.say(Phrase::FromSelectDestination),
                    &[station_name(*lang, rail.city(origin))],
                ),
                (Some(origin), Some(destination)) => fill(
                    lang.say(Phrase::TicketPair),
                    &[
                        station_name(*lang, rail.city(origin)),
                        station_name(*lang, rail.city(destination)),
                    ],
                ),
            },
        };
    }

    if let Ok(mut text) = clock_label.single_mut() {
        text.0 = speed_name(*lang, clock.clock_label()).to_owned();
    }

    if let Ok(mut border) = depart.single_mut() {
        // Live at the counter with a complete ticket, and live again once the
        // train is in, where it books the next journey.
        let ready = *phase.get() == GamePhase::Arrived
            || (desk.is_ready() && *phase.get() == GamePhase::AtTheCounter);
        *border = BorderColor::all(if ready { theme::VISITED } else { theme::INK_FAINT });
    }
}

/// Keeps the header bar current.
pub fn refresh_header(
    rail: Res<Rail>,
    lang: Res<Language>,
    almanac: Res<Almanac>,
    book: Res<PassportBook>,
    counting: Res<Counting>,
    mut year: Query<
        &mut Text,
        (With<HeaderYear>, Without<HeaderCount>, Without<HeaderBalance>, Without<HeaderSeason>),
    >,
    mut season: Query<
        &mut Text,
        (With<HeaderSeason>, Without<HeaderCount>, Without<HeaderBalance>),
    >,
    mut balance: Query<
        &mut Text,
        (With<HeaderBalance>, Without<HeaderCount>, Without<HeaderSeason>),
    >,
    mut count: Query<&mut Text, With<HeaderCount>>,
) {
    if let Ok(mut text) = balance.single_mut() {
        text.0 = fill(lang.say(Phrase::Piastres), &[&counting.treasury.balance().to_string()]);
    }
    if let Ok(mut text) = year.single_mut() {
        text.0 = almanac.year().to_string();
    }
    if let Ok(mut text) = season.single_mut() {
        // Named, because an unexplained shift in the light is just a bug the
        // player cannot report.
        text.0 = lang.say(theme::Season::phrase(almanac.through_year())).to_owned();
    }
    if let Ok(mut text) = count.single_mut() {
        text.0 = format!(
            "{}\n{}",
            fill(
                lang.say(Phrase::Stamped),
                &[
                    &book.passport.visited_count().to_string(),
                    &book.passport.total_cities().to_string(),
                ]
            ),
            fill(
                lang.say(Phrase::OnTheNetwork),
                &[&rail.open_stations(almanac.year()).count().to_string(),]
            ),
        );
    }
}

/// Keeps the station board current.
pub fn refresh_board(
    rail: Res<Rail>,
    lang: Res<Language>,
    clock: Res<Clock>,
    announcer: Res<Announcer>,
    runs: Query<&TrainRun>,
    mut status: Query<
        &mut Text,
        (
            With<BoardStatus>,
            Without<BoardDetail>,
            Without<BoardAnnouncements>,
            Without<BoardRiders>,
        ),
    >,
    mut detail: Query<
        &mut Text,
        (With<BoardDetail>, Without<BoardAnnouncements>, Without<BoardRiders>),
    >,
    mut announcements: Query<&mut Text, (With<BoardAnnouncements>, Without<BoardRiders>)>,
    mut aboard: Query<&mut Text, With<BoardRiders>>,
    mut bar: Query<&mut Node, With<BoardProgressFill>>,
    riders: Query<&Riders>,
    queues: Res<Queues>,
) {
    let run = runs.iter().next();

    if let Ok(mut text) = aboard.single_mut() {
        text.0 = match riders.iter().next() {
            Some(riders) => fill(
                lang.say(Phrase::Aboard),
                &[&riders.len().to_string(), &yafa_core::traffic::TRAIN_CAPACITY.to_string()],
            ),
            None => {
                fill(lang.say(Phrase::WaitingOnPlatforms), &[&queues.total_waiting().to_string()])
            }
        };
    }

    let (headline, body, progress) = match run {
        None => (
            lang.say(Phrase::WaitingForTicket).to_owned(),
            lang.say(Phrase::CounterOpen).to_owned(),
            0.0,
        ),
        Some(run) => {
            let leg = run.current_leg();
            let headline = {
                let (phrase, at) = match run.phase() {
                    Phase::Boarding => (Phrase::BoardingAt, leg.from),
                    Phase::Running => (Phrase::InTransitTo, leg.to),
                    Phase::Dwelling => (Phrase::StandingAt, leg.from),
                    Phase::Arrived => (Phrase::ArrivedAt, run.route().destination()),
                };
                fill(lang.say(phrase), &[station_name(*lang, rail.city(at))])
            };
            let body = match run.phase() {
                Phase::Arrived => {
                    fill(lang.say(Phrase::RunComplete), &[&format!("{:.0}", run.travelled_km())])
                }
                _ => fill(
                    lang.say(Phrase::RunProgress),
                    &[
                        corridor(*lang, leg.line),
                        &format!("{:.0}", run.travelled_km()),
                        &format!("{:.0}", run.remaining_km()),
                        &format!("{:.0}", run.eta_minutes(&clock.0)),
                    ],
                ),
            };
            (headline, body, run.progress())
        }
    };

    if let Ok(mut text) = status.single_mut() {
        text.0 = headline;
    }
    if let Ok(mut text) = detail.single_mut() {
        text.0 = body;
    }
    if let Ok(mut node) = bar.single_mut() {
        node.width = Val::Percent(progress * 100.0);
    }
    if let Ok(mut text) = announcements.single_mut() {
        text.0 = announcer.lines().map(|line| format!("› {line}")).collect::<Vec<_>>().join("\n");
    }
}

/// Fills the "About the City" panel from whichever station has focus.
pub fn refresh_city_panel(
    rail: Res<Rail>,
    lang: Res<Language>,
    focus: Res<Focus>,
    book: Res<PassportBook>,
    mut parts: Query<(&CityPanelPart, &mut Text, &mut TextColor)>,
) {
    let Some(city) = focus.0 else { return };
    let data = rail.city(city);
    let stamp = book.passport.stamp_for(city);

    for (part, mut text, mut color) in &mut parts {
        text.0 = match part {
            CityPanelPart::Name => station_name(*lang, data).to_owned(),
            CityPanelPart::Subtitle => {
                let exonym = data.exonym.map_or_else(String::new, |e| format!("{e}  ·  "));
                format!("{exonym}{}  ·  {}", corridor(*lang, data.line_type), city.code())
            }
            CityPanelPart::Fact => data.historical_fact.to_owned(),
            CityPanelPart::Stamped => match stamp {
                Some(stamp) => fill(lang.say(Phrase::StampedNth), &[&ordinal(*lang, stamp.order)]),
                None => lang.say(Phrase::NotYetVisited).to_owned(),
            },
        };

        // The one line whose colour carries the station's state.
        if matches!(part, CityPanelPart::Stamped) {
            color.0 = if stamp.is_some() { theme::VISITED } else { theme::INK_SOFT };
        }
    }
}

/// Keeps the passport counter, bar, summary and stamp cards current.
pub fn refresh_passport(
    rail: Res<Rail>,
    lang: Res<Language>,
    book: Res<PassportBook>,
    mut counter: Query<
        &mut Text,
        (
            With<PassportCounter>,
            Without<PassportLines>,
            Without<PassportOdometer>,
            Without<PassportRemaining>,
        ),
    >,
    mut lines: Query<
        &mut Text,
        (With<PassportLines>, Without<PassportOdometer>, Without<PassportRemaining>),
    >,
    mut odometer: Query<&mut Text, (With<PassportOdometer>, Without<PassportRemaining>)>,
    mut remaining: Query<&mut Text, With<PassportRemaining>>,
    mut bar: Query<&mut Node, With<PassportFill>>,
    mut cards: Query<(&StampCard, &mut BackgroundColor, &mut BorderColor)>,
    mut codes: Query<(&StampCardCode, &mut TextColor)>,
    mut orders: Query<
        (&StampCardOrder, &mut Text, &mut TextColor),
        (
            Without<PassportCounter>,
            Without<PassportLines>,
            Without<PassportOdometer>,
            Without<PassportRemaining>,
            Without<StampCardCode>,
        ),
    >,
) {
    let passport = &book.passport;

    if let Ok(mut text) = counter.single_mut() {
        text.0 = fill(
            lang.say(Phrase::VisitedCities),
            &[&passport.visited_count().to_string(), &passport.total_cities().to_string()],
        );
    }
    if let Ok(mut node) = bar.single_mut() {
        node.width = Val::Percent(passport.completion() * 100.0);
    }
    if let Ok(mut text) = lines.single_mut() {
        text.0 = passport
            .line_progress(&rail)
            .iter()
            .map(|progress| {
                format!(
                    "{} {}/{}{}",
                    progress.line.code(),
                    progress.stamped,
                    progress.total,
                    if progress.is_complete() { " ✓" } else { "" },
                )
            })
            .collect::<Vec<_>>()
            .join("    ");
    }
    if let Ok(mut text) = odometer.single_mut() {
        text.0 = fill(
            lang.say(Phrase::Odometer),
            &[
                &passport.journeys_completed().to_string(),
                &format!("{:.0}", passport.total_track_km()),
            ],
        );
    }

    if let Ok(mut text) = remaining.single_mut() {
        let left = passport.unstamped();
        text.0 = match left.split_first() {
            // Name a few, so there is always somewhere obvious to go next.
            Some((_, _)) => {
                let names: Vec<&str> =
                    left.iter().take(3).map(|&c| station_name(*lang, rail.city(c))).collect();
                let more = left.len().saturating_sub(names.len());
                let tail = if more > 0 { format!(" (+{more})") } else { String::new() };
                fill(lang.say(Phrase::StillToCallAt), &[&names.join(", "), &tail])
            }
            None => lang.say(Phrase::AllStamped).to_owned(),
        };
    }

    for (StampCard(city), mut background, mut border) in &mut cards {
        let earned = passport.is_stamped(*city);
        background.0 = if earned { theme::PAPER } else { theme::PAPER_DEEP };
        *border = BorderColor::all(if earned { theme::VISITED } else { theme::BORDER });
    }
    for (StampCardCode(city), mut color) in &mut codes {
        color.0 = if passport.is_stamped(*city) { theme::VISITED } else { theme::INK_FAINT };
    }
    for (StampCardOrder(city), mut text, mut color) in &mut orders {
        match passport.stamp_for(*city) {
            Some(stamp) => {
                text.0 = stamp.order.to_string();
                color.0 = theme::INK_SOFT;
            }
            None => {
                text.0 = String::new();
                color.0 = theme::INK_FAINT;
            }
        }
    }
}

/// Starts a flash on the card that was just stamped.
pub fn flash_new_stamps(
    mut commands: Commands,
    mut stamps: MessageReader<PassportStamped>,
    cards: Query<(Entity, &StampCard)>,
) {
    for stamp in stamps.read() {
        if !stamp.fresh {
            continue;
        }
        for (entity, StampCard(city)) in &cards {
            if *city == stamp.city {
                commands.entity(entity).insert(StampFlash::new());
            }
        }
    }
}

/// Runs the flash down and takes it off again: the card presses in under the
/// stamp and settles back.
pub fn run_stamp_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashing: Query<(
        Entity,
        &StampCard,
        &mut StampFlash,
        &mut BackgroundColor,
        &mut UiTransform,
    )>,
) {
    for (entity, StampCard(city), mut flash, mut background, mut transform) in &mut flashing {
        flash.remaining -= time.delta_secs();
        if flash.remaining <= 0.0 {
            transform.scale = Vec2::ONE;
            commands.entity(entity).remove::<StampFlash>();
            continue;
        }
        let strength = flash.remaining / StampFlash::DURATION;
        background.0 = theme::PAPER.mix(&theme::VISITED, strength * 0.8);
        // Overshoot once, then settle - the press of the stamp coming down.
        transform.scale = Vec2::splat(1.0 + strength * strength * 0.16);
        transform.rotation = Rot2::degrees(card_tilt(*city) * (1.0 + strength * 0.9));
    }
}

/// "first", "second", ... for the stamp wording. Arabic takes the numeral,
/// which is how it would be written there anyway.
fn ordinal(lang: Language, order: u32) -> String {
    if lang.is_rtl() {
        return order.to_string();
    }
    const NAMES: [&str; 11] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth", "eleventh",
    ];
    NAMES
        .get(order.saturating_sub(1) as usize)
        .map_or_else(|| format!("{order}th"), |name| (*name).to_owned())
}

/// Fills the city card's track list, and offers to lay what is still on paper.
pub fn refresh_track_list(
    rail: Res<Rail>,
    lang: Res<Language>,
    focus: Res<Focus>,
    almanac: Res<Almanac>,
    counting: Res<Counting>,
    mut rows: Query<(&CityLinkRow, &mut Node)>,
    mut texts: Query<(&CityLinkText, &mut Text), Without<CityLinkBuyLabel>>,
    mut buys: Query<
        (&CityLinkBuy, &mut UiAction, &mut Node, &mut BorderColor),
        Without<CityLinkRow>,
    >,
    mut labels: Query<(&CityLinkBuyLabel, &mut Text, &mut TextColor)>,
) {
    let Some(city) = focus.0 else { return };
    let year = almanac.year();

    // What each row should say, in the order the links come off the station.
    let links: Vec<(String, Option<(yafa_core::SegmentId, i64, bool)>)> = rail
        .links(city)
        .iter()
        .map(|link| {
            let segment = rail.segment(link.segment);
            let open = segment.opening.is_open(year);
            // Short form in the list: the row has a priced control beside it
            // and no width to spare for "never completed".
            let text = format!(
                "to {} - {:.0} km, {}",
                rail.city(link.to).name,
                segment.track_km(),
                match segment.opening.year() {
                    Some(year) => format!("opened {year}"),
                    None => "on paper".to_owned(),
                },
            );
            let offer = (!open).then(|| {
                let cost = counting.tariff.track(segment.track_m);
                (segment.id, cost, counting.treasury.can_afford(cost))
            });
            (text, offer)
        })
        .take(MAX_LINKS)
        .collect();

    for (CityLinkRow(index), mut node) in &mut rows {
        node.display = if *index < links.len() { Display::Flex } else { Display::None };
    }
    for (CityLinkText(index), mut text) in &mut texts {
        text.0 = links.get(*index).map(|(line, _)| line.clone()).unwrap_or_default();
    }
    for (CityLinkBuy(index), mut action, mut node, mut border) in &mut buys {
        match links.get(*index).and_then(|(_, offer)| *offer) {
            Some((segment, _, affordable)) => {
                node.display = Display::Flex;
                *action = UiAction::OpenLine(segment);
                *border = BorderColor::all(if affordable { theme::VISITED } else { theme::BORDER });
            }
            None => node.display = Display::None,
        }
    }
    for (CityLinkBuyLabel(index), mut text, mut color) in &mut labels {
        match links.get(*index).and_then(|(_, offer)| *offer) {
            Some((_, cost, affordable)) => {
                text.0 = fill(lang.say(Phrase::LayLine), &[&cost.to_string()]);
                color.0 = if affordable { theme::VISITED } else { theme::INK_FAINT };
            }
            None => text.0.clear(),
        }
    }
}

/// Brings up the arrival card when the train draws in, and slides it away
/// again four seconds later.
pub fn run_arrival_card(
    time: Res<Time>,
    rail: Res<Rail>,
    mut arrivals: MessageReader<TrainReached>,
    mut cards: Query<(&mut ArrivalCard, &mut Node, &mut UiTransform)>,
    mut parts: Query<(&ArrivalPart, &mut Text)>,
    mut city_cards: Query<&mut Node, (With<CityPanelSheet>, Without<ArrivalCard>)>,
) {
    let Ok((mut card, mut node, mut transform)) = cards.single_mut() else { return };

    // The last station reached this frame wins: at the fastest clock the
    // train can pass two, and the newer one is the one worth reading.
    if let Some(latest) = arrivals.read().last() {
        card.remaining = ArrivalCard::DURATION;
        card.city = Some(latest.city);

        let data = rail.city(latest.city);
        for (part, mut text) in &mut parts {
            text.0 = match part {
                // Both names on the card, whichever way the interface reads.
                ArrivalPart::Name => data.name.to_owned(),
                ArrivalPart::Arabic => data.arabic.to_owned(),
                ArrivalPart::Fact => data.historical_fact.to_owned(),
            };
        }
    }

    if card.remaining <= 0.0 {
        if node.display != Display::None {
            node.display = Display::None;
            if let Ok(mut city) = city_cards.single_mut() {
                city.display = Display::Flex;
            }
        }
        return;
    }

    card.remaining = (card.remaining - time.delta_secs()).max(0.0);
    node.display = Display::Flex;
    if let Ok(mut city) = city_cards.single_mut() {
        city.display = Display::None;
    }
    transform.translation = Val2::new(Val::Px(card.offset(RIGHT_WIDTH)), Val::Px(0.0));
}

/// What a corridor is called, in the language showing.
fn corridor(lang: Language, line: LineType) -> &'static str {
    lang.say(match line {
        LineType::Coastal => Phrase::CoastalLine,
        LineType::Mountain => Phrase::MountainLine,
        LineType::Junction => Phrase::JunctionLinks,
    })
}

/// A station's name, in the language showing.
fn station_name(lang: Language, city: &yafa_core::City) -> &'static str {
    match lang {
        Language::English => city.name,
        Language::Arabic => city.arabic,
    }
}

/// Tears the interface down and builds it again in the other language.
///
/// Rebuilding rather than re-labelling: the language changes the column order
/// and the direction of every line of text as well as the words, and there is
/// no part of the layout it leaves alone. The refresh systems repopulate the
/// live readouts on the next frame.
pub fn rebuild_on_language_change(
    mut commands: Commands,
    lang: Res<Language>,
    built: Option<Res<BuiltLanguage>>,
    roots: Query<Entity, With<InterfaceRoot>>,
) {
    // Compared against what `build_ui` actually built, not against change
    // detection and not against the first language this system happens to
    // see. A toggle that lands before this system's first run would otherwise
    // be recorded as already done and silently dropped.
    let Some(built) = built else { return };
    if built.0 == *lang {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.run_system_cached(build_ui);
    // The board's log was written in the old language and cannot be
    // re-rendered, so it starts again rather than standing there in a script
    // the player has just switched away from.
    commands.run_system_cached(restart_announcements);
}

/// Swaps the language when the toggle is used.
pub fn apply_language_toggle(
    mut requests: MessageReader<ActionRequested>,
    mut lang: ResMut<Language>,
) {
    for &ActionRequested(action) in requests.read() {
        if action == UiAction::ToggleLanguage {
            *lang = lang.other();
        }
    }
}

/// What a timetable speed is called, in the language showing.
fn speed_name(lang: Language, preset: &str) -> &'static str {
    lang.say(match preset {
        "Local" => Phrase::Local,
        "Mail" => Phrase::Mail,
        _ => Phrase::Express,
    })
}

/// Empties the board and says where things stand, in the language showing.
pub fn restart_announcements(
    rail: Res<Rail>,
    almanac: Res<Almanac>,
    lang: Res<Language>,
    mut announcer: ResMut<Announcer>,
) {
    announcer.clear();
    announcer.say(fill(
        lang.say(Phrase::AnnYearStanding),
        &[
            &almanac.year().to_string(),
            &rail.open_stations(almanac.year()).count().to_string(),
            &yafa_core::CITY_COUNT.to_string(),
        ],
    ));
    announcer.say(lang.say(Phrase::AnnCounterOpen));
}

/// Winds the departure board down at the chosen timetable speed.
pub fn tick_departure_board(time: Res<Time>, clock: Res<Clock>, mut board: ResMut<DepartureBoard>) {
    let seconds_per_minute = DepartureBoard::rate(clock.time_scale);
    if seconds_per_minute > 0.0 {
        board.tick(time.delta_secs() / seconds_per_minute);
    }
}

/// Fills the board from the station the ticket departs from.
///
/// The two directions are the station's first two open neighbours, which on
/// a through station are the two ways out of it and at a terminus is the one
/// way there is.
pub fn refresh_departure_board(
    rail: Res<Rail>,
    desk: Res<Desk>,
    almanac: Res<Almanac>,
    lang: Res<Language>,
    board: Res<DepartureBoard>,
    mut panels: Query<&mut Node, (With<DepartureBoardPanel>, Without<DepartureRow>)>,
    mut rows: Query<(&DepartureRow, &mut Node), Without<DepartureBoardPanel>>,
    mut squares: Query<(&DepartureLine, &mut BackgroundColor)>,
    mut numbers: Query<(&ChildOf, &mut Text), (Without<DepartureDest>, Without<DepartureTime>)>,
    mut dests: Query<(&DepartureDest, &mut Text), Without<DepartureTime>>,
    mut times: Query<(&DepartureTime, &mut Text)>,
    lines: Query<(Entity, &DepartureLine)>,
) {
    let year = almanac.year();
    let heading: Vec<(CityId, LineType)> = desk
        .origin
        .map(|origin| {
            rail.links(origin)
                .iter()
                .filter(|link| rail.is_open(link.segment, year))
                .map(|link| (link.to, rail.segment(link.segment).line))
                .take(2)
                .collect()
        })
        .unwrap_or_default();

    if let Ok(mut node) = panels.single_mut() {
        node.display = if desk.origin.is_some() { Display::Flex } else { Display::None };
    }

    for (DepartureRow(row), mut node) in &mut rows {
        // Rows 0 and 1 are the first direction, 2 and 3 the second.
        node.display = if heading.len() > row / 2 { Display::Flex } else { Display::None };
    }
    for (DepartureLine(row), mut colour) in &mut squares {
        if let Some((_, line)) = heading.get(row / 2) {
            colour.0 =
                if row % 2 == 0 { theme::line_color(*line) } else { theme::line_color_dim(*line) };
        }
    }
    for (parent, mut text) in &mut numbers {
        if let Ok((_, DepartureLine(row))) = lines.get(parent.parent()) {
            text.0 = heading
                .get(row / 2)
                .map(|(_, line)| line_number(*line).to_owned())
                .unwrap_or_default();
        }
    }
    for (DepartureDest(row), mut text) in &mut dests {
        text.0 = heading
            .get(row / 2)
            .map(|(city, _)| station_name(*lang, rail.city(*city)).to_owned())
            .unwrap_or_default();
    }
    for (DepartureTime(row), mut text) in &mut times {
        text.0 = if heading.len() > row / 2 {
            let left = board.minutes[*row];
            if left < 0.5 {
                lang.say(Phrase::DepartingNow).to_owned()
            } else {
                fill(lang.say(Phrase::Minutes), &[&format!("{:.0}", left)])
            }
        } else {
            lang.say(Phrase::NoDeparture).to_owned()
        };
    }
}

/// The number a line carries on the map and the board.
fn line_number(line: LineType) -> &'static str {
    match line {
        LineType::Coastal => "1",
        LineType::Mountain => "2",
        LineType::Junction => "·",
    }
}
