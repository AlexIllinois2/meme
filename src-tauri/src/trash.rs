//! 回收站模块 - 处理文件/文件夹的软删除
//! 将删除的文件移动到 ~/.local/share/meme/trash/ 目录

use std::path::{Path, PathBuf};
use std::fs;
use crate::db::get_app_data_dir;

/// 获取回收站目录路径
pub fn get_trash_dir() -> PathBuf {
    get_app_data_dir().join("trash")
}

/// 确保回收站目录存在
fn ensure_trash_dir() -> Result<PathBuf, String> {
    let trash_dir = get_trash_dir();
    fs::create_dir_all(&trash_dir)
        .map_err(|e| format!("创建回收站目录失败: {}", e))?;
    Ok(trash_dir)
}

/// 生成唯一的回收站目标路径
/// 如果目标已存在，添加时间戳后缀
fn generate_trash_target(original_path: &Path) -> Result<PathBuf, String> {
    let trash_dir = ensure_trash_dir()?;
    
    let file_name = original_path
        .file_name()
        .ok_or("无法获取文件/文件夹名称")?
        .to_string_lossy();
    
    // 尝试使用原始名称
    let target = trash_dir.join(&*file_name);
    
    if !target.exists() {
        return Ok(target);
    }
    
    // 已存在，添加时间戳
    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    
    // 分离文件名和扩展名（如果是文件）
    let (stem, ext) = if let Some(ext) = original_path.extension() {
        let stem = original_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        (stem.to_string(), format!(".{}", ext.to_string_lossy()))
    } else {
        (file_name.to_string(), String::new())
    };
    
    let new_name = format!("{}_{}{}", stem, timestamp, ext);
    let target = trash_dir.join(new_name);
    
    if !target.exists() {
        return Ok(target);
    }
    
    // 极少数情况下仍然冲突，添加随机数
    let new_name = format!("{}_{}_{}{}", stem, timestamp, rand::random::<u16>(), ext);
    Ok(trash_dir.join(new_name))
}

/// 移动文件或文件夹到回收站
/// 
/// # Arguments
/// * `source_path` - 要删除的文件/文件夹路径
/// 
/// # Returns
/// * `Ok(())` - 移动成功
/// * `Err(String)` - 移动失败，返回错误信息
pub fn move_to_trash(source_path: &str) -> Result<(), String> {
    let source = Path::new(source_path);
    
    if !source.exists() {
        // 源文件不存在，可能是之前已经被删除或从未创建
        // 这不算是错误，直接返回成功
        return Ok(());
    }
    
    let target = generate_trash_target(source)?;
    
    // 执行移动
    fs::rename(source, &target)
        .map_err(|e| format!("移动到回收站失败: {} -> {} ({})", source_path, target.display(), e))?;
    
    Ok(())
}

/// 清空回收站（可选功能）
pub fn empty_trash() -> Result<usize, String> {
    let trash_dir = get_trash_dir();
    
    if !trash_dir.exists() {
        return Ok(0);
    }
    
    let mut count = 0;
    for entry in fs::read_dir(&trash_dir)
        .map_err(|e| format!("读取回收站失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            fs::remove_dir_all(&path)
                .map_err(|e| format!("删除文件夹失败 {}: {}", path.display(), e))?;
        } else {
            fs::remove_file(&path)
                .map_err(|e| format!("删除文件失败 {}: {}", path.display(), e))?;
        }
        count += 1;
    }
    
    Ok(count)
}

/// 获取回收站中的项目列表（可选功能）
pub fn list_trash() -> Result<Vec<(String, String, u64)>, String> {
    let trash_dir = get_trash_dir();
    
    if !trash_dir.exists() {
        return Ok(vec![]);
    }
    
    let mut items = Vec::new();
    for entry in fs::read_dir(&trash_dir)
        .map_err(|e| format!("读取回收站失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let path = entry.path();
        
        let name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let item_type = if path.is_dir() { "folder" } else { "file" }.to_string();
        
        let size = if path.is_dir() {
            0 // 文件夹大小计算较复杂，暂时返回0
        } else {
            fs::metadata(&path)
                .map(|m| m.len())
                .unwrap_or(0)
        };
        
        items.push((name, item_type, size));
    }
    
    Ok(items)
}