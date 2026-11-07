//! 设置命令。

use tauri::AppHandle;

use crate::core::error::AppResult;
use crate::core::model::{AppSettings, SettingsPatch};
use crate::core::settings;
use crate::core::util::lock;

use super::run_blocking;

/// 读取设置与应用目录（只读、幂等）。
#[tauri::command]
pub async fn settings_get(app: AppHandle) -> AppResult<AppSettings> {
    run_blocking(app, |state| {
        let conn = lock(&state.db)?;
        settings::snapshot(&conn, &state.paths)
    })
    .await
}

/// 合并并保存设置；非法取值返回 `InvalidInput`。
#[tauri::command]
pub async fn settings_set(app: AppHandle, patch: SettingsPatch) -> AppResult<AppSettings> {
    run_blocking(app, move |state| {
        let conn = lock(&state.db)?;
        settings::update(&conn, &state.paths, &patch)?;
        settings::snapshot(&conn, &state.paths)
    })
    .await
}
