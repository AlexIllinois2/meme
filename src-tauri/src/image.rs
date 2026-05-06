//! 图片处理模块
//! 
//! 提供图片上传、复制、删除、搜索等功能

use rusqlite::params;
use std::path::PathBuf;
use crate::{db::init_db, models::Image};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_fs::FsExt;
use image::ImageEncoder;

#[cfg(not(target_os = "android"))]
use clipboard_rs::{Clipboard, ClipboardContext};

/// 剪贴板图片数据结构
#[derive(serde::Serialize)]
pub struct ClipboardImage {
	/// 图片二进制数据
	pub data: Vec<u8>,
	/// 图片格式
	pub format: String,
}

/// 从剪贴板读取图片
#[tauri::command]
pub fn paste_image_from_clipboard<R: tauri::Runtime>(
	_app: tauri::AppHandle<R>,
) -> Result<ClipboardImage, String> {
	let clipboard = _app.clipboard();
	
	match clipboard.read_image() {
		Ok(image) => {
			let rgba_data: Vec<u8> = image.rgba().into();
			let width = image.width();
			let height = image.height();
			
			match image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
				width, height, rgba_data
			) {
				Some(buffer) => {
					let mut png_bytes: Vec<u8> = Vec::new();
					let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
					match encoder.write_image(
						&buffer, width, height, image::ColorType::Rgba8
					) {
						Ok(_) => Ok(ClipboardImage {
							data: png_bytes,
							format: "png".to_string(),
						}),
						Err(e) => Err(format!("Failed to encode PNG: {}", e)),
					}
				}
				None => Err("Failed to create image buffer".to_string()),
			}
		}
		Err(e) => Err(format!("Failed to read clipboard image: {}", e)),
	}
}

/// 检测图片格式
fn detect_image_format(data: &[u8]) -> Option<&'static str> {
	if data.len() < 8 {
		return None;
	}
	
	if data.starts_with(b"\x89PNG") {
		Some("png")
	} else if data.starts_with(b"\xFF\xD8\xFF") {
		Some("jpg")
	} else if data.starts_with(b"GIF89a") || data.starts_with(b"GIF87a") {
		Some("gif")
	} else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
		Some("webp")
	} else if data.starts_with(b"BM") {
		Some("bmp")
	} else {
		None
	}
}

/// 使用系统命令读取剪贴板图片（桌面端）
#[cfg(not(target_os = "android"))]
fn read_clipboard_with_system_command(temp_dir: &str) -> Result<String, String> {
	let timestamp = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or_default()
		.as_millis();
	
	#[cfg(target_os = "linux")]
	{
		let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
		
		if session_type == "wayland" {
			let output = std::process::Command::new("wl-paste")
				.args(&["--type", "image/png"])
				.output();
			
			if let Ok(output) = output {
				if output.status.success() && !output.stdout.is_empty() {
					let temp_file_name = format!("pasted-{}.png", timestamp);
					let temp_path = std::path::Path::new(temp_dir).join(&temp_file_name);
					if let Some(parent) = temp_path.parent() {
						std::fs::create_dir_all(parent)
							.map_err(|e| format!("Failed to create temp dir: {}", e))?;
					}
					std::fs::write(&temp_path, &output.stdout)
						.map_err(|e| format!("Failed to write image: {}", e))?;
					return Ok(temp_path.to_string_lossy().to_string());
				}
			}
		}
	}
	
	Err("System clipboard command failed".to_string())
}

/// 从剪贴板读取图片（桌面端）
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn paste_image_from_clipboard_raw<R: tauri::Runtime>(
	_app: tauri::AppHandle<R>,
	temp_dir: String,
) -> Result<String, String> {
	let ctx = ClipboardContext::new().map_err(|e| {
		format!("Failed to create clipboard context: {}", e)
	})?;
	
	let files = ctx.get_files().map_err(|e| {
		format!("Failed to read files from clipboard: {}", e)
	})?;
	
	if !files.is_empty() {
		let src_path = std::path::Path::new(&files[0]);
		if !src_path.exists() {
			return Err("剪贴板中的文件不存在".to_string());
		}
		
		let extension = src_path.extension()
			.and_then(|e| e.to_str())
			.unwrap_or("png")
			.to_lowercase();
		
		let timestamp = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis();
		
		let temp_file_name = format!("pasted-{}.{}", timestamp, extension);
		let temp_path = std::path::Path::new(&temp_dir).join(&temp_file_name);
		
		if let Some(parent) = temp_path.parent() {
			std::fs::create_dir_all(parent)
				.map_err(|e| format!("Failed to create temp dir: {}", e))?;
		}
		
		std::fs::copy(src_path, &temp_path).map_err(|e| {
			format!("Failed to copy file: {}", e)
		})?;
		
		return Ok(temp_path.to_string_lossy().to_string());
	}
	
	match read_clipboard_with_system_command(&temp_dir) {
		Ok(path) => return Ok(path),
		Err(_) => {
			let clipboard = _app.clipboard();
			match clipboard.read_image() {
				Ok(image) => {
					let rgba_data: Vec<u8> = image.rgba().into();
					let width = image.width();
					let height = image.height();
					
					let timestamp = std::time::SystemTime::now()
						.duration_since(std::time::UNIX_EPOCH)
						.unwrap_or_default()
						.as_millis();
					let temp_file_name = format!("pasted-{}.png", timestamp);
					let temp_path = std::path::Path::new(&temp_dir).join(&temp_file_name);
					
					if let Some(parent) = temp_path.parent() {
						std::fs::create_dir_all(parent)
							.map_err(|e| format!("Failed to create temp dir: {}", e))?;
					}
					
					let buffer = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
						width, height, rgba_data
					).ok_or("Failed to create image buffer")?;
					
					buffer.save(&temp_path)
						.map_err(|e| format!("Failed to save image: {}", e))?;
					
					Ok(temp_path.to_string_lossy().to_string())
				}
				Err(e) => Err(format!("剪贴板中没有图片或读取失败: {}", e)),
			}
		}
	}
}

/// 从剪贴板读取图片（Android 端 - 不支持）
#[cfg(target_os = "android")]
#[tauri::command]
pub fn paste_image_from_clipboard_raw<R: tauri::Runtime>(
	_app: tauri::AppHandle<R>,
	_temp_dir: String,
) -> Result<String, String> {
	Err("剪贴板粘贴功能在 Android 端暂不支持".to_string())
}

/// 获取分组下的所有图片
#[tauri::command]
pub fn get_images_by_group(group_id: i32) -> Result<Vec<Image>, String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	let mut stmt = conn.prepare(
		"SELECT id, image_path, thumbnail_path, share_count, group_id, mode_id 
		 FROM images 
		 WHERE group_id = ? 
		 ORDER BY share_count DESC, id ASC"
	).map_err(|e| e.to_string())?;
	
	let images = stmt.query_map(params![group_id], |row| {
		Ok(Image {
			id: row.get(0)?,
			image_path: row.get(1)?,
			thumbnail_path: row.get(2)?,
			share_count: row.get(3)?,
			group_id: row.get(4)?,
			mode_id: row.get(5)?,
		})
	}).map_err(|e| e.to_string())?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| e.to_string())?;
	
	Ok(images)
}

/// 搜索图片
#[tauri::command]
pub fn search_images(keyword: String, pinyin: bool, acronym: bool) -> Result<Vec<Image>, String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	let search_pattern = format!("%{}%", keyword);
	
	let query = if pinyin && acronym {
		"SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id 
		 FROM images i
		 JOIN keyword_group_links kgl ON i.group_id = kgl.group_id
		 JOIN keywords k ON kgl.keyword_id = k.id
		 WHERE k.keyword LIKE ?1 OR k.pinyin LIKE ?1 OR k.acronym LIKE ?1
		 ORDER BY i.share_count DESC"
	} else if pinyin {
		"SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id 
		 FROM images i
		 JOIN keyword_group_links kgl ON i.group_id = kgl.group_id
		 JOIN keywords k ON kgl.keyword_id = k.id
		 WHERE k.keyword LIKE ?1 OR k.pinyin LIKE ?1
		 ORDER BY i.share_count DESC"
	} else if acronym {
		"SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id 
		 FROM images i
		 JOIN keyword_group_links kgl ON i.group_id = kgl.group_id
		 JOIN keywords k ON kgl.keyword_id = k.id
		 WHERE k.keyword LIKE ?1 OR k.acronym LIKE ?1
		 ORDER BY i.share_count DESC"
	} else {
		"SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id 
		 FROM images i
		 JOIN keyword_group_links kgl ON i.group_id = kgl.group_id
		 JOIN keywords k ON kgl.keyword_id = k.id
		 WHERE k.keyword LIKE ?1
		 ORDER BY i.share_count DESC"
	};
	
	let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
	
	let images = stmt.query_map(params![search_pattern], |row| {
		Ok(Image {
			id: row.get(0)?,
			image_path: row.get(1)?,
			thumbnail_path: row.get(2)?,
			share_count: row.get(3)?,
			group_id: row.get(4)?,
			mode_id: row.get(5)?,
		})
	}).map_err(|e| e.to_string())?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| e.to_string())?;
	
	Ok(images)
}

/// 内部函数：增加分享次数
fn increment_share_count_internal(
	conn: &rusqlite::Connection, 
	image_id: i32, 
	group_id: Option<i32>
) -> Result<(), String> {
	conn.execute(
		"UPDATE images SET share_count = share_count + 1 WHERE id = ?",
		params![image_id],
	).map_err(|e| e.to_string())?;
	
	if let Some(gid) = group_id {
		conn.execute(
			"UPDATE groups SET share_count = share_count + 1 WHERE id = ?",
			params![gid],
		).map_err(|e| e.to_string())?;
	} else {
		let g_id: i32 = conn.query_row(
			"SELECT group_id FROM images WHERE id = ?",
			params![image_id],
			|row| row.get(0)
		).map_err(|e| e.to_string())?;
		
		conn.execute(
			"UPDATE groups SET share_count = share_count + 1 WHERE id = ?",
			params![g_id],
		).map_err(|e| e.to_string())?;
	}
	
	Ok(())
}

/// 分享图片
#[tauri::command]
pub fn share_image(image_id: i32) -> Result<(), String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	increment_share_count_internal(&conn, image_id, None)
}

/// 复制图片到剪贴板（桌面端）
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn copy_image(image_id: i32) -> Result<(), String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	
	let image_path: String = conn.query_row(
		"SELECT image_path FROM images WHERE id = ?",
		params![image_id],
		|row| row.get(0)
	).map_err(|e| e.to_string())?;
	
	increment_share_count_internal(&conn, image_id, None)?;
	
	let ctx = ClipboardContext::new().map_err(|e| {
		format!("Failed to create clipboard context: {}", e)
	})?;
	
	let abs_path = std::fs::canonicalize(&image_path).map_err(|e| {
		format!("Failed to get absolute path: {}", e)
	})?;
	
	ctx.set_files(vec![abs_path.to_string_lossy().to_string()]).map_err(|e| {
		format!("Failed to copy file to clipboard: {}", e)
	})?;
	
	Ok(())
}

/// 复制图片到剪贴板（Android 端）
#[cfg(target_os = "android")]
#[tauri::command]
pub fn copy_image(image_id: i32) -> Result<(), String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	increment_share_count_internal(&conn, image_id, None)?;
	Err("图片复制到剪贴板功能在 Android 端暂不支持".to_string())
}

/// 增加分享次数
#[tauri::command]
pub fn increment_share_count(image_id: i32, group_id: Option<i32>) -> Result<(), String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	increment_share_count_internal(&conn, image_id, group_id)
}

/// 批量复制图片
#[tauri::command]
pub fn copy_images(image_ids: Vec<i32>) -> Result<(), String> {
	if image_ids.is_empty() {
		return Ok(());
	}
	
	for image_id in &image_ids {
		let _ = copy_image(*image_id);
	}
	
	Ok(())
}

/// 删除图片
#[tauri::command]
pub fn delete_images(image_ids: Vec<i32>) -> Result<(), String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	
	for image_id in image_ids {
		if let Ok((image_path, thumb_path)) = conn.query_row(
			"SELECT image_path, thumbnail_path FROM images WHERE id = ?",
			params![image_id],
			|row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
		) {
			let _ = std::fs::remove_file(&image_path);
			if let Some(thumb) = thumb_path {
				let _ = std::fs::remove_file(&thumb);
			}
		}
		
		conn.execute("DELETE FROM images WHERE id = ?", params![image_id])
			.map_err(|e| e.to_string())?;
	}
	
	Ok(())
}

/// 移动图片到其他分组
#[tauri::command]
pub fn move_images(image_ids: Vec<i32>, target_group_id: i32) -> Result<(), String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	
	let target_mode_id: i32 = conn.query_row(
		"SELECT mode_id FROM groups WHERE id = ?",
		params![target_group_id],
		|row| row.get(0)
	).map_err(|e| e.to_string())?;
	
	for image_id in image_ids {
		conn.execute(
			"UPDATE images SET group_id = ?, mode_id = ? WHERE id = ?",
			params![target_group_id, target_mode_id, image_id]
		).map_err(|e| e.to_string())?;
	}
	
	Ok(())
}

/// Android 端上传图片（接收 Base64 编码的图片数据）
/// 
/// 前端读取 content:// URI 的数据并转为 Base64 传递给后端
#[tauri::command]
pub async fn upload_images_android(
	images_data: Vec<AndroidImageData>,
	group_id: i32,
	mode_id: i32,
) -> Result<usize, String> {
	log::info!("[Android] upload_images_android 开始");
	log::info!("[Android] 图片数量: {}", images_data.len());
	log::info!("[Android] group_id: {}, mode_id: {}", group_id, mode_id);
	
	let conn = init_db().map_err(|e| e.to_string())?;
	
	// 获取分组信息
	let group_folder: String = conn.query_row(
		"SELECT folder_path FROM groups WHERE id = ?",
		params![group_id],
		|row| row.get(0)
	).map_err(|e| format!("获取分组信息失败: {}", e))?;
	
	log::info!("[Android] 分组文件夹: {}", group_folder);
	
	let mut success_count = 0;
	
	for image_data in &images_data {
		log::info!("[Android] 处理图片: {}", image_data.name);
		
		// 解码 Base64 数据
		let data = base64_decode(&image_data.data)?;
		log::info!("[Android] 解码数据成功，大小: {} bytes", data.len());
		
		// 检测图片格式
		let ext = detect_image_format(&data).unwrap_or("png");
		log::info!("[Android] 检测到格式: {}", ext);
		
		// 生成文件名
		let timestamp = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis();
		let file_name = format!("image-{}.{}", timestamp, ext);
		let dest_path = std::path::Path::new(&group_folder).join(&file_name);
		
		log::info!("[Android] 目标路径: {:?}", dest_path);
		
		// 写入文件
		std::fs::write(&dest_path, &data)
			.map_err(|e| format!("写入文件失败: {}", e))?;
		
		log::info!("[Android] 文件写入成功");
		
		// 插入数据库记录
		let dest_path_str = dest_path.to_string_lossy().to_string();
		conn.execute(
			"INSERT INTO images (image_path, thumbnail_path, share_count, group_id, mode_id) 
			 VALUES (?, NULL, 0, ?, ?)",
			params![dest_path_str, group_id, mode_id]
		).map_err(|e| format!("插入数据库失败: {}", e))?;
		
		log::info!("[Android] 数据库记录插入成功");
		success_count += 1;
	}
	
	log::info!("[Android] upload_images_android 完成，成功上传 {} 张", success_count);
	Ok(success_count)
}

/// Android 图片数据结构
#[derive(serde::Deserialize)]
pub struct AndroidImageData {
	/// 文件名
	pub name: String,
	/// Base64 编码的图片数据
	pub data: String,
}

/// Base64 解码
fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
	// 移除 data URL 前缀（如果有）
	let data = if data.starts_with("data:") {
		// 找到 base64 数据部分
		if let Some(idx) = data.find(";base64,") {
			&data[idx + 8..]
		} else {
			data
		}
	} else {
		data
	};
	
	use base64::{Engine, engine::general_purpose::STANDARD};
	STANDARD.decode(data).map_err(|e| format!("Base64 解码失败: {}", e))
}

/// 桌面端上传图片
#[tauri::command]
pub fn upload_images(
	file_paths: Vec<String>, 
	group_id: i32, 
	mode_id: i32, 
	_meme_dir: String
) -> Result<(), String> {
	log::info!("[Desktop] upload_images 开始");
	log::info!("[Desktop] file_paths: {:?}", file_paths);
	log::info!("[Desktop] group_id: {}, mode_id: {}", group_id, mode_id);
	
	let conn = init_db().map_err(|e| e.to_string())?;
	
	let group_folder: String = conn.query_row(
		"SELECT folder_path FROM groups WHERE id = ?",
		params![group_id],
		|row| row.get(0)
	).map_err(|e| format!("获取分组信息失败: {}", e))?;
	
	log::info!("[Desktop] 分组文件夹: {}", group_folder);
	
	for file_path in &file_paths {
		log::info!("[Desktop] 处理文件: {}", file_path);
		
		let src_path = std::path::Path::new(file_path);
		if !src_path.exists() {
			log::warn!("[Desktop] 文件不存在: {}", file_path);
			continue;
		}
		
		let file_name = src_path.file_name()
			.and_then(|n| n.to_str())
			.unwrap_or("unknown.png");
		
		let dest_path = std::path::Path::new(&group_folder).join(file_name);
		log::info!("[Desktop] 目标路径: {:?}", dest_path);
		
		std::fs::copy(src_path, &dest_path).map_err(|e| {
			format!("复制文件失败 {}: {}", file_path, e)
		})?;
		
		let dest_path_str = dest_path.to_string_lossy().to_string();
		
		let existing_image: Option<i32> = conn.query_row(
			"SELECT id FROM images WHERE image_path = ?",
			params![dest_path_str],
			|row| row.get(0)
		).ok();
		
		if existing_image.is_none() {
			conn.execute(
				"INSERT INTO images (image_path, thumbnail_path, share_count, group_id, mode_id) 
				 VALUES (?, NULL, 0, ?, ?)",
				params![dest_path_str, group_id, mode_id]
			).map_err(|e| e.to_string())?;
			log::info!("[Desktop] 数据库记录插入成功");
		} else {
			log::info!("[Desktop] 图片已存在，跳过");
		}
	}
	
	log::info!("[Desktop] upload_images 完成");
	Ok(())
}

/// 获取图片完整路径
#[tauri::command]
pub fn get_image_full_path(image_id: i32) -> Result<String, String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	
	let image_path: String = conn.query_row(
		"SELECT image_path FROM images WHERE id = ?",
		params![image_id],
		|row| row.get(0)
	).map_err(|e| e.to_string())?;
	
	Ok(image_path)
}

/// 分享图片到应用
#[tauri::command]
pub fn share_image_to_app<R: tauri::Runtime>(
	_app: tauri::AppHandle<R>,
	image_id: i32,
	target_app: String,
) -> Result<(), String> {
	let conn = init_db().map_err(|e| e.to_string())?;
	
	let image_path: String = conn.query_row(
		"SELECT image_path FROM images WHERE id = ?",
		params![image_id],
		|row| row.get(0)
	).map_err(|e| e.to_string())?;
	
	increment_share_count_internal(&conn, image_id, None)?;
	
	#[cfg(target_os = "linux")]
	{
		std::process::Command::new("xdg-open")
			.arg(&image_path)
			.spawn()
			.map_err(|e| format!("Failed to open image: {}", e))?;
	}
	
	#[cfg(target_os = "macos")]
	{
		std::process::Command::new("open")
			.arg(&image_path)
			.spawn()
			.map_err(|e| format!("Failed to open image: {}", e))?;
	}
	
	#[cfg(target_os = "windows")]
	{
		std::process::Command::new("explorer")
			.arg(&image_path)
			.spawn()
			.map_err(|e| format!("Failed to open image: {}", e))?;
	}
	
	#[cfg(target_os = "android")]
	{
		let _ = image_path;
		let _ = target_app;
	}
	
	Ok(())
}

/// 刷新索引
#[tauri::command]
pub fn refresh_index(meme_dir: String) -> Result<String, String> {
	log::info!("刷新索引开始: {}", meme_dir);
	
	let conn = init_db().map_err(|e| e.to_string())?;
	let path = PathBuf::from(&meme_dir);
	
	if !path.exists() {
		std::fs::create_dir_all(&path)
			.map_err(|e| format!("创建目录失败: {}", e))?;
	}
	
	// 删除无效记录
	delete_invalid_images(&conn)?;
	delete_invalid_groups(&conn)?;
	delete_invalid_modes(&conn)?;
	
	// 扫描并新增
	scan_and_add_items(&conn, &meme_dir)?;
	
	log::info!("刷新索引完成");
	Ok("索引刷新成功".to_string())
}

/// 删除无效图片记录
fn delete_invalid_images(conn: &rusqlite::Connection) -> Result<(), String> {
	let mut stmt = conn.prepare("SELECT id, image_path FROM images")
		.map_err(|e| e.to_string())?;
	
	let images: Vec<(i32, String)> = stmt.query_map([], |row| {
		Ok((row.get(0)?, row.get(1)?))
	}).map_err(|e| e.to_string())?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| e.to_string())?;
	
	for (id, image_path) in &images {
		if !std::path::Path::new(image_path).exists() {
			conn.execute("DELETE FROM images WHERE id = ?", params![id])
				.map_err(|e| e.to_string())?;
		}
	}
	
	Ok(())
}

/// 删除无效分组
fn delete_invalid_groups(conn: &rusqlite::Connection) -> Result<(), String> {
	let mut stmt = conn.prepare("SELECT id, folder_path FROM groups")
		.map_err(|e| e.to_string())?;
	
	let groups: Vec<(i32, String)> = stmt.query_map([], |row| {
		Ok((row.get(0)?, row.get(1)?))
	}).map_err(|e| e.to_string())?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| e.to_string())?;
	
	for (id, folder_path) in &groups {
		if !std::path::Path::new(folder_path).exists() {
			conn.execute("DELETE FROM keyword_group_links WHERE group_id = ?", params![id])
				.map_err(|e| e.to_string())?;
			conn.execute("DELETE FROM images WHERE group_id = ?", params![id])
				.map_err(|e| e.to_string())?;
			conn.execute("DELETE FROM groups WHERE id = ?", params![id])
				.map_err(|e| e.to_string())?;
		}
	}
	
	Ok(())
}

/// 删除无效模式
fn delete_invalid_modes(conn: &rusqlite::Connection) -> Result<(), String> {
	let mut stmt = conn.prepare("SELECT id, folder_path FROM modes")
		.map_err(|e| e.to_string())?;
	
	let modes: Vec<(i32, String)> = stmt.query_map([], |row| {
		Ok((row.get(0)?, row.get(1)?))
	}).map_err(|e| e.to_string())?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| e.to_string())?;
	
	for (id, folder_path) in &modes {
		if !std::path::Path::new(folder_path).exists() {
			conn.execute("DELETE FROM groups WHERE mode_id = ?", params![id])
				.map_err(|e| e.to_string())?;
			conn.execute("DELETE FROM modes WHERE id = ?", params![id])
				.map_err(|e| e.to_string())?;
		}
	}
	
	Ok(())
}

/// 扫描并新增模式、分组和图片
fn scan_and_add_items(conn: &rusqlite::Connection, meme_dir: &String) -> Result<(), String> {
	let path = PathBuf::from(meme_dir);
	
	for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
		let entry = entry.map_err(|e| e.to_string())?;
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
			).map_err(|e| e.to_string())?;
			
			conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
				.map_err(|e| e.to_string())?
		};
		
		// 扫描分组
		for group_entry in std::fs::read_dir(&mode_path).map_err(|e| e.to_string())? {
			let group_entry = group_entry.map_err(|e| e.to_string())?;
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
				).map_err(|e| e.to_string())?;
				
				conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
					.map_err(|e| e.to_string())?
			};
			
			// 扫描图片
			for image_entry in std::fs::read_dir(&group_path).map_err(|e| e.to_string())? {
				let image_entry = image_entry.map_err(|e| e.to_string())?;
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
				
				let existing_image: Option<i32> = conn.query_row(
					"SELECT id FROM images WHERE image_path = ?",
					params![image_path_str],
					|row| row.get(0)
				).ok();
				
				if existing_image.is_none() {
					conn.execute(
						"INSERT INTO images (image_path, thumbnail_path, share_count, group_id, mode_id) 
						 VALUES (?, NULL, 0, ?, ?)",
						params![image_path_str, group_id, mode_id]
					).map_err(|e| e.to_string())?;
				}
			}
		}
	}
	
	Ok(())
}
