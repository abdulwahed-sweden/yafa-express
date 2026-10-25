//! Launcher for Yafa Express. The game itself lives in the library crate; see
//! `lib.rs` for how it fits together.

use bevy::prelude::*;
use bevy::window::WindowResolution;
use yafa_game::YafaExpressPlugin;

/// Logical size of the window at launch. The layout is designed to fit
/// comfortably inside it on a 1280-point-wide display.
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 760;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Yafa Express".to_owned(),
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                    ..default()
                }),
                ..default()
            }),
            YafaExpressPlugin,
        ))
        .run()
}
