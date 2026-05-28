//! 图片处理模块
//!
//! 提供图片上传、复制、删除、搜索、索引刷新等功能
//!
//! 剪贴板相关功能已提取到 [`clipboard`] 模块，通过 `pub use` 重新导出。

use rusqlite::params;
use std::path::PathBuf;
use crate::core::{db_state::DbState, models::Image, meme_fs};
use crate::core::error::AppError;

#[cfg(not(target_os = "android"))]
use clipboard_rs::{Clipboard, ClipboardContext};

// 重新导出剪贴板模块的所有公共项，保持向后兼容

/// 检测图片格式

/// 获取分组下的所有图片
#[tauri::command]
pub fn get_images_by_group(state: tauri::State<'_, DbState>, group_id: i32) -> Result<Vec<Image>, AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	let meme_dir: String = conn.query_row(
		"SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
	).unwrap_or_default();
	let mut stmt = conn.prepare(
		"SELECT id, image_path, thumbnail_path, share_count, group_id, mode_id FROM images WHERE group_id = ? ORDER BY share_count DESC, id ASC"
	)?;
	let images = stmt.query_map(params![group_id], |row| {
		Ok(Image {
			id: row.get(0)?,
			image_path: meme_fs::resolve_meme_path(&meme_dir, &row.get::<_,String>(1)?).to_string_lossy().to_string(),
			thumbnail_path: row.get(2)?,
			share_count: row.get(3)?,
			group_id: row.get(4)?,
			mode_id: row.get(5)?,
		})
	})?.collect::<Result<Vec<_>,_>>()?;
	Ok(images)
}

/// 搜索图片
#[tauri::command]
pub fn search_images(state: tauri::State<'_, DbState>, keyword: String, pinyin: bool, acronym: bool) -> Result<Vec<Image>, AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	let meme_dir: String = conn.query_row(
		"SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
	).unwrap_or_default();
	let search_pattern = format!("%{}%", keyword);
	let query = if pinyin && acronym {
		"SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id FROM images i JOIN keyword_group_links kgl ON i.group_id = kgl.group_id JOIN keywords k ON kgl.keyword_id = k.id WHERE k.keyword LIKE ?1 OR k.pinyin LIKE ?1 OR k.acronym LIKE ?1 ORDER BY i.share_count DESC"
	} else if pinyin {
		"SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id FROM images i JOIN keyword_group_links kgl ON i.group_id = kgl.group_id JOIN keywords k ON kgl.keyword_id = k.id WHERE k.keyword LIKE ?1 OR k.pinyin LIKE ?1 ORDER BY i.share_count DESC"
	} else if acronym {
		"SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id FROM images i JOIN keyword_group_links kgl ON i.group_id = kgl.group_id JOIN keywords k ON kgl.keyword_id = k.id WHERE k.keyword LIKE ?1 OR k.acronym LIKE ?1 ORDER BY i.share_count DESC"
	} else {
		"SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id FROM images i JOIN keyword_group_links kgl ON i.group_id = kgl.group_id JOIN keywords k ON kgl.keyword_id = k.id WHERE k.keyword LIKE ?1 ORDER BY i.share_count DESC"
	};
	let mut stmt = conn.prepare(query)?;
	let images = stmt.query_map(params![search_pattern], |row| {
		Ok(Image {
			id: row.get(0)?,
			image_path: meme_fs::resolve_meme_path(&meme_dir, &row.get::<_,String>(1)?).to_string_lossy().to_string(),
			thumbnail_path: row.get(2)?,
			share_count: row.get(3)?,
			group_id: row.get(4)?,
			mode_id: row.get(5)?,
		})
	})?.collect::<Result<Vec<_>,_>>()?;
	Ok(images)
}

/// 内部函数：增加分享次数
fn increment_share_count_internal(
	conn: &rusqlite::Connection, 
	image_id: i32, 
	group_id: Option<i32>
) -> Result<(), AppError> {
	conn.execute(
		"UPDATE images SET share_count = share_count + 1 WHERE id = ?",
		params![image_id],
	)?;
	
	if let Some(gid) = group_id {
		conn.execute(
			"UPDATE groups SET share_count = share_count + 1 WHERE id = ?",
			params![gid],
		)?;
	} else {
		let g_id: i32 = conn.query_row(
			"SELECT group_id FROM images WHERE id = ?",
			params![image_id],
			|row| row.get(0)
		)?;
		
		conn.execute(
			"UPDATE groups SET share_count = share_count + 1 WHERE id = ?",
			params![g_id],
		)?;
	}
	
	Ok(())
}

/// 分享图片
#[tauri::command]
pub fn share_image(state: tauri::State<'_, DbState>, image_id: i32) -> Result<(), AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	increment_share_count_internal(&conn, image_id, None)
}

/// 复制图片到剪贴板（桌面端）
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn copy_image(state: tauri::State<'_, DbState>, image_id: i32) -> Result<(), AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	let raw_path: String = conn.query_row(
		"SELECT image_path FROM images WHERE id = ?", params![image_id], |row| row.get(0)
	)?;
	let meme_dir: String = conn.query_row(
		"SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
	).unwrap_or_default();
	let image_path = meme_fs::resolve_meme_path(&meme_dir, &raw_path).to_string_lossy().to_string();
	increment_share_count_internal(&conn, image_id, None)?;
	let ctx = ClipboardContext::new().map_err(|e| AppError(format!("Failed to create clipboard context: {}", e)))?;
	let abs_path = std::fs::canonicalize(&image_path).map_err(|e| AppError(format!("Failed to get absolute path: {}", e)))?;
	ctx.set_files(vec![abs_path.to_string_lossy().to_string()]).map_err(|e| AppError(format!("Failed to copy file to clipboard: {}", e)))?;
	Ok(())
}

/// 复制图片到剪贴板（Android 端）
#[cfg(target_os = "android")]
#[tauri::command]
pub fn copy_image(state: tauri::State<'_, DbState>, image_id: i32) -> Result<(), AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	increment_share_count_internal(&conn, image_id, None)?;
	Err("图片复制到剪贴板功能在 Android 端暂不支持".into())
}

/// 增加分享次数
#[tauri::command]
pub fn increment_share_count(state: tauri::State<'_, DbState>, image_id: i32, group_id: Option<i32>) -> Result<(), AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	increment_share_count_internal(&conn, image_id, group_id)
}

/// 批量复制图片
#[tauri::command]
pub fn copy_images(state: tauri::State<'_, DbState>, image_ids: Vec<i32>) -> Result<(), AppError> {
	if image_ids.is_empty() {
		return Ok(());
	}
	
	let mut errors: Vec<String> = Vec::new();
	for image_id in &image_ids {
		let result = (|| -> Result<(), AppError> {
			let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
			let raw_path: String = conn.query_row(
				"SELECT image_path FROM images WHERE id = ?", params![image_id], |row| row.get(0)
			)?;
			let meme_dir: String = conn.query_row(
				"SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
			).unwrap_or_default();
			let image_path = meme_fs::resolve_meme_path(&meme_dir, &raw_path).to_string_lossy().to_string();
			increment_share_count_internal(&conn, *image_id, None)?;
			#[cfg(not(target_os = "android"))]
			{
				let ctx = ClipboardContext::new().map_err(|e| AppError(format!("Failed to create clipboard context: {}", e)))?;
				let abs_path = std::fs::canonicalize(&image_path).map_err(|e| AppError(format!("Failed to get absolute path: {}", e)))?;
				ctx.set_files(vec![abs_path.to_string_lossy().to_string()]).map_err(|e| AppError(format!("Failed to copy file to clipboard: {}", e)))?;
			}
			Ok(())
		})();
		if let Err(e) = result {
			log::warn!("复制图片 {} 失败: {}", image_id, e);
			errors.push(e.to_string());
		}
	}
	
	if errors.len() == image_ids.len() {
		return Err(format!("所有 {} 张图片复制失败: {}", errors.len(), errors.join("; ")).into());
	}
	
	Ok(())
}

/// 删除图片
#[tauri::command]
pub fn delete_images(state: tauri::State<'_, DbState>, image_ids: Vec<i32>) -> Result<(), AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	let meme_dir: String = conn.query_row(
		"SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
	).unwrap_or_default();
	for image_id in image_ids {
		if let Ok((raw_path, thumb_path)) = conn.query_row(
			"SELECT image_path, thumbnail_path FROM images WHERE id = ?",
			params![image_id],
			|row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
		) {
			let image_path = meme_fs::resolve_meme_path(&meme_dir, &raw_path);
			if image_path.exists() {
				std::fs::remove_file(&image_path)
					.map_err(|e| AppError(format!("删除图片文件失败 {}: {}", image_path.display(), e)))?;
			}
			if let Some(thumb) = thumb_path {
				let thumb_path = std::path::Path::new(&thumb);
				if thumb_path.exists() {
					std::fs::remove_file(thumb_path)
						.map_err(|e| AppError(format!("删除缩略图失败 {}: {}", thumb, e)))?;
				}
			}
		}
		conn.execute("DELETE FROM images WHERE id = ?", params![image_id])?;
	}
	Ok(())
}

/// 移动图片到其他分组
#[tauri::command]
pub fn move_images(state: tauri::State<'_, DbState>, image_ids: Vec<i32>, target_group_id: i32) -> Result<(), AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	
	let target_mode_id: i32 = conn.query_row(
		"SELECT mode_id FROM groups WHERE id = ?",
		params![target_group_id],
		|row| row.get(0)
	)?;
	
	for image_id in image_ids {
		conn.execute(
			"UPDATE images SET group_id = ?, mode_id = ? WHERE id = ?",
			params![target_group_id, target_mode_id, image_id]
		)?;
	}
	
	Ok(())
}

/// Android 端上传图片（接收 Base64 编码的图片数据）
/// 
/// 前端读取 content:// URI 的数据并转为 Base64 传递给后端
/// 获取图片完整路径
#[tauri::command]
pub fn get_image_full_path(state: tauri::State<'_, DbState>, image_id: i32) -> Result<String, AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	let raw_path: String = conn.query_row(
		"SELECT image_path FROM images WHERE id = ?", params![image_id], |row| row.get(0)
	)?;
	let meme_dir: String = conn.query_row(
		"SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
	).unwrap_or_default();
	Ok(meme_fs::resolve_meme_path(&meme_dir, &raw_path).to_string_lossy().to_string())
}

/// 分享图片到应用
#[tauri::command]
pub fn share_image_to_app<R: tauri::Runtime>(
	state: tauri::State<'_, DbState>,
	_app: tauri::AppHandle<R>,
	image_id: i32,
	_target_app: String,
) -> Result<(), AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	let raw_path: String = conn.query_row(
		"SELECT image_path FROM images WHERE id = ?", params![image_id], |row| row.get(0)
	)?;
	let meme_dir: String = conn.query_row(
		"SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0)
	).unwrap_or_default();
	let image_path = meme_fs::resolve_meme_path(&meme_dir, &raw_path).to_string_lossy().to_string();
	increment_share_count_internal(&conn, image_id, None)?;
	#[cfg(target_os = "linux")]
	{ std::process::Command::new("xdg-open").arg(&image_path).spawn().map_err(|e| AppError(format!("Failed to open image: {}", e)))?; }
	#[cfg(target_os = "macos")]
	{ std::process::Command::new("open").arg(&image_path).spawn().map_err(|e| AppError(format!("Failed to open image: {}", e)))?; }
	#[cfg(target_os = "windows")]
	{ std::process::Command::new("explorer").arg(&image_path).spawn().map_err(|e| AppError(format!("Failed to open image: {}", e)))?; }
	#[cfg(target_os = "android")]
	{ let _ = image_path; let _ = _target_app; }
	Ok(())
}

/// 刷新索引
#[tauri::command]
pub fn refresh_index(state: tauri::State<'_, DbState>, meme_dir: String) -> Result<String, AppError> {
	log::info!("刷新索引开始: {}", meme_dir);
	
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	let path = PathBuf::from(&meme_dir);
	
	if !path.exists() {
		std::fs::create_dir_all(&path)
			.map_err(|e| AppError(format!("创建目录失败: {}", e)))?;
	}
	
	// 删除无效记录
	delete_invalid_images(&conn, &meme_dir)?;
	delete_invalid_groups(&conn)?;
	delete_invalid_modes(&conn)?;
	
	// 扫描并新增
	scan_and_add_items(&conn, &meme_dir)?;
	
	log::info!("刷新索引完成");
	Ok("索引刷新成功".to_string())
}

/// 删除无效图片记录
fn delete_invalid_images(conn: &rusqlite::Connection, meme_dir: &str) -> Result<(), AppError> {
	let mut stmt = conn.prepare("SELECT id, image_path FROM images")?;
	let images: Vec<(i32, String)> = stmt.query_map([], |row| {
		Ok((row.get(0)?, row.get(1)?))
	})?.collect::<Result<Vec<_>,_>>()?;
	for (id, raw_path) in &images {
		let image_path = meme_fs::resolve_meme_path(meme_dir, raw_path);
		if !image_path.exists() {
			conn.execute("DELETE FROM images WHERE id = ?", params![id])?;
		}
	}
	Ok(())
}

/// 删除无效分组
fn delete_invalid_groups(conn: &rusqlite::Connection) -> Result<(), AppError> {
	let mut stmt = conn.prepare("SELECT id, folder_path FROM groups")?;
	
	let groups: Vec<(i32, String)> = stmt.query_map([], |row| {
		Ok((row.get(0)?, row.get(1)?))
	})?.collect::<Result<Vec<_>, _>>()?;
	
	for (id, folder_path) in &groups {
		if !std::path::Path::new(folder_path).exists() {
			conn.execute("DELETE FROM keyword_group_links WHERE group_id = ?", params![id])?;
			conn.execute("DELETE FROM images WHERE group_id = ?", params![id])?;
			conn.execute("DELETE FROM groups WHERE id = ?", params![id])?;
		}
	}
	
	Ok(())
}

/// 删除无效模式
fn delete_invalid_modes(conn: &rusqlite::Connection) -> Result<(), AppError> {
	let mut stmt = conn.prepare("SELECT id, folder_path FROM modes")?;
	
	let modes: Vec<(i32, String)> = stmt.query_map([], |row| {
		Ok((row.get(0)?, row.get(1)?))
	})?.collect::<Result<Vec<_>, _>>()?;
	
	for (id, folder_path) in &modes {
		if !std::path::Path::new(folder_path).exists() {
			conn.execute("DELETE FROM groups WHERE mode_id = ?", params![id])?;
			conn.execute("DELETE FROM modes WHERE id = ?", params![id])?;
		}
	}
	
	Ok(())
}

/// 扫描并新增模式、分组和图片
fn scan_and_add_items(conn: &rusqlite::Connection, meme_dir: &String) -> Result<(), AppError> {
	let path = PathBuf::from(meme_dir);
	
	for entry in std::fs::read_dir(&path)? {
		let entry = entry?;
		let mode_path = entry.path();
		
		if !mode_path.is_dir() {
			continue;
		}
		
		let mode_name = mode_path.file_name()
			.and_then(|n| n.to_str())
			.unwrap_or("")
			.to_string();
		
		if mode_name.starts_with('.') {
			continue;
		}
		
		let existing_mode_id: Option<i32> = conn.query_row(
			"SELECT id FROM modes WHERE folder_path = ?",
			params![mode_path.to_string_lossy()],
			|row| row.get(0)
		).ok();
		
		let mode_id = if let Some(id) = existing_mode_id {
			id
		} else {
			conn.execute(
				"INSERT INTO modes (name, folder_path, sort_order) VALUES (?, ?, 0)",
				params![mode_name, mode_path.to_string_lossy()]
			)?;
			
			conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))?
		};
		
		// 扫描分组
		for group_entry in std::fs::read_dir(&mode_path)? {
			let group_entry = group_entry?;
			let group_path = group_entry.path();
			
			if !group_path.is_dir() {
				continue;
			}
			
			let group_name = group_path.file_name()
				.and_then(|n| n.to_str())
				.unwrap_or("")
				.to_string();
			
			if group_name.starts_with('.') {
				continue;
			}
			
			let existing_group_id: Option<i32> = conn.query_row(
				"SELECT id FROM groups WHERE folder_path = ?",
				params![group_path.to_string_lossy()],
				|row| row.get(0)
			).ok();
			
			let group_id = if let Some(id) = existing_group_id {
				id
			} else {
				conn.execute(
					"INSERT INTO groups (name, folder_path, mode_id, share_count) VALUES (?, ?, ?, 0)",
					params![group_name, group_path.to_string_lossy(), mode_id]
				)?;
				
				conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))?
			};
			
			// 扫描图片
			for image_entry in std::fs::read_dir(&group_path)? {
				let image_entry = image_entry?;
				let image_path = image_entry.path();
				
				if !image_path.is_file() {
					continue;
				}
				
				let extension = image_path.extension()
					.and_then(|e| e.to_str())
					.unwrap_or("")
					.to_lowercase();
				
				if !["png", "jpg", "jpeg", "gif", "webp", "bmp"].contains(&extension.as_str()) {
					continue;
				}
				
			let image_path_str = image_path.to_string_lossy().to_string();
			let relative_path = meme_fs::relative_path(meme_dir, &image_path_str);
			
			let existing_image: Option<i32> = conn.query_row(
				"SELECT id FROM images WHERE image_path = ?",
				params![relative_path], |row| row.get(0)
			).ok();
			
			if existing_image.is_none() {
				conn.execute(
					"INSERT INTO images (image_path, thumbnail_path, share_count, group_id, mode_id) VALUES (?, NULL, 0, ?, ?)",
					params![relative_path, group_id, mode_id]
				)?;
			}
			}
		}
	}
	
	Ok(())
}

// ─── refresh_everything ───

use std::collections::HashSet;
use crate::commands::keyword::{KeywordPinyin, convert_to_pinyin, convert_to_acronym};

const CHUNK_SIZE: usize = 500;

struct FsSnapshot {
	modes: Vec<(String, String)>,
	groups: Vec<(String, String, String)>,
	images: Vec<(String, String)>,
	group_keywords: std::collections::HashMap<String, Vec<String>>,
	keywords_pinyin: std::collections::HashMap<String, KeywordPinyin>,
}

fn scan_filesystem_snapshot(meme_dir: &str) -> Result<FsSnapshot, AppError> {
	let meme_path = PathBuf::from(meme_dir);
	if !meme_path.exists() {
		std::fs::create_dir_all(&meme_path).map_err(|e| AppError(format!("创建目录失败: {}", e)))?;
	}
	let mut modes: Vec<(String, String)> = Vec::new();
	let mut groups: Vec<(String, String, String)> = Vec::new();
	let mut images: Vec<(String, String)> = Vec::new();
	let mut group_keywords: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
	let mut all_keywords = HashSet::new();
	for entry in std::fs::read_dir(&meme_path)? {
		let entry = entry?;
		let mode_path = entry.path();
		if !mode_path.is_dir() { continue; }
		let mode_name = mode_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
		if mode_name.starts_with('.') { continue; }
		let mode_folder = mode_path.to_string_lossy().to_string();
		modes.push((mode_name.clone(), mode_folder.clone()));
		for group_entry in std::fs::read_dir(&mode_path)? {
			let group_entry = group_entry?;
			let group_path = group_entry.path();
			if !group_path.is_dir() { continue; }
			let group_name = group_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
			if group_name.starts_with('.') || group_name.is_empty() { continue; }
			let group_folder = group_path.to_string_lossy().to_string();
			groups.push((group_name.clone(), group_folder.clone(), mode_folder.clone()));
			let keywords: Vec<String> = group_name.split(&[',', '，', '、', ' '][..])
				.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
			if !keywords.is_empty() {
				group_keywords.insert(group_name.clone(), keywords.clone());
				for kw in &keywords { all_keywords.insert(kw.clone()); }
			}
			for image_entry in std::fs::read_dir(&group_path)? {
				let image_entry = image_entry?;
				let image_path = image_entry.path();
				if !image_path.is_file() { continue; }
				let ext = image_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
				if !["png","jpg","jpeg","gif","webp","bmp"].contains(&ext.as_str()) { continue; }
				let abs = image_path.to_string_lossy().to_string();
				let relative = meme_fs::relative_path(meme_dir, &abs);
				images.push((relative, group_folder.clone()));
			}
		}
	}
	let mut keywords_pinyin: std::collections::HashMap<String, KeywordPinyin> = std::collections::HashMap::new();
	for kw in &all_keywords {
		if kw.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)) {
			let py = convert_to_pinyin(kw);
			let abbr = convert_to_acronym(kw);
			if !py.is_empty() || !abbr.is_empty() {
				keywords_pinyin.insert(kw.clone(), KeywordPinyin { pinyin: py, abbr });
			}
		}
	}
	log::info!("快照: {} modes, {} groups, {} images, {} kw-groups, {} pinyin",
		modes.len(), groups.len(), images.len(), group_keywords.len(), keywords_pinyin.len());
	Ok(FsSnapshot { modes, groups, images, group_keywords, keywords_pinyin })
}

struct FullDiff {
	delete_modes: Vec<String>,
	delete_groups: Vec<String>,
	delete_images: Vec<String>,
	delete_keywords: Vec<String>,
	insert_modes: Vec<(String, String)>,
	insert_groups: Vec<(String, String, String)>,
	insert_images: Vec<(String, String)>,
	insert_keywords: Vec<(String, Option<String>, Option<String>)>,
	insert_links: Vec<(String, String)>,
}

fn compute_db_diff(conn: &rusqlite::Connection, snap: &FsSnapshot) -> Result<FullDiff, AppError> {
	let db_mode_paths: HashSet<String> = {
		let mut stmt = conn.prepare("SELECT folder_path FROM modes")?;
		let rows: Vec<rusqlite::Result<String>> = stmt.query_map([], |r| r.get(0))?.collect();
		rows.into_iter().filter_map(|r| r.ok()).collect()
	};
	let db_group_paths: HashSet<String> = {
		let mut stmt = conn.prepare("SELECT folder_path FROM groups")?;
		let rows: Vec<rusqlite::Result<String>> = stmt.query_map([], |r| r.get(0))?.collect();
		rows.into_iter().filter_map(|r| r.ok()).collect()
	};
	let db_image_paths: HashSet<String> = {
		let mut stmt = conn.prepare("SELECT image_path FROM images")?;
		let rows: Vec<rusqlite::Result<String>> = stmt.query_map([], |r| r.get(0))?.collect();
		rows.into_iter().filter_map(|r| r.ok()).collect()
	};
	let db_keywords: HashSet<String> = {
		let mut stmt = conn.prepare("SELECT keyword FROM keywords")?;
		let rows: Vec<rusqlite::Result<String>> = stmt.query_map([], |r| r.get(0))?.collect();
		rows.into_iter().filter_map(|r| r.ok()).collect()
	};
	let snap_mode_paths: HashSet<String> = snap.modes.iter().map(|(_,p)| p.clone()).collect();
	let snap_group_paths: HashSet<String> = snap.groups.iter().map(|(_,p,_)| p.clone()).collect();
	let snap_image_paths: HashSet<String> = snap.images.iter().map(|(r,_)| r.clone()).collect();
	let snap_keywords: HashSet<String> = snap.group_keywords.values().flat_map(|v| v.iter()).cloned().collect();

	let delete_modes: Vec<String> = db_mode_paths.difference(&snap_mode_paths).cloned().collect();
	let delete_groups: Vec<String> = db_group_paths.difference(&snap_group_paths).cloned().collect();
	let delete_images: Vec<String> = db_image_paths.difference(&snap_image_paths).cloned().collect();
	let delete_keywords: Vec<String> = db_keywords.difference(&snap_keywords).cloned().collect();

	let insert_modes: Vec<_> = snap.modes.iter().filter(|(_,p)| !db_mode_paths.contains(p.as_str())).map(|(n,p)| (n.clone(),p.clone())).collect();
	let insert_groups: Vec<_> = snap.groups.iter().filter(|(_,p,_)| !db_group_paths.contains(p.as_str())).map(|(n,p,mp)| (n.clone(),p.clone(),mp.clone())).collect();
	let insert_images: Vec<_> = snap.images.iter().filter(|(r,_)| !db_image_paths.contains(r.as_str())).map(|(r,gp)| (r.clone(),gp.clone())).collect();
	let mut insert_keywords: Vec<(String, Option<String>, Option<String>)> = Vec::new();
	for kw in &snap_keywords {
		if !db_keywords.contains(kw) {
			let py = snap.keywords_pinyin.get(kw).map(|v| v.pinyin.clone());
			let abbr = snap.keywords_pinyin.get(kw).map(|v| v.abbr.clone());
			insert_keywords.push((kw.clone(), py, abbr));
		}
	}
	let mut insert_links: Vec<(String, String)> = Vec::new();
	for (group_name, keywords) in &snap.group_keywords {
		for kw in keywords { insert_links.push((kw.clone(), group_name.clone())); }
	}
	Ok(FullDiff { delete_modes, delete_groups, delete_images, delete_keywords, insert_modes, insert_groups, insert_images, insert_keywords, insert_links })
}

fn apply_diff(conn: &rusqlite::Connection, diff: &FullDiff) -> Result<(), AppError> {
	use rusqlite::params_from_iter;

	conn.execute("DELETE FROM keyword_group_links", [])?;

	for chunk in diff.delete_images.chunks(CHUNK_SIZE) {
		let ph: Vec<String> = chunk.iter().enumerate().map(|(i,_)| format!("?{}",i+1)).collect();
		let sql = format!("DELETE FROM images WHERE image_path IN ({})", ph.join(","));
		conn.execute(&sql, params_from_iter(chunk.iter()))?;
	}
	for chunk in diff.delete_groups.chunks(CHUNK_SIZE) {
		let ph: Vec<String> = chunk.iter().enumerate().map(|(i,_)| format!("?{}",i+1)).collect();
		let sql = format!("DELETE FROM groups WHERE folder_path IN ({})", ph.join(","));
		conn.execute(&sql, params_from_iter(chunk.iter()))?;
	}
	for chunk in diff.delete_modes.chunks(CHUNK_SIZE) {
		let ph: Vec<String> = chunk.iter().enumerate().map(|(i,_)| format!("?{}",i+1)).collect();
		let sql = format!("DELETE FROM modes WHERE folder_path IN ({})", ph.join(","));
		conn.execute(&sql, params_from_iter(chunk.iter()))?;
	}
	for chunk in diff.delete_keywords.chunks(CHUNK_SIZE) {
		let ph: Vec<String> = chunk.iter().enumerate().map(|(i,_)| format!("?{}",i+1)).collect();
		let sql = format!("DELETE FROM keywords WHERE keyword IN ({})", ph.join(","));
		conn.execute(&sql, params_from_iter(chunk.iter()))?;
	}

	if !diff.insert_modes.is_empty() {
		let mut sql = String::from("INSERT INTO modes (name, folder_path, sort_order) VALUES ");
		let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
		for (i, (name, path)) in diff.insert_modes.iter().enumerate() {
			if i > 0 { sql.push(','); }
			sql.push_str(&format!("(?{}, ?{}, 0)", i*2+1, i*2+2));
			params.push(Box::new(name.clone()));
			params.push(Box::new(path.clone()));
		}
		let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
		conn.execute(&sql, refs.as_slice())?;
	}
	for chunk in diff.insert_groups.chunks(CHUNK_SIZE) {
		let mut parts: Vec<String> = Vec::new();
		let mut params: Vec<String> = Vec::new();
		for (i, (name, fp, mfp)) in chunk.iter().enumerate() {
			parts.push(format!("SELECT ?{}, ?{}, m.id, 0 FROM modes m WHERE m.folder_path = ?{}", i*3+1, i*3+2, i*3+3));
			params.push(name.clone()); params.push(fp.clone()); params.push(mfp.clone());
		}
		let sql = format!("INSERT INTO groups (name, folder_path, mode_id, share_count) {}", parts.join(" UNION ALL "));
		conn.execute(&sql, params_from_iter(params.iter()))?;
	}
	for chunk in diff.insert_images.chunks(CHUNK_SIZE) {
		let mut parts: Vec<String> = Vec::new();
		let mut params: Vec<String> = Vec::new();
		for (i, (rp, gfp)) in chunk.iter().enumerate() {
			parts.push(format!("SELECT ?{}, g.id, g.mode_id, 0 FROM groups g WHERE g.folder_path = ?{}", i*2+1, i*2+2));
			params.push(rp.clone()); params.push(gfp.clone());
		}
		let sql = format!("INSERT INTO images (image_path, group_id, mode_id, share_count) {}", parts.join(" UNION ALL "));
		conn.execute(&sql, params_from_iter(params.iter()))?;
	}
	for chunk in diff.insert_keywords.chunks(CHUNK_SIZE) {
		let mut sql = String::from("INSERT INTO keywords (keyword, pinyin, acronym) VALUES ");
		let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
		for (i, (kw, py, abbr)) in chunk.iter().enumerate() {
			if i > 0 { sql.push(','); }
			sql.push_str(&format!("(?{}, ?{}, ?{})", i*3+1, i*3+2, i*3+3));
			params.push(Box::new(kw.clone()));
			params.push(Box::new(py.clone()));
			params.push(Box::new(abbr.clone()));
		}
		let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
		conn.execute(&sql, refs.as_slice())?;
	}
	for chunk in diff.insert_links.chunks(CHUNK_SIZE) {
		let mut parts: Vec<String> = Vec::new();
		let mut params: Vec<String> = Vec::new();
		for (i, (kw, gn)) in chunk.iter().enumerate() {
			parts.push(format!("SELECT k.id, g.id FROM keywords k, groups g WHERE k.keyword = ?{} AND g.name = ?{}", i*2+1, i*2+2));
			params.push(kw.clone()); params.push(gn.clone());
		}
		let sql = format!("INSERT INTO keyword_group_links (keyword_id, group_id) {}", parts.join(" UNION ALL "));
		conn.execute(&sql, params_from_iter(params.iter()))?;
	}
	Ok(())
}

fn refresh_everything(conn: &rusqlite::Connection, meme_dir: &str) -> Result<String, AppError> {
	let snapshot = scan_filesystem_snapshot(meme_dir)?;
    crate::commands::keyword::write_keywords_toml_snapshot(meme_dir, &snapshot.group_keywords, &snapshot.keywords_pinyin)?;
	let diff = compute_db_diff(conn, &snapshot)?;
	log::info!("diff: +{}m -{}m +{}g -{}g +{}i -{}i +{}k -{}k",
		diff.insert_modes.len(), diff.delete_modes.len(),
		diff.insert_groups.len(), diff.delete_groups.len(),
		diff.insert_images.len(), diff.delete_images.len(),
		diff.insert_keywords.len(), diff.delete_keywords.len());
	apply_diff(conn, &diff)?;
	Ok("数据刷新完成".to_string())
}

#[tauri::command]
pub fn full_refresh(state: tauri::State<'_, DbState>, meme_dir: String) -> Result<String, AppError> {
	let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
	conn.execute_batch("BEGIN")?;
	match refresh_everything(&conn, &meme_dir) {
		Ok(msg) => {
			conn.execute_batch("COMMIT").map_err(|e| {
				let _ = conn.execute_batch("ROLLBACK");
				e
			})?;
			Ok(msg)
		}
		Err(e) => { let _ = conn.execute_batch("ROLLBACK"); Err(e) }
	}
}
