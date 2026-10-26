//! Builds the real interface and the real map, headless.
//!
//! Bevy rejects a bundle with two of the same component, and it rejects a
//! system whose queries can alias - both at runtime, both only when the code
//! actually runs. Spawning the whole layout and then ticking every refresh
//! system turns those into ordinary test failures.

use std::time::Duration;

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::sprite::{SpritePickingMode, SpritePickingSettings};
use bevy::state::app::StatesPlugin;
use yafa_core::{CITY_COUNT, CityId, Passport};
use yafa_game::components::*;
use yafa_game::theme::Fonts;
use yafa_game::{GameSet, SimulationPlugin, map, ui};

/// The game with its interface built, but nothing rendering it.
///
/// `map::spawn_water` and the gizmo systems are left out: they are the parts
/// of the presentation layer that need the renderer's own resources.
fn headless_with_ui(name: &str) -> App {
    let mut app = App::new();
    app.add_plugins((InputPlugin, StatesPlugin))
        .add_plugins(SimulationPlugin {
            passport: Passport::new("Test Traveller"),
            passport_path: std::env::temp_dir()
                .join("yafa-express-tests")
                .join(format!("layout-{name}.json")),
            traffic_seed: 0xC0FFEE,
        })
        // No asset server here, so the built-in face stands in for all three.
        .insert_resource(Fonts::builtin())
        .insert_resource(UiPickingSettings { require_markers: true })
        .insert_resource(SpritePickingSettings {
            require_markers: false,
            picking_mode: SpritePickingMode::BoundingBox,
        })
        .add_systems(Startup, (map::spawn_map, ui::build_ui))
        .add_systems(
            Update,
            (
                ui::style_controls,
                ui::refresh_ticket,
                ui::refresh_board,
                ui::refresh_city_panel,
                ui::refresh_track_list,
                ui::refresh_passport,
                ui::flash_new_stamps,
                ui::run_stamp_flash,
                ui::run_arrival_card,
            )
                .chain()
                .in_set(GameSet::Present),
        );
    app.init_resource::<Time>();
    app.update();
    app
}

fn tick(app: &mut App, dt: f32) {
    app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(dt));
    app.update();
}

fn text_of<M: Component>(app: &mut App) -> String {
    let mut query = app.world_mut().query_filtered::<&Text, With<M>>();
    query.single(app.world()).expect("exactly one of this widget").0.clone()
}

#[test]
fn the_whole_layout_spawns_and_refreshes() {
    // Spawning is where a duplicate component in a bundle shows up, and
    // ticking is where two queries that can alias show up.
    let mut app = headless_with_ui("the_whole_layout_spawns_and_refreshes");
    for _ in 0..3 {
        tick(&mut app, 1.0 / 60.0);
    }
}

#[test]
fn there_is_one_marker_and_one_stamp_card_per_station() {
    let mut app = headless_with_ui("there_is_one_marker_and_one_stamp_card_per_station");

    let mut dots = app.world_mut().query::<&StationDot>();
    assert_eq!(dots.iter(app.world()).count(), CITY_COUNT);

    let mut labels = app.world_mut().query::<&StationLabel>();
    assert_eq!(labels.iter(app.world()).count(), CITY_COUNT);

    let mut rows = app.world_mut().query::<&StationRow>();
    assert_eq!(rows.iter(app.world()).count(), CITY_COUNT);

    let mut cards = app.world_mut().query::<&StampCard>();
    assert_eq!(cards.iter(app.world()).count(), CITY_COUNT);

    let mut trains = app.world_mut().query::<&Train>();
    assert_eq!(trains.iter(app.world()).count(), 1);
}

#[test]
fn the_ticket_boxes_follow_the_counter() {
    let mut app = headless_with_ui("the_ticket_boxes_follow_the_counter");
    tick(&mut app, 1.0 / 60.0);

    app.world_mut().write_message(ActionRequested(UiAction::PickCity(CityId::Yafa)));
    tick(&mut app, 1.0 / 60.0);

    let origins: Vec<String> = {
        let mut query = app.world_mut().query::<(&TicketSlot, &Text)>();
        query
            .iter(app.world())
            .filter(|(slot, _)| matches!(slot, TicketSlot::Origin))
            .map(|(_, text)| text.0.clone())
            .collect()
    };
    assert_eq!(origins, vec!["Yafa".to_owned()]);
}

#[test]
fn the_board_reads_out_the_state_of_the_line() {
    let mut app = headless_with_ui("the_board_reads_out_the_state_of_the_line");
    tick(&mut app, 1.0 / 60.0);
    // Case is a presentation choice; what matters is which state it reports.
    assert!(
        text_of::<BoardStatus>(&mut app).to_lowercase().contains("waiting for a ticket"),
        "the idle board should say it is waiting"
    );

    app.world_mut().write_message(ActionRequested(UiAction::PickCity(CityId::Yafa)));
    app.world_mut().write_message(ActionRequested(UiAction::PickCity(CityId::AlQuds)));
    tick(&mut app, 1.0 / 60.0);
    app.world_mut().write_message(ActionRequested(UiAction::Depart));
    tick(&mut app, 1.0 / 60.0);

    let status = text_of::<BoardStatus>(&mut app);
    assert!(status.to_lowercase().contains("yafa"), "the board should name the station: {status}");
}

#[test]
fn the_city_panel_follows_the_focus() {
    let mut app = headless_with_ui("the_city_panel_follows_the_focus");
    app.world_mut().write_message(ActionRequested(UiAction::ShowCity(CityId::Beisan)));
    tick(&mut app, 1.0 / 60.0);

    let names: Vec<String> = {
        let mut query = app.world_mut().query::<(&CityPanelPart, &Text)>();
        query
            .iter(app.world())
            .filter(|(part, _)| matches!(part, CityPanelPart::Name))
            .map(|(_, text)| text.0.clone())
            .collect()
    };
    assert_eq!(names.len(), 1);
    assert!(
        names[0].eq_ignore_ascii_case("Beisan"),
        "the panel should name the focused station, not {:?}",
        names[0]
    );
}

#[test]
fn the_passport_counter_starts_empty_and_is_rendered() {
    let mut app = headless_with_ui("the_passport_counter_starts_empty_and_is_rendered");
    tick(&mut app, 1.0 / 60.0);
    // Content, not casing: the panel is free to shout.
    let counter = text_of::<PassportCounter>(&mut app);
    assert!(counter.to_lowercase().contains("visited cities"), "{counter}");
    assert!(counter.contains(&format!("0 / {CITY_COUNT}")), "{counter}");
}

#[test]
fn every_part_of_the_city_panel_is_spawned_and_filled() {
    // The panel is built from one entity per `CityPanelPart`. A part with a
    // refresh arm but no widget would otherwise just quietly show nothing.
    let mut app = headless_with_ui("every_part_of_the_city_panel_is_spawned_and_filled");
    app.world_mut().write_message(ActionRequested(UiAction::ShowCity(CityId::Yafa)));
    tick(&mut app, 1.0 / 60.0);

    let mut query = app.world_mut().query::<(&CityPanelPart, &Text)>();
    let parts: Vec<(CityPanelPart, String)> =
        query.iter(app.world()).map(|(part, text)| (*part, text.0.clone())).collect();

    assert_eq!(parts.len(), 4, "one widget per part of the panel");
    for (part, text) in &parts {
        assert!(!text.is_empty(), "{part:?} was left blank");
    }
}

#[test]
fn the_track_list_names_the_lines_out_of_a_station() {
    let mut app = headless_with_ui("the_track_list_names_the_lines_out_of_a_station");
    app.world_mut().write_message(ActionRequested(UiAction::ShowCity(CityId::Yafa)));
    tick(&mut app, 1.0 / 60.0);

    let mut query = app.world_mut().query::<(&CityLinkText, &Text)>();
    let rows: Vec<String> = query.iter(app.world()).map(|(_, text)| text.0.clone()).collect();
    assert_eq!(rows.len(), 4, "four rows, enough for the busiest station");

    let filled: Vec<&String> = rows.iter().filter(|line| !line.is_empty()).collect();
    assert_eq!(filled.len(), 2, "Yafa has two lines: Al-Ladd, and Qaisariya on paper");
    assert!(
        filled.iter().any(|line| line.contains("Al-Ladd") && line.contains("1892")),
        "the line to Al-Ladd opened in 1892: {filled:?}"
    );
}

#[test]
fn a_line_still_on_paper_is_offered_for_sale() {
    let mut app = headless_with_ui("a_line_still_on_paper_is_offered_for_sale");
    app.world_mut().write_message(ActionRequested(UiAction::ShowCity(CityId::Yafa)));
    tick(&mut app, 1.0 / 60.0);

    let mut query = app.world_mut().query::<(&CityLinkBuyLabel, &Text)>();
    let offers: Vec<String> = query
        .iter(app.world())
        .map(|(_, text)| text.0.clone())
        .filter(|line| !line.is_empty())
        .collect();

    // Yafa to Qaisariya is 55 km of surveyed line, at ten piastres the km.
    assert_eq!(offers.len(), 1, "one line out of Yafa is still unbuilt: {offers:?}");
    assert!(offers[0].contains("550"), "55 km at 10 a km: {}", offers[0]);
    assert!(offers[0].to_lowercase().contains("lay"), "and says what it does: {}", offers[0]);
}

#[test]
fn the_arrival_card_covers_the_city_card_and_then_goes() {
    let mut app = headless_with_ui("the_arrival_card_covers_the_city_card_and_then_goes");
    tick(&mut app, 1.0 / 60.0);

    let shown = |app: &mut App| -> (bool, bool) {
        let mut arrival = app.world_mut().query_filtered::<&Node, With<ArrivalCard>>();
        let a = arrival.single(app.world()).expect("one arrival card").display;
        let mut city = app.world_mut().query_filtered::<&Node, With<CityPanelSheet>>();
        let c = city.single(app.world()).expect("one city card").display;
        (a != Display::None, c != Display::None)
    };

    assert_eq!(shown(&mut app), (false, true), "the city card holds the slot by default");

    app.world_mut().write_message(TrainReached { city: CityId::AlLadd, final_stop: false });
    tick(&mut app, 1.0 / 60.0);
    assert_eq!(shown(&mut app), (true, false), "the arrival takes the slot");

    let names: Vec<String> = {
        let mut query = app.world_mut().query::<(&ArrivalPart, &Text)>();
        query
            .iter(app.world())
            .filter(|(part, _)| matches!(part, ArrivalPart::Arabic))
            .map(|(_, text)| text.0.clone())
            .collect()
    };
    assert_eq!(names.len(), 1);
    assert!(!names[0].is_empty(), "the card carries the station's second name");

    // Four seconds later it has gone and the city card is back.
    for _ in 0..5 {
        tick(&mut app, 1.0);
    }
    assert_eq!(shown(&mut app), (false, true));
}
