//! 剪贴板操作模块
//!
//! 提供从剪贴板读取图片的功能，包括桌面端（clipboard-rs）和 Android 端实现。
//! 桌面端额外支持通过系统命令读取剪贴板（Wayland wl-paste 回退方案）。

use tauri_plugin_clipboard_manager::ClipboardExt;
use image::ImageEncoder;
use crate::error::AppError;

#[cfg(not(target_os = "android"))]
use clipboard_rs::{Clipboard, ClipboardContext};

/// 剪贴板图片数据结构
#[derive(serde::Serialize)]
pub struct ClipboardImage {
    /// 图片二进制数据
    pub data: Vec<u8>,
    /// 图片格式
    pub format: String,
}

/// 从剪贴板读取图片
#[tauri::command]
pub fn paste_image_from_clipboard<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<ClipboardImage, AppError> {
    let clipboard = _app.clipboard();

    match clipboard.read_image() {
        Ok(image) => {
            let rgba_data: Vec<u8> = image.rgba().into();
            let width = image.width();
            let height = image.height();

            match image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                width, height, rgba_data
            ) {
                Some(buffer) => {
                    let mut png_bytes: Vec<u8> = Vec::new();
                    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                    match encoder.write_image(
                        &buffer, width, height, image::ColorType::Rgba8
                    ) {
                        Ok(_) => Ok(ClipboardImage {
                            data: png_bytes,
                            format: "png".to_string(),
                        }),
                        Err(e) => Err(AppError(format!("Failed to encode PNG: {}", e))),
                    }
                }
                None => Err("Failed to create image buffer".into()),
            }
        }
        Err(e) => Err(AppError(format!("Failed to read clipboard image: {}", e))),
    }
}

/// 使用系统命令读取剪贴板图片（桌面端）
///
/// 主要用于 Wayland 环境，作为 clipboard-rs 的补充方案。
#[cfg(not(target_os = "android"))]
fn read_clipboard_with_system_command(temp_dir: &str) -> Result<String, AppError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    #[cfg(target_os = "linux")]
    {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        if session_type == "wayland" {
            let output = std::process::Command::new("wl-paste")
                .args(&["--type", "image/png"])
                .output();

            if let Ok(output) = output {
                if output.status.success() && !output.stdout.is_empty() {
                    let temp_file_name = format!("pasted-{}.png", timestamp);
                    let temp_path = std::path::Path::new(temp_dir).join(&temp_file_name);
                    if let Some(parent) = temp_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| AppError(format!("Failed to create temp dir: {}", e)))?;
                    }
                    std::fs::write(&temp_path, &output.stdout)
                        .map_err(|e| AppError(format!("Failed to write image: {}", e)))?;
                    return Ok(temp_path.to_string_lossy().to_string());
                }
            }
        }
    }

    Err("System clipboard command failed".into())
}

/// 从剪贴板读取图片（桌面端）
///
/// 优先尝试读取文件，然后使用系统命令方案，最后回退到 tauri-clipboard-manager。
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn paste_image_from_clipboard_raw<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    temp_dir: String,
) -> Result<String, AppError> {
    let ctx = ClipboardContext::new().map_err(|e| {
        AppError(format!("Failed to create clipboard context: {}", e))
    })?;

    let files = ctx.get_files().map_err(|e| {
        AppError(format!("Failed to read files from clipboard: {}", e))
    })?;

    if !files.is_empty() {
        let src_path = std::path::Path::new(&files[0]);
        if !src_path.exists() {
            return Err("剪贴板中的文件不存在".into());
        }

        let extension = src_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let temp_file_name = format!("pasted-{}.{}", timestamp, extension);
        let temp_path = std::path::Path::new(&temp_dir).join(&temp_file_name);

        if let Some(parent) = temp_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError(format!("Failed to create temp dir: {}", e)))?;
        }

        std::fs::copy(src_path, &temp_path).map_err(|e| {
            AppError(format!("Failed to copy file: {}", e))
        })?;

        return Ok(temp_path.to_string_lossy().to_string());
    }

    match read_clipboard_with_system_command(&temp_dir) {
        Ok(path) => return Ok(path),
        Err(_) => {
            let clipboard = _app.clipboard();
            match clipboard.read_image() {
                Ok(image) => {
                    let rgba_data: Vec<u8> = image.rgba().into();
                    let width = image.width();
                    let height = image.height();

                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis();
                    let temp_file_name = format!("pasted-{}.png", timestamp);
                    let temp_path = std::path::Path::new(&temp_dir).join(&temp_file_name);

                    if let Some(parent) = temp_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| AppError(format!("Failed to create temp dir: {}", e)))?;
                    }

                    let buffer = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                        width, height, rgba_data
                    ).ok_or("Failed to create image buffer")?;

                    buffer.save(&temp_path)
                        .map_err(|e| AppError(format!("Failed to save image: {}", e)))?;

                    Ok(temp_path.to_string_lossy().to_string())
                }
                Err(e) => Err(AppError(format!("剪贴板中没有图片或读取失败: {}", e))),
            }
        }
    }
}

/// 从剪贴板读取图片（Android 端 - 不支持）
#[cfg(target_os = "android")]
#[tauri::command]
pub fn paste_image_from_clipboard_raw<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    _temp_dir: String,
) -> Result<String, AppError> {
    Err("剪贴板粘贴功能在 Android 端暂不支持".into())
}
