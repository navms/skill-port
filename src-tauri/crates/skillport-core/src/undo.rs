//! 撤销记录与恢复：删除技能、同步操作。
//!
//! 记录随活动 id 存入 `undo` 表；撤销时先校验现场，再基于备份恢复，
//! 绝不会删除不属于本工具的条目。

use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::error::{AppError, AppResult};
use super::model::*;
use super::paths::{self, AppPaths};
use super::{fsops, repo, sync};

pub const KIND_SKILL_DELETE: &str = "skill-delete";
pub const KIND_SYNC: &str = "sync";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDeletePayload {
    pub skill: Skill,
    pub installs: Vec<Install>,
    pub backup: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPayload {
    pub changes: Vec<SyncChange>,
}

/// 单条同步动作执行前的现场，用于撤销。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncChange {
    pub skill_id: String,
    pub target_id: String,
    pub target_path: String,
    pub previous: Option<Install>,
    /// 被替换的旧副本备份目录。
    pub backup: Option<String>,
    /// 撤销时需要重建指向库的符号链接（原先就是链接）。
    pub recreate_symlink: bool,
}

/// 记录一次技能删除。
pub fn record_skill_delete(
    conn: &Connection,
    activity_id: &str,
    skill: &Skill,
    installs: &[Install],
    backup: &Path,
) -> AppResult<()> {
    let payload = SkillDeletePayload {
        skill: skill.clone(),
        installs: installs.to_vec(),
        backup: backup.display().to_string(),
    };
    let raw = serde_json::to_string(&payload)?;
    repo::put_undo(conn, activity_id, KIND_SKILL_DELETE, &raw)
}

/// 记录一次同步；`changes` 为空时不写记录。
pub fn record_sync(
    conn: &Connection,
    activity_id: &str,
    changes: Vec<SyncChange>,
) -> AppResult<()> {
    if changes.is_empty() {
        return Ok(());
    }
    let raw = serde_json::to_string(&SyncPayload { changes })?;
    repo::put_undo(conn, activity_id, KIND_SYNC, &raw)
}

/// 撤销指定活动对应的操作。
pub fn undo(conn: &Connection, app_paths: &AppPaths, activity_id: &str) -> AppResult<UndoResult> {
    let Some((kind, raw)) = repo::take_undo(conn, activity_id)? else {
        return Ok(failed("没有可撤销的操作"));
    };
    let result = match kind.as_str() {
        KIND_SKILL_DELETE => serde_json::from_str::<SkillDeletePayload>(&raw)
            .map_err(AppError::from)
            .and_then(|payload| undo_skill_delete(conn, app_paths, &payload)),
        KIND_SYNC => serde_json::from_str::<SyncPayload>(&raw)
            .map_err(AppError::from)
            .and_then(|payload| undo_sync(conn, app_paths, &payload)),
        _ => Ok(failed("未知的撤销记录")),
    };
    match result {
        Ok(outcome) if outcome.ok => Ok(outcome),
        Ok(outcome) => {
            // 现场已变化：把记录放回，允许用户稍后重试。
            let _ = repo::put_undo(conn, activity_id, &kind, &raw);
            Ok(outcome)
        }
        Err(error) => {
            let _ = repo::put_undo(conn, activity_id, &kind, &raw);
            Err(error)
        }
    }
}

fn failed(message: &str) -> UndoResult {
    UndoResult {
        ok: false,
        message: Some(message.to_string()),
    }
}

fn undo_skill_delete(
    conn: &Connection,
    app_paths: &AppPaths,
    payload: &SkillDeletePayload,
) -> AppResult<UndoResult> {
    let skill_dir = app_paths.library_dir.join(&payload.skill.slug);
    if repo::get_skill(conn, &payload.skill.id).is_ok() {
        return Ok(failed("技能已存在，无需恢复"));
    }
    if std::fs::symlink_metadata(&skill_dir).is_ok() {
        return Ok(failed("库目录已被占用，无法恢复"));
    }
    let backup = Path::new(&payload.backup);
    if !backup.is_dir() {
        return Ok(failed("备份已不存在"));
    }
    restore_dir(backup, &skill_dir)?;
    repo::insert_skill(conn, &payload.skill)?;
    for install in &payload.installs {
        repo::put_install(conn, install)?;
    }
    let _ = sync::reconcile(conn, app_paths);
    repo::log_activity(
        conn,
        &repo::ActivityRecord::ok("restore")
            .skill(&payload.skill)
            .detail("恢复删除的技能"),
    )?;
    Ok(UndoResult {
        ok: true,
        message: None,
    })
}

fn undo_sync(
    conn: &Connection,
    app_paths: &AppPaths,
    payload: &SyncPayload,
) -> AppResult<UndoResult> {
    if payload.changes.is_empty() {
        return Ok(failed("没有可撤销的同步操作"));
    }
    for change in &payload.changes {
        let current = ownership(app_paths, &change.target_path, &change.skill_id)?;
        if current == Ownership::Foreign {
            return Ok(failed("目标路径已被其他内容占用，无法撤销"));
        }
        if let Some(backup) = &change.backup {
            if !Path::new(backup).is_dir() {
                return Ok(failed("同步备份已不存在，无法撤销"));
            }
        }
    }

    for change in payload.changes.iter().rev() {
        let target_path = Path::new(&change.target_path);
        remove_owned(app_paths, &change.target_path, &change.skill_id)?;
        if let Some(backup) = &change.backup {
            restore_dir(Path::new(backup), target_path)?;
        } else if change.recreate_symlink {
            if let Ok(skill) = repo::get_skill(conn, &change.skill_id) {
                let skill_dir = app_paths.library_dir.join(&skill.slug);
                if skill_dir.is_dir() {
                    if let Some(parent) = target_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|error| AppError::io_at(parent, error))?;
                    }
                    fsops::create_dir_symlink(target_path, &skill_dir)?;
                }
            }
        }
        match &change.previous {
            Some(previous) => repo::put_install(conn, previous)?,
            None => repo::delete_install(conn, &change.skill_id, &change.target_id)?,
        }
    }

    let _ = sync::reconcile(conn, app_paths);
    repo::log_activity(
        conn,
        &repo::ActivityRecord::ok("restore").detail("撤销上一次同步"),
    )?;
    Ok(UndoResult {
        ok: true,
        message: None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ownership {
    Missing,
    Owned,
    Foreign,
}

fn ownership(app_paths: &AppPaths, target_path: &str, skill_id: &str) -> AppResult<Ownership> {
    let path = Path::new(target_path);
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Ownership::Missing)
        }
        Err(error) => return Err(AppError::io_at(path, error)),
    };
    if meta.file_type().is_symlink() {
        return Ok(
            if fsops::symlink_points_into(path, &app_paths.library_dir)? {
                Ownership::Owned
            } else {
                Ownership::Foreign
            },
        );
    }
    if meta.is_dir() {
        return Ok(match fsops::read_sidecar(path) {
            Some(sidecar) if sidecar.skill_id == skill_id => Ownership::Owned,
            _ => Ownership::Foreign,
        });
    }
    Ok(Ownership::Foreign)
}

fn remove_owned(app_paths: &AppPaths, target_path: &str, skill_id: &str) -> AppResult<()> {
    if ownership(app_paths, target_path, skill_id)? != Ownership::Owned {
        return Ok(());
    }
    let path = Path::new(target_path);
    let meta = std::fs::symlink_metadata(path).map_err(|error| AppError::io_at(path, error))?;
    if meta.file_type().is_symlink() {
        return fsops::remove_symlink(path);
    }
    let Some(parent) = path.parent() else {
        return Err(AppError::PathEscape(target_path.to_string()));
    };
    let Some(slug) = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
    else {
        return Err(AppError::PathEscape(target_path.to_string()));
    };
    paths::ensure_write_allowed(parent, path, &slug)?;
    std::fs::remove_dir_all(path).map_err(|error| AppError::io_at(path, error))
}

/// 把备份目录原地恢复到 `destination`；跨卷时退回复制 + 删除。
fn restore_dir(backup: &Path, destination: &Path) -> AppResult<()> {
    if std::fs::symlink_metadata(destination).is_ok() {
        return Err(AppError::Conflict(format!(
            "恢复目标已存在：{}",
            destination.display()
        )));
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|error| AppError::io_at(parent, error))?;
    }
    match std::fs::rename(backup, destination) {
        Ok(()) => Ok(()),
        Err(_) => {
            fsops::copy_tree(backup, destination, "恢复")?;
            std::fs::remove_dir_all(backup).map_err(|error| AppError::io_at(backup, error))
        }
    }
}
