# Bevy Window Utils

A simple crate with utilities such as setting the window icon and taskbar progress indicator.
Works perfectly without issues on linux and windows. (macOS?)
Has only needed bevy dependencies. Supports embedded assets. (bevy-embedded-assets)

Compatible with Bevy 0.15.3 (as of 0.15.4)

Modyfing taskbar progress indicator is only supported on windows and requires the `taskbar` feature.

Example usage:
```rs
use bevy::{
    app::{App, Startup, Update},
    asset::AssetServer,
    ecs::system::{Res, ResMut},
    DefaultPlugins,
};
use bevy_window_utils::{WindowUtils, WindowUtilsPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins((WindowUtilsPlugin, DefaultPlugins))
        .add_systems(
            Startup,
            |assets: Res<AssetServer>, mut window: ResMut<WindowUtils>| {
                window.window_icon = Some(assets.load("my_icon.png"));
            },
        )
        .add_systems(Update, |mut window: ResMut<WindowUtils>| {
            window.taskbar_progress =
                window
                    .taskbar_progress
                    .as_ref()
                    .map(|p| bevy_window_utils::TaskbarProgress {
                        progress: p.progress + 1,
                        max: 100,
                    });
        });
    app.run();
}
```
