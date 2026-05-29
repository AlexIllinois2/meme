use rusqlite::params;
use crate::core::{models::Config, error::AppError};
use crate::core::db_state::DbState;


#[tauri::command]
pub fn get_config(state: tauri::State<'_, DbState>) -> Result<Config, AppError> {
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    
    let mut stmt = conn.prepare(
        "SELECT meme_dir, color_mode, theme_style, last_mode, last_group, share_app, grid_size, pinyin_search, acronym_search, global_floating_window FROM config WHERE id = 1"
    )?;
    
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
            global_floating_window: row.get::<_, i32>(9).unwrap_or(0) != 0,
        })
    }) {
        Ok(config) => {
            log::info!("Config loaded: meme_dir={}", config.meme_dir);
            Ok(config)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            log::info!("No config found, creating default config");
            conn.execute(
                "INSERT INTO config (id, meme_dir, color_mode, theme_style, last_mode, last_group, share_app, grid_size, pinyin_search, acronym_search, global_floating_window) VALUES (1, '', 'system', 'modern', 1, 1, '', 4, 0, 0, 1)",
                [],
            )?;
            Ok(Config {
                meme_dir: String::new(),
                color_mode: "system".to_string(),
                theme_style: "modern".to_string(),
                last_mode: 1, last_group: 1,
                share_app: String::new(),
                grid_size: 4,
                pinyin_search: true, acronym_search: true,
                global_floating_window: true,
            })
        }
        Err(e) => {
            log::error!("Error loading config: {}", e);
            Err(e.into())
        }
    }
}

#[tauri::command]
pub fn update_config(state: tauri::State<'_, DbState>, config: Config) -> Result<(), AppError> {
    log::info!("Updating config: meme_dir={}, color_mode={}, theme_style={}",
        config.meme_dir, config.color_mode, config.theme_style);
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    
    let result = conn.execute(
        "UPDATE config SET meme_dir = ?, color_mode = ?, theme_style = ?, last_mode = ?, last_group = ?, share_app = ?, grid_size = ?, pinyin_search = ?, acronym_search = ?, global_floating_window = ? WHERE id = 1",
        params![config.meme_dir, config.color_mode, config.theme_style, config.last_mode, config.last_group, config.share_app, config.grid_size, config.pinyin_search as i32, config.acronym_search as i32, config.global_floating_window as i32],
    );
    
    match result {
        Ok(rows) => {
            log::info!("Config updated, {} rows affected", rows);
            if rows == 0 {
                log::warn!("Warning: No config row found, inserting new one");
                conn.execute(
                    "INSERT OR REPLACE INTO config (id, meme_dir, color_mode, theme_style, last_mode, last_group, share_app, grid_size, pinyin_search, acronym_search, global_floating_window) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![config.meme_dir, config.color_mode, config.theme_style, config.last_mode, config.last_group, config.share_app, config.grid_size, config.pinyin_search as i32, config.acronym_search as i32, config.global_floating_window as i32],
                )?;
            }
            Ok(())
        }
        Err(e) => {
            log::error!("Database error updating config: {}", e);
            Err(e.into())
        }
    }
}