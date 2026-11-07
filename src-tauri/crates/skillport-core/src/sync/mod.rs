//! 核对（reconcile）、同步计划（plan）、执行（apply）与冲突处理。

mod apply;
mod inspect;
mod owned;
mod planner;

pub use apply::apply_plan;
pub use inspect::Inspection;
pub use owned::{remove_distribution, RemovalOutcome};
pub use planner::build_plan;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::{AppError, AppResult};
use crate::model::*;
use crate::paths::{self, AppPaths};
use crate::util::{new_id, now_iso};
use crate::{fsops, hash, repo, settings};

use inspect::inspect_target;

/// 扫描库与所有目标目录，把实测状态写回 DB。只读文件系统（不修改技能/目标内容）。
pub fn reconcile(conn: &Connection, app_paths: &AppPaths) -> AppResult<ReconcileReport> {
    let checked_at = now_iso();
    let targets = repo::list_targets(conn)?;
    let skills = repo::list_skills(conn)?;
    let skills_by_id: HashMap<&str, &Skill> = skills
        .iter()
        .map(|skill| (skill.id.as_str(), skill))
        .collect();
    let targets_by_id: HashMap<&str, &Target> = targets
        .iter()
        .map(|target| (target.id.as_str(), target))
        .collect();

    // 库是唯一真相源：库内文件被外部修改时刷新权威 hash。
    let mut hash_cache: HashMap<String, Option<String>> = HashMap::new();
    for skill in &skills {
        let skill_dir = app_paths.library_dir.join(&skill.slug);
        let current = if skill_dir.is_dir() {
            hash::tree_hash(&skill_dir).ok()
        } else {
            None
        };
        if let Some(value) = &current {
            if value != &skill.content_hash {
                repo::update_skill_hash(conn, &skill.id, value, &checked_at)?;
            }
        }
        hash_cache.insert(skill.id.clone(), current);
    }

    let installs = repo::list_installs(conn)?;
    let mut checked = 0usize;
    let mut changed = 0usize;
    for install in &installs {
        let Some(skill) = skills_by_id.get(install.skill_id.as_str()) else {
            continue;
        };
        let Some(target) = targets_by_id.get(install.target_id.as_str()) else {
            continue;
        };
        checked += 1;
        let skill_dir = app_paths.library_dir.join(&skill.slug);
        let target_path =
            match paths::guarded_target_path(Path::new(&target.skills_dir), &skill.slug) {
                Ok(path) => path,
                Err(_) => PathBuf::from(&install.target_path),
            };
        let inspection = match hash_cache.get(&skill.id).cloned().flatten() {
            Some(skill_hash) => inspect_target(
                &app_paths.library_dir,
                &skill.id,
                &skill_dir,
                &skill_hash,
                &target_path,
            ),
            None => Inspection::error("库目录缺失"),
        };
        if inspection.status != install.status {
            changed += 1;
        }
        repo::record_observation(
            conn,
            install.id,
            inspection.status,
            inspection.actual_kind,
            inspection.linked_hash.as_deref(),
            inspection.last_error.as_deref(),
            &checked_at,
        )?;
    }

    let after = repo::list_installs(conn)?;
    let enabled_targets: HashSet<&str> = targets
        .iter()
        .filter(|target| target.enabled)
        .map(|target| target.id.as_str())
        .collect();
    let issues = after
        .iter()
        .filter(|install| enabled_targets.contains(install.target_id.as_str()))
        .filter(|install| !install.ignored && install.status.is_issue())
        .count();

    Ok(ReconcileReport {
        checked,
        changed,
        issues,
        checked_at,
    })
}

/// 设置或取消一条分发期望。
///
/// `enabled = false` 会清理本工具拥有的分发并删除 Install 行，返回 `None`。
pub fn set_install(
    conn: &Connection,
    app_paths: &AppPaths,
    skill_id: &str,
    target_id: &str,
    input: &InstallSetInput,
) -> AppResult<Option<Install>> {
    let skill = repo::get_skill(conn, skill_id)?;
    let target = repo::get_target(conn, target_id)?;
    let existing = repo::get_install(conn, skill_id, target_id)?;

    if !input.enabled {
        let keep = settings::get(conn)?.backup_retention as usize;
        let outcome = remove_distribution(app_paths, &skill, &target, true, keep)?;
        repo::delete_install(conn, skill_id, target_id)?;
        let mut record = repo::ActivityRecord::ok("unlink")
            .skill(&skill)
            .target(&target)
            .detail(&format!("取消 {} → {}", skill.slug, target.name));
        if let Some(detail) = &outcome.detail {
            record = record.detail(detail);
        }
        repo::log_activity(conn, &record)?;
        return Ok(None);
    }

    if let Some(install) = &existing {
        if install.status == InstallStatus::Conflict {
            return Err(AppError::Conflict(
                "目标路径中存在外部内容，请先解决冲突".to_string(),
            ));
        }
    }

    let skills_dir = Path::new(&target.skills_dir);
    let target_path = paths::guarded_target_path(skills_dir, &skill.slug).map_err(|error| {
        AppError::InvalidInput(format!("目标目录不可用：{}（{error}）", target.skills_dir))
    })?;
    let mode = input
        .mode
        .or(existing.as_ref().map(|install| install.mode))
        .unwrap_or(target.link_mode);
    let install = repo::set_expectation(
        conn,
        skill_id,
        target_id,
        mode,
        &target_path.to_string_lossy(),
        &now_iso(),
    )?;
    let action = match mode {
        LinkMode::Copy => "copy",
        LinkMode::Symlink => "link",
    };
    repo::log_activity(
        conn,
        &repo::ActivityRecord::ok(action)
            .skill(&skill)
            .target(&target)
            .detail(&format!("{} → {}", skill.slug, target.name)),
    )?;
    Ok(Some(install))
}

/// 处理目标路径上的冲突。
pub fn resolve_install(
    conn: &Connection,
    app_paths: &AppPaths,
    skill_id: &str,
    target_id: &str,
    resolution: ConflictResolution,
) -> AppResult<Option<Install>> {
    let skill = repo::get_skill(conn, skill_id)?;
    let target = repo::get_target(conn, target_id)?;
    let install = repo::get_install(conn, skill_id, target_id)?
        .ok_or_else(|| AppError::NotFound(format!("{} 在 {} 上的分发", skill.slug, target.name)))?;

    match resolution {
        ConflictResolution::Cancel => {
            repo::delete_install(conn, skill_id, target_id)?;
            repo::log_activity(
                conn,
                &repo::ActivityRecord::ok("conflict")
                    .skill(&skill)
                    .target(&target)
                    .detail(&format!("取消 {} 的 Install，外部目录未改动", target.name)),
            )?;
            Ok(None)
        }
        ConflictResolution::Ignore | ConflictResolution::Unignore => {
            let ignored = resolution == ConflictResolution::Ignore;
            repo::set_ignored(conn, skill_id, target_id, ignored)?;
            let detail = if ignored {
                format!("保留并忽略 {} 的冲突", target.name)
            } else {
                format!("取消忽略 {} 的冲突", target.name)
            };
            repo::log_activity(
                conn,
                &repo::ActivityRecord::ok("conflict")
                    .skill(&skill)
                    .target(&target)
                    .detail(&detail),
            )?;
            repo::get_install(conn, skill_id, target_id)
        }
        ConflictResolution::Adopt => {
            let adopted = adopt(conn, app_paths, &skill, &target, install)?;
            repo::log_activity(
                conn,
                &repo::ActivityRecord::ok("conflict")
                    .skill(&skill)
                    .target(&target)
                    .detail(&format!("接管 {} 的外部目录为库副本", target.name)),
            )?;
            Ok(Some(adopted))
        }
    }
}

/// 接管目标路径上的外部条目，使其成为本工具拥有的副本。
fn adopt(
    conn: &Connection,
    app_paths: &AppPaths,
    skill: &Skill,
    target: &Target,
    mut install: Install,
) -> AppResult<Install> {
    let skills_dir = Path::new(&target.skills_dir);
    let target_path = paths::guarded_target_path(skills_dir, &skill.slug)?;
    let skill_dir = app_paths.library_dir.join(&skill.slug);
    let library_hash = hash::tree_hash(&skill_dir)?;
    let inspection = inspect_target(
        &app_paths.library_dir,
        &skill.id,
        &skill_dir,
        &library_hash,
        &target_path,
    );
    if inspection.status != InstallStatus::Conflict {
        return Err(AppError::InvalidInput(
            "该分发当前不是冲突，无需接管".to_string(),
        ));
    }
    let meta = std::fs::symlink_metadata(&target_path)
        .map_err(|error| AppError::io_at(&target_path, error))?;

    let actual_hash = if meta.file_type().is_symlink() {
        let resolved = fsops::resolve_link(&target_path)?;
        if !resolved.is_dir() {
            return Err(AppError::InvalidInput(
                "该链接指向的不是目录，无法接管为副本".to_string(),
            ));
        }
        let temp = skills_dir.join(format!("{}{}", fsops::TMP_PREFIX, new_id()));
        fsops::copy_tree(&resolved, &temp, "接管")?;
        let copied_hash = hash::tree_hash(&temp)?;
        fsops::write_sidecar(&temp, &fsops::Sidecar::new(&skill.id, &copied_hash))?;
        fsops::remove_symlink(&target_path)?;
        if let Err(error) = std::fs::rename(&temp, &target_path) {
            let _ = std::fs::remove_dir_all(&temp);
            return Err(AppError::io_at(&target_path, error));
        }
        copied_hash
    } else if meta.is_dir() {
        let existing_hash = hash::tree_hash(&target_path)?;
        fsops::write_sidecar(
            &target_path,
            &fsops::Sidecar::new(&skill.id, &existing_hash),
        )?;
        existing_hash
    } else {
        return Err(AppError::InvalidInput(
            "目标路径不是目录，无法接管".to_string(),
        ));
    };

    let at = now_iso();
    install.mode = LinkMode::Copy;
    install.actual_kind = Some(ActualKind::Copy);
    install.status = if actual_hash == library_hash {
        InstallStatus::Ok
    } else {
        InstallStatus::Stale
    };
    install.linked_hash = Some(actual_hash);
    install.ignored = false;
    install.last_error = None;
    install.last_checked = Some(at);
    repo::put_install(conn, &install)?;
    Ok(install)
}
