use rusqlite::params;
use crate::{db::init_db, models::CustomShareApp, error::AppError};

#[tauri::command]
pub fn get_custom_share_apps() -> Result<Vec<CustomShareApp>, AppError> {
    let conn = init_db()?;
    
    // 确保表存在
    conn.execute(
        "CREATE TABLE IF NOT EXISTS custom_share_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_name TEXT NOT NULL UNIQUE,
            app_name TEXT
        )",
        [],
    )?;

    let mut stmt = conn.prepare("SELECT id, package_name, app_name FROM custom_share_apps ORDER BY id")?;
    let apps = stmt.query_map([], |row| {
        Ok(CustomShareApp {
            id: row.get(0)?,
            package_name: row.get(1)?,
            app_name: row.get(2)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(apps)
}

#[tauri::command]
pub fn add_custom_share_app(package_name: String, app_name: Option<String>) -> Result<(), AppError> {
    let conn = init_db()?;
    
    conn.execute(
        "INSERT INTO custom_share_apps (package_name, app_name) VALUES (?, ?) ON CONFLICT(package_name) DO UPDATE SET app_name = excluded.app_name",
        params![package_name, app_name],
    )?;

    Ok(())
}

#[tauri::command]
pub fn remove_custom_share_app(id: i32) -> Result<(), AppError> {
    let conn = init_db()?;
    
    conn.execute(
        "DELETE FROM custom_share_apps WHERE id = ?",
        params![id],
    )?;

    Ok(())
}
