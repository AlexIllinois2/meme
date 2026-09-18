//! 核心基础设施
pub mod db;
pub mod error;
pub mod models;
pub mod config;
pub mod db_state;
pub mod meme_fs;
// 窗口状态持久化是桌面端概念，Android 无窗口尺寸/位置 API，不参与编译
#[cfg(not(target_os = "android"))]
pub mod window_state;
