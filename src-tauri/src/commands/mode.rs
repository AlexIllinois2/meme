use rusqlite::params;
use std::path::PathBuf;
use std::fs;
use crate::core::{models::Mode, error::AppError};
use crate::core::db_state::DbState;
use crate::core::meme_fs::{validate_name, sanitize_folder_name};

/// 获取 meme 基础目录
fn get_meme_base_dir(conn: &rusqlite::Connection) -> Result<String, AppError> {
    let meme_dir: String = conn.query_row(
        "SELECT meme_dir FROM config WHERE id = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| AppError(format!("获取配置失败: {}", e)))?;
    
    if meme_dir.is_empty() {
        return Err(AppError("未设置表情包目录，请先在设置中配置".to_string()));
    }
    
    Ok(meme_dir)
}

/// 构建模式的完整文件夹路径
/// 模式文件夹直接放在 meme_dir 下
fn build_mode_folder_path(conn: &rusqlite::Connection, name: &str) -> Result<String, AppError> {
    let base_dir = get_meme_base_dir(conn)?;
    let safe_name = sanitize_folder_name(name);
    Ok(PathBuf::from(base_dir).join(safe_name).to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_modes(state: tauri::State<'_, DbState>) -> Result<Vec<Mode>, AppError> {
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, folder_path FROM modes ORDER BY sort_order ASC, id ASC"
    )?;
    
    let modes = stmt.query_map([], |row| {
        Ok(Mode {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            folder_path: row.get(3)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    
    Ok(modes)
}

#[tauri::command]
pub fn add_mode(state: tauri::State<'_, DbState>, name: String, sort_order: i32) -> Result<i32, AppError> {
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    // 验证名称
    validate_name(&name)?;
    
    // 构建文件夹路径（名称即文件夹名）
    let folder_path = build_mode_folder_path(&conn, &name)?;
    
    // 1. 先创建文件夹（文件优先）
    fs::create_dir_all(&folder_path)?;
    
    // 2. 再插入数据库
    let result = conn.execute(
        "INSERT INTO modes (name, sort_order, folder_path) VALUES (?, ?, ?)",
        params![name, sort_order, folder_path],
    );
    
    if let Err(e) = result {
        // 数据库插入失败，回滚：删除已创建的文件夹
        let _ = fs::remove_dir_all(&folder_path);
        return Err(AppError(format!("保存到数据库失败: {}", e)));
    }
    
    // 获取新插入的模式ID
    let mode_id: i32 = conn.query_row(
        "SELECT last_insert_rowid()",
        [],
        |row| row.get(0)
    )?;
    
    Ok(mode_id)
}

#[tauri::command]
pub fn update_mode(state: tauri::State<'_, DbState>, id: i32, name: String, sort_order: i32) -> Result<(), AppError> {
    log::info!("Updating mode: id={}, name={}", id, name);
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    
    // 验证名称
    validate_name(&name)?;
    
    // 获取旧的模式信息（用于文件夹重命名）
    let old_mode: Option<(String, String)> = conn.query_row(
        "SELECT name, folder_path FROM modes WHERE id = ?",
        params![id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    ).ok();
    
    // 构建新的文件夹路径
    let new_folder_path = build_mode_folder_path(&conn, &name)?;
    
    // 如果需要重命名文件夹
    if let Some((old_name, old_folder_path)) = &old_mode {
        if old_name != &name {
            // 1. 先重命名文件夹（文件优先）
            if PathBuf::from(old_folder_path).exists() {
                fs::rename(old_folder_path, &new_folder_path)
                    .map_err(|e| AppError(format!("重命名文件夹失败: {}", e)))?;
            } else {
                // 旧文件夹不存在，创建新文件夹
                fs::create_dir_all(&new_folder_path)
                    .map_err(|e| AppError(format!("创建新文件夹失败: {}", e)))?;
            }
        }
    }
    
    // 2. 更新数据库
    let rows = conn.execute(
        "UPDATE modes SET name = ?, sort_order = ?, folder_path = ? WHERE id = ?",
        params![name, sort_order, new_folder_path, id],
    ).map_err(|e| {
        // 数据库更新失败，尝试回滚文件夹
        if let Some((old_name, old_folder_path)) = &old_mode {
            if old_name != &name && PathBuf::from(&new_folder_path).exists() {
                let _ = fs::rename(&new_folder_path, old_folder_path);
            }
        }
        AppError(format!("更新数据库失败: {}", e))
    })?;
    
    log::info!("Updated {} rows", rows);
    if rows == 0 {
        return Err(AppError(format!("Mode with id {} not found", id)));
    }
    
    Ok(())
}

/// 内部函数：删除单个模式及其相关数据
fn delete_mode_internal(conn: &rusqlite::Connection, mode_id: i32) -> Result<(), AppError> {
    // 检查模式是否存在
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM modes WHERE id = ?",
        params![mode_id],
        |row| Ok(row.get::<_, i32>(0)? > 0)
    )?;
    
    if !exists {
        return Err(AppError(format!("模式 ID {} 不存在", mode_id)));
    }
    
    log::info!("Deleting mode {} and all related data...", mode_id);
    
    // 获取模式信息（用于删除文件夹）
    let folder_path: String = conn.query_row(
        "SELECT folder_path FROM modes WHERE id = ?",
        params![mode_id],
        |row| row.get(0)
    )?;
    
    // 1. 先删除文件夹（文件优先）
    if !folder_path.is_empty() {
        let path = std::path::Path::new(&folder_path);
        if path.exists() {
            fs::remove_dir_all(path)
                .map_err(|e| AppError(format!("删除模式文件夹失败 {}: {}", folder_path, e)))?;
        }
    }
    
    // 2. 删除数据库记录
    // 由于外键约束可能未正确设置，我们手动按顺序删除
    
    // 2.1 先删除该模式下所有分组的图片
    let images_deleted = conn.execute(
        "DELETE FROM images WHERE group_id IN (SELECT id FROM groups WHERE mode_id = ?)",
        params![mode_id]
    ).map_err(|e| AppError(format!("删除图片失败: {}", e)))?;
    log::info!("Deleted {} images", images_deleted);
    
    // 2.2 删除关键词-分组关联
    let kg_deleted = conn.execute(
        "DELETE FROM keyword_group_links WHERE group_id IN (SELECT id FROM groups WHERE mode_id = ?)",
        params![mode_id]
    ).map_err(|e| AppError(format!("删除关键词关联失败: {}", e)))?;
    log::info!("Deleted {} keyword-group links", kg_deleted);
    
    // 2.3 删除该模式下的所有分组
    let groups_deleted = conn.execute(
        "DELETE FROM groups WHERE mode_id = ?",
        params![mode_id]
    ).map_err(|e| AppError(format!("删除分组失败: {}", e)))?;
    log::info!("Deleted {} groups", groups_deleted);
    
    // 2.4 最后删除模式本身
    let result = conn.execute("DELETE FROM modes WHERE id = ?", params![mode_id]);
    
    match result {
        Ok(rows) => {
            if rows > 0 {
                log::info!("Mode {} deleted successfully", mode_id);
                Ok(())
            } else {
                Err(AppError(format!("Failed to delete mode {}", mode_id)))
            }
        }
        Err(e) => {
            log::error!("Database error deleting mode: {}", e);
            Err(AppError(format!("删除模式失败: {}", e)))
        }
    }
}

#[tauri::command]
pub fn delete_mode(state: tauri::State<'_, DbState>, mode_id: i32) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    delete_mode_internal(&conn, mode_id)
}

#[tauri::command]
pub fn delete_modes(state: tauri::State<'_, DbState>, mode_ids: Vec<i32>) -> Result<(), AppError> {
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    
    let count = mode_ids.len();
    for id in mode_ids {
        log::info!("Deleting mode {}...", id);
        delete_mode_internal(&conn, id)?;
    }
    
    log::info!("Successfully deleted {} modes", count);
    Ok(())
}
