//! 技能库：导入（严格校验）、列表、详情、编辑、删除。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use walkdir::WalkDir;

use super::error::{AppError, AppResult};
use super::model::*;
use super::paths::{self, AppPaths, SLUG_FALLBACK};
use super::util::{new_id, now_iso};
use super::{frontmatter, fsops, hash, logging, repo, settings, sync, undo};

/// 列出技能及其分发状态。`sizeBytes` 从库目录扫描得到，`category` 来自 DB。
pub fn list(conn: &Connection, paths: &AppPaths) -> AppResult<Vec<SkillSummary>> {
    let skills = repo::list_skills(conn)?;
    let mut installs_by_skill: HashMap<String, Vec<Install>> = HashMap::new();
    for install in repo::list_installs(conn)? {
        installs_by_skill
            .entry(install.skill_id.clone())
            .or_default()
            .push(install);
    }
    Ok(skills
        .into_iter()
        .map(|skill| {
            let installs = installs_by_skill.remove(&skill.id).unwrap_or_default();
            summary(skill, paths, installs)
        })
        .collect())
}

fn summary(mut skill: Skill, paths: &AppPaths, install_count: Vec<Install>) -> SkillSummary {
    let dir = paths.library_dir.join(&skill.slug);
    // 外部直接修改库文件时，用实际内容刷新权威 hash。
    if let Ok(current) = hash::tree_hash(&dir) {
        if current != skill.content_hash {
            skill.content_hash = current;
        }
    }
    SkillSummary {
        id: skill.id,
        slug: skill.slug,
        name: skill.name,
        description: skill.description,
        category: skill.category,
        source_kind: skill.source_kind,
        source_ref: skill.source_ref,
        content_hash: skill.content_hash,
        size_bytes: tree_size(&dir),
        created_at: skill.created_at,
        updated_at: skill.updated_at,
        installs: install_count,
    }
}

fn tree_size(dir: &Path) -> u64 {
    WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            !entry
                .path()
                .components()
                .any(|component| hash::is_excluded_name(&component.as_os_str().to_string_lossy()))
        })
        .filter_map(|entry| entry.metadata().ok().map(|meta| meta.len()))
        .sum()
}

/// 技能详情：完整 `SKILL.md` 与分发状态。
pub fn get(conn: &Connection, paths: &AppPaths, id: &str) -> AppResult<SkillDetail> {
    let skill = repo::get_skill(conn, id)?;
    let dir = paths.library_dir.join(&skill.slug);
    let installs = repo::list_installs_for_skill(conn, id)?;
    let summary = summary(skill, paths, installs);
    Ok(SkillDetail {
        skill_md: std::fs::read_to_string(dir.join("SKILL.md")).unwrap_or_default(),
        path: dir.display().to_string(),
        summary,
    })
}

/// 批量导入；每个路径独立处理，失败不影响其他项，也不写库。
pub fn import(conn: &Connection, paths: &AppPaths, sources: &[String]) -> AppResult<ImportReport> {
    let mut taken = repo::all_slugs(conn)?;
    let mut items = Vec::new();

    for source in sources {
        import_source(conn, paths, source, &mut taken, &mut items);
    }

    let imported = items.iter().filter(|item| item.ok).count();
    let skipped = items.len().saturating_sub(imported);
    if !items.is_empty() {
        repo::log_activity(
            conn,
            &repo::ActivityRecord::ok("import")
                .detail(&format!("导入 {imported} 个技能，{skipped} 个被拒绝")),
        )?;
    }
    Ok(ImportReport {
        items,
        imported,
        skipped,
    })
}

fn import_source(
    conn: &Connection,
    paths: &AppPaths,
    source: &str,
    taken: &mut Vec<String>,
    items: &mut Vec<ImportResultItem>,
) {
    let given = PathBuf::from(source.trim());
    if source.trim().is_empty() {
        items.push(rejected(source, None, "路径为空"));
        return;
    }
    let candidates = match collect_candidates(&given) {
        Ok(candidates) => candidates,
        Err(error) => {
            let code = match &error {
                AppError::NotFound(_) => ImportErrorCode::PathNotFound,
                _ => ImportErrorCode::IoError,
            };
            items.push(ImportResultItem {
                path: source.to_string(),
                slug: None,
                ok: false,
                error_code: Some(code),
                detail: Some(error.to_string()),
            });
            return;
        }
    };
    if candidates.is_empty() {
        items.push(ImportResultItem {
            path: source.to_string(),
            slug: None,
            ok: false,
            error_code: Some(ImportErrorCode::MissingSkillFile),
            detail: Some(format!("{} 下没有 SKILL.md", given.display())),
        });
        return;
    }
    for candidate in candidates {
        match import_one(conn, paths, &candidate, taken) {
            Ok(skill) => {
                logging::write(
                    &paths.logs_dir,
                    "info",
                    "import_ok",
                    Some(&format!(
                        "slug={} hash={} source={}",
                        skill.slug,
                        skill.content_hash,
                        candidate.display()
                    )),
                );
                items.push(ImportResultItem {
                    path: candidate.display().to_string(),
                    slug: Some(skill.slug),
                    ok: true,
                    error_code: None,
                    detail: None,
                });
            }
            Err(error) => {
                logging::write(
                    &paths.logs_dir,
                    "error",
                    "import_failed",
                    Some(&format!(
                        "code={} source={}",
                        error.code(),
                        candidate.display()
                    )),
                );
                items.push(ImportResultItem {
                    path: candidate.display().to_string(),
                    slug: None,
                    ok: false,
                    error_code: Some(import_error_code(&error)),
                    detail: Some(error.to_string()),
                });
            }
        }
    }
}

fn rejected(path: &str, slug: Option<String>, detail: &str) -> ImportResultItem {
    ImportResultItem {
        path: path.to_string(),
        slug,
        ok: false,
        error_code: Some(ImportErrorCode::PathNotFound),
        detail: Some(detail.to_string()),
    }
}

fn import_error_code(error: &AppError) -> ImportErrorCode {
    match error {
        AppError::MissingSkillFile(_) => ImportErrorCode::MissingSkillFile,
        AppError::InvalidFrontmatter { .. } => ImportErrorCode::InvalidFrontmatter,
        AppError::MissingName(_) => ImportErrorCode::MissingName,
        AppError::MissingDescription(_) => ImportErrorCode::MissingDescription,
        AppError::InvalidSlug(_) => ImportErrorCode::InvalidSlug,
        AppError::Conflict(_) => ImportErrorCode::DuplicateSlug,
        _ => ImportErrorCode::IoError,
    }
}

fn collect_candidates(source: &Path) -> AppResult<Vec<PathBuf>> {
    let source = std::fs::canonicalize(source).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => AppError::NotFound(source.display().to_string()),
        _ => AppError::io_at(source, error),
    })?;
    if !source.is_dir() {
        return Err(AppError::Validation(format!(
            "导入路径必须是目录：{}",
            source.display()
        )));
    }
    if source.join("SKILL.md").is_file() {
        return Ok(vec![source]);
    }
    let entries = std::fs::read_dir(&source).map_err(|error| AppError::io_at(&source, error))?;
    let mut candidates: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("SKILL.md").is_file())
        .collect();
    candidates.sort();
    Ok(candidates)
}

fn base_slug(name: &str, dir: &Path) -> String {
    let from_name = paths::slugify(name);
    if from_name != SLUG_FALLBACK {
        return from_name;
    }
    let dir_name = dir
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();
    let from_dir = paths::slugify(&dir_name);
    if from_dir != SLUG_FALLBACK {
        from_dir
    } else {
        from_name
    }
}

fn import_one(
    conn: &Connection,
    paths: &AppPaths,
    source: &Path,
    taken: &mut Vec<String>,
) -> AppResult<Skill> {
    if paths::is_inside(source, &paths.library_dir) {
        return Err(AppError::Validation(
            "不能从 SkillPort 库目录本身导入技能".to_string(),
        ));
    }
    let parsed = frontmatter::parse_skill_md(&source.join("SKILL.md"))?;
    let slug = base_slug(&parsed.name, source);
    if slug == SLUG_FALLBACK {
        return Err(AppError::InvalidSlug(format!(
            "无法从名称或目录名生成 slug：{}",
            source.display()
        )));
    }
    paths::validate_slug(&slug)?;
    if taken.iter().any(|item| item == &slug) {
        return Err(AppError::Conflict(format!("slug 已存在：{slug}")));
    }

    let id = new_id();
    let now = now_iso();
    let tmp = paths
        .library_dir
        .join(format!("{}{}", fsops::TMP_PREFIX, id));
    let destination = paths.library_dir.join(&slug);
    let mut guard = TempGuard::new(vec![tmp.clone(), destination.clone()]);

    fsops::copy_tree(source, &tmp, "导入")?;
    // 以复制后的内容为准再校验一次，避免复制过程中文件变化。
    let copied = frontmatter::parse_skill_md(&tmp.join("SKILL.md"))?;
    let content_hash = hash::tree_hash(&tmp)?;
    std::fs::rename(&tmp, &destination).map_err(|error| AppError::io_at(&destination, error))?;

    let skill = Skill {
        id,
        slug: slug.clone(),
        name: copied.name,
        description: copied.description,
        // 分类是 SkillPort 自己的口径，导入时一律为空，用户需要时再设。
        category: None,
        source_kind: SourceKind::Import,
        source_ref: Some(source.display().to_string()),
        content_hash,
        created_at: now.clone(),
        updated_at: now,
    };
    repo::insert_skill(conn, &skill)?;
    guard.disarm();
    taken.push(slug);
    Ok(skill)
}

/// 编辑技能：整篇 `SKILL.md`、仅 name / description，或仅分类。
///
/// 分类是 SkillPort 自有字段，只写 DB；只有内容字段才会改动库目录与内容 hash。
pub fn update(
    conn: &Connection,
    paths: &AppPaths,
    id: &str,
    patch: &SkillPatch,
) -> AppResult<SkillDetail> {
    let mut skill = repo::get_skill(conn, id)?;
    let dir = paths.library_dir.join(&skill.slug);
    let skill_md = dir.join("SKILL.md");

    // `None` = 不改；`Some(None)` = 清除；`Some(Some(v))` = 设为 v。
    let category_patch = patch
        .category
        .as_deref()
        .map(|raw| {
            let trimmed = raw.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        // 与现状相同就当作没改，避免空写与多余的 updated_at / 活动记录。
        .filter(|next| next != &skill.category);
    let edits_file =
        patch.name.is_some() || patch.description.is_some() || patch.skill_md.is_some();

    if !edits_file && category_patch.is_none() {
        return get(conn, paths, id);
    }
    let name_patch = clean_field(patch.name.as_deref(), "name")?;
    let description_patch = clean_field(patch.description.as_deref(), "description")?;

    if edits_file {
        let (name, description, text) = if let Some(content) = &patch.skill_md {
            let parsed = frontmatter::parse_skill_md_str(&skill_md.display().to_string(), content)?;
            let mut mapping = parsed.frontmatter.clone();
            let mut changed = false;
            if let Some(name) = &name_patch {
                frontmatter::set_field(&mut mapping, "name", name);
                changed = true;
            }
            if let Some(description) = &description_patch {
                frontmatter::set_field(&mut mapping, "description", description);
                changed = true;
            }
            let text = if changed {
                frontmatter::render_skill_md(&mapping, &parsed.body)?
            } else {
                content.clone()
            };
            (
                name_patch.clone().unwrap_or(parsed.name),
                description_patch.clone().unwrap_or(parsed.description),
                text,
            )
        } else {
            let parsed = frontmatter::parse_skill_md(&skill_md)?;
            let mut mapping = parsed.frontmatter.clone();
            if let Some(name) = &name_patch {
                frontmatter::set_field(&mut mapping, "name", name);
            }
            if let Some(description) = &description_patch {
                frontmatter::set_field(&mut mapping, "description", description);
            }
            let text = frontmatter::render_skill_md(&mapping, &parsed.body)?;
            (
                name_patch.unwrap_or(parsed.name),
                description_patch.unwrap_or(parsed.description),
                text,
            )
        };

        write_text_atomic(paths, &skill_md, &text)?;
        skill.name = name;
        skill.description = description;
        skill.content_hash = hash::tree_hash(&dir)?;
    }
    // 分类只写 DB，绝不回写 SKILL.md。
    if let Some(category) = category_patch {
        skill.category = category;
    }
    skill.updated_at = now_iso();
    repo::update_skill(conn, &skill)?;
    let detail = if edits_file {
        format!("{} 的 SKILL.md 已更新", skill.slug)
    } else {
        format!("{} 的分类已更新", skill.slug)
    };
    repo::log_activity(
        conn,
        &repo::ActivityRecord::ok("update")
            .skill(&skill)
            .detail(&detail),
    )?;
    if edits_file {
        // 写操作之后立刻核对，让副本目标显示为过期。
        sync::reconcile(conn, paths)?;
    }
    get(conn, paths, id)
}

fn clean_field(value: Option<&str>, field: &str) -> AppResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Err(AppError::InvalidInput(format!("{field} 不能为空")))
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
    }
}

fn write_text_atomic(paths: &AppPaths, destination: &Path, text: &str) -> AppResult<()> {
    let temp = paths
        .library_dir
        .join(format!("{}{}", fsops::TMP_PREFIX, new_id()));
    std::fs::write(&temp, text).map_err(|error| AppError::io_at(&temp, error))?;
    let result = replace_file(&temp, destination);
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result.map_err(|error| AppError::io_at(destination, error))
}

fn replace_file(temp: &Path, destination: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    if destination.exists() {
        // Windows 的 rename 不能覆盖已存在文件；先移除再 rename。
        std::fs::remove_file(destination)?;
    }
    std::fs::rename(temp, destination)
}

/// 删除技能：先清理目标目录中本工具拥有的分发，再移走库目录，最后删 DB 行。
pub fn delete(
    conn: &Connection,
    paths: &AppPaths,
    id: &str,
    options: &DeleteSkillOptions,
) -> AppResult<DeleteSkillResult> {
    let skill = repo::get_skill(conn, id)?;
    let targets = repo::list_targets(conn)?;
    let installs = repo::list_installs_for_skill(conn, id)?;
    let keep = settings::get(conn)?.backup_retention as usize;
    let removed_installs = installs.len();

    for install in &installs {
        if let Some(target) = targets.iter().find(|item| item.id == install.target_id) {
            let outcome =
                sync::remove_distribution(paths, &skill, target, options.delete_targets, keep)?;
            let mut record = repo::ActivityRecord::ok("unlink")
                .skill(&skill)
                .target(target);
            if let Some(detail) = &outcome.detail {
                record = record.detail(detail);
            }
            repo::log_activity(conn, &record)?;
        }
    }

    let skill_dir = paths.library_dir.join(&skill.slug);
    let backup = if std::fs::symlink_metadata(&skill_dir).is_ok() {
        fsops::backup_dir(&paths.backups_dir, &skill.slug, "library", &skill_dir, keep)?
    } else {
        None
    };
    repo::delete_skill_row(conn, id)?;

    let detail = if options.delete_targets {
        format!("已删除 {}，并清理 {removed_installs} 条链接", skill.slug)
    } else {
        format!("已删除 {}，目标目录中的副本保留", skill.slug)
    };
    let activity_id = repo::log_activity(
        conn,
        &repo::ActivityRecord::ok("delete")
            .skill(&skill)
            .detail(&detail),
    )?;

    let undo_id = match &backup {
        Some(path) => {
            undo::record_skill_delete(conn, &activity_id.to_string(), &skill, &installs, path)?;
            Some(activity_id.to_string())
        }
        None => None,
    };
    Ok(DeleteSkillResult {
        undo_id,
        removed_installs,
    })
}

/// 失败/中断时清理本工具刚创建的目录。
struct TempGuard {
    paths: Vec<PathBuf>,
    armed: bool,
}

impl TempGuard {
    fn new(paths: Vec<PathBuf>) -> Self {
        Self { paths, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TempGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        for path in &self.paths {
            if let Ok(meta) = std::fs::symlink_metadata(path) {
                if meta.is_dir() {
                    let _ = std::fs::remove_dir_all(path);
                } else {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}
