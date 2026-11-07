//! 目标（harness）的 CRUD、默认路径探测与目录合法性校验。

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::error::{AppError, AppResult};
use super::model::*;
use super::paths::{self, AppPaths};
use super::util::{new_id, now_iso};
use super::{repo, settings, sync};

/// 已知 harness 的候选技能目录（相对用户主目录）。
///
/// 注意：这些路径按 harness 文档整理，发布前仍需逐版本核实（见 `AGENTS.md` §18）。
/// 检测不到时由用户手动填写，不做猜测性写入。
const CANDIDATES: [(&str, &str, &str); 5] = [
    ("claude-code", "Claude Code", ".claude/skills"),
    ("codex", "Codex CLI", ".codex/skills"),
    ("cursor", "Cursor", ".cursor/skills"),
    ("gemini-cli", "Gemini CLI", ".gemini/skills"),
    ("opencode", "OpenCode", ".config/opencode/skills"),
];

pub fn list(conn: &Connection) -> AppResult<Vec<Target>> {
    repo::list_targets(conn)
}

pub fn create(conn: &Connection, paths: &AppPaths, input: &TargetCreateInput) -> AppResult<Target> {
    paths::validate_slug(&input.slug)?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("目标名称不能为空".to_string()));
    }
    let skills_dir = paths::validate_skills_dir(&input.skills_dir)?;
    if skills_dir.exists() && !skills_dir.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "目标路径已存在但不是目录：{}",
            skills_dir.display()
        )));
    }
    if repo::find_target_by_slug(conn, &input.slug)?.is_some() {
        return Err(AppError::Conflict(format!(
            "目标 slug 已存在：{}",
            input.slug
        )));
    }
    let mut dirs: Vec<PathBuf> = repo::list_targets(conn)?
        .into_iter()
        .map(|target| PathBuf::from(target.skills_dir))
        .collect();
    dirs.push(skills_dir.clone());
    paths::validate_library_placement(&paths.library_dir, dirs)?;

    let now = now_iso();
    let target = Target {
        id: new_id(),
        slug: input.slug.clone(),
        name: name.to_string(),
        kind: "skills_dir".to_string(),
        skills_dir: skills_dir.display().to_string(),
        link_mode: input.link_mode,
        enabled: input.enabled,
        detected: false,
        created_at: now.clone(),
        updated_at: now,
    };
    repo::insert_target(conn, &target).map_err(map_constraint)?;
    repo::log_activity(
        conn,
        &repo::ActivityRecord::ok("target")
            .target(&target)
            .detail(&format!("添加目标 {}", target.name)),
    )?;
    Ok(target)
}

pub fn update(
    conn: &Connection,
    paths: &AppPaths,
    id: &str,
    patch: &TargetUpdatePatch,
) -> AppResult<Target> {
    let mut target = repo::get_target(conn, id)?;
    if let Some(name) = &patch.name {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::InvalidInput("目标名称不能为空".to_string()));
        }
        target.name = name.to_string();
    }
    if let Some(slug) = &patch.slug {
        paths::validate_slug(slug)?;
        if slug != &target.slug {
            if let Some(existing) = repo::find_target_by_slug(conn, slug)? {
                if existing.id != target.id {
                    return Err(AppError::Conflict(format!("目标 slug 已存在：{slug}")));
                }
            }
            target.slug = slug.clone();
        }
    }
    let mut skills_dir_changed = false;
    if let Some(raw) = &patch.skills_dir {
        let skills_dir = paths::validate_skills_dir(raw)?;
        if skills_dir.display().to_string() != target.skills_dir {
            if skills_dir.exists() && !skills_dir.is_dir() {
                return Err(AppError::InvalidInput(format!(
                    "目标路径已存在但不是目录：{}",
                    skills_dir.display()
                )));
            }
            let mut dirs: Vec<PathBuf> = repo::list_targets(conn)?
                .into_iter()
                .filter(|item| item.id != id)
                .map(|item| PathBuf::from(item.skills_dir))
                .collect();
            dirs.push(skills_dir.clone());
            paths::validate_library_placement(&paths.library_dir, dirs)?;
            target.skills_dir = skills_dir.display().to_string();
            skills_dir_changed = true;
        }
    }
    if let Some(mode) = patch.link_mode {
        target.link_mode = mode;
    }
    if let Some(enabled) = patch.enabled {
        target.enabled = enabled;
    }
    target.updated_at = now_iso();
    repo::update_target_row(conn, &target).map_err(map_constraint)?;
    if skills_dir_changed {
        repo::retarget_installs(conn, id, &target.skills_dir, &target.updated_at)?;
    }
    repo::log_activity(
        conn,
        &repo::ActivityRecord::ok("target")
            .target(&target)
            .detail(&format!("更新目标 {}", target.name)),
    )?;
    Ok(target)
}

pub fn delete(
    conn: &Connection,
    paths: &AppPaths,
    id: &str,
    options: &DeleteTargetOptions,
) -> AppResult<()> {
    let target = repo::get_target(conn, id)?;
    let keep = settings::get(conn)?.backup_retention as usize;
    for install in repo::list_installs_for_target(conn, id)? {
        let skill = repo::get_skill(conn, &install.skill_id)?;
        if options.unlink {
            let outcome = sync::remove_distribution(paths, &skill, &target, true, keep)?;
            let mut record = repo::ActivityRecord::ok("unlink")
                .skill(&skill)
                .target(&target);
            if let Some(detail) = &outcome.detail {
                record = record.detail(detail);
            }
            repo::log_activity(conn, &record)?;
        }
        repo::delete_install(conn, &skill.id, id)?;
    }
    repo::delete_target_row(conn, id)?;
    repo::log_activity(
        conn,
        &repo::ActivityRecord::ok("target")
            .target(&target)
            .detail(&format!("删除目标 {}", target.name)),
    )?;
    Ok(())
}

/// 依据用户主目录给出候选目标；只报告存在性，不自动创建。
pub fn detect_suggestions(conn: &Connection, home: &Path) -> AppResult<Vec<SuggestedTarget>> {
    let existing: Vec<PathBuf> = repo::list_targets(conn)?
        .into_iter()
        .map(|target| paths::canonical_or_normalized(Path::new(&target.skills_dir)))
        .collect();
    Ok(CANDIDATES
        .iter()
        .map(|(slug, name, relative)| {
            let dir = paths::normalize_lexically(&home.join(relative));
            let normalized = paths::canonical_or_normalized(&dir);
            SuggestedTarget {
                slug: (*slug).to_string(),
                name: (*name).to_string(),
                skills_dir: dir.display().to_string(),
                exists: dir.is_dir(),
                already_added: existing.iter().any(|item| item == &normalized),
            }
        })
        .collect())
}

fn map_constraint(error: AppError) -> AppError {
    if let AppError::Db(message) = &error {
        if message.contains("targets.skills_dir") {
            return AppError::Conflict("该目标目录已被其他目标占用".to_string());
        }
        if message.contains("targets.slug") {
            return AppError::Conflict("目标 slug 已存在".to_string());
        }
    }
    error
}
