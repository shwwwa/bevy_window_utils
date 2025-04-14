#[allow(unused_imports)]
use std::os::raw::c_void;

#[cfg(target_os = "windows")]
use ::winit::platform::windows::WindowExtWindows;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use bevy::{app::Plugin, ecs::system::Resource};
#[cfg(target_os = "windows")]
use w::prelude::{shell_ITaskbarList3, Handle};
#[cfg(target_os = "windows")]
use w::{ITaskbarList4, HWND};
#[cfg(target_os = "windows")]
use winsafe::{self as w, co};

pub struct WindowUtilsPlugin;

impl Plugin for WindowUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowUtils>()
            .add_systems(Update, window_utils_resource_updated);
    }
}

#[cfg(feature = "taskbar")]
pub struct TaskbarProgress {
    pub progress: u64,
    pub max: u64,
}

#[derive(Resource, Default)]
pub struct WindowUtils {
    #[cfg(feature = "taskbar")]
    pub taskbar_progress: Option<TaskbarProgress>,
    pub window_icon: Option<bevy::asset::Handle<Image>>,
}

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
                    i.data.clone(),
                    i.texture_descriptor.size.width,
                    i.texture_descriptor.size.height,
                )
                .ok()
            });
        for window in windows.windows.iter() {
            window.1.set_window_icon(icon.clone())
        }
        // currently only windows is supported
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
                        // i really hate this. winnit hods HWND as an integer while winsafe uses a pointer
                        unsafe {
                            let hwnd = HWND::from_ptr(window.1.hwnd() as *mut c_void);
                            itbl.SetProgressValue(&hwnd, progress.progress, progress.max)
                                .unwrap();
                            itbl.SetProgressState(&hwnd, co::TBPF::NORMAL).unwrap();
                        }
                    }
                }
            }
        }
    }
}
