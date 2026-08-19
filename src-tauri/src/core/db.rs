use crate::core::error::AppError;
use rusqlite::{Connection, params};
use std::path::PathBuf;


/// 获取应用数据目录
pub fn get_app_data_dir() -> PathBuf {
    // Android 使用应用私有目录
    #[cfg(target_os = "android")]
    {
        // Android 上返回 /data/data/com.v.meme/files
        PathBuf::from("/data/data/com.v.meme/files")
    }
    
    // 桌面端使用标准目录
    #[cfg(not(target_os = "android"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".local/share/meme")
    }
}

/// 获取数据库路径
pub fn get_db_path() -> PathBuf {
    get_app_data_dir().join("meme.db")
}

/// 初始化数据库连接并创建表结构
pub fn init_db() -> Result<Connection, AppError> {
    let db_path = get_db_path();
    
    // 确保目录存在（静默创建，不打印日志）
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let conn = Connection::open(&db_path)?;
    
    // 启用外键支持
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    
    // 创建配置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            meme_dir TEXT NOT NULL DEFAULT '',
            color_mode TEXT NOT NULL DEFAULT 'system',
            last_mode INTEGER DEFAULT 1,
            last_group INTEGER DEFAULT 1,
            share_app TEXT DEFAULT '',
            grid_size INTEGER DEFAULT 4,
            pinyin_search INTEGER DEFAULT 1,
            acronym_search INTEGER DEFAULT 1,
            theme_style TEXT NOT NULL DEFAULT 'modern',
            global_floating_window INTEGER DEFAULT 1
        )",
        [],
    )?;

    // 插入默认配置（如果不存在）
    let default_meme_dir = {
        #[cfg(target_os = "android")]
        { "/storage/emulated/0/meme".to_string() }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        { format!("{}/meme", std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())) }
        #[cfg(target_os = "windows")]
        { format!("{}\\meme", std::env::var("USERPROFILE").unwrap_or_else(|_| "C:".into())) }
        #[cfg(not(any(target_os = "android", target_os = "linux", target_os = "macos", target_os = "windows")))]
        { String::new() }
    };
    conn.execute(
        "INSERT OR IGNORE INTO config (id, meme_dir) VALUES (1, ?1)",
        params![default_meme_dir],
    )?;
    
    // 创建模式表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS modes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0,
            folder_path TEXT NOT NULL DEFAULT ''
        )",
        [],
    )?;
    
    // 创建分组表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            folder_path TEXT NOT NULL,
            share_count INTEGER DEFAULT 0,
            mode_id INTEGER NOT NULL,
            FOREIGN KEY (mode_id) REFERENCES modes(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    // 创建关键词表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS keywords (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            keyword TEXT NOT NULL UNIQUE,
            pinyin TEXT,
            acronym TEXT
        )",
        [],
    )?;
    
    // 创建图片表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            image_path TEXT NOT NULL,
            thumbnail_path TEXT,
            share_count INTEGER DEFAULT 0,
            group_id INTEGER NOT NULL,
            mode_id INTEGER NOT NULL,
            sticker_data BLOB,
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
            FOREIGN KEY (mode_id) REFERENCES modes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 创建关键词-分组关联表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS keyword_group_links (
            keyword_id INTEGER NOT NULL,
            group_id INTEGER NOT NULL,
            PRIMARY KEY (keyword_id, group_id),
            FOREIGN KEY (keyword_id) REFERENCES keywords(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    // 创建索引以优化查询性能
    conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_groups_mode_share ON groups(mode_id, share_count DESC);
        CREATE INDEX IF NOT EXISTS idx_images_group_share ON images(group_id, share_count DESC);
        CREATE INDEX IF NOT EXISTS idx_images_mode ON images(mode_id);
        CREATE INDEX IF NOT EXISTS idx_keywords_keyword ON keywords(keyword);
        CREATE INDEX IF NOT EXISTS idx_keyword_group_links_keyword ON keyword_group_links(keyword_id);
        CREATE INDEX IF NOT EXISTS idx_keyword_group_links_group ON keyword_group_links(group_id);
        CREATE INDEX IF NOT EXISTS idx_keywords_pinyin ON keywords(pinyin);
        CREATE INDEX IF NOT EXISTS idx_keywords_acronym ON keywords(acronym);
    ")?;

    Ok(conn)
}
