//! 轻量文件日志：记录路径、错误码与 hash，不记录技能正文。
//!
//! 单文件上限 5 MB，超出后轮转为 `skillport.log.1`（只保留一份）。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::error::{AppError, AppResult};
use super::util::now_iso;

pub const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;
pub const LOG_NAME: &str = "skillport.log";

pub fn log_path(dir: &Path) -> PathBuf {
    dir.join(LOG_NAME)
}

/// 创建日志目录；若已有日志超过上限则先轮转。
pub fn init(dir: &Path) -> AppResult<()> {
    fs::create_dir_all(dir).map_err(|error| AppError::io_at(dir, error))?;
    rotate_if_needed(dir).map_err(|error| AppError::io_at(dir, error))?;
    Ok(())
}

/// 追加一行结构化日志；写入失败静默忽略，不影响主流程。
pub fn write(dir: &Path, level: &str, event: &str, detail: Option<&str>) {
    let _ = rotate_if_needed(dir);
    let line = format!(
        "{} level={} event={} detail={}\n",
        now_iso(),
        level,
        event,
        detail.unwrap_or("-")
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path(dir))
    {
        let _ = file.write_all(line.as_bytes());
    }
}

fn rotate_if_needed(dir: &Path) -> std::io::Result<()> {
    let path = log_path(dir);
    let Ok(meta) = fs::metadata(&path) else {
        return Ok(());
    };
    if meta.len() < MAX_LOG_BYTES {
        return Ok(());
    }
    let rotated = dir.join("skillport.log.1");
    if rotated.exists() {
        fs::remove_file(&rotated)?;
    }
    fs::rename(&path, &rotated)
}
