//! 技能库命令。

use tauri::AppHandle;

use crate::core::error::{AppError, AppResult};
use crate::core::model::*;
use crate::core::util::lock;
use crate::core::{library, repo};

use super::{emit, run_blocking, EVENT_LIBRARY_CHANGED};

/// 列出全部技能及分发状态。只读、幂等。
#[tauri::command]
pub async fn library_list(app: AppHandle) -> AppResult<Vec<SkillSummary>> {
    run_blocking(app, |state| {
        let conn = lock(&state.db)?;
        library::list(&conn, &state.paths)
    })
    .await
}

/// 读取技能详情（文件清单 + 分发状态）。只读、幂等。
#[tauri::command]
pub async fn library_get(app: AppHandle, id: String) -> AppResult<SkillDetail> {
    run_blocking(app, move |state| {
        let conn = lock(&state.db)?;
        library::get(&conn, &state.paths, &id)
    })
    .await
}

/// 批量导入技能目录。严格校验；失败项不写库、不碰目标目录。
#[tauri::command]
pub async fn library_import(app: AppHandle, paths: Vec<String>) -> AppResult<ImportReport> {
    let report = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        library::import(&conn, &state.paths, &paths)
    })
    .await?;
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(report)
}

/// 编辑技能的 `SKILL.md`（写入后重算 hash）或仅分类（只写 DB，不碰文件）。
#[tauri::command]
pub async fn library_update(
    app: AppHandle,
    id: String,
    patch: SkillPatch,
) -> AppResult<SkillDetail> {
    let detail = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        library::update(&conn, &state.paths, &id, &patch)
    })
    .await?;
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(detail)
}

/// 删除技能。先清理归属本工具的分发，再备份库目录，最后删 DB 行。
///
/// 返回可用于撤销的活动 id 与删除的分发数。
#[tauri::command]
pub async fn library_delete(
    app: AppHandle,
    id: String,
    options: DeleteSkillOptions,
) -> AppResult<DeleteSkillResult> {
    let result = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        library::delete(&conn, &state.paths, &id, &options)
    })
    .await?;
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(result)
}

/// 在系统文件管理器中显示技能目录。只读。
#[tauri::command]
pub async fn library_reveal(app: AppHandle, id: String) -> AppResult<()> {
    let path = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        let skill = repo::get_skill(&conn, &id)?;
        let dir = state.paths.library_dir.join(&skill.slug);
        if !dir.is_dir() {
            return Err(AppError::NotFound(format!(
                "技能目录不存在：{}",
                dir.display()
            )));
        }
        Ok(dir)
    })
    .await?;
    tauri::async_runtime::spawn_blocking(move || super::reveal_path(&path))
        .await
        .map_err(|error| AppError::Internal(format!("后台任务失败：{error}")))?
}
