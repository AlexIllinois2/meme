use rusqlite::params;
use std::path::PathBuf;
use std::collections::HashMap;
use pinyin::ToPinyin;
use crate::{db::init_db, models::Keyword};
use serde::{Deserialize, Serialize};

/// 使用 pinyin crate 转换拼音
pub fn convert_to_pinyin(text: &str) -> String {
    let mut result = String::new();
    for pinyin_opt in text.to_pinyin() {
        if let Some(py) = pinyin_opt {
            result.push_str(py.plain());
        }
    }
    result
}

/// 生成首字母缩写
pub fn convert_to_acronym(text: &str) -> String {
    let mut result = String::new();
    for pinyin_opt in text.to_pinyin() {
        if let Some(py) = pinyin_opt {
            let plain = py.plain();
            if let Some(first_char) = plain.chars().next() {
                result.push(first_char);
            }
        }
    }
    result
}

/// keywords.toml 结构定义
/// 关键词只与分组关联，与模式无关
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct KeywordsToml {
    #[serde(default)]
    pub groups_keywords: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub keywords_pinyin: HashMap<String, KeywordPinyin>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KeywordPinyin {
    pub pinyin: String,
    pub abbr: String,
}

/// 获取所有关键词
/// 关键词只关联分组，不关联模式
#[tauri::command]
pub fn get_all_keywords() -> Result<Vec<Keyword>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT k.id, k.keyword, k.pinyin, k.acronym,
                (SELECT COUNT(*) FROM keyword_group_links WHERE keyword_id = k.id) as group_count
         FROM keywords k
         ORDER BY k.keyword"
    ).map_err(|e| e.to_string())?;
    
    let mut keywords_with_links: Vec<Keyword> = Vec::new();
    
    let keyword_rows = stmt.query_map([], |row| {
        Ok(Keyword {
            id: row.get(0)?,
            keyword: row.get(1)?,
            pinyin: row.get(2)?,
            acronym: row.get(3)?,
            group_count: row.get(4)?,
            mode_count: None,
            group_ids: None,
            mode_ids: None,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    for mut kw in keyword_rows {
        // 查询关联的分组ID
        let mut group_stmt = conn.prepare(
            "SELECT group_id FROM keyword_group_links WHERE keyword_id = ?"
        ).map_err(|e| e.to_string())?;
        let group_ids: Vec<i32> = group_stmt.query_map(params![kw.id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        kw.group_ids = Some(group_ids);
        // mode_ids 始终为 None，关键词不直接关联模式
        
        keywords_with_links.push(kw);
    }
    
    Ok(keywords_with_links)
}

/// 添加关键词
/// 自动为中文关键词生成拼音和首字母缩写
#[tauri::command]
pub fn add_keyword(
    keyword: String,
    pinyin: Option<String>,
    acronym: Option<String>,
    group_ids: Vec<i32>,
) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 自动为中文关键词生成拼音
    let has_chinese = keyword.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
    
    let final_pinyin = if has_chinese && pinyin.is_none() {
        Some(convert_to_pinyin(&keyword))
    } else {
        pinyin
    };
    
    let final_acronym = if has_chinese && acronym.is_none() {
        Some(convert_to_acronym(&keyword))
    } else {
        acronym
    };
    
    // 插入关键词
    conn.execute(
        "INSERT INTO keywords (keyword, pinyin, acronym) VALUES (?, ?, ?)",
        params![keyword, final_pinyin, final_acronym],
    ).map_err(|e| e.to_string())?;
    
    // 获取新插入的关键词ID
    let keyword_id = conn.last_insert_rowid() as i32;
    
    // 插入关联关系
    for group_id in group_ids {
        conn.execute(
            "INSERT OR IGNORE INTO keyword_group_links (keyword_id, group_id) VALUES (?, ?)",
            params![keyword_id, group_id],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// 更新关键词
/// 自动为中文关键词生成拼音和首字母缩写
#[tauri::command]
pub fn update_keyword(
    id: i32,
    keyword: String,
    pinyin: Option<String>,
    acronym: Option<String>,
    group_ids: Vec<i32>,
) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 自动为中文关键词生成拼音
    let has_chinese = keyword.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
    
    let final_pinyin = if has_chinese && pinyin.is_none() {
        Some(convert_to_pinyin(&keyword))
    } else {
        pinyin
    };
    
    let final_acronym = if has_chinese && acronym.is_none() {
        Some(convert_to_acronym(&keyword))
    } else {
        acronym
    };
    
    // 更新关键词
    conn.execute(
        "UPDATE keywords SET keyword = ?, pinyin = ?, acronym = ? WHERE id = ?",
        params![keyword, final_pinyin, final_acronym, id],
    ).map_err(|e| e.to_string())?;
    
    // 删除旧的关联关系
    conn.execute("DELETE FROM keyword_group_links WHERE keyword_id = ?", params![id])
        .map_err(|e| e.to_string())?;
    
    // 插入新的关联关系
    for group_id in group_ids {
        conn.execute(
            "INSERT OR IGNORE INTO keyword_group_links (keyword_id, group_id) VALUES (?, ?)",
            params![id, group_id],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// 内部函数：删除单个关键词
fn delete_keyword_internal(conn: &rusqlite::Connection, keyword_id: i32) -> Result<(), String> {
    conn.execute("DELETE FROM keywords WHERE id = ?", params![keyword_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_keyword(keyword_id: i32) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    delete_keyword_internal(&conn, keyword_id)
}

#[tauri::command]
pub fn delete_keywords(keyword_ids: Vec<i32>) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    for id in keyword_ids {
        delete_keyword_internal(&conn, id)?;
    }
    Ok(())
}

/// 从文件加载关键词配置
fn load_keywords_from_file(meme_dir: String) -> Result<KeywordsToml, String> {
    let keywords_file = PathBuf::from(&meme_dir).join("keywords.toml");
    
    if !keywords_file.exists() {
        return Ok(KeywordsToml::default());
    }
    
    let content = std::fs::read_to_string(&keywords_file)
        .map_err(|e| format!("Failed to read keywords.toml: {}", e))?;
    
    let keywords_toml: KeywordsToml = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse keywords.toml: {}", e))?;
    
    Ok(keywords_toml)
}

/// 生成关键词配置文件 - 按需修改，不重写整个文件
/// 使用 toml_edit 保留原有格式，只修改变更的行
#[tauri::command]
pub fn generate_keywords_file(meme_dir: String, _generate_pinyin: bool, _generate_acronym: bool) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // ========== 1. 构建目标数据 ==========
    
    // 获取所有分组及其关键词
    let mut stmt = conn.prepare(
        "SELECT id, name FROM groups ORDER BY name"
    ).map_err(|e| e.to_string())?;
    
    let groups: Vec<(i32, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    
    // 构建目标 groups_keywords
    let mut target_groups_keywords: HashMap<String, Vec<String>> = HashMap::new();
    
    for (group_id, group_name) in &groups {
        // 获取该分组的所有关键词
        let mut keywords = Vec::new();
        
        // 从分组名解析关键词
        let name_keywords: Vec<String> = group_name.split(&[',', '，', '、', ' '][..])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        keywords.extend(name_keywords.clone());
        
        // 获取关联的关键词
        let mut kw_stmt = conn.prepare(
            "SELECT k.keyword FROM keywords k
             JOIN keyword_group_links kgl ON k.id = kgl.keyword_id
             WHERE kgl.group_id = ?"
        ).map_err(|e| e.to_string())?;
        
        let existing: Vec<String> = kw_stmt.query_map(params![group_id], |row| {
            Ok(row.get::<_, String>(0)?)
        }).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        
        for kw in existing {
            if !keywords.contains(&kw) {
                keywords.push(kw);
            }
        }
        
        if !keywords.is_empty() {
            target_groups_keywords.insert(group_name.clone(), keywords);
        }
    }
    
    // 构建目标 keywords_pinyin
    let mut target_keywords_pinyin: HashMap<String, KeywordPinyin> = HashMap::new();
    
    for keywords in target_groups_keywords.values() {
        for keyword in keywords {
            // 只处理包含中文的关键词
            let has_chinese = keyword.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
            if !has_chinese {
                continue;
            }
            
            // 查询数据库中的拼音
            let (stored_pinyin, stored_acronym): (Option<String>, Option<String>) = conn.query_row(
                "SELECT pinyin, acronym FROM keywords WHERE keyword = ?",
                params![keyword],
                |row| Ok((row.get(0)?, row.get(1)?))
            ).unwrap_or((None, None));
            
            let pinyin = stored_pinyin.unwrap_or_else(|| convert_to_pinyin(keyword));
            let abbr = stored_acronym.unwrap_or_else(|| convert_to_acronym(keyword));
            
            if !pinyin.is_empty() || !abbr.is_empty() {
                target_keywords_pinyin.insert(keyword.clone(), KeywordPinyin { pinyin, abbr });
            }
        }
    }
    
    // ========== 2. 读取现有 TOML 数据 ==========
    
    let keywords_file = PathBuf::from(&meme_dir).join("keywords.toml");
    
    // 读取现有数据（如果不存在则视为空）
    let (existing_groups_keywords, existing_keywords_pinyin): (HashMap<String, Vec<String>>, HashMap<String, KeywordPinyin>) = 
        if keywords_file.exists() {
            match load_keywords_from_file(meme_dir.clone()) {
                Ok(data) => (data.groups_keywords, data.keywords_pinyin),
                Err(_) => (HashMap::new(), HashMap::new()),
            }
        } else {
            (HashMap::new(), HashMap::new())
        };
    
    // ========== 3. 构建变更容器 ==========
    
    // groups_keywords 变更容器
    let mut groups_to_add: Vec<(String, Vec<String>)> = Vec::new();
    let mut groups_to_update: Vec<(String, Vec<String>)> = Vec::new();
    let mut groups_to_remove: Vec<String> = Vec::new();
    
    // 计算需要删除的分组
    for (group_name, _) in &existing_groups_keywords {
        if !target_groups_keywords.contains_key(group_name) {
            groups_to_remove.push(group_name.clone());
        }
    }
    
    // 计算需要新增或更新的分组
    for (group_name, keywords) in &target_groups_keywords {
        match existing_groups_keywords.get(group_name) {
            None => {
                // 新增
                groups_to_add.push((group_name.clone(), keywords.clone()));
            }
            Some(existing) => {
                // 检查是否需要更新
                if existing != keywords {
                    groups_to_update.push((group_name.clone(), keywords.clone()));
                }
            }
        }
    }
    
    // keywords_pinyin 变更容器
    let mut pinyin_to_add: Vec<(String, KeywordPinyin)> = Vec::new();
    let mut pinyin_to_update: Vec<(String, KeywordPinyin)> = Vec::new();
    let mut pinyin_to_remove: Vec<String> = Vec::new();
    
    // 收集所有目标关键词
    let all_target_keywords: Vec<&String> = target_groups_keywords.values()
        .flat_map(|v| v.iter())
        .collect();
    
    // 计算需要删除的拼音条目
    for (keyword, _) in &existing_keywords_pinyin {
        if !all_target_keywords.contains(&keyword) {
            pinyin_to_remove.push(keyword.clone());
        }
    }
    
    // 计算需要新增或更新的拼音
    for (keyword, pinyin_info) in &target_keywords_pinyin {
        match existing_keywords_pinyin.get(keyword) {
            None => {
                // 新增
                pinyin_to_add.push((keyword.clone(), pinyin_info.clone()));
            }
            Some(existing) => {
                // 检查是否需要更新
                if existing != pinyin_info {
                    pinyin_to_update.push((keyword.clone(), pinyin_info.clone()));
                }
            }
        }
    }
    
    // ========== 4. 判断是否需要更新 ==========
    
    let needs_update = !groups_to_add.is_empty() 
        || !groups_to_update.is_empty() 
        || !groups_to_remove.is_empty()
        || !pinyin_to_add.is_empty()
        || !pinyin_to_update.is_empty()
        || !pinyin_to_remove.is_empty();
    
    if !needs_update {
        return Ok(());
    }
    
    // ========== 5. 应用变更到 TOML 文档 ==========
    
    let mut doc = if keywords_file.exists() {
        let content = std::fs::read_to_string(&keywords_file)
            .map_err(|e| format!("Failed to read keywords.toml: {}", e))?;
        content.parse::<toml_edit::DocumentMut>()
            .map_err(|e| format!("Failed to parse TOML: {}", e))?
    } else {
        // 创建新文档
        let mut doc = toml_edit::DocumentMut::new();
        doc.insert("groups_keywords", toml_edit::table());
        doc.insert("keywords_pinyin", toml_edit::table());
        doc
    };
    
    // 确保两个表存在
    if !doc.contains_key("groups_keywords") {
        doc.insert("groups_keywords", toml_edit::table());
    }
    if !doc.contains_key("keywords_pinyin") {
        doc.insert("keywords_pinyin", toml_edit::table());
    }
    
    // 获取 groups_keywords 表
    let groups_table = doc["groups_keywords"].as_table_mut()
        .ok_or("Failed to get groups_keywords table")?;
    
    // 执行删除操作
    for group_name in groups_to_remove {
        groups_table.remove(&group_name);
    }
    
    // 执行更新操作
    for (group_name, keywords) in groups_to_update {
        let keywords_array = toml_edit::Array::from_iter(keywords.iter().cloned());
        groups_table[&group_name] = toml_edit::Item::Value(keywords_array.into());
    }
    
    // 执行新增操作
    for (group_name, keywords) in groups_to_add {
        let keywords_array = toml_edit::Array::from_iter(keywords.iter().cloned());
        groups_table[&group_name] = toml_edit::Item::Value(keywords_array.into());
    }
    
    // 获取 keywords_pinyin 表
    let pinyin_table = doc["keywords_pinyin"].as_table_mut()
        .ok_or("Failed to get keywords_pinyin table")?;
    
    // 执行删除操作
    for keyword in pinyin_to_remove {
        pinyin_table.remove(&keyword);
    }
    
    // 执行更新操作
    for (keyword, pinyin_info) in pinyin_to_update {
        let mut inline_table = toml_edit::InlineTable::new();
        inline_table.insert("pinyin", pinyin_info.pinyin.clone().into());
        inline_table.insert("abbr", pinyin_info.abbr.clone().into());
        pinyin_table[&keyword] = toml_edit::Item::Value(inline_table.into());
    }
    
    // 执行新增操作
    for (keyword, pinyin_info) in pinyin_to_add {
        let mut inline_table = toml_edit::InlineTable::new();
        inline_table.insert("pinyin", pinyin_info.pinyin.clone().into());
        inline_table.insert("abbr", pinyin_info.abbr.clone().into());
        pinyin_table[&keyword] = toml_edit::Item::Value(inline_table.into());
    }
    
    // ========== 6. 写入文件 ==========
    
    let content = if keywords_file.exists() {
        // 编辑现有文件，直接输出文档内容（toml_edit 会保留原有注释）
        doc.to_string()
    } else {
        // 创建新文件，添加注释头
        format!(
            "# Keywords Configuration\n# Auto-generated file\n# Format:\n# [groups_keywords] - 分组关键词映射\n# [keywords_pinyin] - 关键词拼音信息（仅中文需要）\n\n{}",
            doc.to_string()
        )
    };
    
    std::fs::write(&keywords_file, content)
        .map_err(|e| format!("Failed to write keywords file: {}", e))?;
    
    Ok(())
}

/// 解析并导入 keywords.toml 到数据库
#[tauri::command]
pub fn import_keywords_from_file(meme_dir: String) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let keywords_toml = load_keywords_from_file(meme_dir)?;
    
    // 导入 groups_keywords
    for (group_name, keywords) in &keywords_toml.groups_keywords {
        // 查找分组ID
        let group_id: Option<i32> = conn.query_row(
            "SELECT id FROM groups WHERE name = ?",
            params![group_name],
            |row| row.get(0)
        ).ok();
        
        if let Some(gid) = group_id {
            for keyword in keywords {
                // 插入或获取关键词ID
                let keyword_id: i32 = match conn.query_row(
                    "SELECT id FROM keywords WHERE keyword = ?",
                    params![keyword],
                    |row| row.get::<_, i32>(0)
                ) {
                    Ok(id) => id,
                    Err(_) => {
                        // 检查是否需要生成拼音
                        let has_chinese = keyword.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
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
                        
                        conn.execute(
                            "INSERT INTO keywords (keyword, pinyin, acronym) VALUES (?, ?, ?)",
                            params![keyword, pinyin, acronym],
                        ).map_err(|e| e.to_string())?;
                        
                        conn.last_insert_rowid() as i32
                    }
                };
                
                // 建立关键词-分组关联
                conn.execute(
                    "INSERT OR IGNORE INTO keyword_group_links (keyword_id, group_id) VALUES (?, ?)",
                    params![keyword_id, gid],
                ).map_err(|e| e.to_string())?;
            }
        }
    }
    
    // 导入 keywords_pinyin（更新现有关键词的拼音）
    for (keyword, pinyin_info) in &keywords_toml.keywords_pinyin {
        conn.execute(
            "UPDATE keywords SET pinyin = ?, acronym = ? WHERE keyword = ?",
            params![pinyin_info.pinyin, pinyin_info.abbr, keyword],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// TOML字符串转义
fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

/// 同步关键词到文件
/// 自动从配置读取 meme_dir 并生成关键词文件
#[tauri::command]
pub fn sync_keywords_to_file() -> Result<(), String> {
    // 从配置中获取 meme_dir
    let conn = init_db().map_err(|e| e.to_string())?;
    let meme_dir: String = conn.query_row(
        "SELECT meme_dir FROM config WHERE id = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    let config_row: (i32, i32) = conn.query_row(
        "SELECT pinyin_search, acronym_search FROM config WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).unwrap_or((0, 0));
    
    generate_keywords_file(meme_dir, config_row.0 != 0, config_row.1 != 0)
}

/// 根据分组名获取关联的所有关键词名称（跨所有模式）
#[tauri::command]
pub fn get_keywords_by_group_name(group_name: String) -> Result<Vec<String>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 查询所有同名分组关联的关键词名称
    let mut stmt = conn.prepare(
        "SELECT DISTINCT k.keyword
         FROM keywords k
         JOIN keyword_group_links kgl ON k.id = kgl.keyword_id
         JOIN groups g ON kgl.group_id = g.id
         WHERE g.name = ?
         ORDER BY k.keyword"
    ).map_err(|e| e.to_string())?;
    
    let keywords: Vec<String> = stmt.query_map(params![group_name], |row| {
        row.get::<_, String>(0)
    }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    Ok(keywords)
}

/// 从所有同名分组中移除与指定关键词的关联
#[tauri::command]
pub fn remove_keyword_from_group_name(keyword: String, group_name: String) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 查询关键词的 ID
    let keyword_id: i32 = conn.query_row(
        "SELECT id FROM keywords WHERE keyword = ?",
        params![keyword],
        |row| row.get(0)
    ).map_err(|_| format!("关键词 '{}' 不存在", keyword))?;
    
    // 找到所有同名分组的 ID
    let mut stmt = conn.prepare(
        "SELECT id FROM groups WHERE name = ?"
    ).map_err(|e| e.to_string())?;
    
    let group_ids: Vec<i32> = stmt.query_map(params![group_name], |row| {
        row.get::<_, i32>(0)
    }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    // 删除这些分组与关键词的关联
    for group_id in group_ids {
        conn.execute(
            "DELETE FROM keyword_group_links WHERE keyword_id = ? AND group_id = ?",
            params![keyword_id, group_id],
        ).map_err(|e| e.to_string())?;
    }
    
    // 同步更新 keywords.toml
    sync_keywords_to_file()?;
    
    Ok(())
}

/// 添加关键词到所有同名分组
/// 如果关键词不存在则新建，存在则直接使用
#[tauri::command]
pub fn add_keyword_to_group_name(keyword: String, group_name: String) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 检查关键词是否已存在
    let existing_id: Option<i32> = conn.query_row(
        "SELECT id FROM keywords WHERE keyword = ?",
        params![keyword],
        |row| row.get(0)
    ).ok();
    
    let keyword_id = match existing_id {
        Some(id) => id,
        None => {
            // 新建关键词
            let has_chinese = keyword.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
            let pinyin = if has_chinese {
                Some(convert_to_pinyin(&keyword))
            } else {
                None
            };
            let acronym = if has_chinese {
                Some(convert_to_acronym(&keyword))
            } else {
                None
            };
            
            conn.execute(
                "INSERT INTO keywords (keyword, pinyin, acronym) VALUES (?, ?, ?)",
                params![keyword, pinyin, acronym],
            ).map_err(|e| e.to_string())?;
            
            conn.last_insert_rowid() as i32
        }
    };
    
    // 找到所有同名分组的 ID
    let mut stmt = conn.prepare(
        "SELECT id FROM groups WHERE name = ?"
    ).map_err(|e| e.to_string())?;
    
    let group_ids: Vec<i32> = stmt.query_map(params![group_name], |row| {
        row.get::<_, i32>(0)
    }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    // 建立这些分组与关键词的关联
    for group_id in group_ids {
        conn.execute(
            "INSERT OR IGNORE INTO keyword_group_links (keyword_id, group_id) VALUES (?, ?)",
            params![keyword_id, group_id],
        ).map_err(|e| e.to_string())?;
    }
    
    // 同步更新 keywords.toml
    sync_keywords_to_file()?;
    
    Ok(())
}
