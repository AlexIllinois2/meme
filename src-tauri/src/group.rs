use rusqlite::params;
use std::path::PathBuf;
use std::fs;
use crate::{db::init_db, models::Group, trash::move_to_trash};
use crate::keyword::{convert_to_pinyin, convert_to_acronym};

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

/// 获取模式对应的文件夹路径
fn get_mode_folder_path(mode_id: i32) -> Result<String, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let folder_path: String = conn.query_row(
        "SELECT folder_path FROM modes WHERE id = ?",
        params![mode_id],
        |row| row.get(0)
    ).map_err(|e| format!("获取模式路径失败: {}", e))?;
    
    Ok(folder_path)
}

/// 构建分组的完整文件夹路径
/// 分组文件夹放在对应模式的文件夹下
fn build_group_folder_path(mode_id: i32, name: &str) -> Result<String, String> {
    let mode_path = get_mode_folder_path(mode_id)?;
    let safe_name = sanitize_folder_name(name);
    Ok(PathBuf::from(mode_path).join(safe_name).to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_groups_by_mode(mode_id: i32) -> Result<Vec<Group>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
         FROM groups g 
         LEFT JOIN modes m ON g.mode_id = m.id 
         WHERE g.mode_id = ? 
         ORDER BY g.share_count DESC, g.id ASC"
    ).map_err(|e| e.to_string())?;
    
    let groups = stmt.query_map(params![mode_id], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            folder_path: row.get(2)?,
            share_count: row.get(3)?,
            mode_id: row.get(4)?,
            mode_name: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    Ok(groups)
}

#[tauri::command]
pub fn get_all_groups() -> Result<Vec<Group>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
         FROM groups g 
         LEFT JOIN modes m ON g.mode_id = m.id 
         ORDER BY g.share_count DESC, g.id ASC"
    ).map_err(|e| e.to_string())?;
    
    let groups = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            folder_path: row.get(2)?,
            share_count: row.get(3)?,
            mode_id: row.get(4)?,
            mode_name: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    Ok(groups)
}

#[tauri::command]
pub fn get_all_groups_simple() -> Result<Vec<Group>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
         FROM groups g 
         LEFT JOIN modes m ON g.mode_id = m.id 
         ORDER BY g.name"
    ).map_err(|e| e.to_string())?;
    
    let groups = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            folder_path: row.get(2)?,
            share_count: row.get(3)?,
            mode_id: row.get(4)?,
            mode_name: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    Ok(groups)
}

/// 为分组自动创建关键词关联
fn auto_create_group_keywords(conn: &rusqlite::Connection, group_id: i32, group_name: &str) -> Result<(), String> {
    // 解析分组名获取关键词（按中英文逗号、顿号、空格分割）
    let keywords: Vec<String> = group_name.split(&[',', '，', '、', ' '][..])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    // 为每个关键词创建记录并建立关联
    for keyword in &keywords {
        let has_chinese = keyword.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
        
        // 生成拼音和首字母缩写
        let pinyin = if has_chinese {
            Some(convert_to_pinyin(keyword))
        } else {
            None
        };
        let acronym = if has_chinese {
            Some(convert_to_acronym(keyword))
        } else {
            None
        };
        
        // 插入关键词（如果不存在）
        conn.execute(
            "INSERT OR IGNORE INTO keywords (keyword, pinyin, acronym) VALUES (?, ?, ?)",
            params![keyword, pinyin, acronym],
        ).map_err(|e| e.to_string())?;
        
        // 获取关键词ID
        let keyword_id: i32 = conn.query_row(
            "SELECT id FROM keywords WHERE keyword = ?",
            params![keyword],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;
        
        // 建立关键词-分组关联
        conn.execute(
            "INSERT OR IGNORE INTO keyword_group_links (keyword_id, group_id) VALUES (?, ?)",
            params![keyword_id, group_id],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn add_group(name: String, mode_id: i32) -> Result<i32, String> {
    // 验证名称
    validate_name(&name)?;
    
    // 构建文件夹路径（名称即文件夹名）
    let folder_path = build_group_folder_path(mode_id, &name)?;
    
    // 1. 先创建文件夹（文件优先）
    fs::create_dir_all(&folder_path)
        .map_err(|e| format!("创建分组文件夹失败: {}", e))?;
    
    // 2. 再插入数据库
    let conn = init_db().map_err(|e| e.to_string())?;
    
    let result = conn.execute(
        "INSERT INTO groups (name, folder_path, share_count, mode_id) VALUES (?, ?, 0, ?)",
        params![name, folder_path, mode_id],
    );
    
    if let Err(e) = result {
        // 数据库插入失败，回滚：删除已创建的文件夹
        let _ = fs::remove_dir_all(&folder_path);
        return Err(format!("保存到数据库失败: {}", e));
    }
    
    // 获取新插入的分组ID
    let group_id: i32 = conn.query_row(
        "SELECT last_insert_rowid()",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    // 自动创建关键词关联
    auto_create_group_keywords(&conn, group_id, &name)?;
    
    Ok(group_id)
}

#[tauri::command]
pub fn update_group(id: i32, name: String, mode_id: i32) -> Result<(), String> {
    // 验证名称
    validate_name(&name)?;
    
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 获取旧的分组信息（用于文件夹重命名）
    let old_group: Option<(String, i32, String)> = conn.query_row(
        "SELECT name, mode_id, folder_path FROM groups WHERE id = ?",
        params![id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?, row.get::<_, String>(2)?))
    ).ok();
    
    // 构建新的文件夹路径
    let new_folder_path = build_group_folder_path(mode_id, &name)?;
    
    // 如果需要重命名或移动文件夹
    if let Some((old_name, old_mode_id, old_folder_path)) = &old_group {
        if old_name != &name || old_mode_id != &mode_id {
            // 1. 先处理文件夹（文件优先）
            if PathBuf::from(old_folder_path).exists() {
                // 如果路径不同，需要移动/重命名
                if old_folder_path != &new_folder_path {
                    fs::rename(old_folder_path, &new_folder_path)
                        .map_err(|e| format!("重命名/移动文件夹失败: {}", e))?;
                }
            } else {
                // 旧文件夹不存在，创建新文件夹
                fs::create_dir_all(&new_folder_path)
                    .map_err(|e| format!("创建新文件夹失败: {}", e))?;
            }
        }
    }
    
    // 2. 更新数据库
    let result = conn.execute(
        "UPDATE groups SET name = ?, folder_path = ?, mode_id = ? WHERE id = ?",
        params![name, new_folder_path, mode_id, id],
    );
    
    if let Err(e) = &result {
        // 数据库更新失败，尝试回滚文件夹
        if let Some((old_name, old_mode_id, old_folder_path)) = &old_group {
            if (old_name != &name || old_mode_id != &mode_id) && PathBuf::from(&new_folder_path).exists() {
                let _ = fs::rename(&new_folder_path, old_folder_path);
            }
        }
        return Err(format!("更新数据库失败: {}", e));
    }
    
    // 删除旧的关键词关联并重新创建
    conn.execute("DELETE FROM keyword_group_links WHERE group_id = ?", params![id])
        .map_err(|e| e.to_string())?;
    auto_create_group_keywords(&conn, id, &name)?;
    
    Ok(())
}

/// 内部函数：删除单个分组
fn delete_group_internal(conn: &rusqlite::Connection, group_id: i32) -> Result<(), String> {
    // 获取分组信息（用于删除文件夹）
    let folder_path: String = conn.query_row(
        "SELECT folder_path FROM groups WHERE id = ?",
        params![group_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    // 1. 先移动文件夹到回收站（文件优先）
    if !folder_path.is_empty() {
        let _ = move_to_trash(&folder_path);
    }
    
    // 2. 删除关键词关联
    conn.execute("DELETE FROM keyword_group_links WHERE group_id = ?", params![group_id])
        .map_err(|e| e.to_string())?;
    
    // 3. 删除分组
    conn.execute("DELETE FROM groups WHERE id = ?", params![group_id])
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn delete_group(group_id: i32) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    delete_group_internal(&conn, group_id)
}

#[tauri::command]
pub fn delete_groups(group_ids: Vec<i32>) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    for id in group_ids {
        delete_group_internal(&conn, id)?;
    }
    Ok(())
}

/// 搜索分组
/// pinyin_search 和 acronym_search 控制是否按拼音/缩写搜索
#[tauri::command]
pub fn search_groups(
    keyword: String,
    mode_id: Option<i32>,
    pinyin_search: bool,
    acronym_search: bool,
) -> Result<Vec<Group>, String> {
    // 如果搜索内容为空，返回所有分组
    if keyword.trim().is_empty() {
        if let Some(mid) = mode_id {
            return get_groups_by_mode(mid);
        } else {
            return get_all_groups();
        }
    }
    
    let conn = init_db().map_err(|e| e.to_string())?;
    let search_pattern = format!("%{}%", keyword);
    
    // 根据搜索选项构建查询和参数
    let groups = if let Some(mid) = mode_id {
        let query = match (pinyin_search, acronym_search) {
            (false, false) => {
                // 只按关键词搜索
                "SELECT DISTINCT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
                 FROM groups g
                 LEFT JOIN modes m ON g.mode_id = m.id
                 JOIN keyword_group_links kgl ON g.id = kgl.group_id
                 JOIN keywords k ON kgl.keyword_id = k.id
                 WHERE g.mode_id = ? AND k.keyword LIKE ?
                 ORDER BY g.share_count DESC, g.id ASC"
            }
            (true, false) => {
                // 关键词 + 拼音
                "SELECT DISTINCT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
                 FROM groups g
                 LEFT JOIN modes m ON g.mode_id = m.id
                 JOIN keyword_group_links kgl ON g.id = kgl.group_id
                 JOIN keywords k ON kgl.keyword_id = k.id
                 WHERE g.mode_id = ? AND (k.keyword LIKE ? OR k.pinyin LIKE ?)
                 ORDER BY g.share_count DESC, g.id ASC"
            }
            (false, true) => {
                // 关键词 + 缩写
                "SELECT DISTINCT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
                 FROM groups g
                 LEFT JOIN modes m ON g.mode_id = m.id
                 JOIN keyword_group_links kgl ON g.id = kgl.group_id
                 JOIN keywords k ON kgl.keyword_id = k.id
                 WHERE g.mode_id = ? AND (k.keyword LIKE ? OR k.acronym LIKE ?)
                 ORDER BY g.share_count DESC, g.id ASC"
            }
            (true, true) => {
                // 关键词 + 拼音 + 缩写
                "SELECT DISTINCT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
                 FROM groups g
                 LEFT JOIN modes m ON g.mode_id = m.id
                 JOIN keyword_group_links kgl ON g.id = kgl.group_id
                 JOIN keywords k ON kgl.keyword_id = k.id
                 WHERE g.mode_id = ? AND (k.keyword LIKE ? OR k.pinyin LIKE ? OR k.acronym LIKE ?)
                 ORDER BY g.share_count DESC, g.id ASC"
            }
        };
        
        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
        
        match (pinyin_search, acronym_search) {
            (false, false) => {
                stmt.query_map(params![mid, search_pattern], |row| {
                    Ok(Group {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        folder_path: row.get(2)?,
                        share_count: row.get(3)?,
                        mode_id: row.get(4)?,
                        mode_name: row.get(5)?,
                    })
                }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
            }
            (true, false) | (false, true) => {
                stmt.query_map(params![mid, search_pattern, search_pattern], |row| {
                    Ok(Group {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        folder_path: row.get(2)?,
                        share_count: row.get(3)?,
                        mode_id: row.get(4)?,
                        mode_name: row.get(5)?,
                    })
                }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
            }
            (true, true) => {
                stmt.query_map(params![mid, search_pattern, search_pattern, search_pattern], |row| {
                    Ok(Group {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        folder_path: row.get(2)?,
                        share_count: row.get(3)?,
                        mode_id: row.get(4)?,
                        mode_name: row.get(5)?,
                    })
                }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
            }
        }
    } else {
        let query = match (pinyin_search, acronym_search) {
            (false, false) => {
                "SELECT DISTINCT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
                 FROM groups g
                 LEFT JOIN modes m ON g.mode_id = m.id
                 JOIN keyword_group_links kgl ON g.id = kgl.group_id
                 JOIN keywords k ON kgl.keyword_id = k.id
                 WHERE k.keyword LIKE ?
                 ORDER BY g.share_count DESC, g.id ASC"
            }
            (true, false) => {
                "SELECT DISTINCT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
                 FROM groups g
                 LEFT JOIN modes m ON g.mode_id = m.id
                 JOIN keyword_group_links kgl ON g.id = kgl.group_id
                 JOIN keywords k ON kgl.keyword_id = k.id
                 WHERE k.keyword LIKE ? OR k.pinyin LIKE ?
                 ORDER BY g.share_count DESC, g.id ASC"
            }
            (false, true) => {
                "SELECT DISTINCT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
                 FROM groups g
                 LEFT JOIN modes m ON g.mode_id = m.id
                 JOIN keyword_group_links kgl ON g.id = kgl.group_id
                 JOIN keywords k ON kgl.keyword_id = k.id
                 WHERE k.keyword LIKE ? OR k.acronym LIKE ?
                 ORDER BY g.share_count DESC, g.id ASC"
            }
            (true, true) => {
                "SELECT DISTINCT g.id, g.name, g.folder_path, g.share_count, g.mode_id, m.name 
                 FROM groups g
                 LEFT JOIN modes m ON g.mode_id = m.id
                 JOIN keyword_group_links kgl ON g.id = kgl.group_id
                 JOIN keywords k ON kgl.keyword_id = k.id
                 WHERE k.keyword LIKE ? OR k.pinyin LIKE ? OR k.acronym LIKE ?
                 ORDER BY g.share_count DESC, g.id ASC"
            }
        };
        
        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
        
        match (pinyin_search, acronym_search) {
            (false, false) => {
                stmt.query_map(params![search_pattern], |row| {
                    Ok(Group {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        folder_path: row.get(2)?,
                        share_count: row.get(3)?,
                        mode_id: row.get(4)?,
                        mode_name: row.get(5)?,
                    })
                }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
            }
            (true, false) | (false, true) => {
                stmt.query_map(params![search_pattern, search_pattern], |row| {
                    Ok(Group {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        folder_path: row.get(2)?,
                        share_count: row.get(3)?,
                        mode_id: row.get(4)?,
                        mode_name: row.get(5)?,
                    })
                }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
            }
            (true, true) => {
                stmt.query_map(params![search_pattern, search_pattern, search_pattern], |row| {
                    Ok(Group {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        folder_path: row.get(2)?,
                        share_count: row.get(3)?,
                        mode_id: row.get(4)?,
                        mode_name: row.get(5)?,
                    })
                }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
            }
        }
    };
    
    Ok(groups)
}

/// 重建所有分组的关键词关联
#[tauri::command]
pub fn rebuild_all_group_keywords() -> Result<i32, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 获取所有分组
    let mut stmt = conn.prepare(
        "SELECT id, name FROM groups"
    ).map_err(|e| e.to_string())?;
    
    let groups: Vec<(i32, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    
    let mut count = 0;
    for (group_id, group_name) in groups {
        // 删除旧的关键词关联
        conn.execute(
            "DELETE FROM keyword_group_links WHERE group_id = ?",
            params![group_id],
        ).map_err(|e| e.to_string())?;
        
        // 重新创建关键词关联
        auto_create_group_keywords(&conn, group_id, &group_name)?;
        count += 1;
    }
    
    Ok(count)
}