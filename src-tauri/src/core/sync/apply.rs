//! 执行同步计划。每一步先备份/校验归属，再原子落位。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use rusqlite::Connection;

use crate::core::error::{AppError, AppResult};
use crate::core::model::{
    ActualKind, LinkMode, PlanAction, Skill, SyncPlan, SyncProgress, SyncProgressStatus,
    SyncResult, SyncResultItem, Target,
};
use crate::core::paths::{self, AppPaths};
use crate::core::undo::SyncChange;
use crate::core::util::{new_id, now_iso};
use crate::core::{fsops, hash, logging, repo, settings, undo};

/// 目标路径上已有的条目归属。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Existing {
    None,
    OwnedSymlink,
    OwnedCopy,
    Foreign,
}

/// 按计划顺序执行；每步之间检查取消标记。已完成的步骤不回滚。
pub fn apply_plan(
    conn: &Connection,
    app_paths: &AppPaths,
    plan: &SyncPlan,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(SyncProgress),
) -> AppResult<SyncResult> {
    let keep = settings::get(conn)?.backup_retention as usize;
    let actionable: Vec<&PlanAction> = plan
        .actions
        .iter()
        .filter(|action| action.kind.is_actionable())
        .collect();
    let total = actionable.len();
    let mut applied = 0usize;
    let mut failed = 0usize;
    let mut cancelled = false;
    let mut items = Vec::new();
    let mut changes = Vec::new();

    for (index, action) in actionable.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        let previous = repo::get_install(conn, &action.skill_id, &action.target_id)?;
        let outcome = execute_action(conn, app_paths, action, keep);
        let (status, item) = match outcome {
            Ok(executed) => {
                applied += 1;
                changes.push(SyncChange {
                    skill_id: action.skill_id.clone(),
                    target_id: action.target_id.clone(),
                    target_path: executed.target_path.display().to_string(),
                    recreate_symlink: recreate_symlink(previous.as_ref()),
                    previous,
                    backup: executed.backup.map(|path| path.display().to_string()),
                });
                (
                    SyncProgressStatus::Ok,
                    SyncResultItem {
                        action_id: action.id.clone(),
                        ok: true,
                        error_code: None,
                        message: None,
                    },
                )
            }
            Err(error) => {
                failed += 1;
                record_failure(conn, app_paths, action, &error)?;
                let status = if error.code() == "Conflict" {
                    SyncProgressStatus::Conflict
                } else {
                    SyncProgressStatus::Failed
                };
                (
                    status,
                    SyncResultItem {
                        action_id: action.id.clone(),
                        ok: false,
                        error_code: Some(error.code().to_string()),
                        message: Some(error.to_string()),
                    },
                )
            }
        };
        items.push(item);
        progress(SyncProgress {
            plan_id: plan.id.clone(),
            completed: index + 1,
            total,
            action_id: Some(action.id.clone()),
            status,
        });
    }

    let undo_id = if applied > 0 {
        Some(record_summary(
            conn,
            &actionable,
            applied,
            failed,
            cancelled,
            changes,
        )?)
    } else {
        None
    };

    Ok(SyncResult {
        plan_id: plan.id.clone(),
        applied,
        failed,
        cancelled,
        undo_id,
        items,
    })
}

fn recreate_symlink(previous: Option<&crate::core::model::Install>) -> bool {
    match previous {
        Some(install) => {
            install.actual_kind == Some(ActualKind::Symlink)
                && matches!(
                    install.status,
                    crate::core::model::InstallStatus::Ok
                        | crate::core::model::InstallStatus::Stale
                )
        }
        None => false,
    }
}

fn record_failure(
    conn: &Connection,
    app_paths: &AppPaths,
    action: &PlanAction,
    error: &AppError,
) -> AppResult<()> {
    let at = now_iso();
    repo::mark_error(
        conn,
        &action.skill_id,
        &action.target_id,
        &error.to_string(),
        &at,
    )?;
    repo::log_activity(
        conn,
        &repo::ActivityRecord {
            action: "sync".to_string(),
            skill_id: Some(action.skill_id.clone()),
            skill_name: Some(action.skill_slug.clone()),
            target_id: Some(action.target_id.clone()),
            target_name: Some(action.target_name.clone()),
            result: "failed".to_string(),
            detail: Some(error.code().to_string()),
        },
    )?;
    logging::write(
        &app_paths.logs_dir,
        "error",
        "sync_failed",
        Some(&format!(
            "skill={} target={} code={} path={}",
            action.skill_slug,
            action.target_name,
            error.code(),
            action.target_path
        )),
    );
    Ok(())
}

fn record_summary(
    conn: &Connection,
    actionable: &[&PlanAction],
    applied: usize,
    failed: usize,
    cancelled: bool,
    changes: Vec<SyncChange>,
) -> AppResult<String> {
    let detail = if cancelled {
        format!("同步已取消，完成 {applied} 项")
    } else if failed > 0 {
        format!("同步完成 {applied} 项，{failed} 项失败")
    } else {
        format!("同步完成 {applied} 项")
    };
    let mut record = repo::ActivityRecord::ok("sync").detail(&detail);
    if actionable.len() == 1 {
        if let Some(action) = actionable.first() {
            if let Ok(skill) = repo::get_skill(conn, &action.skill_id) {
                record = record.skill(&skill);
            }
            if let Ok(target) = repo::get_target(conn, &action.target_id) {
                record = record.target(&target);
            }
        }
    }
    let activity_id = repo::log_activity(conn, &record)?;
    undo::record_sync(conn, &activity_id.to_string(), changes)?;
    Ok(activity_id.to_string())
}

/// 一次成功执行的现场：撤销需要的真实目标路径与旧副本备份。
struct Executed {
    target_path: PathBuf,
    backup: Option<PathBuf>,
}

fn execute_action(
    conn: &Connection,
    app_paths: &AppPaths,
    action: &PlanAction,
    keep: usize,
) -> AppResult<Executed> {
    let skill = repo::get_skill(conn, &action.skill_id)?;
    let target = repo::get_target(conn, &action.target_id)?;
    let skill_dir = app_paths.library_dir.join(&skill.slug);
    if !skill_dir.is_dir() {
        return Err(AppError::NotFound(format!(
            "库目录缺失：{}",
            skill_dir.display()
        )));
    }
    let skill_hash = hash::tree_hash(&skill_dir)?;
    let (skills_dir, target_path) = prepare_target(app_paths, &skill, &target)?;
    let existing = classify(app_paths, &target_path, &skill.id)?;
    if existing == Existing::Foreign {
        return Err(conflict_exists(&target_path));
    }

    let backup = match (action.mode, existing) {
        (LinkMode::Symlink, Existing::OwnedCopy) => Some(backup_owned(
            app_paths,
            &skill,
            &target,
            &target_path,
            keep,
        )?),
        (LinkMode::Copy, Existing::OwnedCopy) => {
            let staged = stage_copy(&skill_dir, &skills_dir, &skill, &skill_hash)?;
            let backup = backup_owned(app_paths, &skill, &target, &target_path, keep)?;
            staged.finalize(&target_path)?;
            Some(backup)
        }
        (LinkMode::Symlink, Existing::OwnedSymlink) => {
            fsops::remove_symlink(&target_path)?;
            None
        }
        (LinkMode::Copy, Existing::OwnedSymlink) => {
            fsops::remove_symlink(&target_path)?;
            stage_copy(&skill_dir, &skills_dir, &skill, &skill_hash)?.finalize(&target_path)?;
            None
        }
        (LinkMode::Copy, Existing::None) => {
            stage_copy(&skill_dir, &skills_dir, &skill, &skill_hash)?.finalize(&target_path)?;
            None
        }
        (LinkMode::Symlink, Existing::None) => None,
        (_, Existing::Foreign) => return Err(conflict_exists(&target_path)),
    };

    if action.mode == LinkMode::Symlink {
        paths::ensure_write_allowed(&skills_dir, &target_path, &skill.slug)?;
        fsops::create_dir_symlink(&target_path, &skill_dir)?;
    }

    let actual_kind = match action.mode {
        LinkMode::Symlink => ActualKind::Symlink,
        LinkMode::Copy => ActualKind::Copy,
    };
    repo::mark_synced(
        conn,
        &skill.id,
        &target.id,
        actual_kind,
        &skill_hash,
        &now_iso(),
    )?;
    Ok(Executed {
        target_path,
        backup,
    })
}

fn backup_owned(
    app_paths: &AppPaths,
    skill: &Skill,
    target: &Target,
    target_path: &Path,
    keep: usize,
) -> AppResult<PathBuf> {
    fsops::backup_dir(
        &app_paths.backups_dir,
        &skill.slug,
        &target.slug,
        target_path,
        keep,
    )?
    .ok_or_else(|| AppError::Internal("备份未生成".to_string()))
}

fn prepare_target(
    app_paths: &AppPaths,
    skill: &Skill,
    target: &Target,
) -> AppResult<(PathBuf, PathBuf)> {
    let skills_dir = PathBuf::from(&target.skills_dir);
    if !skills_dir.is_absolute() {
        return Err(AppError::InvalidInput(format!(
            "目标目录必须是绝对路径：{}",
            skills_dir.display()
        )));
    }
    paths::validate_library_placement(&app_paths.library_dir, [skills_dir.clone()])?;
    if skills_dir.exists() {
        if !skills_dir.is_dir() {
            return Err(AppError::Conflict(format!(
                "目标路径不是目录：{}",
                skills_dir.display()
            )));
        }
    } else {
        std::fs::create_dir_all(&skills_dir)
            .map_err(|error| AppError::io_at(&skills_dir, error))?;
    }
    let target_path = paths::guarded_target_path(&skills_dir, &skill.slug)?;
    paths::ensure_write_allowed(&skills_dir, &target_path, &skill.slug)?;
    Ok((skills_dir, target_path))
}

fn classify(app_paths: &AppPaths, target_path: &Path, skill_id: &str) -> AppResult<Existing> {
    let meta = match std::fs::symlink_metadata(target_path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Existing::None),
        Err(error) => return Err(AppError::io_at(target_path, error)),
    };
    if meta.file_type().is_symlink() {
        return if fsops::symlink_points_into(target_path, &app_paths.library_dir)? {
            Ok(Existing::OwnedSymlink)
        } else {
            Ok(Existing::Foreign)
        };
    }
    if meta.is_dir() {
        return match fsops::read_sidecar(target_path) {
            Some(sidecar) if sidecar.skill_id == skill_id => Ok(Existing::OwnedCopy),
            _ => Ok(Existing::Foreign),
        };
    }
    Ok(Existing::Foreign)
}

fn conflict_exists(target_path: &Path) -> AppError {
    AppError::Conflict(format!(
        "目标位置已有不属于本技能的内容，拒绝覆盖：{}",
        target_path.display()
    ))
}

/// 先在同卷临时目录准备完整副本，再原子 rename 到位。
struct StagedCopy {
    temp: PathBuf,
    armed: bool,
}

impl StagedCopy {
    fn finalize(mut self, target_path: &Path) -> AppResult<()> {
        if std::fs::symlink_metadata(target_path).is_ok() {
            return Err(conflict_exists(target_path));
        }
        std::fs::rename(&self.temp, target_path)
            .map_err(|error| AppError::io_at(target_path, error))?;
        self.armed = false;
        Ok(())
    }
}

impl Drop for StagedCopy {
    fn drop(&mut self) {
        if self.armed {
            let _ = std::fs::remove_dir_all(&self.temp);
        }
    }
}

fn stage_copy(
    skill_dir: &Path,
    skills_dir: &Path,
    skill: &Skill,
    skill_hash: &str,
) -> AppResult<StagedCopy> {
    let temp = skills_dir.join(format!("{}{}", fsops::TMP_PREFIX, new_id()));
    std::fs::create_dir_all(&temp).map_err(|error| AppError::io_at(&temp, error))?;
    let staged = StagedCopy { temp, armed: true };
    fsops::copy_tree(skill_dir, &staged.temp, "技能")?;
    fsops::write_sidecar(&staged.temp, &fsops::Sidecar::new(&skill.id, skill_hash))?;
    Ok(staged)
}
