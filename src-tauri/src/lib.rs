// Meme Manager - Tauri 后端入口
// 模块化架构：所有业务逻辑已拆分到独立模块

// 声明模块（子目录）
pub mod core;
pub mod commands;

// 导出必要的类型和函数供其他模块使用
pub use core::models::*;
pub use core::error::AppError;
pub use core::db_state::DbState;

#[cfg(target_os = "android")]
use tauri::Emitter;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 应用启动时初始化一次数据库连接（全局共享）
    let db_conn = core::db::init_db().expect("Failed to initialize database");

    let mut builder = tauri::Builder::default()
        .manage(DbState(std::sync::Mutex::new(db_conn)))
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
        core::config::get_config,
        core::config::update_config,
        // 模式
        commands::mode::get_modes,
        commands::mode::add_mode,
        commands::mode::update_mode,
        commands::mode::delete_mode,
        commands::mode::delete_modes,
        // 分组
        commands::group::get_groups_by_mode,
        commands::group::get_all_groups,
        commands::group::get_all_groups_simple,
        commands::group::add_group,
        commands::group::update_group,
        commands::group::delete_group,
        commands::group::delete_groups,
        commands::group::search_groups,
        // 关键词
        commands::keyword::generate_keywords_file,
        commands::keyword::sync_keywords_to_file,
        // 分组名关键词管理（跨所有模式）
        commands::keyword::get_keywords_by_group_name,
        commands::keyword::remove_keyword_from_group_name,
        commands::keyword::add_keyword_to_group_name,
        // 分组关键词重建
        commands::group::rebuild_all_group_keywords,
        // 图片
        commands::image::get_images_by_group,
        commands::image::search_images,
        commands::image::share_image,
        commands::image::copy_image,
        commands::image::delete_images,
        commands::image::move_images,
        commands::upload::upload_images,
        commands::image::get_image_full_path,
        commands::image::share_image_to_app,
        commands::image::increment_share_count,
        commands::image::copy_images,
        // 剪贴板粘贴（桌面端支持，Android 返回错误）
        commands::clipboard::paste_image_from_clipboard,
        commands::clipboard::paste_image_from_clipboard_raw,
        // 应用控制
        exit_app,
        // 索引刷新
        commands::image::refresh_index,
        commands::image::full_refresh,
        // Android 图片上传（Base64）
        commands::upload::upload_images_android,
        // 存储权限检查
        core::meme_fs::check_storage_accessible,
        // 自定义分享应用
        commands::custom_share::get_custom_share_apps,
        commands::custom_share::add_custom_share_app,
        commands::custom_share::remove_custom_share_app,
        // 保存图片到相册
        commands::save_image::save_image_to_gallery,
        // 批量删除（合并三次 RPC）
        commands::batch_delete::batch_delete,
        // Android 平台专有命令
        #[cfg(target_os = "android")]
        commands::android_picker::select_directory_android,
        #[cfg(target_os = "android")]
        trigger_search_focus,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 退出应用命令
#[tauri::command]
fn exit_app(_app_handle: tauri::AppHandle) {
    std::process::exit(0);
}

// 悬浮窗触发搜索聚焦的命令 - Android 平台
#[cfg(target_os = "android")]
#[tauri::command]
fn trigger_search_focus(app_handle: tauri::AppHandle) -> Result<(), String> {
    app_handle.emit("triggerSearchFocus", ()).map_err(|e| e.to_string())?;
    Ok(())
}
