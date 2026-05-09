// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// fn main() {
//     meme_lib::run()
// }


use tauri::webview::WebviewWindowBuilder;
use wry::WebContext;
use std::path::PathBuf;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 获取应用数据目录（安卓上自动指向应用私有目录）
            let app_dir = app.path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            
            // WebView 持久化目录
            let web_context_dir = app_dir.join("webview_data");
            std::fs::create_dir_all(&web_context_dir).ok();
            
            // 创建持久化 WebContext
            let web_context = WebContext::new(Some(web_context_dir));
            
            let _window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into())
            )
            .web_context(web_context)  // 使用持久化上下文
            .build()?;
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error");
}