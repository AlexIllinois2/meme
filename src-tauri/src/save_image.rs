use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use tauri::Emitter;
use tauri::Manager;

#[tauri::command]
pub async fn save_image_to_gallery(
    app_handle: tauri::AppHandle,
    image_data: String,
    filename: String,
) -> Result<String, String> {
    let bytes = BASE64
        .decode(&image_data)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    #[cfg(not(target_os = "android"))]
    {
        let save_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("saved_images");

        std::fs::create_dir_all(&save_dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;

        let file_path = save_dir.join(&filename);
        std::fs::write(&file_path, &bytes)
            .map_err(|e| format!("写入失败: {}", e))?;

        Ok(file_path.to_string_lossy().to_string())
    }

    #[cfg(target_os = "android")]
    {
        let cache_dir = app_handle
            .path()
            .app_cache_dir()
            .unwrap_or_else(|_| std::env::temp_dir());

        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("创建缓存目录失败: {}", e))?;

        let temp_path = cache_dir.join(&filename);
        std::fs::write(&temp_path, &bytes)
            .map_err(|e| format!("写入临时文件失败: {}", e))?;

        let path_str = temp_path.to_string_lossy().to_string();
        let display_name = filename.trim_end_matches(|c| c == '.');

        app_handle
            .emit("saveImageToGallery", serde_json::json!({
                "path": &path_str,
                "displayName": display_name,
            }))
            .map_err(|e| format!("发送事件失败: {}", e))?;

        Ok(path_str)
    }
}