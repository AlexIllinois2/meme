//! 图片上传模块
//!
//! 处理桌面端和 Android 端的图片上传逻辑

use rusqlite::params;
use crate::core::{db_state::DbState, meme_fs};

/// Android 图片数据结构
#[derive(serde::Deserialize)]
pub struct AndroidImageData {
    /// 文件名
    pub name: String,
    /// Base64 编码的图片数据
    pub data: String,
}

/// 检测图片格式
fn detect_image_format(data: &[u8]) -> Option<&'static str> {
    if data.len() < 8 { return None; }
    if data.starts_with(b"\x89PNG") { Some("png") }
    else if data.starts_with(b"\xFF\xD8\xFF") { Some("jpg") }
    else if data.starts_with(b"GIF89a") || data.starts_with(b"GIF87a") { Some("gif") }
    else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" { Some("webp") }
    else if data.starts_with(b"BM") { Some("bmp") }
    else { None }
}

/// Base64 解码
fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    let data = if data.starts_with("data:") {
        if let Some(idx) = data.find(";base64,") {
            &data[idx + 8..]
        } else { data }
    } else { data };
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.decode(data).map_err(|e| format!("Base64 解码失败: {}", e))
}

/// 桌面端上传图片
#[tauri::command]
pub fn upload_images(
    state: tauri::State<'_, DbState>,
    file_paths: Vec<String>,
    group_id: i32,
    mode_id: i32,
    meme_dir: String,
) -> Result<(), crate::core::error::AppError> {
    use crate::core::error::AppError;
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    let group_folder: String = conn.query_row(
        "SELECT folder_path FROM groups WHERE id = ?", params![group_id],
        |row| row.get(0)
    ).map_err(|e| AppError(format!("获取分组信息失败: {}", e)))?;

    for file_path in &file_paths {
        let src_path = std::path::Path::new(file_path);
        if !src_path.exists() { continue; }
        let file_name = src_path.file_name()
            .and_then(|n| n.to_str()).unwrap_or("unknown.png");
        let dest_path = std::path::Path::new(&group_folder).join(file_name);
        std::fs::copy(src_path, &dest_path)
            .map_err(|e| AppError(format!("复制文件失败 {}: {}", file_path, e)))?;
        let dest_path_str = dest_path.to_string_lossy().to_string();
        let relative_path = meme_fs::relative_path(&meme_dir, &dest_path_str);
        let existing: Option<i32> = conn.query_row(
            "SELECT id FROM images WHERE image_path = ?",
            params![relative_path], |row| row.get(0)
        ).ok();
        if existing.is_none() {
            conn.execute(
                "INSERT INTO images (image_path, thumbnail_path, share_count, group_id, mode_id) VALUES (?, NULL, 0, ?, ?)",
                params![relative_path, group_id, mode_id]
            )?;
        }
    }
    Ok(())
}

/// Android 端上传图片（接收 Base64 编码的图片数据）
#[tauri::command]
pub async fn upload_images_android(
    state: tauri::State<'_, DbState>,
    images_data: Vec<AndroidImageData>,
    group_id: i32,
    mode_id: i32,
) -> Result<usize, crate::core::error::AppError> {
    use crate::core::error::AppError;
    log::info!("[Android] upload_images_android 开始, 图片数量: {}", images_data.len());
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;

    let meme_dir: String = conn.query_row(
        "SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
    ).unwrap_or_default();

    let group_folder: String = conn.query_row(
        "SELECT folder_path FROM groups WHERE id = ?", params![group_id],
        |row| row.get(0)
    ).map_err(|e| AppError(format!("获取分组信息失败: {}", e)))?;

    let mut success_count = 0;
    for image_data in &images_data {
        let data = base64_decode(&image_data.data)?;
        let ext = detect_image_format(&data).unwrap_or("png");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
        let file_name = format!("image-{}.{}", timestamp, ext);
        let dest_path = std::path::Path::new(&group_folder).join(&file_name);
        std::fs::write(&dest_path, &data)
            .map_err(|e| AppError(format!("写入文件失败: {}", e)))?;
        let dest_path_str = dest_path.to_string_lossy().to_string();
        let relative_path = meme_fs::relative_path(&meme_dir, &dest_path_str);
        conn.execute(
            "INSERT INTO images (image_path, thumbnail_path, share_count, group_id, mode_id) VALUES (?, NULL, 0, ?, ?)",
            params![relative_path, group_id, mode_id]
        ).map_err(|e| AppError(format!("插入数据库失败: {}", e)))?;
        success_count += 1;
    }
    log::info!("[Android] upload_images_android 完成，成功上传 {} 张", success_count);
    Ok(success_count)
}
