//! 窗口状态持久化（替代 tauri-plugin-window-state）
//!
//! 原插件把状态硬编码存到 `app_config_dir()`（Linux 为 `~/.config/com.v.meme`），
//! 无法自定义目录。这里自实现同样的功能，将窗口状态保存到
//! `~/.local/share/meme/window-state.json`，与 DB、webview 数据统一放在 meme 下。
//!
//! 文件格式与原插件兼容（`label -> state` 的 map），旧文件可直接迁移复用；
//! 未知字段（visible/decorated/fullscreen）由 serde 默认忽略。
//!
//! 行为与原插件一致：
//! - 窗口移动/缩放/关闭时更新内存缓存
//! - 应用退出（`RunEvent::Exit`）时写盘
//! - 启动时恢复位置/尺寸/最大化，且只恢复到仍存在的显示器上（防止外接屏拔出后窗口跑到屏幕外）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent};

/// 窗口状态字段与原插件保持一致；只用到位置/尺寸/最大化，其余字段不序列化
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WindowState {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    // 最大化前的位置（最大化会把窗口放到屏幕角落，恢复时用它还原）
    prev_x: i32,
    prev_y: i32,
    maximized: bool,
}

/// 应用级共享缓存：label -> 窗口状态
pub struct WindowStateCache(pub Arc<Mutex<HashMap<String, WindowState>>>);

/// 状态文件路径：`~/.local/share/meme/window-state.json`
pub fn state_file_path() -> PathBuf {
    crate::core::db::get_app_data_dir().join("window-state.json")
}

/// 迁移旧版窗口状态文件（`~/.config/com.v.meme/.window-state.json`，由原插件写入）
///
/// 搬移后旧配置目录若已空则一并删除，使 `~/.config` 下不再残留 com.v.meme。
#[cfg(not(target_os = "android"))]
pub fn migrate_legacy_config_dir() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let legacy_dir = PathBuf::from(&home).join(".config/com.v.meme");
    let legacy_file = legacy_dir.join(".window-state.json");
    let target = state_file_path();

    if legacy_file.exists() && !target.exists() {
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::rename(&legacy_file, &target) {
            Ok(()) => log::info!(
                "已迁移窗口状态 {} -> {}",
                legacy_file.display(),
                target.display()
            ),
            Err(e) => log::warn!("迁移窗口状态文件失败: {e}"),
        }
    }

    // 旧配置目录已空则删除
    if legacy_dir.exists()
        && legacy_dir
            .read_dir()
            .map(|mut d| d.next().is_none())
            .unwrap_or(false)
    {
        let _ = std::fs::remove_dir(&legacy_dir);
    }
}

/// 加载磁盘上的窗口状态缓存
pub fn load_cached_state() -> HashMap<String, WindowState> {
    std::fs::read_to_string(state_file_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 恢复窗口状态并挂载事件监听（位置/尺寸变化写入缓存，关闭前刷新）
pub fn restore_and_track(window: &WebviewWindow, cache: Arc<Mutex<HashMap<String, WindowState>>>) {
    let label = window.label().to_string();

    let state = {
        let mut c = cache.lock().unwrap();
        c.entry(label.clone()).or_insert_with(WindowState::default).clone()
    };

    if let Err(e) = restore(window, &state) {
        log::warn!("恢复窗口状态失败: {e}");
    }

    let window_clone = window.clone();
    let cache_clone = cache.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Moved(position) => {
            if !window_clone.is_minimized().unwrap_or_default() {
                let mut c = cache_clone.lock().unwrap();
                if let Some(state) = c.get_mut(&label) {
                    state.prev_x = state.x;
                    state.prev_y = state.y;
                    state.x = position.x;
                    state.y = position.y;
                }
            }
        }
        WindowEvent::Resized(size) => {
            if !window_clone.is_minimized().unwrap_or_default()
                && !window_clone.is_maximized().unwrap_or_default()
            {
                let mut c = cache_clone.lock().unwrap();
                if let Some(state) = c.get_mut(&label) {
                    state.width = size.width;
                    state.height = size.height;
                }
            }
        }
        WindowEvent::CloseRequested { .. } => {
            if let Some(state) = cache_clone.lock().unwrap().get_mut(&label) {
                update_live_state(&window_clone, state);
            }
        }
        _ => {}
    });
}

/// 应用退出时把所有窗口状态写盘
pub fn save_all(app: &AppHandle) {
    let cache = app.state::<WindowStateCache>();
    let states = cache.0.lock().unwrap().clone();
    let path = state_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_vec_pretty(&states) {
        Ok(bytes) => {
            if let Err(e) = std::fs::write(&path, bytes) {
                log::warn!("保存窗口状态失败: {e}");
            }
        }
        Err(e) => log::warn!("序列化窗口状态失败: {e}"),
    }
}

/// 用当前窗口的实际状态刷新缓存（关闭窗口前调用，捕捉最大化等状态）
fn update_live_state(window: &WebviewWindow, state: &mut WindowState) {
    let is_maximized = window.is_maximized().unwrap_or_default();
    let is_minimized = window.is_minimized().unwrap_or_default();
    state.maximized = is_maximized;
    if !is_maximized && !is_minimized {
        if let Ok(size) = window.inner_size() {
            if size.width > 0 && size.height > 0 {
                state.width = size.width;
                state.height = size.height;
            }
        }
        if let Ok(pos) = window.outer_position() {
            state.x = pos.x;
            state.y = pos.y;
        }
    }
}

/// 恢复位置/尺寸/最大化；位置只在保存的窗口与某个显示器相交时才恢复，
/// 避免外接屏拔出后窗口跑到屏幕外
fn restore(window: &WebviewWindow, state: &WindowState) -> tauri::Result<()> {
    if state == &WindowState::default() {
        return Ok(());
    }

    if state.width > 0 && state.height > 0 {
        window.set_size(PhysicalSize::new(state.width, state.height))?;
    }

    let position = PhysicalPosition::new(state.x, state.y);
    let size = PhysicalSize::new(state.width.max(1), state.height.max(1));
    for monitor in window.available_monitors()? {
        if monitor_intersects(&monitor, position, size) {
            let (x, y) = if state.maximized {
                (state.prev_x, state.prev_y)
            } else {
                (state.x, state.y)
            };
            window.set_position(PhysicalPosition::new(x, y))?;
            break;
        }
    }

    if state.maximized {
        window.maximize()?;
    }

    Ok(())
}

/// 窗口四角任一落在显示器范围内即视为相交
fn monitor_intersects(
    monitor: &Monitor,
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
) -> bool {
    let PhysicalPosition { x, y } = *monitor.position();
    let PhysicalSize { width, height } = *monitor.size();

    let left = x;
    let right = x + width as i32;
    let top = y;
    let bottom = y + height as i32;

    [
        (position.x, position.y),
        (position.x + size.width as i32, position.y),
        (position.x, position.y + size.height as i32),
        (
            position.x + size.width as i32,
            position.y + size.height as i32,
        ),
    ]
    .into_iter()
    .any(|(x, y)| x >= left && x < right && y >= top && y < bottom)
}