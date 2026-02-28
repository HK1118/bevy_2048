#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod game;

use std::time::Duration;

use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};
#[cfg(feature = "dev_native")]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Window {
                    title: "2048".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
    )
    .add_plugins(game::GamePlugin)
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::reactive(Duration::from_secs(1)),
        unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(1)),
    });

    #[cfg(feature = "dev_native")]
    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new());

    app.run()
}
