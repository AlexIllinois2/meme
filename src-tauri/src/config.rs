use rusqlite::params;
use crate::{db::init_db, models::Config};


#[tauri::command]
pub fn get_config() -> Result<Config, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT meme_dir, color_mode, theme_style, last_mode, last_group, share_app, grid_size, pinyin_search, acronym_search FROM config WHERE id = 1"
    ).map_err(|e| e.to_string())?;
    
    match stmt.query_row([], |row| {
        Ok(Config {
            meme_dir: row.get(0)?,
            color_mode: row.get(1)?,
            theme_style: row.get(2)?,
            last_mode: row.get(3)?,
            last_group: row.get(4)?,
            share_app: row.get(5)?,
            grid_size: row.get(6)?,
            pinyin_search: row.get::<_, i32>(7)? != 0,
            acronym_search: row.get::<_, i32>(8)? != 0,
        })
    }) {
        Ok(config) => {
            eprintln!("Config loaded: meme_dir={}", config.meme_dir);
            Ok(config)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            eprintln!("No config found, creating default config");
            // 如果配置不存在，创建默认配置
            conn.execute(
                "INSERT INTO config (id, meme_dir, color_mode, theme_style, last_mode, last_group, share_app, grid_size, pinyin_search, acronym_search) VALUES (1, '', 'system', 'modern', 1, 1, '', 4, 0, 0)",
                [],
            ).map_err(|e| e.to_string())?;
            
            Ok(Config {
                meme_dir: String::new(),
                color_mode: "system".to_string(),
                theme_style: "modern".to_string(),
                last_mode: 1,
                last_group: 1,
                share_app: String::new(),
                grid_size: 4,
                pinyin_search: false,
                acronym_search: false,
            })
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn update_config(config: Config) -> Result<(), String> {
    eprintln!("Updating config: meme_dir={}, color_mode={}, theme_style={}", config.meme_dir, config.color_mode, config.theme_style);
    let conn = init_db().map_err(|e| {
        eprintln!("Failed to init db for config update: {}", e);
        e.to_string()
    })?;
    
    let result = conn.execute(
        "UPDATE config SET meme_dir = ?, color_mode = ?, theme_style = ?, last_mode = ?, last_group = ?, share_app = ?, grid_size = ?, pinyin_search = ?, acronym_search = ? WHERE id = 1",
        params![config.meme_dir, config.color_mode, config.theme_style, config.last_mode, config.last_group, config.share_app, config.grid_size, config.pinyin_search as i32, config.acronym_search as i32],
    );
    
    match result {
        Ok(rows) => {
            eprintln!("Config updated, {} rows affected", rows);
            if rows == 0 {
                eprintln!("Warning: No config row found, inserting new one");
                // 如果不存在则插入
                conn.execute(
                    "INSERT OR REPLACE INTO config (id, meme_dir, color_mode, theme_style, last_mode, last_group, share_app, grid_size, pinyin_search, acronym_search) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![config.meme_dir, config.color_mode, config.theme_style, config.last_mode, config.last_group, config.share_app, config.grid_size, config.pinyin_search as i32, config.acronym_search as i32],
                ).map_err(|e| {
                    eprintln!("Failed to insert config: {}", e);
                    e.to_string()
                })?;
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Database error updating config: {}", e);
            Err(e.to_string())
        }
    }
}