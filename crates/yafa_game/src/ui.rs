//! The interface: a header bar, and one column of four cards beside the map -
//! Ticket, Now, the station in focus, and the Passport.
//!
//! The map is the station list: there is no second list of names to read.
//! The layout is built once at startup; after that a handful of refresh
//! systems copy state onto the widgets. Each widget is found by a marker
//! component carrying the [`CityId`] it belongs to, so nothing has to keep an
//! index into a list that might move.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::LineHeight;
use bevy::ui::Pressed;
use bevy::ui_widgets::Button;
use yafa_core::{CityId, Network, Phase};

use crate::components::*;
use crate::theme::{self, Fonts};

/// The single column of cards beside the map.
const COLUMN_WIDTH: f32 = 372.0;
/// The header bar across the top.
const HEADER_HEIGHT: f32 = 58.0;
/// Space between cards, and around the page.
const GUTTER: f32 = 14.0;

// --- Construction ---------------------------------------------------------

/// Lays out the whole interface: a header bar across the top, the map filling
/// everything under it, and one column of cards down the right-hand side.
pub fn build_ui(mut commands: Commands, rail: Res<Rail>, fonts: Res<Fonts>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            // The map lies underneath: only the cards themselves swallow
            // clicks, so stations stay reachable everywhere else.
            Pickable::IGNORE,
            Name::new("Interface"),
        ))
        .with_children(|root| {
            header_bar(root, &fonts);
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Stretch,
                    padding: UiRect::all(Val::Px(GUTTER)),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|body| {
                right_column(body, &rail, &fonts);
            });
        });
}

/// The navy bar: the game's name on the left, the passport count on the right.
fn header_bar(root: &mut ChildSpawnerCommands, fonts: &Fonts) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(HEADER_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::axes(Val::Px(22.0), Val::Px(0.0)),
            ..default()
        },
        BackgroundColor(theme::HEADER_BAR),
        Pickable { should_block_lower: true, is_hoverable: false },
        children![
            (
                Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Baseline, column_gap: Val::Px(14.0), ..default() },
                Pickable::IGNORE,
                children![
                    (
                        Text::new("Yafa Express"),
                        fonts.sign_bold(theme::HEADING + 2.0),
                        TextColor(theme::ON_HEADER),
                        Pickable::IGNORE,
                    ),
                    (
                        Text::new("يافا إكسبرس"),
                        fonts.sign(theme::HEADING),
                        TextColor(theme::ON_HEADER.with_alpha(0.8)),
                        Pickable::IGNORE,
                    ),
                ],
            ),
            (
                PassportCounter,
                Text::new("Visited cities  0 / 11"),
                fonts.sign(theme::BODY),
                TextColor(theme::ON_HEADER),
                Pickable::IGNORE,
            ),
        ],
    ));
}

// --- The column ------------------------------------------------------------

fn right_column(body: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts) {
    body.spawn((
        Node {
            width: Val::Px(COLUMN_WIDTH),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(GUTTER),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|column| {
        ticket_card(column, fonts);
        now_card(column, fonts);
        city_card(column, fonts);
        passport_card(column, rail, fonts);
    });
}

/// From, To, Depart. Everything the player needs to start a journey, and
/// nothing else.
fn ticket_card(column: &mut ChildSpawnerCommands, fonts: &Fonts) {
    column
        .spawn((theme::panel_node(), theme::panel_chrome()))
        .with_children(|panel| {
            panel.spawn(theme::panel_header(fonts, "Ticket", "click two stations on the map"));
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
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            border: UiRect::left(Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            row_gap: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(theme::PAPER_DEEP),
        BorderColor::all(accent),
        Pickable::IGNORE,
        children![
            (
                Text::new(caption.to_owned()),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ),
            (
                slot,
                Text::new("-"),
                fonts.sign_bold(theme::HEADING),
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
                column_gap: Val::Px(8.0),
                width: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(4.0)),
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
                    fonts.sign_bold(theme::BODY + 1.0),
                    TextColor(theme::ON_ACCENT),
                    Pickable::IGNORE,
                )],
            ));
            row.spawn((
                control_chrome(UiAction::ClearTicket),
                control_node(),
                children![(
                    Text::new("Clear"),
                    fonts.sign(theme::BODY),
                    TextColor(theme::INK),
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
                    TextColor(theme::INK),
                    Pickable::IGNORE,
                )],
            ));
        });
}

/// What the train is doing right now, in one line, with a bar under it.
fn now_card(column: &mut ChildSpawnerCommands, fonts: &Fonts) {
    column
        .spawn((theme::panel_node(), theme::panel_chrome()))
        .with_children(|panel| {
            panel.spawn(theme::panel_header(fonts, "Now", "Space depart · Esc clear · Tab speed"));
            panel.spawn((
                BoardStatus,
                Text::new("Waiting for a ticket"),
                fonts.sign_bold(theme::HEADING),
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
            progress_bar(panel, BoardProgressFill, theme::ACCENT);
            panel.spawn((
                BoardAnnouncements,
                Text::new(""),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                LineHeight::RelativeToFont(1.5),
                Pickable::IGNORE,
            ));
        });
}

/// A filled bar. `fill` marks the inner node so a refresh system can set its
/// width.
fn progress_bar(panel: &mut ChildSpawnerCommands, fill: impl Component, color: Color) {
    panel
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(10.0),
                border_radius: BorderRadius::all(Val::Px(5.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme::BORDER),
            Pickable::IGNORE,
        ))
        .with_child((
            fill,
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(color),
            Pickable::IGNORE,
        ));
}

/// The station in focus: its two names, one sentence of history, its lines,
/// and whether it is stamped.
fn city_card(column: &mut ChildSpawnerCommands, fonts: &Fonts) {
    column.spawn((
        theme::panel_node(),
        theme::panel_chrome(),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Baseline,
                    column_gap: Val::Px(12.0),
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    (
                        CityPanelPart::Name,
                        Text::new("Yafa"),
                        fonts.sign_bold(theme::DISPLAY),
                        TextColor(theme::INK_HEADER),
                        Pickable::IGNORE,
                    ),
                    (
                        CityPanelPart::Subtitle,
                        Text::new(""),
                        fonts.sign(theme::HEADING),
                        TextColor(theme::INK_SOFT),
                        Pickable::IGNORE,
                    ),
                ],
            ),
            (
                CityPanelPart::Fact,
                Text::new(""),
                fonts.prose(theme::BODY),
                TextColor(theme::INK),
                LineHeight::RelativeToFont(1.5),
                Pickable::IGNORE,
            ),
            (
                CityPanelPart::Links,
                Text::new(""),
                fonts.data(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                LineHeight::RelativeToFont(1.5),
                Pickable::IGNORE,
            ),
            (
                CityPanelPart::Stamped,
                Text::new(""),
                fonts.sign_bold(theme::CAPTION),
                // Overwritten by `refresh_city_panel` on the first frame.
                TextColor(theme::INK_SOFT),
                Pickable::IGNORE,
            ),
        ],
    ));
}

fn passport_card(column: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts) {
    column
        .spawn((Node { flex_grow: 1.0, ..theme::panel_node() }, theme::panel_chrome()))
        .with_children(|panel| {
            panel.spawn(theme::panel_header(fonts, "Passport", "one stamp per station"));
            progress_bar(panel, PassportFill, theme::VISITED);
            panel.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    (
                        PassportLines,
                        Text::new(""),
                        fonts.data(theme::CAPTION),
                        TextColor(theme::INK_SOFT),
                        Pickable::IGNORE,
                    ),
                    (
                        PassportOdometer,
                        Text::new(""),
                        fonts.data(theme::CAPTION),
                        TextColor(theme::INK_SOFT),
                        Pickable::IGNORE,
                    ),
                ],
            ));
            stamp_grid(panel, rail, fonts);
            panel.spawn((
                PassportRemaining,
                Text::new(""),
                fonts.prose(theme::CAPTION),
                TextColor(theme::INK_SOFT),
                LineHeight::RelativeToFont(1.5),
                Node { margin: UiRect::top(Val::Px(4.0)), ..default() },
                Pickable::IGNORE,
            ));
        });
}

/// The stamps: one chip per station, grey until it is earned, green after.
fn stamp_grid(panel: &mut ChildSpawnerCommands, rail: &Network, fonts: &Fonts) {
    panel
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                margin: UiRect::top(Val::Px(4.0)),
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
                    UiTransform::default(),
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(theme::PAPER_DEEP),
                    BorderColor::all(theme::BORDER),
                    children![
                        (
                            StampCardCode(city),
                            Text::new(data.arabic),
                            fonts.sign_bold(theme::BODY),
                            TextColor(theme::INK_FAINT),
                            Pickable::IGNORE,
                        ),
                        (
                            Text::new(data.name),
                            fonts.prose(theme::CAPTION),
                            TextColor(theme::INK_SOFT),
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
                ));
            }
        });
}

/// The layout of a clickable control. Extend it with `..control_node()`
/// rather than adding a second `Node`, which Bevy rejects as a duplicate.
fn control_node() -> Node {
    Node {
        padding: UiRect::axes(Val::Px(16.0), Val::Px(9.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
    }
}

/// Everything a clickable control needs except its layout. The depart button
/// is filled with the accent; the others are quiet outlines.
fn control_chrome(action: UiAction) -> impl Bundle {
    let primary = action == UiAction::Depart;
    (
        Button,
        action,
        Hovered::default(),
        Pickable::default(),
        BackgroundColor(if primary { theme::ACCENT } else { theme::PAPER }),
        BorderColor::all(if primary { theme::ACCENT } else { theme::SLATE }),
    )
}

// --- Refresh --------------------------------------------------------------

/// Highlights controls under the pointer. The depart button keeps its accent
/// fill and only darkens; the outline buttons tint.
pub fn style_controls(
    mut controls: Query<
        (&UiAction, &Hovered, Has<Pressed>, &mut BackgroundColor),
        Without<StampCard>,
    >,
) {
    for (action, hovered, pressed, mut background) in &mut controls {
        let primary = *action == UiAction::Depart;
        background.0 = match (primary, pressed, hovered.get()) {
            (true, true, _) => theme::ACCENT.mix(&theme::INK, 0.3),
            (true, false, true) => theme::ACCENT.mix(&theme::INK, 0.12),
            (true, false, false) => theme::ACCENT,
            (false, true, _) => theme::PAPER_PRESSED,
            (false, false, true) => theme::PAPER_DEEP,
            (false, false, false) => theme::PAPER,
        };
    }
}

/// Writes the ticket, and keeps the station list in step with it.
pub fn refresh_ticket(
    rail: Res<Rail>,
    desk: Res<Desk>,
    clock: Res<Clock>,
    phase: Res<State<GamePhase>>,
    mut slots: Query<(&TicketSlot, &mut Text), (Without<TicketRouteText>, Without<ClockLabel>)>,
    mut route_text: Query<&mut Text, (With<TicketRouteText>, Without<ClockLabel>)>,
    mut clock_label: Query<&mut Text, With<ClockLabel>>,
    mut depart: Query<&mut BackgroundColor, With<DepartButton>>,
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

    if let Ok(mut background) = depart.single_mut() {
        // Live at the counter with a complete ticket, and live again once the
        // train is in, where it books the next journey. Until then it is
        // washed out, so the eye knows there is nothing to press yet.
        let ready = *phase.get() == GamePhase::Arrived
            || (desk.is_ready() && *phase.get() == GamePhase::AtTheCounter);
        if !ready {
            background.0 = theme::ACCENT.mix(&theme::PAPER, 0.6);
        }
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
            CityPanelPart::Subtitle => data.arabic.to_owned(),
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
                None => "Not yet stamped".to_owned(),
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
        background.0 = if earned { theme::VISITED.mix(&theme::PAPER, 0.85) } else { theme::PAPER_DEEP };
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
    for (entity, StampCard(_), mut flash, mut background, mut transform) in &mut flashing {
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
