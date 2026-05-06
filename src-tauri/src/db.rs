use rusqlite::{Connection, Result};
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
pub fn init_db() -> Result<Connection> {
    let db_path = get_db_path();
    
    // 确保目录存在（静默创建，不打印日志）
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some(e.to_string())
            )
        })?;
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
            pinyin_search INTEGER DEFAULT 0,
            acronym_search INTEGER DEFAULT 0
        )",
        [],
    )?;
    
    // 检查并添加 pinyin_search 列（兼容旧数据库）
    let has_pinyin_search: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('config') WHERE name='pinyin_search'",
        [],
        |row| row.get::<_, i32>(0)
    ).unwrap_or(0) > 0;
    
    if !has_pinyin_search {
        conn.execute("ALTER TABLE config ADD COLUMN pinyin_search INTEGER DEFAULT 0", [])?;
    }
    
    // 检查并添加 acronym_search 列（兼容旧数据库）
    let has_acronym_search: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('config') WHERE name='acronym_search'",
        [],
        |row| row.get::<_, i32>(0)
    ).unwrap_or(0) > 0;
    
    if !has_acronym_search {
        conn.execute("ALTER TABLE config ADD COLUMN acronym_search INTEGER DEFAULT 0", [])?;
    }
    
    // 检查并添加 theme_style 列（兼容旧数据库）
    let has_theme_style: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('config') WHERE name='theme_style'",
        [],
        |row| row.get::<_, i32>(0)
    ).unwrap_or(0) > 0;
    
    if !has_theme_style {
        conn.execute("ALTER TABLE config ADD COLUMN theme_style TEXT DEFAULT 'modern'", [])?;
    }
    
    // 插入默认配置（如果不存在）
    conn.execute(
        "INSERT OR IGNORE INTO config (id) VALUES (1)",
        [],
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
    
    // 检查并添加 folder_path 列(兼容旧数据库)
    let has_folder_path: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('modes') WHERE name='folder_path'",
        [],
        |row| row.get::<_, i32>(0)
    ).unwrap_or(0) > 0;
    
    if !has_folder_path {
        conn.execute("ALTER TABLE modes ADD COLUMN folder_path TEXT NOT NULL DEFAULT ''", [])?;
    }
    
    // 检查并修复 images 表的 thumbnail_path 字段（兼容旧数据库）
    let thumb_nullable: bool = conn.query_row(
        "SELECT \"notnull\" FROM pragma_table_info('images') WHERE name='thumbnail_path'",
        [],
        |row| row.get::<_, i32>(0)
    ).unwrap_or(0) == 0; // notnull=0 表示允许 NULL
    
    if !thumb_nullable {
        // SQLite 不支持 ALTER COLUMN，需要重建表
        eprintln!("Fixing images table thumbnail_path column to allow NULL...");
        
        // 检查旧表是否有 created_at 列
        let has_created_at: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('images') WHERE name='created_at'",
            [],
            |row| row.get::<_, i32>(0)
        ).unwrap_or(0) > 0;
        
        if has_created_at {
            conn.execute_batch("
                BEGIN TRANSACTION;
                CREATE TABLE IF NOT EXISTS images_new (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    image_path TEXT NOT NULL,
                    thumbnail_path TEXT,
                    share_count INTEGER DEFAULT 0,
                    group_id INTEGER NOT NULL,
                    mode_id INTEGER NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
                    FOREIGN KEY (mode_id) REFERENCES modes(id) ON DELETE CASCADE
                );
                INSERT INTO images_new (id, image_path, thumbnail_path, share_count, group_id, mode_id, created_at) 
                SELECT id, image_path, thumbnail_path, share_count, group_id, mode_id, created_at FROM images;
                DROP TABLE images;
                ALTER TABLE images_new RENAME TO images;
                CREATE INDEX IF NOT EXISTS idx_images_group_share ON images(group_id, share_count DESC);
                CREATE INDEX IF NOT EXISTS idx_images_mode ON images(mode_id);
                COMMIT;
            ")?;
        } else {
            conn.execute_batch("
                BEGIN TRANSACTION;
                CREATE TABLE IF NOT EXISTS images_new (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    image_path TEXT NOT NULL,
                    thumbnail_path TEXT,
                    share_count INTEGER DEFAULT 0,
                    group_id INTEGER NOT NULL,
                    mode_id INTEGER NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
                    FOREIGN KEY (mode_id) REFERENCES modes(id) ON DELETE CASCADE
                );
                INSERT INTO images_new (id, image_path, thumbnail_path, share_count, group_id, mode_id) 
                SELECT id, image_path, thumbnail_path, share_count, group_id, mode_id FROM images;
                DROP TABLE images;
                ALTER TABLE images_new RENAME TO images;
                CREATE INDEX IF NOT EXISTS idx_images_group_share ON images(group_id, share_count DESC);
                CREATE INDEX IF NOT EXISTS idx_images_mode ON images(mode_id);
                COMMIT;
            ")?;
        }
        
        eprintln!("Fixed images table successfully");
    }
    
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
    
    // 迁移：如果 keywords 表有 abbreviation 列但没有 acronym 列，重命名
    let has_abbreviation: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('keywords') WHERE name='abbreviation'",
        [],
        |row| row.get::<_, i32>(0)
    ).unwrap_or(0) > 0;
    
    let has_acronym: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('keywords') WHERE name='acronym'",
        [],
        |row| row.get::<_, i32>(0)
    ).unwrap_or(0) > 0;
    
    if has_abbreviation && !has_acronym {
        eprintln!("Migrating keywords table: renaming abbreviation to acronym");
        conn.execute("ALTER TABLE keywords RENAME COLUMN abbreviation TO acronym", [])?;
    } else if !has_acronym {
        // 如果两列都没有，添加 acronym 列
        eprintln!("Adding acronym column to keywords table");
        conn.execute("ALTER TABLE keywords ADD COLUMN acronym TEXT", [])?;
    }
    
    // 创建图片表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            image_path TEXT NOT NULL,
            thumbnail_path TEXT,
            share_count INTEGER DEFAULT 0,
            group_id INTEGER NOT NULL,
            mode_id INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
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
