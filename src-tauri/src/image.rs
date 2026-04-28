use rusqlite::params;
use std::path::PathBuf;
use crate::{db::init_db, models::Image};
use tauri_plugin_clipboard_manager::ClipboardExt;
use clipboard_rs::{Clipboard, ClipboardContext};
use image::ImageEncoder;

#[derive(serde::Serialize)]
pub struct ClipboardImage {
    pub data: Vec<u8>,
    pub format: String,
}

#[tauri::command]
pub fn paste_image_from_clipboard<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<ClipboardImage, String> {
    // 使用 tauri-plugin-clipboard-manager 读取剪贴板图片
    let clipboard = _app.clipboard();
    
    // 尝试读取图片 (read_image 不是异步的)
    match clipboard.read_image() {
        Ok(image) => {
            // 获取图片的 RGBA 数据
            let rgba_data: Vec<u8> = image.rgba().into();
            let width = image.width();
            let height = image.height();
            
            // 将 RGBA 数据编码为 PNG 格式
            match image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba_data) {
                Some(buffer) => {
                    let mut png_bytes: Vec<u8> = Vec::new();
                    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                    match encoder.write_image(&buffer, width, height, image::ColorType::Rgba8) {
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

/// 检测数据是否为图片并返回扩展名
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

/// 使用系统命令读取剪贴板图片数据
fn read_clipboard_with_system_command(temp_dir: &str) -> Result<String, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    
    #[cfg(target_os = "linux")]
    {
        // 首先检测是 X11 还是 Wayland
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        
        if session_type == "wayland" {
            // 尝试使用 wl-paste (Wayland)
            let output = std::process::Command::new("wl-paste")
                .args(&["--type", "image/png"])
                .output();
            
            if let Ok(output) = output {
                if output.status.success() && !output.stdout.is_empty() {
                    let temp_file_name = format!("pasted-{}.png", timestamp);
                    let temp_path = std::path::Path::new(temp_dir).join(&temp_file_name);
                    if let Some(parent) = temp_path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create temp dir: {}", e))?;
                    }
                    std::fs::write(&temp_path, &output.stdout).map_err(|e| format!("Failed to write image: {}", e))?;
                    return Ok(temp_path.to_string_lossy().to_string());
                }
            }
            
            // 尝试其他图片格式
            for mime_type in &["image/jpeg", "image/gif", "image/webp", "image/bmp", "image/x-qt-image"] {
                let output = std::process::Command::new("wl-paste")
                    .args(&["--type", mime_type])
                    .output();
                
                if let Ok(output) = output {
                    if output.status.success() && !output.stdout.is_empty() {
                        if let Some(ext) = detect_image_format(&output.stdout) {
                            let temp_file_name = format!("pasted-{}.{}", timestamp, ext);
                            let temp_path = std::path::Path::new(temp_dir).join(&temp_file_name);
                            if let Some(parent) = temp_path.parent() {
                                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create temp dir: {}", e))?;
                            }
                            std::fs::write(&temp_path, &output.stdout).map_err(|e| format!("Failed to write image: {}", e))?;
                            return Ok(temp_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        } else {
            // X11 - 首先获取可用的剪贴板格式
            let targets_output = std::process::Command::new("xclip")
                .args(&["-selection", "clipboard", "-t", "TARGETS", "-o"])
                .output();
            
            let available_formats: Vec<String> = if let Ok(output) = targets_output {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            };
            
            // 优先尝试的 MIME 类型
            let preferred_types = vec![
                "image/png",
                "image/jpeg", 
                "image/jpg",
                "image/gif",
                "image/webp",
                "image/bmp",
                "image/x-qt-image",  // Qt 应用（如 Linux QQ）
                "image/x-png",
                "application/x-qt-image",
                "image/tiff",
            ];
            
            // 如果有可用的格式列表，按优先级尝试
            for mime_type in &preferred_types {
                if available_formats.is_empty() || available_formats.contains(&mime_type.to_string()) {
                    let output = std::process::Command::new("xclip")
                        .args(&["-selection", "clipboard", "-t", mime_type, "-o"])
                        .output();
                    
                    if let Ok(output) = output {
                        if output.status.success() && !output.stdout.is_empty() {
                            if let Some(ext) = detect_image_format(&output.stdout) {
                                let temp_file_name = format!("pasted-{}.{}", timestamp, ext);
                                let temp_path = std::path::Path::new(temp_dir).join(&temp_file_name);
                                if let Some(parent) = temp_path.parent() {
                                    std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create temp dir: {}", e))?;
                                }
                                std::fs::write(&temp_path, &output.stdout).map_err(|e| format!("Failed to write image: {}", e))?;
                                return Ok(temp_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
            
            // 尝试 text/uri-list（文件路径列表）
            let output = std::process::Command::new("xclip")
                .args(&["-selection", "clipboard", "-t", "text/uri-list", "-o"])
                .output();
            
            if let Ok(output) = output {
                if output.status.success() && !output.stdout.is_empty() {
                    let uri_list = String::from_utf8_lossy(&output.stdout);
                    for line in uri_list.lines() {
                        let line = line.trim();
                        if line.starts_with("file://") {
                            let path = &line[7..]; // 移除 file:// 前缀
                            let path = percent_encoding::percent_decode_str(path)
                                .decode_utf8_lossy()
                                .to_string();
                            let src_path = std::path::Path::new(&path);
                            if src_path.exists() {
                                let extension = src_path.extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("png")
                                    .to_lowercase();
                                let temp_file_name = format!("pasted-{}.{}", timestamp, extension);
                                let temp_path = std::path::Path::new(temp_dir).join(&temp_file_name);
                                if let Some(parent) = temp_path.parent() {
                                    std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create temp dir: {}", e))?;
                                }
                                std::fs::copy(src_path, &temp_path).map_err(|e| format!("Failed to copy file: {}", e))?;
                                return Ok(temp_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
            
            // 回退到 xclip 默认类型
            let output = std::process::Command::new("xclip")
                .args(&["-selection", "clipboard", "-o"])
                .output();
            
            if let Ok(output) = output {
                if output.status.success() && !output.stdout.is_empty() {
                    if let Some(ext) = detect_image_format(&output.stdout) {
                        let temp_file_name = format!("pasted-{}.{}", timestamp, ext);
                        let temp_path = std::path::Path::new(temp_dir).join(&temp_file_name);
                        if let Some(parent) = temp_path.parent() {
                            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create temp dir: {}", e))?;
                        }
                        std::fs::write(&temp_path, &output.stdout).map_err(|e| format!("Failed to write image: {}", e))?;
                        return Ok(temp_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 osascript 读取剪贴板
        let output = std::process::Command::new("osascript")
            .args(&["-e", "try
    set imageData to (the clipboard as «class PNGf»)
    return imageData
on error
    return \"\"
end try"])
            .output();
        
        if let Ok(output) = output {
            if output.status.success() && !output.stdout.is_empty() {
                std::fs::write(&temp_path, &output.stdout).map_err(|e| format!("Failed to write image: {}", e))?;
                return Ok(temp_path.to_string_lossy().to_string());
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 PowerShell
        let ps_script = format!(
            "Add-Type -AssemblyName System.Windows.Forms
            $img = [Windows.Forms.Clipboard]::GetImage()
            if ($img) {{
                $img.Save('{}')
                'OK'
            }} else {{
                'NO_IMAGE'
            }}",
            temp_path.to_string_lossy().replace("\\", "\\\\")
        );
        
        let output = std::process::Command::new("powershell.exe")
            .args(&["-Command", &ps_script])
            .output();
        
        if let Ok(output) = output {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if result == "OK" {
                return Ok(temp_path.to_string_lossy().to_string());
            }
        }
    }
    
    Err("System clipboard command failed".to_string())
}

#[tauri::command]
pub fn paste_image_from_clipboard_raw<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    temp_dir: String,
) -> Result<String, String> {
    // 首先尝试使用 clipboard-rs 读取文件（保留 GIF 动画）
    let ctx = ClipboardContext::new().map_err(|e| {
        format!("Failed to create clipboard context: {}", e)
    })?;
    
    // 尝试获取剪贴板中的文件路径
    let files = ctx.get_files().map_err(|e| {
        format!("Failed to read files from clipboard: {}", e)
    })?;
    
    if !files.is_empty() {
        // 复制第一个文件到临时目录
        let src_path = std::path::Path::new(&files[0]);
        if !src_path.exists() {
            return Err("剪贴板中的文件不存在".to_string());
        }
        
        // 检测文件类型
        let extension = src_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();
        
        // 支持 GIF 动画
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        
        let temp_file_name = format!("pasted-{}.{}", timestamp, extension);
        let temp_path = std::path::Path::new(&temp_dir).join(&temp_file_name);
        
        // 确保临时目录存在
        if let Some(parent) = temp_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create temp dir: {}", e))?;
        }
        
        // 复制文件（保留原始格式和动画）
        std::fs::copy(src_path, &temp_path).map_err(|e| {
            format!("Failed to copy file: {}", e)
        })?;
        
        return Ok(temp_path.to_string_lossy().to_string());
    }
    
    // 尝试使用系统命令读取剪贴板图片
    match read_clipboard_with_system_command(&temp_dir) {
        Ok(path) => return Ok(path),
        Err(_) => {
            // 系统命令失败，尝试 Tauri 方式
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
                        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create temp dir: {}", e))?;
                    }
                    
                    let buffer = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba_data)
                        .ok_or("Failed to create image buffer")?;
                    
                    buffer.save(&temp_path).map_err(|e| format!("Failed to save image: {}", e))?;
                    
                    Ok(temp_path.to_string_lossy().to_string())
                }
                Err(e) => Err(format!("剪贴板中没有图片或读取失败: {}", e)),
            }
        }
    }
}


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
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    Ok(images)
}

#[tauri::command]
pub fn search_images(keyword: String, pinyin: bool, acronym: bool) -> Result<Vec<Image>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    let search_pattern = format!("%{}%", keyword);
    
    // 根据配置构建不同的查询
    let query = if pinyin && acronym {
        "SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id 
         FROM images i
         JOIN keyword_group_links kgl ON i.group_id = kgl.group_id
         JOIN keywords k ON kgl.keyword_id = k.id
         WHERE k.keyword LIKE ? OR k.pinyin LIKE ? OR k.acronym LIKE ?
         ORDER BY i.share_count DESC"
    } else if pinyin {
        "SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id 
         FROM images i
         JOIN keyword_group_links kgl ON i.group_id = kgl.group_id
         JOIN keywords k ON kgl.keyword_id = k.id
         WHERE k.keyword LIKE ? OR k.pinyin LIKE ?
         ORDER BY i.share_count DESC"
    } else if acronym {
        "SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id 
         FROM images i
         JOIN keyword_group_links kgl ON i.group_id = kgl.group_id
         JOIN keywords k ON kgl.keyword_id = k.id
         WHERE k.keyword LIKE ? OR k.acronym LIKE ?
         ORDER BY i.share_count DESC"
    } else {
        "SELECT DISTINCT i.id, i.image_path, i.thumbnail_path, i.share_count, i.group_id, i.mode_id 
         FROM images i
         JOIN keyword_group_links kgl ON i.group_id = kgl.group_id
         JOIN keywords k ON kgl.keyword_id = k.id
         WHERE k.keyword LIKE ?
         ORDER BY i.share_count DESC"
    };
    
    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
    
    let images = if pinyin && acronym {
        stmt.query_map(
            params![search_pattern, search_pattern, search_pattern],
            |row| {
                Ok(Image {
                    id: row.get(0)?,
                    image_path: row.get(1)?,
                    thumbnail_path: row.get(2)?,
                    share_count: row.get(3)?,
                    group_id: row.get(4)?,
                    mode_id: row.get(5)?,
                })
            }
        ).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    } else if pinyin {
        stmt.query_map(
            params![search_pattern, search_pattern],
            |row| {
                Ok(Image {
                    id: row.get(0)?,
                    image_path: row.get(1)?,
                    thumbnail_path: row.get(2)?,
                    share_count: row.get(3)?,
                    group_id: row.get(4)?,
                    mode_id: row.get(5)?,
                })
            }
        ).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    } else if acronym {
        stmt.query_map(
            params![search_pattern, search_pattern],
            |row| {
                Ok(Image {
                    id: row.get(0)?,
                    image_path: row.get(1)?,
                    thumbnail_path: row.get(2)?,
                    share_count: row.get(3)?,
                    group_id: row.get(4)?,
                    mode_id: row.get(5)?,
                })
            }
        ).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    } else {
        stmt.query_map(
            params![search_pattern],
            |row| {
                Ok(Image {
                    id: row.get(0)?,
                    image_path: row.get(1)?,
                    thumbnail_path: row.get(2)?,
                    share_count: row.get(3)?,
                    group_id: row.get(4)?,
                    mode_id: row.get(5)?,
                })
            }
        ).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    };
    
    Ok(images)
}

/// 内部函数：增加分享次数
fn increment_share_count_internal(conn: &rusqlite::Connection, image_id: i32, group_id: Option<i32>) -> Result<(), String> {
    // 更新图片分享次数
    conn.execute(
        "UPDATE images SET share_count = share_count + 1 WHERE id = ?",
        params![image_id],
    ).map_err(|e| e.to_string())?;
    
    // 如果提供了 group_id，更新分组分享次数
    if let Some(gid) = group_id {
        conn.execute(
            "UPDATE groups SET share_count = share_count + 1 WHERE id = ?",
            params![gid],
        ).map_err(|e| e.to_string())?;
    } else {
        // 否则查询分组ID并更新
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

#[tauri::command]
pub fn share_image(image_id: i32) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    increment_share_count_internal(&conn, image_id, None)
}

#[tauri::command]
pub fn copy_image(image_id: i32) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 获取图片路径
    let image_path: String = conn.query_row(
        "SELECT image_path FROM images WHERE id = ?",
        params![image_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    // 增加分享次数
    increment_share_count_internal(&conn, image_id, None)?;
    
    // 使用 clipboard-rs 复制文件到剪贴板（保留GIF动画）
    use clipboard_rs::{Clipboard, ClipboardContext};
    
    let ctx = ClipboardContext::new().map_err(|e| {
        format!("Failed to create clipboard context: {}", e)
    })?;
    
    // 将文件路径转换为绝对路径
    let abs_path = std::fs::canonicalize(&image_path).map_err(|e| {
        format!("Failed to get absolute path: {}", e)
    })?;
    
    // 复制文件到剪贴板（支持GIF动图）
    ctx.set_files(vec![abs_path.to_string_lossy().to_string()]).map_err(|e| {
        format!("Failed to copy file to clipboard: {}", e)
    })?;
    
    Ok(())
}

#[tauri::command]
pub fn increment_share_count(image_id: i32, group_id: Option<i32>) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    increment_share_count_internal(&conn, image_id, group_id)
}

#[tauri::command]
pub fn copy_images(image_ids: Vec<i32>) -> Result<(), String> {
    if image_ids.is_empty() {
        return Ok(());
    }
    
    for image_id in &image_ids {
        // 增加分享次数
        let _ = copy_image(*image_id);
    }
    
    Ok(())
}

#[tauri::command]
pub fn delete_images(image_ids: Vec<i32>) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    for image_id in image_ids {
        // 先获取图片路径以便删除文件
        if let Ok((image_path, thumb_path)) = conn.query_row(
            "SELECT image_path, thumbnail_path FROM images WHERE id = ?",
            params![image_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        ) {
            // 删除文件
            let _ = std::fs::remove_file(&image_path);
            if let Some(thumb) = thumb_path {
                let _ = std::fs::remove_file(&thumb);
            }
        }
        
        // 删除数据库记录
        conn.execute("DELETE FROM images WHERE id = ?", params![image_id])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn move_images(image_ids: Vec<i32>, target_group_id: i32) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 获取目标分组的mode_id
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

#[tauri::command]
pub fn upload_images(file_paths: Vec<String>, group_id: i32, mode_id: i32, _meme_dir: String) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 获取分组信息
    let (_group_name, group_folder): (String, String) = conn.query_row(
        "SELECT name, folder_path FROM groups WHERE id = ?",
        params![group_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).map_err(|e| e.to_string())?;
    
    for file_path in &file_paths {
        let src_path = std::path::Path::new(file_path);
        if !src_path.exists() {
            continue;
        }
        
        let file_name = src_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.png");
        
        let dest_path = std::path::Path::new(&group_folder).join(file_name);
        
        // 复制文件到分组文件夹
        std::fs::copy(src_path, &dest_path).map_err(|e| {
            format!("Failed to copy file {}: {}", file_path, e)
        })?;
        
        let dest_path_str = dest_path.to_string_lossy().to_string();
        
        // 检查图片是否已存在
        let existing_image: Option<i32> = conn.query_row(
            "SELECT id FROM images WHERE image_path = ?",
            params![dest_path_str],
            |row| row.get(0)
        ).ok();
                
        if existing_image.is_none() {
            // 新增图片记录
            conn.execute(
                "INSERT INTO images (image_path, thumbnail_path, share_count, group_id, mode_id) VALUES (?, NULL, 0, ?, ?)",
                params![dest_path_str, group_id, mode_id]
            ).map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

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

#[tauri::command]
pub fn share_image_to_app(image_id: i32, _app: String) -> Result<(), String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 获取图片路径
    let image_path: String = conn.query_row(
        "SELECT image_path FROM images WHERE id = ?",
        params![image_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    // 增加分享次数
    increment_share_count_internal(&conn, image_id, None)?;
    
    // 使用系统默认应用打开图片
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
    
    Ok(())
}

/// 内部函数：删除无效图片记录
fn delete_invalid_images(conn: &rusqlite::Connection) -> Result<(), String> {
    let mut stmt = conn.prepare("SELECT id, image_path FROM images")
        .map_err(|e| e.to_string())?;
    let images: Vec<(i32, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    let mut invalid_image_ids: Vec<i32> = Vec::new();
    for (id, image_path) in &images {
        if !std::path::Path::new(image_path).exists() {
            invalid_image_ids.push(*id);
        }
    }
    
    // 批量删除无效图片
    if !invalid_image_ids.is_empty() {
        for id in &invalid_image_ids {
            conn.execute("DELETE FROM images WHERE id = ?", params![id])
                .map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

/// 内部函数：删除无效分组
fn delete_invalid_groups(conn: &rusqlite::Connection) -> Result<(), String> {
    let mut stmt = conn.prepare("SELECT id, folder_path FROM groups")
        .map_err(|e| e.to_string())?;
    let groups: Vec<(i32, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    let mut invalid_group_ids: Vec<i32> = Vec::new();
    for (id, folder_path) in &groups {
        if !std::path::Path::new(folder_path).exists() {
            invalid_group_ids.push(*id);
        }
    }
    
    // 批量删除无效分组及其关联
    if !invalid_group_ids.is_empty() {
        for id in &invalid_group_ids {
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

/// 内部函数：删除无效模式
fn delete_invalid_modes(conn: &rusqlite::Connection) -> Result<(), String> {
    let mut stmt = conn.prepare("SELECT id, folder_path FROM modes")
        .map_err(|e| e.to_string())?;
    let modes: Vec<(i32, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    let mut invalid_mode_ids: Vec<i32> = Vec::new();
    for (id, folder_path) in &modes {
        if !std::path::Path::new(folder_path).exists() {
            invalid_mode_ids.push(*id);
        }
    }
    
    // 批量删除无效模式
    // keyword_mode_links 已废弃，关键词是全局的，通过 keyword_group_links 关联到分组
    if !invalid_mode_ids.is_empty() {
        for id in &invalid_mode_ids {
            conn.execute("DELETE FROM groups WHERE mode_id = ?", params![id])
                .map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM modes WHERE id = ?", params![id])
                .map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

/// 内部函数：扫描并新增模式、分组和图片
fn scan_and_add_items(conn: &rusqlite::Connection, meme_dir: &String) -> Result<(), String> {
    let path = PathBuf::from(meme_dir);
    
    // 扫描文件系统，批量新增模式
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
        
        // 检查模式是否已存在
        let existing_mode_id: Option<i32> = conn.query_row(
            "SELECT id FROM modes WHERE folder_path = ?",
            params![mode_path.to_string_lossy()],
            |row| row.get(0)
        ).ok();
        
        let mode_id = if let Some(id) = existing_mode_id {
            id
        } else {
            // 新增模式
            conn.execute(
                "INSERT INTO modes (name, folder_path, sort_order) VALUES (?, ?, 0)",
                params![mode_name, mode_path.to_string_lossy()]
            ).map_err(|e| e.to_string())?;
            
            conn.query_row(
                "SELECT last_insert_rowid()",
                [],
                |row| row.get(0)
            ).map_err(|e| e.to_string())?
        };
        
        // 扫描分组文件夹
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
            
            // 检查分组是否已存在
            let existing_group_id: Option<i32> = conn.query_row(
                "SELECT id FROM groups WHERE folder_path = ?",
                params![group_path.to_string_lossy()],
                |row| row.get(0)
            ).ok();
            
            let group_id = if let Some(id) = existing_group_id {
                id
            } else {
                // 新增分组
                conn.execute(
                    "INSERT INTO groups (name, folder_path, mode_id, share_count) VALUES (?, ?, ?, 0)",
                    params![group_name, group_path.to_string_lossy(), mode_id]
                ).map_err(|e| e.to_string())?;
                
                conn.query_row(
                    "SELECT last_insert_rowid()",
                    [],
                    |row| row.get(0)
                ).map_err(|e| e.to_string())?
            };
            
            // 扫描图片文件
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
                
                if !["jpg", "jpeg", "png", "gif", "webp", "bmp"].contains(&extension.as_str()) {
                    continue;
                }
                
                let image_path_str = image_path.to_string_lossy().to_string();
                
                // 检查图片是否已存在
                let existing_image: Option<i32> = conn.query_row(
                    "SELECT id FROM images WHERE image_path = ?",
                    params![image_path_str],
                    |row| row.get(0)
                ).ok();
                
                if existing_image.is_none() {
                    // 新增图片记录
                    conn.execute(
                        "INSERT INTO images (image_path, thumbnail_path, share_count, group_id, mode_id) VALUES (?, NULL, 0, ?, ?)",
                        params![image_path_str, group_id, mode_id]
                    ).map_err(|e| e.to_string())?;
                }
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn refresh_index(meme_dir: String) -> Result<(), String> {
    if meme_dir.is_empty() {
        return Err("表情包目录未设置".to_string());
    }
    
    let path = PathBuf::from(&meme_dir);
    if !path.exists() {
        return Err("表情包目录不存在".to_string());
    }
    
    let conn = init_db().map_err(|e| e.to_string())?;
    
    // 1. 批量删除无效图片记录
    delete_invalid_images(&conn)?;
    
    // 2. 批量删除无效分组（文件夹不存在的分组）
    delete_invalid_groups(&conn)?;
    
    // 3. 批量删除无效模式（文件夹不存在的模式）
    delete_invalid_modes(&conn)?;
    
    // 4. 扫描文件系统，批量新增模式
    scan_and_add_items(&conn, &meme_dir)?;
    
    Ok(())
}