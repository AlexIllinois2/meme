//! 文件系统路径解析模块
//!
//! 全平台统一：数据库中只存储相对于 meme 根目录的路径，
//! 所有实际文件操作前通过本模块将相对路径解析为绝对路径。

use std::path::{Path, PathBuf};
use crate::core::error::AppError;
/// 非法文件名字符
pub const INVALID_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// 验证名称是否合法（不包含非法字符）
pub fn validate_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError("名称不能为空".to_string()));
    }

    if name.trim() != name {
        return Err(AppError("名称首尾不能有空格".to_string()));
    }

    for c in name.chars() {
        if INVALID_CHARS.contains(&c) {
            return Err(AppError(format!("名称包含非法字符: '{}'", c)));
        }
    }

    Ok(())
}

/// 生成安全的文件夹名称（替换非法字符）
pub fn sanitize_folder_name(name: &str) -> String {
    name.chars()
        .map(|c| if INVALID_CHARS.contains(&c) { '_' } else { c })
        .collect()
}

/// 将相对于 meme 根目录的路径解析为绝对路径
///
/// 如果路径已经是绝对路径，直接返回（向后兼容未迁移的数据）
pub fn resolve_meme_path(meme_dir: &str, relative: &str) -> PathBuf {
    let path = PathBuf::from(relative);
    if path.is_absolute() {
        return path;
    }
    let relative = relative.trim_start_matches('/');
    PathBuf::from(meme_dir).join(relative)
}

/// 从绝对路径中提取相对于 meme_dir 的相对路径
pub fn relative_path(meme_dir: &str, absolute: &str) -> String {
    let prefix = if meme_dir.ends_with('/') {
        meme_dir.to_string()
    } else {
        format!("{}/", meme_dir)
    };
    absolute
        .strip_prefix(&prefix)
        .unwrap_or(absolute)
        .to_string()
}

/// 旧版 WebKit 数据目录：`~/.local/share/com.v.meme`（仅桌面端）
#[cfg(not(target_os = "android"))]
fn legacy_webview_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".local/share/com.v.meme")
}

/// 迁移旧版 WebKit 数据目录到 `~/.local/share/meme/webview`
///
/// 历史版本中 webview 数据（localStorage、缓存等）由 Tauri 自动放在
/// `~/.local/share/com.v.meme`（基于 identifier），与 DB 所在目录不统一。
/// 现在窗口创建时通过 `data_directory` 指定到 `meme/webview`，
/// 此函数在启动时把旧目录整体搬过去（含 localStorage，避免丢失前端持久化数据）。
#[cfg(not(target_os = "android"))]
pub fn migrate_legacy_webview_data_dir() {
    let legacy = legacy_webview_data_dir();
    if !legacy.exists() {
        return;
    }

    let target = crate::core::db::get_app_data_dir().join("webview");
    if target.exists() {
        // 新目录已初始化（说明新版本已运行过），只补迁 localStorage，再清理旧目录
        migrate_localstorage(&legacy, &target);
        let _ = std::fs::remove_dir_all(&legacy);
        return;
    }

    // 新目录尚未创建：整体搬移，localStorage 与缓存全部保留
    if let Some(parent) = target.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::rename(&legacy, &target) {
        Ok(()) => log::info!(
            "已迁移旧版 webview 数据目录 {} -> {}",
            legacy.display(),
            target.display()
        ),
        Err(e) => {
            // 搬移失败（罕见），至少保留 localStorage；旧目录留待用户自行处理
            log::warn!("迁移旧版 webview 数据目录失败: {e}");
            migrate_localstorage(&legacy, &target);
        }
    }
}

/// 仅复制 localStorage（前端持久化数据），旧目录保留
#[cfg(not(target_os = "android"))]
fn migrate_localstorage(legacy: &Path, target: &Path) {
    let src = legacy.join("localstorage");
    let dst = target.join("localstorage");
    if !src.exists() || dst.exists() {
        return;
    }
    if let Some(parent) = dst.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match copy_dir_recursive(&src, &dst) {
        Ok(()) => log::info!("已复制 localStorage -> {}", dst.display()),
        Err(e) => log::warn!("复制 localStorage 失败: {e}"),
    }
}

/// 递归复制目录（std 无内置实现）
#[cfg(not(target_os = "android"))]
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            std::fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

/// 检查存储目录是否可访问
///
/// Android 端 `MANAGE_EXTERNAL_STORAGE` 权限未授予时，
/// 即使 SAF 选择器返回了路径，`std::fs` 操作也可能失败。
/// 此函数用于在操作前检查权限状态。
/// 使用 `read_dir` 而非 `metadata`，因为某些 Android 版本上
/// `metadata` 可能成功但 `read_dir` 失败。
#[tauri::command]
pub fn check_storage_accessible(meme_dir: String) -> Result<bool, AppError> {
    let path = resolve_meme_path(&meme_dir, "");
    match std::fs::read_dir(&path) {
        Ok(_) => Ok(true),
        Err(e) => {
            log::warn!("Storage not accessible: {:?} - {}", path, e);
            Ok(false)
        }
    }
}
