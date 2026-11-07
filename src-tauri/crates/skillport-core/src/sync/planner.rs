//! 生成同步计划（只读，不修改文件系统）。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::AppResult;
use crate::hash;
use crate::model::*;
use crate::paths::AppPaths;
use crate::plan::PLAN_TTL;
use crate::util::new_id;
use crate::{paths, repo};

use super::inspect::{inspect_target, Inspection};

/// 根据当前期望与实测状态生成计划。
pub fn build_plan(
    conn: &Connection,
    app_paths: &AppPaths,
    filter: &SyncFilter,
) -> AppResult<SyncPlan> {
    let targets = repo::list_targets(conn)?;
    let skills = repo::list_skills(conn)?;
    let installs = repo::list_installs(conn)?;
    let skills_by_id: HashMap<&str, &Skill> = skills
        .iter()
        .map(|skill| (skill.id.as_str(), skill))
        .collect();
    let targets_by_id: HashMap<&str, &Target> = targets
        .iter()
        .map(|target| (target.id.as_str(), target))
        .collect();

    let mut hash_cache: HashMap<String, Option<String>> = HashMap::new();
    let mut actions = Vec::new();

    for install in &installs {
        let Some(skill) = skills_by_id.get(install.skill_id.as_str()) else {
            continue;
        };
        let Some(target) = targets_by_id.get(install.target_id.as_str()) else {
            continue;
        };
        if !target.enabled || !passes_filter(filter, &install.skill_id, &install.target_id) {
            continue;
        }

        let skill_dir = app_paths.library_dir.join(&skill.slug);
        let skills_dir = Path::new(&target.skills_dir);
        let fallback_path = skills_dir.join(&skill.slug);
        let target_path = match paths::guarded_target_path(skills_dir, &skill.slug) {
            Ok(path) => path,
            Err(error) => {
                actions.push(action(
                    skill,
                    target,
                    fallback_path,
                    install,
                    SyncActionKind::Conflict,
                    InstallStatus::Error,
                    Some(error.to_string()),
                ));
                continue;
            }
        };

        let skill_hash = match cached_hash(&mut hash_cache, &skill.id, &skill_dir) {
            Ok(Some(value)) => value,
            Ok(None) => {
                actions.push(action(
                    skill,
                    target,
                    target_path,
                    install,
                    SyncActionKind::Conflict,
                    InstallStatus::Error,
                    Some("库目录缺失，无法同步".to_string()),
                ));
                continue;
            }
            Err(error) => {
                actions.push(action(
                    skill,
                    target,
                    target_path,
                    install,
                    SyncActionKind::Conflict,
                    InstallStatus::Error,
                    Some(error.to_string()),
                ));
                continue;
            }
        };

        let inspection = inspect_target(
            &app_paths.library_dir,
            &skill.id,
            &skill_dir,
            &skill_hash,
            &target_path,
        );
        if inspection.status == InstallStatus::Ok
            && inspection.actual_kind == Some(kind_of(install.mode))
        {
            continue;
        }
        let (kind, reason) = decide(install, &inspection);
        actions.push(action(
            skill,
            target,
            target_path,
            install,
            kind,
            inspection.status,
            reason,
        ));
    }

    Ok(SyncPlan {
        id: new_id(),
        created_at: crate::util::now_iso(),
        expires_at: expires_at(PLAN_TTL.as_secs()),
        actions,
    })
}

fn kind_of(mode: LinkMode) -> ActualKind {
    match mode {
        LinkMode::Symlink => ActualKind::Symlink,
        LinkMode::Copy => ActualKind::Copy,
    }
}

fn passes_filter(filter: &SyncFilter, skill_id: &str, target_id: &str) -> bool {
    if let Some(id) = &filter.skill_id {
        if id != skill_id {
            return false;
        }
    }
    if let Some(id) = &filter.target_id {
        if id != target_id {
            return false;
        }
    }
    true
}

fn cached_hash(
    cache: &mut HashMap<String, Option<String>>,
    skill_id: &str,
    skill_dir: &Path,
) -> AppResult<Option<String>> {
    if let Some(value) = cache.get(skill_id) {
        return Ok(value.clone());
    }
    let value = if skill_dir.is_dir() {
        Some(hash::tree_hash(skill_dir)?)
    } else {
        None
    };
    cache.insert(skill_id.to_string(), value.clone());
    Ok(value)
}

fn decide(install: &Install, inspection: &Inspection) -> (SyncActionKind, Option<String>) {
    let mode_label = match install.mode {
        LinkMode::Symlink => "符号链接",
        LinkMode::Copy => "副本",
    };
    match inspection.status {
        InstallStatus::Missing => {
            let reason = if inspection.owned {
                "重新创建悬空链接"
            } else {
                "创建符号链接或副本"
            };
            (SyncActionKind::Create, Some(reason.to_string()))
        }
        InstallStatus::Ok => (
            SyncActionKind::Repair,
            Some(format!("按设置改为{mode_label}")),
        ),
        InstallStatus::Stale => (SyncActionKind::Repair, Some("更新过期副本".to_string())),
        InstallStatus::Conflict => {
            if install.ignored {
                (SyncActionKind::Skip, Some("已保留并忽略冲突".to_string()))
            } else {
                (
                    SyncActionKind::Conflict,
                    Some(
                        inspection
                            .last_error
                            .clone()
                            .unwrap_or_else(|| "目标路径被占用".to_string()),
                    ),
                )
            }
        }
        InstallStatus::Error => (
            SyncActionKind::Repair,
            Some(
                inspection
                    .last_error
                    .clone()
                    .unwrap_or_else(|| "重试上次失败的操作".to_string()),
            ),
        ),
        InstallStatus::Pending => (
            SyncActionKind::Repair,
            Some("同步状态未知，将重新写入".to_string()),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn action(
    skill: &Skill,
    target: &Target,
    target_path: PathBuf,
    install: &Install,
    kind: SyncActionKind,
    state: InstallStatus,
    reason: Option<String>,
) -> PlanAction {
    PlanAction {
        id: format!("{}:{}", skill.id, target.id),
        skill_id: skill.id.clone(),
        skill_slug: skill.slug.clone(),
        skill_name: skill.name.clone(),
        target_id: target.id.clone(),
        target_name: target.name.clone(),
        target_path: target_path.display().to_string(),
        mode: install.mode,
        kind,
        state,
        reason,
    }
}

fn expires_at(seconds: u64) -> String {
    (time::OffsetDateTime::now_utc() + time::Duration::seconds(seconds as i64))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| crate::util::now_iso())
}
