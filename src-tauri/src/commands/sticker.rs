//! 表情图(sticker)生成与缓存模块
//!
//! 策略：表情图(GIF, 240x240, 256 色调色板压缩)在**后台异步生成**，不阻塞分享。
//! 首次分享某张图片时先直接发送原图，同时触发后台任务用原图生成标准表情图，
//! 存入 images 表的 `sticker_data`(BLOB) 字段并写出磁盘缓存
//! `<meme_dir>/.sticker_cache/<hash>.gif`；下次分享同一张图时表情图已就绪，
//! 直接复用缓存，不再读取或处理原图。
//!
//! 缓存文件名用图片**相对路径的哈希**而非自增 id：清除应用数据/重建索引后
//! id 会重新分配，按路径哈希命名可保证缓存仍能正确复用，也不会错配给别的图片。

use std::collections::HashSet;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

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

/// 根据数据库中的 `image_path`(相对路径)计算稳定的表情图缓存键。
///
/// 使用相对路径而非自增 id：用户清除应用数据或重建索引后 id 会重新分配，
/// 若以 id 命名缓存文件，旧缓存可能被错配给另一张图。相对路径在重扫/换库
/// 后保持不变，同一文件仍能命中同一缓存。
pub fn sticker_cache_key(image_path: &str) -> String {
    // FNV-1a 64位哈希(固定算法，跨平台/跨版本结果稳定)
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in image_path.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

/// 表情图缓存文件路径：`<meme_dir>/.sticker_cache/<key>.gif`
pub fn sticker_cache_path(meme_dir: &str, image_path: &str) -> PathBuf {
    Path::new(meme_dir)
        .join(CACHE_DIR_NAME)
        .join(format!("{}.gif", sticker_cache_key(image_path)))
}

/// 后台生成任务去重集合：同一图片同一时刻只保留一个生成任务
static PENDING_GENERATIONS: OnceLock<Mutex<HashSet<i32>>> = OnceLock::new();

/// 查询表情图是否已就绪(磁盘缓存或 DB BLOB)。
///
/// 已就绪时确保缓存文件存在并返回其路径；未就绪返回 `Ok(None)`（**不触发**生成），
/// 调用方应先分享原图，再通过 [`spawn_sticker_generation`] 在后台异步生成。
pub fn get_ready_sticker_path(
    conn: &rusqlite::Connection,
    image_id: i32,
    meme_dir: &str,
) -> Result<Option<PathBuf>, AppError> {
    let (sticker_blob, image_path): (Option<Vec<u8>>, String) = conn
        .query_row(
            "SELECT sticker_data, image_path FROM images WHERE id = ?",
            params![image_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| AppError(format!("查询图片失败: {}", e)))?;

    let cache_dir = Path::new(meme_dir).join(CACHE_DIR_NAME);
    // 文件名与图片相对路径绑定(而非自增 id)，文件存在即说明属于当前图片
    let cache_path = sticker_cache_path(meme_dir, &image_path);

    // 快速路径：缓存文件已存在，直接复用
    if cache_path.exists() {
        return Ok(Some(cache_path));
    }

    // 已有 BLOB：补写缓存文件后复用
    if let Some(blob) = sticker_blob {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| AppError(format!("创建表情图缓存目录失败: {}", e)))?;
        std::fs::write(&cache_path, &blob)
            .map_err(|e| AppError(format!("写出表情图缓存失败: {}", e)))?;
        return Ok(Some(cache_path));
    }

    Ok(None)
}

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
/// 优先复用已有缓存/BLOB；均不存在时执行生成(供后台生成任务与兼容调用使用)。
/// 若原图格式无法解码(如 webp 未启用解码)，则回退返回原图路径，分享仍可用。
pub fn ensure_sticker_path(
    conn: &rusqlite::Connection,
    image_id: i32,
    meme_dir: &str,
) -> Result<PathBuf, AppError> {
    // 已就绪(缓存文件或 BLOB)直接复用，不再处理原图
    if let Some(path) = get_ready_sticker_path(conn, image_id, meme_dir)? {
        return Ok(path);
    }

    // 没有缓存/BLOB：读取原图并生成
    let raw_path: String = conn
        .query_row(
            "SELECT image_path FROM images WHERE id = ?",
            params![image_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError(format!("查询图片失败: {}", e)))?;
    let cache_dir = Path::new(meme_dir).join(CACHE_DIR_NAME);
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| AppError(format!("创建表情图缓存目录失败: {}", e)))?;
    let cache_path = sticker_cache_path(meme_dir, &raw_path);
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

/// 后台异步生成某图片的表情图(立即返回，不阻塞调用方)。
///
/// 同一图片同一时刻只保留一个生成任务；生成结果写入 DB(`sticker_data`)与磁盘缓存，
/// 供下次复制/分享直接使用。任务失败仅记录日志，不影响本次已发出的原图。
pub fn spawn_sticker_generation(image_id: i32) {
    let pending = PENDING_GENERATIONS.get_or_init(|| Mutex::new(HashSet::new()));
    {
        let mut set = pending.lock().unwrap_or_else(|e| e.into_inner());
        if !set.insert(image_id) {
            return; // 已在生成中，跳过
        }
    }

    let spawned = std::thread::Builder::new()
        .name(format!("sticker-gen-{}", image_id))
        .spawn(move || {
            let result = generate_sticker_job(image_id);
            if let Err(e) = result {
                log::warn!("后台生成表情图失败 image_id={}: {}", image_id, e);
            }
            // 无论成败都解除去重，允许后续再次触发
            pending
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&image_id);
        });

    if let Err(e) = spawned {
        // 线程创建失败(极罕见)：解除去重，允许下次重试
        log::warn!("启动表情图后台生成线程失败 image_id={}: {}", image_id, e);
        pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&image_id);
    }
}

/// 后台生成任务：使用独立数据库连接执行生成，避免占用主连接/主线程。
fn generate_sticker_job(image_id: i32) -> Result<(), AppError> {
    let conn = rusqlite::Connection::open(crate::core::db::get_db_path())?;
    conn.busy_timeout(std::time::Duration::from_secs(15))?;
    let meme_dir: String = conn
        .query_row("SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0))
        .unwrap_or_default();
    if meme_dir.trim().is_empty() {
        return Err(AppError("未设置表情包目录，跳过表情图生成".to_string()));
    }
    // 已就绪(如并发已生成)时内部直接复用，否则执行生成
    ensure_sticker_path(&conn, image_id, &meme_dir)?;
    Ok(())
}

/// 获取某图片已就绪的表情图磁盘路径(供前端分享/复制使用，**不触发**生成)
///
/// 已生成时返回 `.gif` 缓存路径；未生成返回 `null`。前端收到 `null` 时应
/// 先直接发送原图，再调用 [`generate_sticker_async`] 触发后台异步生成。
/// 该命令不增加分享计数(分享计数由 `share_image` 等单独维护)。
#[tauri::command]
pub fn get_sticker_if_ready(
    state: tauri::State<'_, crate::core::db_state::DbState>,
    image_id: i32,
) -> Result<Option<String>, AppError> {
    let conn = state.lock().map_err(|e| AppError(e.to_string()))?;
    let meme_dir: String = conn
        .query_row("SELECT meme_dir FROM config WHERE id = 1", [], |row| row.get(0))
        .unwrap_or_default();
    Ok(get_ready_sticker_path(&conn, image_id, &meme_dir)?
        .map(|p| p.to_string_lossy().to_string()))
}

/// 触发某图片表情图的后台异步生成(立即返回，不阻塞)
#[tauri::command]
pub fn generate_sticker_async(
    _state: tauri::State<'_, crate::core::db_state::DbState>,
    image_id: i32,
) -> Result<(), AppError> {
    spawn_sticker_generation(image_id);
    Ok(())
}
