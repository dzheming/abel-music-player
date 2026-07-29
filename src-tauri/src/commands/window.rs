use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};

use super::portable::get_portable_dir;

#[derive(Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

fn state_file_path() -> PathBuf {
    get_portable_dir().join("window-state.json")
}

#[tauri::command]
pub fn save_window_state(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;

    let is_maximized = window.is_maximized().unwrap_or(false);

    if is_maximized {
        let path = state_file_path();
        if path.exists() {
            let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(mut state) = serde_json::from_str::<WindowState>(&json) {
                state.maximized = true;
                let json = serde_json::to_string(&state).map_err(|e| e.to_string())?;
                fs::write(&path, json).map_err(|e| e.to_string())?;
            }
        }
        return Ok(());
    }

    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;

    let state = WindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized: false,
    };

    let json = serde_json::to_string(&state).map_err(|e| e.to_string())?;
    fs::write(state_file_path(), json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn restore_window_state(app: AppHandle) -> Result<(), String> {
    let path = state_file_path();
    if !path.exists() {
        return Ok(());
    }

    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let state: WindowState = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let valid = monitor_contains_position(state.x, state.y)
        || monitor_contains_position(state.x + state.width as i32, state.y)
        || monitor_contains_position(state.x, state.y + state.height as i32)
        || monitor_contains_position(state.x + state.width as i32, state.y + state.height as i32);
    

    let window = app.get_webview_window("main").ok_or("Window not found")?;

    if valid {
        use tauri::{LogicalPosition, LogicalSize};
        window.set_position(LogicalPosition::new(state.x as f64, state.y as f64)).map_err(|e| e.to_string())?;
        window.set_size(LogicalSize::new(state.width as f64, state.height as f64)).map_err(|e| e.to_string())?;
    
    }

    Ok(())
}

/// 显示器最大坐标值 (覆盖多屏 16K 分辨率场景)
const MAX_MONITOR_DIMENSION: i32 = 16384;
/// 允许窗口略微超出屏幕边界仍视为有效位置
const OFFSCREEN_MARGIN: i32 = 200;

fn monitor_contains_position(x: i32, y: i32) -> bool {
    x >= -OFFSCREEN_MARGIN && x < MAX_MONITOR_DIMENSION && y >= -OFFSCREEN_MARGIN && y < MAX_MONITOR_DIMENSION
}

#[tauri::command]
pub fn set_taskbar_icon(app: AppHandle, icon_base64: String) -> Result<(), String> {
    let data = icon_base64
        .split(',')
        .last()
        .unwrap_or(&icon_base64);

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| e.to_string())?;

    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();

    let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);

    let window = app.get_webview_window("main").ok_or("Window not found")?;
    window.set_icon(icon).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn reset_taskbar_icon(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    let icon_bytes = include_bytes!("../../icons/icon.png");
    let img = image::load_from_memory(icon_bytes).map_err(|e| e.to_string())?;
    let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);
    window.set_icon(icon).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(target_os = "macos")]
mod power_macos {
    use std::sync::atomic::{AtomicU32, Ordering};

    static ASSERTION_ID: AtomicU32 = AtomicU32::new(0);

    extern "C" {
        fn IOPMAssertionCreateWithName(
            assertion_type: *const std::ffi::c_void,
            level: u32,
            reason: *const std::ffi::c_void,
            assertion_id: *mut u32,
        ) -> i32;
        fn IOPMAssertionRelease(assertion_id: u32) -> i32;
    }

    pub fn prevent() {
        use core_foundation::string::CFString;
        use core_foundation::base::TCFType;

        let assertion_type = CFString::new("PreventUserIdleDisplaySleep");
        let reason = CFString::new("Audio playback");
        let mut assertion_id: u32 = 0;

        let ret = unsafe {
            IOPMAssertionCreateWithName(
                assertion_type.as_concrete_TypeRef() as *const _,
                255, // kIOPMAssertionLevelOn
                reason.as_concrete_TypeRef() as *const _,
                &mut assertion_id,
            )
        };
        if ret != 0 {
            eprintln!("[power] IOPMAssertionCreateWithName failed: {}", ret);
            return;
        }

        let old = ASSERTION_ID.swap(assertion_id, Ordering::Relaxed);
        if old != 0 {
            unsafe { IOPMAssertionRelease(old); }
        }
    }

    pub fn allow() {
        let id = ASSERTION_ID.swap(0, Ordering::Relaxed);
        if id != 0 {
            unsafe { IOPMAssertionRelease(id); }
        }
    }
}

#[tauri::command]
pub fn prevent_sleep() {
    #[cfg(target_os = "windows")]
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
    }

    #[cfg(target_os = "macos")]
    power_macos::prevent();
}

#[tauri::command]
pub fn allow_sleep() {
    #[cfg(target_os = "windows")]
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }

    #[cfg(target_os = "macos")]
    power_macos::allow();
}