//! IPC 命令层：只做参数校验、状态注入、调用 core 与事件通知。

pub mod library;
pub mod settings;
pub mod sync;
pub mod targets;

use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::core::error::{AppError, AppResult};
use crate::state::AppState;

pub const EVENT_LIBRARY_CHANGED: &str = "library://changed";
pub const EVENT_RECONCILE_FINISHED: &str = "reconcile://finished";
pub const EVENT_SYNC_PROGRESS: &str = "sync://progress";
pub const EVENT_SYNC_FINISHED: &str = "sync://finished";

/// 在阻塞线程池中执行数据库/文件操作，避免占用 IPC 线程。
pub(crate) async fn run_blocking<T, F>(app: AppHandle, work: F) -> AppResult<T>
where
    T: Send + 'static,
    F: FnOnce(&AppState) -> AppResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        work(&state)
    })
    .await
    .map_err(|error| AppError::Internal(format!("后台任务失败：{error}")))?
}

/// 事件只做通知，关键结果仍由命令返回值承载。
pub(crate) fn emit<S>(app: &AppHandle, event: &str, payload: S)
where
    S: serde::Serialize + Clone,
{
    use tauri::Emitter;
    let _ = app.emit(event, payload);
}

/// 在系统文件管理器中定位一个路径（选中它，而不是只打开父目录）。
pub(crate) fn reveal_path(path: &Path) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map_err(|error| AppError::io_at(path, error))?;
    }
    #[cfg(target_os = "windows")]
    {
        let argument = format!("/select,{}", path.display());
        std::process::Command::new("explorer")
            .arg(argument)
            .spawn()
            .map_err(|error| AppError::io_at(path, error))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target: &Path = match path.parent() {
            Some(parent) => parent,
            None => path,
        };
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(|error| AppError::io_at(target, error))?;
    }
    Ok(())
}
