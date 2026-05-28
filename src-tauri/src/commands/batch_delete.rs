//! 批量删除命令
//!
//! 将前端三次 invoke（delete_modes + delete_groups + delete_images）
//! 合并为一次 Rust 调用，减少 RPC 往返 + 保证原子性。

use serde::Deserialize;
use rusqlite::params;
use crate::core::{db_state::DbState, error::AppError, meme_fs};

#[derive(Deserialize)]
pub struct BatchDeleteRequest {
    pub mode_ids: Vec<i32>,
    pub group_ids: Vec<i32>,
    pub image_ids: Vec<i32>,
}

#[tauri::command]
pub fn batch_delete(
    state: tauri::State<'_, DbState>,
    req: BatchDeleteRequest,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;

    // 获取 meme_dir（图片删除需要解析路径）
    let meme_dir: String = conn.query_row(
        "SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
    ).unwrap_or_default();

    // 1. 删除模式（级联删除分组 + 图片 + 关键词链接）
    for mode_id in &req.mode_ids {
        // 先删文件
        let folder_path: String = conn.query_row(
            "SELECT folder_path FROM modes WHERE id = ?", params![mode_id], |row| row.get(0)
        ).unwrap_or_default();
        if !folder_path.is_empty() {
            let path = std::path::Path::new(&folder_path);
            if path.exists() {
                std::fs::remove_dir_all(path)
                    .map_err(|e| AppError(format!("删除模式文件夹失败 {}: {}", folder_path, e)))?;
            }
        }
        // 删数据库
        conn.execute("DELETE FROM images WHERE group_id IN (SELECT id FROM groups WHERE mode_id = ?)", params![mode_id])?;
        conn.execute("DELETE FROM keyword_group_links WHERE group_id IN (SELECT id FROM groups WHERE mode_id = ?)", params![mode_id])?;
        conn.execute("DELETE FROM groups WHERE mode_id = ?", params![mode_id])?;
        conn.execute("DELETE FROM modes WHERE id = ?", params![mode_id])?;
    }

    // 2. 删除分组（级联删除图片）
    for group_id in &req.group_ids {
        let folder_path: String = conn.query_row(
            "SELECT folder_path FROM groups WHERE id = ?", params![group_id], |row| row.get(0)
        ).unwrap_or_default();
        if !folder_path.is_empty() {
            let path = std::path::Path::new(&folder_path);
            if path.exists() {
                std::fs::remove_dir_all(path)
                    .map_err(|e| AppError(format!("删除分组文件夹失败 {}: {}", folder_path, e)))?;
            }
        }
        conn.execute("DELETE FROM images WHERE group_id = ?", params![group_id])?;
        conn.execute("DELETE FROM keyword_group_links WHERE group_id = ?", params![group_id])?;
        conn.execute("DELETE FROM groups WHERE id = ?", params![group_id])?;
    }

    // 3. 删除独立图片（不在已删分组中的）
    for image_id in &req.image_ids {
        // 跳过已通过分组删除的图片
        let already_deleted: bool = conn.query_row(
            "SELECT COUNT(*) FROM images WHERE id = ?", params![image_id], |row| row.get::<_, i32>(0)
        ).unwrap_or(0) == 0;
        if already_deleted { continue; }

        if let Ok((raw_path, thumb_path)) = conn.query_row::<(String, Option<String>), _, _>(
            "SELECT image_path, thumbnail_path FROM images WHERE id = ?",
            params![image_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        ) {
            let image_path = meme_fs::resolve_meme_path(&meme_dir, &raw_path);
            if image_path.exists() {
                let _ = std::fs::remove_file(&image_path);
            }
            if let Some(thumb) = thumb_path {
                let thumb_path = std::path::Path::new(&thumb);
                if thumb_path.exists() {
                    let _ = std::fs::remove_file(thumb_path);
                }
            }
        }
        conn.execute("DELETE FROM images WHERE id = ?", params![image_id])?;
    }

    Ok(())
}
