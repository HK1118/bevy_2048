#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod game;

use bevy::{asset::AssetMetaCheck, prelude::*};
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
    .add_plugins(game::GamePlugin);

    #[cfg(feature = "dev_native")]
    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new());

    app.run()
}
