//! 目标（harness）命令。

use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::core::error::{AppError, AppResult};
use crate::core::model::*;
use crate::core::util::lock;
use crate::core::{icons, repo, target};

use super::{emit, run_blocking, EVENT_LIBRARY_CHANGED};

/// 列出所有目标。只读、幂等。
#[tauri::command]
pub async fn targets_list(app: AppHandle) -> AppResult<Vec<Target>> {
    run_blocking(app, |state| {
        let conn = lock(&state.db)?;
        target::list(&conn)
    })
    .await
}

/// 给出候选目标目录及存在性；不创建、不写入任何路径。
#[tauri::command]
pub async fn target_detect_suggestions(app: AppHandle) -> AppResult<Vec<SuggestedTarget>> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| AppError::Internal(format!("无法获取用户主目录：{error}")))?;
    run_blocking(app, move |state| {
        let conn = lock(&state.db)?;
        target::detect_suggestions(&conn, &home)
    })
    .await
}

/// 新增目标。校验 slug、绝对路径与库目录位置。
#[tauri::command]
pub async fn target_create(app: AppHandle, input: TargetCreateInput) -> AppResult<Target> {
    let created = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        target::create(&conn, &state.paths, &input)
    })
    .await?;
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(created)
}

/// 修改目标。路径变化时刷新该目标下 Install 的期望路径。
#[tauri::command]
pub async fn target_update(
    app: AppHandle,
    id: String,
    patch: TargetUpdatePatch,
) -> AppResult<Target> {
    let updated = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        target::update(&conn, &state.paths, &id, &patch)
    })
    .await?;
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(updated)
}

/// 删除目标；`unlink` 为真时清理归属本工具的分发条目。
#[tauri::command]
pub async fn target_delete(
    app: AppHandle,
    id: String,
    options: DeleteTargetOptions,
) -> AppResult<()> {
    run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        target::delete(&conn, &state.paths, &id, &options)
    })
    .await?;
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(())
}

/// 在系统文件管理器中打开目标的技能目录。只读、幂等。
///
/// 目录不存在时返回 `NotFound`；不会创建目录，也不修改任何文件。
#[tauri::command]
pub async fn target_reveal(app: AppHandle, id: String) -> AppResult<()> {
    let dir = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        let target = repo::get_target(&conn, &id)?;
        let dir = PathBuf::from(&target.skills_dir);
        if !dir.is_dir() {
            return Err(AppError::NotFound(format!(
                "技能目录不存在：{}",
                dir.display()
            )));
        }
        Ok(dir)
    })
    .await?;
    tauri::async_runtime::spawn_blocking(move || super::open_path(&dir))
        .await
        .map_err(|error| AppError::Internal(format!("后台任务失败：{error}")))?
}

/// 列出 harness 图标；读取应用数据目录下的 `icons/*.svg`，用户可直接增删。只读、幂等。
#[tauri::command]
pub async fn target_icon_list(app: AppHandle) -> AppResult<Vec<TargetIconAsset>> {
    run_blocking(app, |state| icons::list(&state.paths)).await
}

/// 在系统文件管理器中打开图标目录；目录为空时先补齐默认图标。
#[tauri::command]
pub async fn target_icon_reveal(app: AppHandle) -> AppResult<()> {
    let dir = run_blocking(app.clone(), |state| {
        icons::ensure_defaults(&state.paths)?;
        Ok(icons::dir(&state.paths))
    })
    .await?;
    tauri::async_runtime::spawn_blocking(move || super::reveal_path(&dir))
        .await
        .map_err(|error| AppError::Internal(format!("后台任务失败：{error}")))?
}

/// 打开系统目录选择对话框；取消时返回 `null`。
#[tauri::command]
pub async fn target_pick_dir(
    app: AppHandle,
    initial_dir: Option<String>,
) -> AppResult<Option<String>> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = app.dialog().file();
        if let Some(dir) = initial_dir.filter(|value| !value.trim().is_empty()) {
            dialog = dialog.set_directory(dir);
        }
        dialog.blocking_pick_folder()
    })
    .await
    .map_err(|error| AppError::Internal(format!("对话框任务失败：{error}")))?;
    match picked {
        None => Ok(None),
        Some(file_path) => {
            let path = file_path
                .into_path()
                .map_err(|error| AppError::Internal(format!("无法解析所选目录：{error}")))?;
            Ok(Some(path.display().to_string()))
        }
    }
}
