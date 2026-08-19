//! 表情图(sticker)生成与缓存模块
//!
//! 懒加载策略：第一次分享某张图片时才用原图生成一张标准表情图
//! (GIF, 240x240, 256 色调色板压缩)，存入 images 表的 `sticker_data`(BLOB) 字段，
//! 并写出磁盘缓存 `<meme_dir>/.sticker_cache/<id>.gif` 供复制/分享使用。
//! 后续分享直接复用缓存，不再读取或处理原图。

use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::imageops;
use image::AnimationDecoder;
use image::GenericImageView;
use rusqlite::params;

use crate::core::error::AppError;
use crate::core::meme_fs;

const STICKER_SIZE: u32 = 240;
/// 动画帧上限：兼顾流畅度与体积(目标 < 1MB)
const MAX_FRAMES: usize = 50;
const CACHE_DIR_NAME: &str = ".sticker_cache";

/// 将图片等比缩放并居中铺在 `STICKER_SIZE x STICKER_SIZE` 的透明画布上
fn fit_and_pad(img: &image::DynamicImage) -> image::DynamicImage {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return image::DynamicImage::new_rgba8(STICKER_SIZE, STICKER_SIZE);
    }
    let scale = (STICKER_SIZE as f32 / w as f32).min(STICKER_SIZE as f32 / h as f32);
    let nw = ((w as f32 * scale).round().max(1.0)) as u32;
    let nh = ((h as f32 * scale).round().max(1.0)) as u32;
    let resized = img.resize(nw, nh, imageops::FilterType::Lanczos3);
    let mut canvas = image::DynamicImage::new_rgba8(STICKER_SIZE, STICKER_SIZE); // 全透明
    let x = ((STICKER_SIZE - nw) / 2) as i64;
    let y = ((STICKER_SIZE - nh) / 2) as i64;
    imageops::overlay(&mut canvas, &resized, x, y);
    canvas
}

/// 由原图字节生成标准表情图 GIF 字节
fn generate_sticker_bytes(src_bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    let is_gif = src_bytes.starts_with(b"GIF89a") || src_bytes.starts_with(b"GIF87a");

    let mut out: Vec<u8> = Vec::new();
    {
        let mut encoder = gif::Encoder::new(
            &mut out,
            STICKER_SIZE as u16,
            STICKER_SIZE as u16,
            &[],
        )
        .map_err(|e| AppError(format!("创建 GIF 编码器失败: {}", e)))?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| AppError(format!("设置 GIF 循环失败: {}", e)))?;

        if is_gif {
            if let Ok(decoder) =
                image::codecs::gif::GifDecoder::new(Cursor::new(src_bytes.to_vec()))
            {
                let frames = decoder.into_frames();
                let mut count = 0usize;
                for frame_res in frames {
                    if count >= MAX_FRAMES {
                        break;
                    }
                    let frame = match frame_res {
                        Ok(f) => f,
                        Err(_) => break,
                    };
                    let delay = frame.delay();
                    let (n, d) = delay.numer_denom_ms();
                    let ms = if d > 0 { (n as f64) / (d as f64) } else { 100.0 };
                    // GIF 帧延迟单位为 10ms(即厘秒)，下限 2(=20ms) 避免播放过快
                    let cs = (ms / 10.0).round().max(2.0) as u16;

                    let buf = frame.into_buffer();
                    let dyn_img = image::DynamicImage::ImageRgba8(buf);
                    let padded = fit_and_pad(&dyn_img);
                    let mut raw = padded.into_rgba8().into_raw();
                    let mut gif_frame =
                        gif::Frame::from_rgba(STICKER_SIZE as u16, STICKER_SIZE as u16, &mut raw);
                    gif_frame.delay = cs;
                    encoder
                        .write_frame(&gif_frame)
                        .map_err(|e| AppError(format!("写入 GIF 帧失败: {}", e)))?;
                    count += 1;
                }

                // 动画解码异常(0 帧)时退化为静态图
                if count == 0 {
                    let img = image::load_from_memory(src_bytes)
                        .map_err(|e| AppError(format!("解码图片失败: {}", e)))?;
                    let padded = fit_and_pad(&img);
                    let mut raw = padded.into_rgba8().into_raw();
                    let gif_frame =
                        gif::Frame::from_rgba(STICKER_SIZE as u16, STICKER_SIZE as u16, &mut raw);
                    encoder
                        .write_frame(&gif_frame)
                        .map_err(|e| AppError(format!("写入 GIF 帧失败: {}", e)))?;
                }
            } else {
                // GIF 头但解码失败，按静态处理
                let img = image::load_from_memory(src_bytes)
                    .map_err(|e| AppError(format!("解码图片失败: {}", e)))?;
                let padded = fit_and_pad(&img);
                let mut raw = padded.into_rgba8().into_raw();
                let gif_frame =
                    gif::Frame::from_rgba(STICKER_SIZE as u16, STICKER_SIZE as u16, &mut raw);
                encoder
                    .write_frame(&gif_frame)
                    .map_err(|e| AppError(format!("写入 GIF 帧失败: {}", e)))?;
            }
        } else {
            let img = image::load_from_memory(src_bytes)
                .map_err(|e| AppError(format!("解码图片失败: {}", e)))?;
            let padded = fit_and_pad(&img);
            let mut raw = padded.into_rgba8().into_raw();
            let gif_frame =
                gif::Frame::from_rgba(STICKER_SIZE as u16, STICKER_SIZE as u16, &mut raw);
            encoder
                .write_frame(&gif_frame)
                .map_err(|e| AppError(format!("写入 GIF 帧失败: {}", e)))?;
        }
    }

    Ok(out)
}

/// 确保某图片的表情图已生成，返回其磁盘缓存路径。
///
/// 优先复用已有缓存/BLOB；均不存在时执行懒生成(第一次分享)。
/// 若原图格式无法解码(如 webp 未启用解码)，则回退返回原图路径，分享仍可用。
pub fn ensure_sticker_path(
    conn: &rusqlite::Connection,
    image_id: i32,
    meme_dir: &str,
) -> Result<PathBuf, AppError> {
    let (sticker_blob, raw_path): (Option<Vec<u8>>, String) = conn
        .query_row(
            "SELECT sticker_data, image_path FROM images WHERE id = ?",
            params![image_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| AppError(format!("查询图片失败: {}", e)))?;

    let cache_dir = Path::new(meme_dir).join(CACHE_DIR_NAME);
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| AppError(format!("创建表情图缓存目录失败: {}", e)))?;
    let cache_path = cache_dir.join(format!("{}.gif", image_id));

    // 快速路径：缓存文件已存在，直接复用
    if cache_path.exists() {
        return Ok(cache_path);
    }

    // 已有 BLOB：写出即可
    if let Some(blob) = sticker_blob {
        std::fs::write(&cache_path, &blob)
            .map_err(|e| AppError(format!("写出表情图缓存失败: {}", e)))?;
        return Ok(cache_path);
    }

    // 没有 BLOB：懒生成(第一次分享时)
    let src_path = meme_fs::resolve_meme_path(meme_dir, &raw_path);
    if !src_path.exists() {
        return Err(AppError(format!("原图不存在: {}", src_path.display())));
    }
    let src_bytes =
        std::fs::read(&src_path).map_err(|e| AppError(format!("读取原图失败: {}", e)))?;

    match generate_sticker_bytes(&src_bytes) {
        Ok(gif_bytes) => {
            conn.execute(
                "UPDATE images SET sticker_data = ? WHERE id = ?",
                params![gif_bytes, image_id],
            )
            .map_err(|e| AppError(format!("保存表情图失败: {}", e)))?;
            std::fs::write(&cache_path, &gif_bytes)
                .map_err(|e| AppError(format!("写出表情图缓存失败: {}", e)))?;
            Ok(cache_path)
        }
        Err(e) => {
            // 解码失败(如 webp)，回退使用原图，保证分享可用
            log::warn!("表情图生成失败，回退使用原图 {}: {}", image_id, e);
            Ok(src_path)
        }
    }
}

/// 获取某图片的表情图磁盘路径(供前端复制/分享使用)
///
/// 该命令只负责确保表情图存在并返回路径，不增加分享计数
/// (分享计数由 `share_image` 单独维护)。
#[tauri::command]
pub fn get_sticker_path(
    state: tauri::State<'_, crate::core::db_state::DbState>,
    image_id: i32,
) -> Result<String, AppError> {
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    let meme_dir: String = conn
        .query_row("SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0))
        .unwrap_or_default();
    let path = ensure_sticker_path(&conn, image_id, &meme_dir)?;
    Ok(path.to_string_lossy().to_string())
}
