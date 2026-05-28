//! 全局数据库连接状态
//! 通过 Tauri 的 `app.manage()` 注入，避免每个命令都重复打开连接

use std::sync::{Mutex, MutexGuard, PoisonError};
use rusqlite::Connection;

pub struct DbState(pub Mutex<Connection>);

impl DbState {
    /// 获取数据库连接锁
    /// 配合 `tauri::State` 的 Deref 使用：`state.lock()`
    pub fn lock(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.0.lock().map_err(|e: PoisonError<_>| e.to_string())
    }
}
