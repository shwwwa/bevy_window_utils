# Bevy Window Utils

A simple crate with utilities such as setting the window icon and taskbar progress indicator.
Supports linux and windows. (macOS?)
Has only needed bevy/winit dependencies, depends on winsafe/winit crate (exposes winsafe/winit functions). 
Supports embedded assets and other bevy plugins.

Compatible with Bevy 0.16.0 (as of 0.16.0)

Modifying taskbar progress indicator is only supported on windows 7+ and requires the `taskbar` feature.

Example usage:
```rs
use bevy::prelude::*;
use bevy_window_utils::{WindowUtils, WindowUtilsPlugin};

fn main() {
    let mut app = App::new();

    // You can initialize icon instead of default.
    app.add_plugins((DefaultPlugins, WindowUtilsPlugin::default()))
        .add_systems(
            Startup,
            |assets: Res<AssetServer>, mut window: ResMut<WindowUtils>| {
		// You can set icon on runtime:
                window.window_icon = Some(assets.load("icon.png"));
                window.taskbar_progress = Some(bevy_window_utils::TaskbarProgress {
                    progress: 30,
		    ..Default::default()
                });
            },
        )
        .add_systems(Update, |mut window: ResMut<WindowUtils>| {
            window.taskbar_progress =
                window
                    .taskbar_progress
                    .as_ref()
                    .map(|p| bevy_window_utils::TaskbarProgress {
                        progress: p.progress + 1,
			..Default::default()
                    });
        });
    app.run();
}
```
