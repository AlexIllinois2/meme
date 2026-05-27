use rusqlite::params;
use std::path::PathBuf;
use std::collections::HashMap;
use pinyin::ToPinyin;
use crate::db::init_db;
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

/// 生成关键词配置文件 - 只扫描文件系统，不依赖数据库
/// 根据模式/分组文件夹名自动生成关键词
#[tauri::command]
pub fn generate_keywords_file(meme_dir: String, _generate_pinyin: bool, _generate_acronym: bool) -> Result<(), String> {
    use std::fs;
    
    let meme_path = PathBuf::from(&meme_dir);
    
    if !meme_path.exists() {
        return Err(format!("表情包目录不存在: {}", meme_dir));
    }
    
    // ========== 1. 扫描文件系统构建目标数据 ==========
    
    let mut target_groups_keywords: HashMap<String, Vec<String>> = HashMap::new();
    
    // 遍历所有模式目录
    let entries = fs::read_dir(&meme_path)
        .map_err(|e| format!("无法读取目录: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        
        // 只处理目录（模式）
        if !path.is_dir() {
            continue;
        }
        
        let mode_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        // 遍历模式下的分组目录
        let group_entries = fs::read_dir(&path)
            .map_err(|e| format!("无法读取模式目录 {}: {}", mode_name, e))?;
        
        for group_entry in group_entries {
            let group_entry = group_entry.map_err(|e| format!("读取分组目录失败: {}", e))?;
            let group_path = group_entry.path();
            
            // 只处理目录（分组）
            if !group_path.is_dir() {
                continue;
            }
            
            let group_name = group_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            if group_name.is_empty() {
                continue;
            }
            
            // 从分组名解析关键词（支持逗号、顿号、空格分隔）
            let keywords: Vec<String> = group_name.split(&[',', '，', '、', ' '][..])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            if !keywords.is_empty() {
                target_groups_keywords.insert(group_name.clone(), keywords);
            }
        }
    }
    
    // ========== 2. 构建拼音数据 ==========
    
    let mut target_keywords_pinyin: HashMap<String, KeywordPinyin> = HashMap::new();
    
    for keywords in target_groups_keywords.values() {
        for keyword in keywords {
            // 只处理包含中文的关键词
            let has_chinese = keyword.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
            if !has_chinese {
                continue;
            }
            
            let pinyin = convert_to_pinyin(keyword);
            let abbr = convert_to_acronym(keyword);
            
            if !pinyin.is_empty() || !abbr.is_empty() {
                target_keywords_pinyin.insert(keyword.clone(), KeywordPinyin { pinyin, abbr });
            }
        }
    }
    
    // ========== 3. 读取现有 TOML 数据 ==========
    
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
    
    // ========== 4. 构建变更容器 ==========
    
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
    
    // ========== 5. 判断是否需要更新 ==========
    
    let needs_update = !groups_to_add.is_empty() 
        || !groups_to_update.is_empty() 
        || !groups_to_remove.is_empty()
        || !pinyin_to_add.is_empty()
        || !pinyin_to_update.is_empty()
        || !pinyin_to_remove.is_empty();
    
    if !needs_update {
        log::info!("关键词文件无需更新");
        return Ok(());
    }
    
    // ========== 6. 应用变更到 TOML 文档 ==========
    
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
    
    // ========== 7. 写入文件 ==========
    
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

/// 内部函数：直接写入 keywords.toml（全量覆盖，不做 diff-merge）
pub(crate) fn write_keywords_toml_snapshot(
    meme_dir: &str,
    groups_keywords: &HashMap<String, Vec<String>>,
    keywords_pinyin: &HashMap<String, KeywordPinyin>,
) -> Result<(), String> {
    let keywords_file = PathBuf::from(meme_dir).join("keywords.toml");

    let mut doc = toml_edit::DocumentMut::new();
    doc.insert("groups_keywords", toml_edit::table());
    doc.insert("keywords_pinyin", toml_edit::table());

    if let Some(groups_table) = doc["groups_keywords"].as_table_mut() {
        let mut sorted_groups: Vec<_> = groups_keywords.iter().collect();
        sorted_groups.sort_by_key(|(k, _)| *k);
        for (group_name, keywords) in sorted_groups {
            let mut sorted_kw = keywords.clone();
            sorted_kw.sort();
            groups_table[group_name] =
                toml_edit::Item::Value(toml_edit::Array::from_iter(sorted_kw).into());
        }
    }

    if let Some(pinyin_table) = doc["keywords_pinyin"].as_table_mut() {
        let mut sorted_pinyin: Vec<_> = keywords_pinyin.iter().collect();
        sorted_pinyin.sort_by_key(|(k, _)| *k);
        for (keyword, info) in sorted_pinyin {
            let mut t = toml_edit::InlineTable::new();
            t.insert("pinyin", info.pinyin.clone().into());
            t.insert("abbr", info.abbr.clone().into());
            pinyin_table[keyword] = toml_edit::Item::Value(t.into());
        }
    }

    let content = format!(
        "# Keywords Configuration\n# Auto-generated file\n# Format:\n# [groups_keywords] - 分组关键词映射\n# [keywords_pinyin] - 关键词拼音信息（仅中文需要）\n\n{}",
        doc.to_string()
    );

    std::fs::write(&keywords_file, content)
        .map_err(|e| format!("Failed to write keywords file: {}", e))?;

    Ok(())
}
