//! # Yafa Express
//!
//! A 2D top-down train simulation across the historical railways of Palestine.
//! Buy a ticket at the counter, watch the train work its way along the coastal
//! and mountain lines, and collect a passport stamp at every station it calls
//! at on the way.
//!
//! ## Shape of the code
//!
//! * [`yafa_core`] - the rules, with no engine attached: the network graph,
//!   routing, the journey state machine, the ticket desk and the passport. It
//!   is unit-tested on its own.
//! * [`components`] - the ECS vocabulary: components, messages, resources,
//!   states.
//! * [`map`] - projecting the network onto the screen, drawing track, and the
//!   locomotive.
//! * [`systems`] - the gameplay: ticketing, movement, stamping, saving.
//! * [`ui`] - the ticket counter, station board, city panel and passport.
//! * [`theme`] - the palette and the shared chrome.
//!
//! Stations and track are addressed by `CityId` / `SegmentId` throughout.
//! Nothing holds a reference into the network, so a system is free to read the
//! map while mutating the passport and the train in the same breath.
//!
//! The app is split into [`SimulationPlugin`] - the rules, which need no
//! window - and [`PresentationPlugin`] - everything that draws. The split is
//! what lets the integration tests drive a whole journey headless.

#![forbid(unsafe_code)]
// A Bevy system declares its whole data dependency in its signature, and a
// disjoint-query filter is genuinely a long type. Both lints fire on ordinary,
// readable ECS code, so they are off here rather than worked around.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod components;
pub mod map;
pub mod systems;
pub mod theme;
pub mod ui;

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::sprite::{SpritePickingMode, SpritePickingSettings};
use yafa_core::{Network, Passport};

use components::*;
use theme::Fonts;

/// Simulation runs first, then everything that only reads state to show it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    Simulate,
    Present,
}

/// The rules of the game: state, messages and the systems that move the train
/// and fill the passport. Needs no window, no renderer and no assets.
pub struct SimulationPlugin {
    /// The passport to start from.
    pub passport: Passport,
    /// Where it is written back to.
    pub passport_path: PathBuf,
}

impl SimulationPlugin {
    /// Reads the player's passport from its usual place, falling back to a
    /// blank one if the file is missing or damaged.
    pub fn from_disk() -> Self {
        let passport_path = Passport::default_path();
        let passport = Passport::load(&passport_path).unwrap_or_else(|error| {
            warn!("could not read {}: {error}", passport_path.display());
            Passport::default()
        });
        Self { passport, passport_path }
    }
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        let network = Network::new();
        let view = map::make_map_view(&network);

        app.init_state::<GamePhase>()
            .insert_resource(Rail(network))
            .insert_resource(view)
            .insert_resource(PassportBook::new(self.passport.clone(), self.passport_path.clone()))
            .init_resource::<Desk>()
            .init_resource::<Clock>()
            .init_resource::<Focus>()
            .init_resource::<Announcer>()
            .init_resource::<JourneyScratch>()
            .add_message::<ActionRequested>()
            .add_message::<TrainReached>()
            .add_message::<TrainDeparted>()
            .add_message::<PassportStamped>()
            .add_message::<JourneyFinished>()
            .add_observer(systems::on_control_activated)
            .add_observer(systems::on_station_clicked)
            .add_systems(Startup, systems::opening_announcement)
            // The gameplay chain, in the order the day actually happens:
            // someone presses a button, the train moves, the clerk stamps, the
            // board announces it, and the passport is filed away.
            .add_systems(
                Update,
                (
                    systems::keyboard_shortcuts,
                    systems::apply_actions,
                    systems::train_movement_system,
                    systems::passport_stamp_system,
                    systems::record_completed_journeys,
                    systems::settle_on_arrival,
                    systems::announce,
                    systems::save_passport,
                )
                    .chain()
                    .in_set(GameSet::Simulate),
            )
            .configure_sets(Update, GameSet::Simulate.before(GameSet::Present));
    }
}

/// Everything that draws: the map, the interface, and the systems that copy
/// state onto them.
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        // The faces are compiled into the binary, so this only needs somewhere
        // to register them. `DefaultPlugins` is added before this plugin, so
        // the store is already there; a headless app that skips it inserts
        // `Fonts::builtin()` for itself instead.
        let fonts = match app.world_mut().get_resource_mut::<Assets<Font>>() {
            Some(mut assets) => Fonts::embedded(&mut assets),
            None => Fonts::builtin(),
        };

        app.insert_resource(fonts)
            // Only entities that ask for it take part in picking, so the UI's
            // transparent containers never swallow a click meant for the map.
            .insert_resource(UiPickingSettings { require_markers: true })
            .insert_resource(SpritePickingSettings {
                require_markers: false,
                picking_mode: SpritePickingMode::BoundingBox,
            })
            .add_systems(Startup, (map::spawn_map, map::spawn_water, ui::build_ui))
            .add_systems(
                Update,
                (
                    ui::style_controls,
                    ui::refresh_ticket,
                    ui::refresh_board,
                    ui::refresh_city_panel,
                    ui::refresh_passport,
                    ui::flash_new_stamps,
                    // The flash paints over the card colour the refresh just
                    // set, so it has to come after it.
                    ui::run_stamp_flash,
                )
                    .chain()
                    .in_set(GameSet::Present),
            )
            .add_systems(
                Update,
                // The country first, then the track over it, then everything
                // that marks a place on it.
                (
                    map::draw_country,
                    map::draw_shorelines,
                    map::draw_furniture,
                    map::draw_network,
                    map::draw_station_rings,
                    map::draw_steam,
                )
                    .chain()
                    .in_set(GameSet::Present),
            );
    }
}

/// The whole game.
pub struct YafaExpressPlugin;

impl Plugin for YafaExpressPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SimulationPlugin::from_disk(), PresentationPlugin));
    }
}
