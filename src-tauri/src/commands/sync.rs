//! 分发、核对、同步与活动日志命令。

use tauri::{AppHandle, Manager};

use crate::core::error::{AppError, AppResult};
use crate::core::model::*;
use crate::core::util::lock;
use crate::core::{repo, sync, undo};
use crate::state::AppState;

use super::{
    emit, run_blocking, EVENT_LIBRARY_CHANGED, EVENT_RECONCILE_FINISHED, EVENT_SYNC_FINISHED,
    EVENT_SYNC_PROGRESS,
};

/// 设置或取消技能 × 目标的分发期望。
///
/// `enabled = false` 会清理本工具拥有的分发并返回 `null`。
#[tauri::command]
pub async fn installs_set(
    app: AppHandle,
    skill_id: String,
    target_id: String,
    input: InstallSetInput,
) -> AppResult<Option<Install>> {
    let result = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        sync::set_install(&conn, &state.paths, &skill_id, &target_id, &input)
    })
    .await?;
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(result)
}

/// 处理目标路径上的冲突（接管 / 忽略 / 取消忽略 / 取消 Install）。
#[tauri::command]
pub async fn installs_resolve(
    app: AppHandle,
    skill_id: String,
    target_id: String,
    resolution: ConflictResolution,
) -> AppResult<Option<Install>> {
    let result = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        sync::resolve_install(&conn, &state.paths, &skill_id, &target_id, resolution)
    })
    .await?;
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(result)
}

/// 扫描库与目标目录，刷新所有实测状态。不修改技能与目标内容。
#[tauri::command]
pub async fn reconcile(app: AppHandle) -> AppResult<ReconcileReport> {
    let report = run_blocking(app.clone(), |state| {
        let conn = lock(&state.db)?;
        sync::reconcile(&conn, &state.paths)
    })
    .await?;
    emit(&app, EVENT_RECONCILE_FINISHED, report.clone());
    emit(
        &app,
        EVENT_LIBRARY_CHANGED,
        serde_json::json!({ "schema": 1 }),
    );
    Ok(report)
}

/// 生成同步计划（5 分钟有效）；不修改文件系统。
#[tauri::command]
pub async fn sync_plan(app: AppHandle, filter: Option<SyncFilter>) -> AppResult<SyncPlan> {
    let filter = filter.unwrap_or_default();
    let (plan, _cancel) = run_blocking(app, move |state| {
        let conn = lock(&state.db)?;
        let plan = sync::build_plan(&conn, &state.paths, &filter)?;
        let cancel = state.plans.insert(plan.clone())?;
        Ok((plan, cancel))
    })
    .await?;
    Ok(plan)
}

/// 执行计划；通过 `sync://progress` 报告进度，`sync://finished` 通知结束。
#[tauri::command]
pub async fn sync_apply(app: AppHandle, plan_id: String) -> AppResult<SyncResult> {
    let (plan, cancel) = {
        let state = app.state::<AppState>();
        state.plans.begin_apply(&plan_id)?
    };
    let handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        let conn = lock(&state.db)?;
        let mut on_progress = |progress: SyncProgress| {
            emit(&handle, EVENT_SYNC_PROGRESS, progress);
        };
        let outcome = sync::apply_plan(&conn, &state.paths, &plan, &cancel, &mut on_progress);
        let _ = state.plans.finish(&plan.id);
        outcome
    })
    .await
    .map_err(|error| AppError::Internal(format!("后台任务失败：{error}")))?;

    if let Ok(summary) = &result {
        emit(&app, EVENT_SYNC_FINISHED, summary.clone());
        emit(
            &app,
            EVENT_LIBRARY_CHANGED,
            serde_json::json!({ "schema": 1 }),
        );
    }
    result
}

/// 请求取消正在执行的计划；已完成步骤不回滚。
#[tauri::command]
pub async fn sync_cancel(app: AppHandle, plan_id: String) -> AppResult<()> {
    let state = app.state::<AppState>();
    state.plans.cancel(&plan_id)
}

/// 分页读取操作日志。只读、幂等。
#[tauri::command]
pub async fn activity_list(
    app: AppHandle,
    query: Option<ActivityQuery>,
) -> AppResult<ActivityPage> {
    let query = query.unwrap_or_default();
    let cursor = match query.cursor.as_deref() {
        None | Some("") => None,
        Some(raw) => Some(
            raw.parse::<i64>()
                .map_err(|_| AppError::InvalidInput(format!("无效的分页游标：{raw}")))?,
        ),
    };
    run_blocking(app, move |state| {
        let conn = lock(&state.db)?;
        repo::list_activity(&conn, query.limit.unwrap_or(30), cursor)
    })
    .await
}

/// 撤销一条活动记录对应的操作（删除技能 / 同步）。
#[tauri::command]
pub async fn activity_undo(app: AppHandle, activity_id: String) -> AppResult<UndoResult> {
    let result = run_blocking(app.clone(), move |state| {
        let conn = lock(&state.db)?;
        undo::undo(&conn, &state.paths, &activity_id)
    })
    .await?;
    if result.ok {
        emit(
            &app,
            EVENT_LIBRARY_CHANGED,
            serde_json::json!({ "schema": 1 }),
        );
    }
    Ok(result)
}
