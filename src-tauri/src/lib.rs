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
mod trash;

// 导出必要的类型和函数供其他模块使用
pub use db::init_db;
pub use models::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
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
            keyword::get_all_keywords,
            keyword::add_keyword,
            keyword::update_keyword,
            keyword::delete_keyword,
            keyword::delete_keywords,
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
            // 剪贴板粘贴
            image::paste_image_from_clipboard_raw,
            // 索引刷新
            image::refresh_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
