use serde::{Deserialize, Serialize};

// ==================== 数据结构定义 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mode {
    pub id: i32,
    pub name: String,
    pub sort_order: i32,
    pub folder_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub folder_path: String,
    pub share_count: i32,
    pub mode_id: i32,
    pub mode_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    pub id: i32,
    pub image_path: String,
    pub thumbnail_path: Option<String>,
    pub share_count: i32,
    pub group_id: i32,
    pub mode_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub meme_dir: String,
    pub color_mode: String,
    pub theme_style: String,
    pub last_mode: i32,
    pub last_group: i32,
    pub share_app: String,
    pub grid_size: i32,
    pub pinyin_search: bool,
    pub acronym_search: bool,
    pub global_floating_window: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomShareApp {
    pub id: i32,
    pub package_name: String,
    pub app_name: Option<String>,
}
