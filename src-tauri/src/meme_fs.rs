//! 文件系统路径解析模块
//!
//! 全平台统一：数据库中只存储相对于 meme 根目录的路径，
//! 所有实际文件操作前通过本模块将相对路径解析为绝对路径。

use std::path::PathBuf;

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

/// 检查存储目录是否可访问
///
/// Android 端 `MANAGE_EXTERNAL_STORAGE` 权限未授予时，
/// 即使 SAF 选择器返回了路径，`std::fs` 操作也可能失败。
/// 此函数用于在操作前检查权限状态。
/// 使用 `read_dir` 而非 `metadata`，因为某些 Android 版本上
/// `metadata` 可能成功但 `read_dir` 失败。
#[tauri::command]
pub fn check_storage_accessible(meme_dir: String) -> Result<bool, String> {
    let path = resolve_meme_path(&meme_dir, "");
    match std::fs::read_dir(&path) {
        Ok(_) => Ok(true),
        Err(e) => {
            log::warn!("Storage not accessible: {:?} - {}", path, e);
            Ok(false)
        }
    }
}
