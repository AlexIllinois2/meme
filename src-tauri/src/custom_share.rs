use rusqlite::params;
use crate::{db::init_db, models::CustomShareApp};

#[tauri::command]
pub fn get_custom_share_apps() -> Result<Vec<CustomShareApp>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 确保表存在
    conn.execute(
        "CREATE TABLE IF NOT EXISTS custom_share_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            package_name TEXT NOT NULL UNIQUE,
            app_name TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, package_name, app_name FROM custom_share_apps ORDER BY id").map_err(|e| e.to_string())?;
    let apps = stmt.query_map([], |row| {
        Ok(CustomShareApp {
            id: row.get(0)?,
            package_name: row.get(1)?,
            app_name: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(apps)
}

#[tauri::command]
pub fn add_custom_share_app(package_name: String, app_name: Option<String>) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO custom_share_apps (package_name, app_name) VALUES (?, ?) ON CONFLICT(package_name) DO UPDATE SET app_name = excluded.app_name",
        params![package_name, app_name],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_custom_share_app(id: i32) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "DELETE FROM custom_share_apps WHERE id = ?",
        params![id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}
