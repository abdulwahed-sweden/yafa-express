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
use crate::theme::{self, Fonts};

const LEFT_WIDTH: f32 = 318.0;
const RIGHT_WIDTH: f32 = 348.0;
const BOARD_WIDTH: f32 = 520.0;
/// Stamp cards, four to a row inside the passport.
const CARD_WIDTH: f32 = 74.0;
const CARD_HEIGHT: f32 = 48.0;

// --- Construction ---------------------------------------------------------

/// Lays out the whole interface.
pub fn build_ui(mut commands: Commands, rail: Res<Rail>, fonts: Res<Fonts>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
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
            ticket_counter(root, &rail, &fonts);
            centre_column(root, &fonts);
            right_column(root, &rail, &fonts);
        });
}

// --- Ticket counter -------------------------------------------------------

fn ticket_counter(root: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts) {
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
                    "Ticket Counter",
                    "Choose where you board and where you are bound",
                ));
                ticket_stub(panel, fonts);
                panel.spawn((
                    TicketRouteText,
                    Text::new("Select a departure station"),
                    fonts.data(theme::CAPTION),
                    TextColor(theme::INK_SOFT),
                    LineHeight::RelativeToFont(1.45),
                    Pickable::IGNORE,
                ));
                control_row(panel, fonts);
                station_list(panel, rail, fonts);
                keyboard_hints(panel, fonts);
            });
    });
}

/// The two boxes on the ticket itself.
fn ticket_stub(panel: &mut ChildSpawnerCommands, fonts: &Fonts) {
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
            slot_box(row, fonts, "From", TicketSlot::Origin, theme::VISITED);
            slot_box(row, fonts, "To", TicketSlot::Destination, theme::PENDING);
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

fn control_row(panel: &mut ChildSpawnerCommands, fonts: &Fonts) {
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
                    Text::new("Depart"),
                    fonts.sign_bold(theme::BODY),
                    TextColor(theme::INK),
                    Pickable::IGNORE,
                )],
            ));
            row.spawn((
                control_chrome(UiAction::ClearTicket),
                control_node(),
                children![(
                    Text::new("Clear"),
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
                    Text::new("Express"),
                    fonts.sign(theme::BODY),
                    TextColor(theme::INK_SOFT),
                    Pickable::IGNORE,
                )],
            ));
        });
}

/// The shortcuts, set at the foot of the counter where there is room for them.
fn keyboard_hints(panel: &mut ChildSpawnerCommands, fonts: &Fonts) {
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
                Text::new("Space depart · Esc clear · Tab timetable"),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_FAINT),
                Pickable::IGNORE,
            ),
        ],
    ));
}

/// Every station, grouped by the corridor it belongs to.
fn station_list(panel: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts) {
    panel
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                width: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(3.0)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|list| {
            for line in LineType::ALL {
                list.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(7.0),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![
                        (
                            Node { width: Val::Px(16.0), height: Val::Px(3.0), ..default() },
                            BackgroundColor(theme::line_color(line)),
                            Pickable::IGNORE,
                        ),
                        (
                            Text::new(line.label().to_owned()),
                            fonts.data(theme::CAPTION),
                            TextColor(theme::INK_SOFT),
                            Pickable::IGNORE,
                        ),
                    ],
                ));

                for city in rail.cities_on(line) {
                    station_row(list, fonts, city.id, city.name, city.exonym);
                }
            }
        });
}

fn station_row(
    list: &mut ChildSpawnerCommands,
    fonts: &Fonts,
    city: CityId,
    name: &str,
    exonym: Option<&'static str>,
) {
    let subtitle = exonym.map_or_else(String::new, |e| e.to_owned());

    list.spawn((
        Button,
        UiAction::PickCity(city),
        StationRow(city),
        Hovered::default(),
        Pickable::default(),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
            border: UiRect::left(Val::Px(2.0)),
            width: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(theme::PAPER),
        BorderColor::all(Color::NONE),
        children![
            (
                Text::new(city.code()),
                fonts.data(theme::CAPTION),
                TextColor(theme::SLATE),
                Pickable::IGNORE,
            ),
            (
                Text::new(name.to_owned()),
                fonts.sign(theme::SIGN),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ),
            (
                Text::new(subtitle),
                fonts.prose(theme::CAPTION),
                TextColor(theme::INK_FAINT),
                Node { flex_grow: 1.0, ..default() },
                Pickable::IGNORE,
            ),
            (
                StationRowNote(city),
                Text::new(""),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ),
        ],
    ));
}

// --- Centre: the title and the station board ------------------------------

fn centre_column(root: &mut ChildSpawnerCommands, fonts: &Fonts) {
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
        centre.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(1.0),
                ..default()
            },
            Pickable::IGNORE,
            children![
                (
                    Text::new("Yafa Express"),
                    fonts.sign_bold(theme::DISPLAY),
                    TextColor(theme::INK_HEADER),
                    Pickable::IGNORE,
                ),
                (
                    Text::new("the railways of historical Palestine"),
                    fonts.prose(theme::CAPTION),
                    TextColor(theme::INK_FAINT),
                    Pickable::IGNORE,
                ),
            ],
        ));

        station_board(centre, fonts);
    });
}

fn station_board(centre: &mut ChildSpawnerCommands, fonts: &Fonts) {
    centre
        .spawn((Node { width: Val::Px(BOARD_WIDTH), ..theme::panel_node() }, theme::panel_chrome()))
        .with_children(|panel| {
            panel.spawn(theme::panel_header(
                fonts,
                "Station Board",
                "Departures, arrivals and the state of the line",
            ));
            panel.spawn((
                BoardStatus,
                Text::new("Waiting for a ticket"),
                fonts.sign(theme::DISPLAY),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ));
            panel.spawn((
                BoardDetail,
                Text::new("The counter is open."),
                fonts.prose(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                LineHeight::RelativeToFont(1.4),
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

fn right_column(root: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts) {
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
        city_panel(column, fonts);
        passport_panel(column, rail, fonts);
    });
}

fn city_panel(column: &mut ChildSpawnerCommands, fonts: &Fonts) {
    column.spawn((
        theme::panel_node(),
        theme::panel_chrome(),
        children![
            theme::panel_header(
                fonts,
                "About the City",
                "Click any station, on the map or in the list"
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
            (
                CityPanelPart::Links,
                Text::new(""),
                fonts.data(theme::CAPTION),
                TextColor(theme::SLATE),
                LineHeight::RelativeToFont(1.5),
                Pickable::IGNORE,
            ),
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

fn passport_panel(column: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts) {
    column
        .spawn((Node { flex_grow: 1.0, ..theme::panel_node() }, theme::panel_chrome()))
        .with_children(|panel| {
            panel.spawn(theme::panel_header(
                fonts,
                "Passport",
                "One stamp for every station called at",
            ));
            panel.spawn((
                PassportCounter,
                Text::new("Visited cities  0 / 11"),
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
            stamp_grid(panel, rail, fonts);
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
fn stamp_grid(panel: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts) {
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
                            Text::new(data.name),
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
    desk: Res<Desk>,
    clock: Res<Clock>,
    book: Res<PassportBook>,
    phase: Res<State<GamePhase>>,
    mut slots: Query<(&TicketSlot, &mut Text), (Without<TicketRouteText>, Without<ClockLabel>)>,
    mut route_text: Query<&mut Text, (With<TicketRouteText>, Without<ClockLabel>)>,
    mut clock_label: Query<&mut Text, With<ClockLabel>>,
    mut rows: Query<(&StationRow, &Hovered, &mut BackgroundColor, &mut BorderColor)>,
    mut notes: Query<
        (&StationRowNote, &mut Text, &mut TextColor),
        (Without<TicketSlot>, Without<TicketRouteText>, Without<ClockLabel>),
    >,
    mut depart: Query<&mut BorderColor, (With<DepartButton>, Without<StationRow>)>,
) {
    for (slot, mut text) in &mut slots {
        let city = match slot {
            TicketSlot::Origin => desk.origin,
            TicketSlot::Destination => desk.destination,
        };
        text.0 = city.map_or_else(|| "-".to_owned(), |c| rail.city(c).name.to_owned());
    }

    if let Ok(mut text) = route_text.single_mut() {
        text.0 = match desk.issue(&rail) {
            Some(route) => {
                // Corridor codes, not full names: the long form wrapped to
                // three lines and pushed the station list down the panel.
                let lines: Vec<&str> = route.lines_used().iter().map(|l| l.code()).collect();
                format!(
                    "{:.0} km  ·  {} calls  ·  {:.0} min  ·  {}",
                    route.total_km(),
                    route.legs.len(),
                    clock.running_minutes(route.total_m),
                    lines.join(" + "),
                )
            }
            None => desk.summary(&rail),
        };
    }

    if let Ok(mut text) = clock_label.single_mut() {
        text.0 = clock.clock_label().to_owned();
    }

    for (StationRow(city), hovered, mut background, mut border) in &mut rows {
        let (bg, accent) = if desk.origin == Some(*city) {
            (theme::PAPER_PRESSED, theme::VISITED)
        } else if desk.destination == Some(*city) {
            (theme::PAPER_PRESSED, theme::PENDING)
        } else if hovered.get() {
            (theme::PAPER_DEEP, theme::INK_FAINT)
        } else {
            (theme::PAPER, Color::NONE)
        };
        background.0 = bg;
        *border = BorderColor::all(accent);
    }

    for (StationRowNote(city), mut text, mut color) in &mut notes {
        let (note, tint) = if desk.origin == Some(*city) {
            ("From", theme::VISITED)
        } else if desk.destination == Some(*city) {
            ("To", theme::PENDING)
        } else if book.passport.is_stamped(*city) {
            ("✓", theme::VISITED)
        } else {
            ("", theme::INK_FAINT)
        };
        text.0 = note.to_owned();
        color.0 = tint;
    }

    if let Ok(mut border) = depart.single_mut() {
        // Live at the counter with a complete ticket, and live again once the
        // train is in, where it books the next journey.
        let ready = *phase.get() == GamePhase::Arrived
            || (desk.is_ready() && *phase.get() == GamePhase::AtTheCounter);
        *border = BorderColor::all(if ready { theme::VISITED } else { theme::INK_FAINT });
    }
}

/// Keeps the station board current.
pub fn refresh_board(
    rail: Res<Rail>,
    clock: Res<Clock>,
    announcer: Res<Announcer>,
    runs: Query<&TrainRun>,
    mut status: Query<
        &mut Text,
        (With<BoardStatus>, Without<BoardDetail>, Without<BoardAnnouncements>),
    >,
    mut detail: Query<&mut Text, (With<BoardDetail>, Without<BoardAnnouncements>)>,
    mut announcements: Query<&mut Text, With<BoardAnnouncements>>,
    mut fill: Query<&mut Node, With<BoardProgressFill>>,
) {
    let run = runs.iter().next();

    let (headline, body, progress) = match run {
        None => (
            "Waiting for a ticket".to_owned(),
            "The counter is open. Pick a departure and a destination.".to_owned(),
            0.0,
        ),
        Some(run) => {
            let leg = run.current_leg();
            let headline = match run.phase() {
                Phase::Boarding => format!("Boarding at {}", rail.city(leg.from).name),
                Phase::Running => format!("In transit to {}", rail.city(leg.to).name),
                Phase::Dwelling => format!("Standing at {}", rail.city(leg.from).name),
                Phase::Arrived => {
                    format!("Arrived at {}", rail.city(run.route().destination()).name)
                }
            };
            let body = match run.phase() {
                Phase::Arrived => format!(
                    "{:.0} km run complete. Depart again, or press Space, for a new ticket.",
                    run.travelled_km()
                ),
                _ => format!(
                    "{} - {:.0} km run, {:.0} km to go, about {:.0} min on the timetable.",
                    leg.line.label(),
                    run.travelled_km(),
                    run.remaining_km(),
                    run.eta_minutes(&clock.0),
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
    if let Ok(mut node) = fill.single_mut() {
        node.width = Val::Percent(progress * 100.0);
    }
    if let Ok(mut text) = announcements.single_mut() {
        text.0 = announcer.lines().map(|line| format!("› {line}")).collect::<Vec<_>>().join("\n");
    }
}

/// Fills the "About the City" panel from whichever station has focus.
pub fn refresh_city_panel(
    rail: Res<Rail>,
    focus: Res<Focus>,
    book: Res<PassportBook>,
    mut parts: Query<(&CityPanelPart, &mut Text, &mut TextColor)>,
) {
    let Some(city) = focus.0 else { return };
    let data = rail.city(city);
    let stamp = book.passport.stamp_for(city);

    for (part, mut text, mut color) in &mut parts {
        text.0 = match part {
            CityPanelPart::Name => data.name.to_owned(),
            CityPanelPart::Subtitle => {
                let exonym = data.exonym.map_or_else(String::new, |e| format!("{e}  ·  "));
                format!("{exonym}{}  ·  {}", data.line_type.label(), city.code())
            }
            CityPanelPart::Fact => data.historical_fact.to_owned(),
            CityPanelPart::Links => rail
                .links(city)
                .iter()
                .map(|link| {
                    let segment = rail.segment(link.segment);
                    format!(
                        "to {} - {:.0} km, {}",
                        rail.city(link.to).name,
                        segment.track_km(),
                        segment.opening.label(),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
            CityPanelPart::Stamped => match stamp {
                Some(stamp) => {
                    format!("Stamped - the {} station in this passport", ordinal(stamp.order))
                }
                None => "Not yet visited - buy a ticket through here".to_owned(),
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
    mut fill: Query<&mut Node, With<PassportFill>>,
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
        text.0 =
            format!("Visited cities  {} / {}", passport.visited_count(), passport.total_cities());
    }
    if let Ok(mut node) = fill.single_mut() {
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
        text.0 = format!(
            "{} journeys  ·  {:.0} km of track behind you",
            passport.journeys_completed(),
            passport.total_track_km(),
        );
    }

    if let Ok(mut text) = remaining.single_mut() {
        let left = passport.unstamped();
        text.0 = match left.split_first() {
            // Name a few, so there is always somewhere obvious to go next.
            Some((_, _)) => {
                let names: Vec<&str> = left.iter().take(3).map(|&c| rail.city(c).name).collect();
                let more = left.len().saturating_sub(names.len());
                let tail = if more > 0 { format!(", and {more} more") } else { String::new() };
                format!("Still to call at: {}{tail}.", names.join(", "))
            }
            None => "Every station on the network is stamped.".to_owned(),
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

/// "first", "second", ... for the stamp wording.
fn ordinal(order: u32) -> String {
    const NAMES: [&str; 11] = [
        "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
        "tenth", "eleventh",
    ];
    NAMES
        .get(order.saturating_sub(1) as usize)
        .map_or_else(|| format!("{order}th"), |name| (*name).to_owned())
}
