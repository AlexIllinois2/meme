use rusqlite::params;
use std::path::PathBuf;
use std::fs;
use crate::{db::init_db, models::Mode, trash::move_to_trash};

/// 非法文件名字符
const INVALID_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// 验证名称是否合法（不包含非法字符）
fn validate_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("名称不能为空".to_string());
    }
    
    if name.trim() != name {
        return Err("名称首尾不能有空格".to_string());
    }
    
    for c in name.chars() {
        if INVALID_CHARS.contains(&c) {
            return Err(format!("名称包含非法字符: '{}'", c));
        }
    }
    
    Ok(())
}

/// 生成安全的文件夹名称（替换非法字符）
fn sanitize_folder_name(name: &str) -> String {
    name.chars()
        .map(|c| if INVALID_CHARS.contains(&c) { '_' } else { c })
        .collect()
}

/// 获取 meme 基础目录
fn get_meme_base_dir() -> Result<String, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let meme_dir: String = conn.query_row(
        "SELECT meme_dir FROM config WHERE id = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| format!("获取配置失败: {}", e))?;
    
    if meme_dir.is_empty() {
        return Err("未设置表情包目录，请先在设置中配置".to_string());
    }
    
    Ok(meme_dir)
}

/// 构建模式的完整文件夹路径
/// 模式文件夹直接放在 meme_dir 下
fn build_mode_folder_path(name: &str) -> Result<String, String> {
    let base_dir = get_meme_base_dir()?;
    let safe_name = sanitize_folder_name(name);
    Ok(PathBuf::from(base_dir).join(safe_name).to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_modes() -> Result<Vec<Mode>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, folder_path FROM modes ORDER BY sort_order ASC, id ASC"
    ).map_err(|e| e.to_string())?;
    
    let modes = stmt.query_map([], |row| {
        Ok(Mode {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            folder_path: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    Ok(modes)
}

#[tauri::command]
pub fn add_mode(name: String, sort_order: i32) -> Result<i32, String> {
    // 验证名称
    validate_name(&name)?;
    
    // 构建文件夹路径（名称即文件夹名）
    let folder_path = build_mode_folder_path(&name)?;
    
    // 1. 先创建文件夹（文件优先）
    fs::create_dir_all(&folder_path)
        .map_err(|e| format!("创建模式文件夹失败: {}", e))?;
    
    // 2. 再插入数据库
    let conn = init_db().map_err(|e| e.to_string())?;
    
    let result = conn.execute(
        "INSERT INTO modes (name, sort_order, folder_path) VALUES (?, ?, ?)",
        params![name, sort_order, folder_path],
    );
    
    if let Err(e) = result {
        // 数据库插入失败，回滚：删除已创建的文件夹
        let _ = fs::remove_dir_all(&folder_path);
        return Err(format!("保存到数据库失败: {}", e));
    }
    
    // 获取新插入的模式ID
    let mode_id: i32 = conn.query_row(
        "SELECT last_insert_rowid()",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    Ok(mode_id)
}

#[tauri::command]
pub fn update_mode(id: i32, name: String, sort_order: i32) -> Result<(), String> {
    eprintln!("Updating mode: id={}, name={}", id, name);
    
    // 验证名称
    validate_name(&name)?;
    
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 获取旧的模式信息（用于文件夹重命名）
    let old_mode: Option<(String, String)> = conn.query_row(
        "SELECT name, folder_path FROM modes WHERE id = ?",
        params![id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    ).ok();
    
    // 构建新的文件夹路径
    let new_folder_path = build_mode_folder_path(&name)?;
    
    // 如果需要重命名文件夹
    if let Some((old_name, old_folder_path)) = &old_mode {
        if old_name != &name {
            // 1. 先重命名文件夹（文件优先）
            if PathBuf::from(old_folder_path).exists() {
                fs::rename(old_folder_path, &new_folder_path)
                    .map_err(|e| format!("重命名文件夹失败: {}", e))?;
            } else {
                // 旧文件夹不存在，创建新文件夹
                fs::create_dir_all(&new_folder_path)
                    .map_err(|e| format!("创建新文件夹失败: {}", e))?;
            }
        }
    }
    
    // 2. 更新数据库
    let result = conn.execute(
        "UPDATE modes SET name = ?, sort_order = ?, folder_path = ? WHERE id = ?",
        params![name, sort_order, new_folder_path, id],
    );
    
    if let Err(e) = result {
        // 数据库更新失败，尝试回滚文件夹
        if let Some((old_name, old_folder_path)) = &old_mode {
            if old_name != &name && PathBuf::from(&new_folder_path).exists() {
                let _ = fs::rename(&new_folder_path, old_folder_path);
            }
        }
        return Err(format!("更新数据库失败: {}", e));
    }
    
    match result {
        Ok(rows) => {
            eprintln!("Updated {} rows", rows);
            if rows == 0 {
                return Err(format!("Mode with id {} not found", id));
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(e.to_string())
        }
    }
}

/// 内部函数：删除单个模式及其相关数据
fn delete_mode_internal(conn: &rusqlite::Connection, mode_id: i32) -> Result<(), String> {
    // 检查模式是否存在
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM modes WHERE id = ?",
        params![mode_id],
        |row| Ok(row.get::<_, i32>(0)? > 0)
    ).map_err(|e| e.to_string())?;
    
    if !exists {
        return Err(format!("模式 ID {} 不存在", mode_id));
    }
    
    eprintln!("Deleting mode {} and all related data...", mode_id);
    
    // 获取模式信息（用于删除文件夹）
    let folder_path: String = conn.query_row(
        "SELECT folder_path FROM modes WHERE id = ?",
        params![mode_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    // 1. 先移动文件夹到回收站（文件优先）
    if !folder_path.is_empty() {
        let _ = move_to_trash(&folder_path);
    }
    
    // 2. 删除数据库记录
    // 由于外键约束可能未正确设置，我们手动按顺序删除
    
    // 2.1 先删除该模式下所有分组的图片
    let images_deleted = conn.execute(
        "DELETE FROM images WHERE group_id IN (SELECT id FROM groups WHERE mode_id = ?)",
        params![mode_id]
    ).map_err(|e| format!("删除图片失败: {}", e))?;
    eprintln!("Deleted {} images", images_deleted);
    
    // 2.2 删除关键词-分组关联
    let kg_deleted = conn.execute(
        "DELETE FROM keyword_group_links WHERE group_id IN (SELECT id FROM groups WHERE mode_id = ?)",
        params![mode_id]
    ).map_err(|e| format!("删除关键词关联失败: {}", e))?;
    eprintln!("Deleted {} keyword-group links", kg_deleted);
    
    // 2.3 删除该模式下的所有分组
    let groups_deleted = conn.execute(
        "DELETE FROM groups WHERE mode_id = ?",
        params![mode_id]
    ).map_err(|e| format!("删除分组失败: {}", e))?;
    eprintln!("Deleted {} groups", groups_deleted);
    
    // 2.4 最后删除模式本身
    let result = conn.execute("DELETE FROM modes WHERE id = ?", params![mode_id]);
    
    match result {
        Ok(rows) => {
            if rows > 0 {
                eprintln!("Mode {} deleted successfully", mode_id);
                Ok(())
            } else {
                Err(format!("Failed to delete mode {}", mode_id))
            }
        }
        Err(e) => {
            eprintln!("Database error deleting mode: {}", e);
            Err(format!("删除模式失败: {}", e))
        }
    }
}

#[tauri::command]
pub fn delete_mode(mode_id: i32) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    delete_mode_internal(&conn, mode_id)
}

#[tauri::command]
pub fn delete_modes(mode_ids: Vec<i32>) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    let count = mode_ids.len();
    for id in mode_ids {
        eprintln!("Deleting mode {}...", id);
        delete_mode_internal(&conn, id)?;
    }
    
    eprintln!("Successfully deleted {} modes", count);
    Ok(())
}