//! Launcher for Yafa Express. The game itself lives in the library crate; see
//! `lib.rs` for how it fits together.

use bevy::prelude::*;
use bevy::window::WindowResolution;
use yafa_game::YafaExpressPlugin;

/// Logical size of the window at launch.
///
/// Twenty-three stations drawn at 3.6 pixels to the kilometre need a map 434
/// by 686 pixels, which no longer fits beside the panels in the old 1280 by
/// 760 frame.
const WINDOW_WIDTH: u32 = 1440;
const WINDOW_HEIGHT: u32 = 940;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Yafa Express".to_owned(),
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                    // On the web the canvas takes the page, and the browser
                    // keeps its own keyboard shortcuts rather than having the
                    // game swallow them.
                    #[cfg(target_arch = "wasm32")]
                    fit_canvas_to_parent: true,
                    #[cfg(target_arch = "wasm32")]
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            YafaExpressPlugin,
        ))
        .run()
}
