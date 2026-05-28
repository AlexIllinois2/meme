use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use tauri::Emitter;
use tauri::Manager;
use crate::error::AppError;

#[tauri::command]
pub async fn save_image_to_gallery(
    app_handle: tauri::AppHandle,
    image_data: String,
    filename: String,
) -> Result<String, AppError> {
    let bytes = BASE64
        .decode(&image_data)
        .map_err(|e| AppError(format!("Base64 解码失败: {}", e)))?;

    // 路径安全：只取 filename 的最后一个组件，防止路径遍历
    let safe_filename = std::path::Path::new(&filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.png")
        .to_string();

    #[cfg(not(target_os = "android"))]
    {
        let save_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("saved_images");

        std::fs::create_dir_all(&save_dir)
            .map_err(|e| AppError(format!("创建目录失败: {}", e)))?;

        let file_path = save_dir.join(&safe_filename);
        std::fs::write(&file_path, &bytes)
            .map_err(|e| AppError(format!("写入失败: {}", e)))?;

        Ok(file_path.to_string_lossy().to_string())
    }

    #[cfg(target_os = "android")]
    {
        let cache_dir = app_handle
            .path()
            .app_cache_dir()
            .unwrap_or_else(|_| std::env::temp_dir());

        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| AppError(format!("创建缓存目录失败: {}", e)))?;

        let temp_path = cache_dir.join(&safe_filename);
        std::fs::write(&temp_path, &bytes)
            .map_err(|e| AppError(format!("写入临时文件失败: {}", e)))?;

        let path_str = temp_path.to_string_lossy().to_string();
        let display_name = safe_filename.trim_end_matches(|c| c == '.');

        app_handle
            .emit("saveImageToGallery", serde_json::json!({
                "path": &path_str,
                "displayName": display_name,
            }))
            .map_err(|e| AppError(format!("发送事件失败: {}", e)))?;

        Ok(path_str)
    }
}
