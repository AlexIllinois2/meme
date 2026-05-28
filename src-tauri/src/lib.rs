// Meme Manager - Tauri 后端入口
// 模块化架构：所有业务逻辑已拆分到独立模块

// 声明模块
mod models;
mod db;
mod config;
mod mode;
mod group;
mod keyword;
mod image;
mod custom_share;
mod meme_fs;
mod save_image;
#[cfg(target_os = "android")]
mod android_picker;

// 导出必要的类型和函数供其他模块使用
pub use db::init_db;
pub use models::*;

// 导入 Emitter trait 以使用 emit 方法
use tauri::Emitter;

// 悬浮窗触发搜索聚焦的命令 - Android 平台
#[cfg(target_os = "android")]
#[tauri::command]
fn trigger_search_focus(app_handle: tauri::AppHandle) -> Result<(), String> {
    // 发送事件到前端
    app_handle.emit("triggerSearchFocus", ()).map_err(|e| e.to_string())?;
    Ok(())
}

// 退出应用命令
#[tauri::command]
fn exit_app(app_handle: tauri::AppHandle) {
    std::process::exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init());
    
    // Android 平台添加 share 插件
    #[cfg(target_os = "android")]
    {
        builder = builder.plugin(tauri_plugin_share::init());
    }
    
    // Android 平台添加 android-fs 插件
    #[cfg(target_os = "android")]
    {
        builder = builder.plugin(tauri_plugin_android_fs::init());
    }
    
    // 统一 invoke_handler（平台差异通过 cfg 属性在项级别处理）
    builder = builder.invoke_handler(tauri::generate_handler![
        // 配置
        config::get_config,
        config::update_config,
        // 模式
        mode::get_modes,
        mode::add_mode,
        mode::update_mode,
        mode::delete_mode,
        mode::delete_modes,
        // 分组
        group::get_groups_by_mode,
        group::get_all_groups,
        group::get_all_groups_simple,
        group::add_group,
        group::update_group,
        group::delete_group,
        group::delete_groups,
        group::search_groups,
        // 关键词
        keyword::generate_keywords_file,
        keyword::sync_keywords_to_file,
        // 分组名关键词管理（跨所有模式）
        keyword::get_keywords_by_group_name,
        keyword::remove_keyword_from_group_name,
        keyword::add_keyword_to_group_name,
        // 分组关键词重建
        group::rebuild_all_group_keywords,
        // 图片
        image::get_images_by_group,
        image::search_images,
        image::share_image,
        image::copy_image,
        image::delete_images,
        image::move_images,
        image::upload_images,
        image::get_image_full_path,
        image::share_image_to_app,
        image::increment_share_count,
        image::copy_images,
        // 剪贴板粘贴（桌面端支持，Android 返回错误）
        image::paste_image_from_clipboard,
        image::paste_image_from_clipboard_raw,
        // 应用控制
        exit_app,
        // 索引刷新
        image::refresh_index,
        image::full_refresh,
        // Android 图片上传（Base64）
        image::upload_images_android,
        // 存储权限检查
        meme_fs::check_storage_accessible,
        // 自定义分享应用
        custom_share::get_custom_share_apps,
        custom_share::add_custom_share_app,
        custom_share::remove_custom_share_app,
        // 保存图片到相册
        save_image::save_image_to_gallery,
        // Android 平台专有命令
        #[cfg(target_os = "android")]
        android_picker::select_directory_android,
        #[cfg(target_os = "android")]
        trigger_search_focus,
    ]);
    
    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
