//! The interface: the ticket counter, the station board, the city panel and
//! the passport.
//!
//! The layout is built once at startup; after that a handful of refresh
//! systems copy state onto the widgets. Each widget is found by a marker
//! component carrying the [`CityId`] it belongs to, so nothing has to keep an
//! index into a list that might move.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::Button;
use yafa_core::{CityId, LineType, Network, Phase};

use crate::components::*;
use crate::theme;

const LEFT_WIDTH: f32 = 318.0;
const RIGHT_WIDTH: f32 = 348.0;
const BOARD_WIDTH: f32 = 520.0;

// --- Construction ---------------------------------------------------------

/// Lays out the whole interface.
pub fn build_ui(mut commands: Commands, rail: Res<Rail>) {
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
            ticket_counter(root, &rail);
            centre_column(root);
            right_column(root, &rail);
        });
}

// --- Ticket counter -------------------------------------------------------

fn ticket_counter(root: &mut ChildSpawnerCommands, rail: &Network) {
    root.spawn((
        Node { width: Val::Px(LEFT_WIDTH), flex_direction: FlexDirection::Column, ..default() },
        Pickable::IGNORE,
    ))
    .with_children(|column| {
        column
            .spawn((Node { flex_grow: 1.0, ..theme::panel_node() }, theme::panel_chrome()))
            .with_children(|panel| {
                panel.spawn(theme::panel_header(
                    "Ticket Counter",
                    "Choose where you board and where you are bound",
                ));
                ticket_stub(panel);
                panel.spawn((
                    TicketRouteText,
                    Text::new("Select a departure station"),
                    theme::text_font(theme::SIZE_SMALL),
                    TextColor(theme::INK_SOFT),
                    Pickable::IGNORE,
                ));
                control_row(panel);
                station_list(panel, rail);
            });
    });
}

/// The two boxes on the ticket itself.
fn ticket_stub(panel: &mut ChildSpawnerCommands) {
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
            slot_box(row, "FROM", TicketSlot::Origin, theme::OLIVE);
            slot_box(row, "TO", TicketSlot::Destination, theme::STAMP_RED);
        });
}

fn slot_box(row: &mut ChildSpawnerCommands, caption: &str, slot: TicketSlot, accent: Color) {
    row.spawn((
        Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(9.0), Val::Px(7.0)),
            border: UiRect::left(Val::Px(3.0)),
            row_gap: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(theme::PAPER_DEEP),
        BorderColor::all(accent),
        Pickable::IGNORE,
        children![
            (
                Text::new(caption.to_owned()),
                theme::text_font(theme::SIZE_TINY),
                TextColor(theme::INK_FAINT),
                Pickable::IGNORE,
            ),
            (
                slot,
                Text::new("-"),
                theme::text_font(theme::SIZE_BODY),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ),
        ],
    ));
}

fn control_row(panel: &mut ChildSpawnerCommands) {
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
                    Text::new("DEPART"),
                    theme::text_font(theme::SIZE_SMALL),
                    TextColor(theme::INK),
                    Pickable::IGNORE,
                )],
            ));
            row.spawn((
                control_chrome(UiAction::ClearTicket),
                control_node(),
                children![(
                    Text::new("CLEAR"),
                    theme::text_font(theme::SIZE_SMALL),
                    TextColor(theme::INK_SOFT),
                    Pickable::IGNORE,
                )],
            ));
            row.spawn((
                control_chrome(UiAction::CycleClock),
                control_node(),
                children![(
                    ClockLabel,
                    Text::new("EXPRESS"),
                    theme::text_font(theme::SIZE_SMALL),
                    TextColor(theme::INK_SOFT),
                    Pickable::IGNORE,
                )],
            ));
        });
}

/// Every station, grouped by the corridor it belongs to.
fn station_list(panel: &mut ChildSpawnerCommands, rail: &Network) {
    panel
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                width: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(4.0)),
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
                        column_gap: Val::Px(6.0),
                        margin: UiRect::top(Val::Px(7.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![
                        (
                            Node { width: Val::Px(14.0), height: Val::Px(3.0), ..default() },
                            BackgroundColor(theme::line_color(line)),
                            Pickable::IGNORE,
                        ),
                        (
                            Text::new(line.label().to_uppercase()),
                            theme::text_font(theme::SIZE_TINY),
                            TextColor(theme::INK_SOFT),
                            Pickable::IGNORE,
                        ),
                    ],
                ));

                for city in rail.cities_on(line) {
                    station_row(list, city.id, city.name, city.exonym);
                }
            }
        });
}

fn station_row(
    list: &mut ChildSpawnerCommands,
    city: CityId,
    name: &str,
    exonym: Option<&'static str>,
) {
    let subtitle = exonym.map_or_else(String::new, |e| format!("({e})"));

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
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border: UiRect::left(Val::Px(2.0)),
            width: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(theme::PAPER),
        BorderColor::all(Color::NONE),
        children![
            (
                Text::new(city.code()),
                theme::text_font(theme::SIZE_TINY),
                TextColor(theme::BRASS_DEEP),
                Pickable::IGNORE,
            ),
            (
                Text::new(name.to_owned()),
                theme::text_font(theme::SIZE_BODY),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ),
            (
                Text::new(subtitle),
                theme::text_font(theme::SIZE_TINY),
                TextColor(theme::INK_FAINT),
                Node { flex_grow: 1.0, ..default() },
                Pickable::IGNORE,
            ),
            (
                StationRowNote(city),
                Text::new(""),
                theme::text_font(theme::SIZE_TINY),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ),
        ],
    ));
}

// --- Centre: the title and the station board ------------------------------

fn centre_column(root: &mut ChildSpawnerCommands) {
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
                ..default()
            },
            Pickable::IGNORE,
            children![
                (
                    Text::new("YAFA EXPRESS"),
                    theme::text_font(theme::SIZE_TITLE),
                    TextColor(theme::BRASS),
                    Pickable::IGNORE,
                ),
                (
                    Text::new("the railways of historical Palestine"),
                    theme::text_font(theme::SIZE_TINY),
                    TextColor(theme::INK_FAINT),
                    Pickable::IGNORE,
                ),
            ],
        ));

        station_board(centre);
    });
}

fn station_board(centre: &mut ChildSpawnerCommands) {
    centre
        .spawn((Node { width: Val::Px(BOARD_WIDTH), ..theme::panel_node() }, theme::panel_chrome()))
        .with_children(|panel| {
            panel.spawn(theme::panel_header(
                "Station Board",
                "Departures, arrivals and the state of the line",
            ));
            panel.spawn((
                BoardStatus,
                Text::new("Waiting for a ticket"),
                theme::text_font(theme::SIZE_DISPLAY),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ));
            panel.spawn((
                BoardDetail,
                Text::new("The counter is open."),
                theme::text_font(theme::SIZE_SMALL),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ));
            progress_bar(panel, BoardProgressFill, theme::OLIVE);
            panel.spawn((
                BoardAnnouncements,
                Text::new(""),
                theme::text_font(theme::SIZE_SMALL),
                TextColor(theme::INK_SOFT),
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
                height: Val::Px(7.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::PAPER_DEEP),
            BorderColor::all(theme::BRASS_DEEP),
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

fn right_column(root: &mut ChildSpawnerCommands, rail: &Network) {
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
        city_panel(column);
        passport_panel(column, rail);
    });
}

fn city_panel(column: &mut ChildSpawnerCommands) {
    column.spawn((
        theme::panel_node(),
        theme::panel_chrome(),
        children![
            theme::panel_header("About the City", "Click any station, on the map or in the list"),
            (
                CityPanelPart::Name,
                Text::new("Yafa"),
                theme::text_font(theme::SIZE_DISPLAY),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ),
            (
                CityPanelPart::Subtitle,
                Text::new(""),
                theme::text_font(theme::SIZE_TINY),
                TextColor(theme::INK_FAINT),
                Pickable::IGNORE,
            ),
            (
                CityPanelPart::Fact,
                Text::new(""),
                theme::text_font(theme::SIZE_SMALL),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ),
            (
                CityPanelPart::Links,
                Text::new(""),
                theme::text_font(theme::SIZE_TINY),
                TextColor(theme::BRASS_DEEP),
                Pickable::IGNORE,
            ),
            (
                CityPanelPart::Stamped,
                Text::new(""),
                theme::text_font(theme::SIZE_TINY),
                TextColor(theme::STAMP_RED),
                Pickable::IGNORE,
            ),
        ],
    ));
}

fn passport_panel(column: &mut ChildSpawnerCommands, rail: &Network) {
    column
        .spawn((Node { flex_grow: 1.0, ..theme::panel_node() }, theme::panel_chrome()))
        .with_children(|panel| {
            panel.spawn(theme::panel_header("Passport", "One stamp for every station called at"));
            panel.spawn((
                PassportCounter,
                Text::new("Visited Cities: 0 / 11"),
                // Large rather than display size: at 26px the line wraps
                // inside the passport's column.
                theme::text_font(theme::SIZE_LARGE),
                TextColor(theme::INK),
                Pickable::IGNORE,
            ));
            progress_bar(panel, PassportFill, theme::STAMP_RED);
            panel.spawn((
                PassportLines,
                Text::new(""),
                theme::text_font(theme::SIZE_SMALL),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ));
            panel.spawn((
                PassportOdometer,
                Text::new(""),
                theme::text_font(theme::SIZE_TINY),
                TextColor(theme::INK_FAINT),
                Pickable::IGNORE,
            ));
            stamp_grid(panel, rail);
        });
}

/// The grid of stamp cards, one per station, locked until it is earned.
fn stamp_grid(panel: &mut ChildSpawnerCommands, rail: &Network) {
    panel
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                margin: UiRect::top(Val::Px(6.0)),
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
                    Node {
                        width: Val::Px(74.0),
                        height: Val::Px(46.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(1.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(theme::PAPER_DEEP),
                    BorderColor::all(theme::INK_FAINT),
                    children![
                        (
                            StampCardCode(city),
                            Text::new(city.code()),
                            theme::text_font(theme::SIZE_HEADING),
                            TextColor(theme::INK_FAINT),
                            Pickable::IGNORE,
                        ),
                        (
                            Text::new(data.name),
                            theme::text_font(theme::SIZE_TINY),
                            TextColor(theme::INK_FAINT),
                            Pickable::IGNORE,
                        ),
                        (
                            StampCardOrder(city),
                            Text::new("locked"),
                            theme::text_font(theme::SIZE_TINY),
                            TextColor(theme::INK_FAINT),
                            Pickable::IGNORE,
                        ),
                    ],
                ));
            }
        });
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
        BorderColor::all(theme::BRASS_DEEP),
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
                    "{:.0} km  -  {} calls  -  {:.0} min  -  {}",
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
        text.0 = clock.clock_label().to_uppercase();
    }

    for (StationRow(city), hovered, mut background, mut border) in &mut rows {
        let (bg, accent) = if desk.origin == Some(*city) {
            (theme::PAPER_PRESSED, theme::OLIVE)
        } else if desk.destination == Some(*city) {
            (theme::PAPER_PRESSED, theme::STAMP_RED)
        } else if hovered.get() {
            (theme::PAPER_DEEP, theme::BRASS)
        } else {
            (theme::PAPER, Color::NONE)
        };
        background.0 = bg;
        *border = BorderColor::all(accent);
    }

    for (StationRowNote(city), mut text, mut color) in &mut notes {
        let (note, tint) = if desk.origin == Some(*city) {
            ("FROM", theme::OLIVE)
        } else if desk.destination == Some(*city) {
            ("TO", theme::STAMP_RED)
        } else if book.passport.is_stamped(*city) {
            ("stamped", theme::BRASS_DEEP)
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
        *border = BorderColor::all(if ready { theme::OLIVE } else { theme::INK_FAINT });
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
        text.0 = announcer.lines().collect::<Vec<_>>().join("\n");
    }
}

/// Fills the "About the City" panel from whichever station has focus.
pub fn refresh_city_panel(
    rail: Res<Rail>,
    focus: Res<Focus>,
    book: Res<PassportBook>,
    mut parts: Query<(&CityPanelPart, &mut Text)>,
) {
    let Some(city) = focus.0 else { return };
    let data = rail.city(city);
    let stamp = book.passport.stamp_for(city);

    for (part, mut text) in &mut parts {
        text.0 = match part {
            CityPanelPart::Name => data.name.to_owned(),
            CityPanelPart::Subtitle => {
                let exonym = data.exonym.map_or_else(String::new, |e| format!("{e}  -  "));
                format!("{exonym}{}  -  station code {}", data.line_type.label(), city.code())
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
                    format!("STAMPED - the {} station in this passport", ordinal(stamp.order))
                }
                None => "NOT YET VISITED - buy a ticket through here".to_owned(),
            },
        };
    }
}

/// Keeps the passport counter, bar, summary and stamp cards current.
pub fn refresh_passport(
    rail: Res<Rail>,
    book: Res<PassportBook>,
    mut counter: Query<
        &mut Text,
        (With<PassportCounter>, Without<PassportLines>, Without<PassportOdometer>),
    >,
    mut lines: Query<&mut Text, (With<PassportLines>, Without<PassportOdometer>)>,
    mut odometer: Query<&mut Text, With<PassportOdometer>>,
    mut fill: Query<&mut Node, With<PassportFill>>,
    mut cards: Query<(&StampCard, &mut BackgroundColor, &mut BorderColor)>,
    mut codes: Query<(&StampCardCode, &mut TextColor)>,
    mut orders: Query<
        (&StampCardOrder, &mut Text, &mut TextColor),
        (
            Without<PassportCounter>,
            Without<PassportLines>,
            Without<PassportOdometer>,
            Without<StampCardCode>,
        ),
    >,
) {
    let passport = &book.passport;

    if let Ok(mut text) = counter.single_mut() {
        text.0 = passport.counter();
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
                    "{}  {} / {}{}",
                    progress.line.code(),
                    progress.stamped,
                    progress.total,
                    if progress.is_complete() { "  complete" } else { "" },
                )
            })
            .collect::<Vec<_>>()
            .join("     ");
    }
    if let Ok(mut text) = odometer.single_mut() {
        text.0 = format!(
            "{} journeys  -  {:.0} km of track behind you",
            passport.journeys_completed(),
            passport.total_track_km(),
        );
    }

    for (StampCard(city), mut background, mut border) in &mut cards {
        let earned = passport.is_stamped(*city);
        background.0 = if earned { theme::PAPER } else { theme::PAPER_DEEP };
        *border = BorderColor::all(if earned {
            theme::line_color(rail.city(*city).line_type)
        } else {
            theme::INK_FAINT
        });
    }
    for (StampCardCode(city), mut color) in &mut codes {
        color.0 = if passport.is_stamped(*city) { theme::STAMP_RED } else { theme::INK_FAINT };
    }
    for (StampCardOrder(city), mut text, mut color) in &mut orders {
        match passport.stamp_for(*city) {
            Some(stamp) => {
                text.0 = format!("no. {}", stamp.order);
                color.0 = theme::INK_SOFT;
            }
            None => {
                text.0 = "locked".to_owned();
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

/// Runs the flash down and takes it off again.
pub fn run_stamp_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashing: Query<(Entity, &mut StampFlash, &mut BackgroundColor)>,
) {
    for (entity, mut flash, mut background) in &mut flashing {
        flash.remaining -= time.delta_secs();
        if flash.remaining <= 0.0 {
            commands.entity(entity).remove::<StampFlash>();
            continue;
        }
        let strength = flash.remaining / StampFlash::DURATION;
        background.0 = theme::PAPER.mix(&theme::BRASS, strength * 0.8);
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
