// Android 平台专用的目录选择器
// 使用 tauri-plugin-android-fs 插件的原生能力

use crate::error::AppError;

/// Android 端选择目录
/// 使用 SAF (Storage Access Framework) 弹出系统原生目录选择器，
/// 并将返回的 SAF URI 解析为真实文件系统路径
#[tauri::command]
pub async fn select_directory_android(app_handle: tauri::AppHandle) -> Result<String, AppError> {
    use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, convert_dir_path_to_string};
    use percent_encoding::percent_decode_str;

    match app_handle.android_fs().show_open_dir_dialog() {
        Ok(Some(dir_path)) => {
            // DirPath 的字段是 pub(crate)，外部无法直接访问
            // 通过序列化 JSON 来提取 topTreeUri 和 relativeTerms
            let json_str = convert_dir_path_to_string(&dir_path)
                .map_err(|e| AppError(format!("序列化目录路径失败: {e}")))?;
            let json: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| AppError(format!("解析目录JSON失败: {e}")))?;

            let top_tree_uri = json["topTreeUri"]
                .as_str()
                .ok_or(AppError("无法获取 topTreeUri".into()))?;
            let relative_terms: Vec<String> = json["relativeTerms"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            // 从 SAF URI 中提取真实路径
            // URI 格式: content://com.android.externalstorage.documents/tree/primary%3Ameme
            let tree_part = top_tree_uri
                .rsplit("/tree/")
                .next()
                .ok_or(AppError(format!("无法解析 SAF URI: {top_tree_uri}")))?;

            // URL 解码 %3A → :, %2F → / 等
            let decoded = percent_decode_str(tree_part)
                .decode_utf8()
                .map_err(|e| AppError(format!("URL 解码失败: {e}")))?;

            // 解析 volume:path 格式
            // primary → /storage/emulated/0
            // XXXX-XXXX → /storage/XXXX-XXXX
            let (volume, path) = decoded
                .split_once(':')
                .ok_or(AppError(format!("无法解析路径段: {decoded}")))?;

            let base = if volume == "primary" {
                "/storage/emulated/0".to_string()
            } else {
                format!("/storage/{volume}")
            };

            let mut full_path = std::path::PathBuf::from(&base);
            if !path.is_empty() {
                full_path.push(path);
            }
            for term in &relative_terms {
                full_path.push(term);
            }

            Ok(full_path.to_string_lossy().to_string())
        }
        Ok(None) => Err(AppError("用户取消了选择".into())),
        Err(e) => Err(AppError(format!("选择目录失败: {e}"))),
    }
}
