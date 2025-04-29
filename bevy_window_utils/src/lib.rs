#![allow(clippy::pedantic)]
#![warn(missing_docs)]

//! A crate that defines an interface for extended windowing support in Bevy.

#[allow(unused_imports)]
use std::os::raw::c_void;

use bevy_ecs::{prelude::*, resource::Resource};
use bevy_app::{App, Plugin, Update};
use bevy_asset::Assets;
use bevy_image::Image;
use bevy_log::{warn, warn_once};
use bevy_window::PrimaryWindow;
use bevy_winit::WinitWindows;
#[cfg(target_os = "windows")]
use winit::raw_window_handle;
#[cfg(target_os = "windows")]
use winit::raw_window_handle::HasWindowHandle;

#[cfg(target_os = "windows")]
use w::prelude::{Handle, shell_ITaskbarList3};
#[cfg(target_os = "windows")]
use w::{HWND, ITaskbarList4};
#[cfg(target_os = "windows")]
use winsafe::{self as w, co};

/** A [`Plugin`] that defines an interface for extended windowing support in Bevy.
You can initialize window icon here, or just use default function.
Adds barely exposed things to bevy like setting window icons, taskbar progress, or other winit/winsafe options. */
#[derive(Default)]
pub struct WindowUtilsPlugin {
    /** What window icon to set on initialization. */
    pub icon: Option<bevy_asset::Handle<Image>>,
}

impl Plugin for WindowUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowUtils {
            window_icon: self.icon.clone(),
            ..Default::default()
        })
        .add_systems(Update, window_utils_resource_updated)
        .add_systems(Update, update_is_maximized);
    }
}

/** Struct for taskbar progress. Requires `taskbar` feature.
  Provides useful interface from COM:
  <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-itaskbarlist3>

  Supports:
  - Windows (7+)
*/
#[cfg(feature = "taskbar")]
pub struct TaskbarProgress {
    /** Indicates the proportion of the operation that has been completed. */
    pub progress: u64,
    /** Indicates the value progress will have when the operation is complete. */
    pub max: u64,
    /// Indicates the type and state of the progress indicator displayed on a taskbar button.
    /// Note that a call to `SetProgressValue` should switch a progress indicator
    /// currently in an indeterminate mode ([`TaskbarState::Indeterminate`]) to a normal ([`TaskbarState::Normal`]) display
    /// and clear the [`TaskbarState::Indeterminate`] flag, but then is overwritten with state change, so it needs manual change.
    pub state: TaskbarState,
    /// Automatically stops the progress when [`TaskbarProgress::progress`] exceeds [`TaskbarProgress::max`].
    pub auto_no_progress: bool,
}

impl Default for TaskbarProgress {
    fn default() -> Self {
        TaskbarProgress {
            progress: 0,
            max: 100,
            state: TaskbarState::Normal,
            auto_no_progress: true,
        }
    }
}

/// Sets the type and state of the progress indicator displayed on a taskbar button.
/// Note that a call to `SetProgressValue` should switch a progress indicator
/// currently in an indeterminate mode ([`TaskbarState::Indeterminate`]) to a normal ([`TaskbarState::Normal`]) display
/// and clear the ([`TaskbarState::Indeterminate`]) flag, but is overwritten with state change, so it needs manual change.
#[derive(Copy, Clone)]
pub enum TaskbarState {
    /// Stops displaying progress and returns the button to its normal state.
    /// Call this method with this flag to dismiss the progress bar when the
    /// operation is complete or cancelled.
    NoProgress = 0x0,
    /// The progress indicator does not grow in size but cycles repeatedly
    /// along the length of the taskbar button. This indicates activity without
    /// specifying what proportion of the progress is complete. Progress is
    /// taking place but there is no prediction as to how long the operation
    /// will take.
    Indeterminate = 0x1,
    /// The progress indicator grows in size from left to right in proportion to
    /// the estimated amount of the operation completed. This is a determinate
    /// progress indicator; a prediction is being made as to the duration of the
    /// operation.
    Normal = 0x2,
    /// The progress indicator turns red to show that an error has occurred in
    /// one of the windows that is broadcasting progress. This is a determinate
    /// state. If the progress indicator is in the indeterminate state it
    /// switches to a red determinate display of a generic percentage not
    /// indicative of actual progress.
    Error = 0x4,
    /// The progress indicator turns yellow to show that progress is currently
    /// stopped in one of the windows but can be resumed by the user. No error
    /// condition exists and nothing is preventing the progress from continuing.
    /// This is a determinate state. If the progress indicator is in the
    /// indeterminate state it switches to a yellow determinate display of a
    /// generic percentage not indicative of actual progress.
    Paused = 0x8,
}

/** Main resource with access to additional exposed things from winit. */
#[derive(Resource, Default)]
pub struct WindowUtils {
    #[cfg(feature = "taskbar")]
    /// Current taskbar progress. Supports only windows 7+. Requires `taskbar` feature.
    ///
    /// Provides useful interface from COM:
    /// <https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-itaskbarlist3>
    ///
    /// Supports:
    /// - Windows (7+)
    pub taskbar_progress: Option<TaskbarProgress>,
    /// Contains handle to window's icon. If resource is invalid throws `bevy_asset::server` error to console.
    ///
    /// Supports:
    /// - Windows
    /// - Linux
    /// - MacOS (?)
    pub window_icon: Option<bevy_asset::Handle<Image>>,
    /// Automatically changes its value whether window is maximized or not. Returns [`None`] if error happened.
    /// Requires existence of primary window.
    /// Supports:
    /// - Windows
    /// - Linux
    /// - MacOS (?)
    pub is_maximized: Option<bool>,
}

/** Checks every frame if window is maximized. */
fn update_is_maximized(
    mut window_utils: ResMut<WindowUtils>,
    windows: NonSend<WinitWindows>,
    window: Query<EntityRef, With<PrimaryWindow>>,
) {
    for entity in window.iter() {
        match windows.get_window(entity.id()) {
            Some(window_wrapper) => {
                window_utils.is_maximized = Some(window_wrapper.is_maximized());
            }
            None => {
                warn_once!("winit is_maximized() interception failed, couldn't get the window.");
                window_utils.is_maximized = None;
            }
        }
    }
}

/** Checks every frame if window utils resource was updated. */
fn window_utils_resource_updated(
    window_utils: Res<WindowUtils>,
    windows: NonSend<WinitWindows>,
    assets: Res<Assets<Image>>,
) {
    if assets.is_changed() || window_utils.is_changed() {
        let icon = window_utils
            .window_icon
            .as_ref()
            .and_then(|i| assets.get(i))
            .and_then(|i| {
                ::winit::window::Icon::from_rgba(
                    // safe unwrap because it is not being used as a storage texture.
                    i.data.clone().unwrap(),
                    i.texture_descriptor.size.width,
                    i.texture_descriptor.size.height,
                )
                .ok()
            });

        for window in windows.windows.iter() {
            window.1.set_window_icon(icon.clone())
        }

        // feat(taskbar): supports only windows
        #[cfg(all(feature = "taskbar", target_os = "windows"))]
        if window_utils.is_changed() {
            {
                if let Some(progress) = &window_utils.taskbar_progress {
                    for window in windows.windows.iter() {
                        let itbl: ITaskbarList4 = w::CoCreateInstance(
                            &co::CLSID::TaskbarList,
                            None,
                            co::CLSCTX::INPROC_SERVER,
                        )
                        .unwrap();
                        // unsafe: winit holds HWND as an `NonZeroIsize` while winsafe uses a pointer.
                        // requires `rwh_06` feature (gets `raw_window_handle v0.6`) from `winit` that is provided by default.
                        unsafe {
                            match window.1.window_handle() {
                                Ok(handle) => {
                                    // We know for sure that enum is Win32, so access hwnd directly
                                    if let raw_window_handle::RawWindowHandle::Win32(win_handle) =
                                        handle.as_raw()
                                    {
                                        let hwnd = HWND::from_ptr(
                                            isize::from(win_handle.hwnd) as *mut c_void
                                        );
                                        itbl.SetProgressValue(
                                            &hwnd,
                                            progress.progress,
                                            progress.max,
                                        )
                                        .unwrap();
                                        if progress.auto_no_progress
                                            && progress.progress >= progress.max
                                        {
                                            itbl.SetProgressState(&hwnd, co::TBPF::NOPROGRESS)
                                                .unwrap();
                                        } else {
                                            itbl.SetProgressState(
                                                &hwnd,
                                                co::TBPF::from_raw(progress.state as u32),
                                            )
                                            .unwrap();
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Couldn't set taskbar progress: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
