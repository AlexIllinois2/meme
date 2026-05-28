//! 统一的错误类型
//!
//! 消除了所有 `map_err(|e| e.to_string())` 样板代码。
//! 所有 Tauri 命令返回 `Result<_, AppError>` 即可自动转换。

use serde::Serialize;

#[derive(Debug)]
pub struct AppError(pub String);

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError(format!("数据库错误: {}", e))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError(format!("IO错误: {}", e))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError(format!("JSON错误: {}", e))
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(e: base64::DecodeError) -> Self {
        AppError(format!("Base64解码错误: {}", e))
    }
}

impl From<String> for AppError {
    fn from(e: String) -> Self {
        AppError(e)
    }
}

impl From<&str> for AppError {
    fn from(e: &str) -> Self {
        AppError(e.to_string())
    }
}

// Tauri 要求错误实现 Serialize
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}
