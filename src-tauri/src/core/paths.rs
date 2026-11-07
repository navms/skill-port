//! 应用数据目录、slug 规则与所有目标写入前的路径守卫。

use std::path::{Component, Path, PathBuf};

use super::error::{AppError, AppResult};

/// 覆盖应用数据目录的环境变量（测试 / 便携模式）。
pub const ENV_DATA_DIR: &str = "SKILLPORT_DATA_DIR";
pub const MIN_SLUG_LEN: usize = 2;
pub const MAX_SLUG_LEN: usize = 63;
/// slug 生成失败时的兜底名。
pub const SLUG_FALLBACK: &str = "skill";
const RESERVED_SLUGS: [&str; 2] = ["skillport", "node_modules"];

/// 应用数据目录下的固定布局。
#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub library_dir: PathBuf,
    pub db_path: PathBuf,
    pub backups_dir: PathBuf,
    pub logs_dir: PathBuf,
    /// harness 图标：放在数据目录里，用户可直接增删，无需重新打包。
    pub icons_dir: PathBuf,
}

impl AppPaths {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        Self {
            library_dir: data_dir.join("library"),
            db_path: data_dir.join("db.sqlite3"),
            backups_dir: data_dir.join("backups"),
            logs_dir: data_dir.join("logs"),
            icons_dir: data_dir.join("icons"),
            data_dir,
        }
    }

    /// 读取 `SKILLPORT_DATA_DIR`；为空视为未设置。
    pub fn env_data_dir() -> Option<PathBuf> {
        env_data_dir()
    }

    /// 创建库、备份、日志、图标目录。
    pub fn ensure_dirs(&self) -> AppResult<()> {
        for dir in [
            &self.data_dir,
            &self.library_dir,
            &self.backups_dir,
            &self.logs_dir,
            &self.icons_dir,
        ] {
            std::fs::create_dir_all(dir).map_err(|error| AppError::io_at(dir, error))?;
        }
        Ok(())
    }
}

/// 读取 `SKILLPORT_DATA_DIR`；为空视为未设置。
pub fn env_data_dir() -> Option<PathBuf> {
    std::env::var_os(ENV_DATA_DIR)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// 校验 slug：`^[a-z0-9]+(-[a-z0-9]+)*$`，2–63 字符，拒绝保留名。
pub fn validate_slug(slug: &str) -> AppResult<()> {
    if slug.len() < MIN_SLUG_LEN || slug.len() > MAX_SLUG_LEN {
        return Err(AppError::InvalidSlug(format!(
            "长度必须在 {MIN_SLUG_LEN}–{MAX_SLUG_LEN} 之间：{slug}"
        )));
    }
    if !slug.is_ascii() {
        return Err(AppError::InvalidSlug(format!("只允许 ASCII：{slug}")));
    }
    if slug.contains("..") || RESERVED_SLUGS.contains(&slug) {
        return Err(AppError::InvalidSlug(format!("保留名称：{slug}")));
    }
    let valid = slug.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    });
    if !valid {
        return Err(AppError::InvalidSlug(format!(
            "只允许小写字母、数字和单个连字符：{slug}"
        )));
    }
    Ok(())
}

/// 从显示名清洗出 slug；结果不满足长度时返回兜底名。
pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in name.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(lower);
        } else {
            pending_dash = true;
        }
    }
    if out.len() > MAX_SLUG_LEN {
        out.truncate(MAX_SLUG_LEN);
        while out.ends_with('-') {
            out.pop();
        }
    }
    if out.len() < MIN_SLUG_LEN {
        return SLUG_FALLBACK.to_string();
    }
    out
}

/// 生成不与 `taken` 冲突的 slug；冲突时追加 `-2`、`-3`…。
pub fn unique_slug(base: &str, taken: &[String]) -> String {
    let mut base = if base.is_empty() {
        SLUG_FALLBACK.to_string()
    } else {
        base.to_string()
    };
    if base.len() > MAX_SLUG_LEN {
        base.truncate(MAX_SLUG_LEN);
        while base.ends_with('-') {
            base.pop();
        }
    }
    if !taken.iter().any(|item| item == &base) {
        return base;
    }
    for suffix_number in 2..10_000u32 {
        let suffix = format!("-{suffix_number}");
        let max_stem = MAX_SLUG_LEN.saturating_sub(suffix.len());
        let mut stem = base.clone();
        if stem.len() > max_stem {
            stem.truncate(max_stem);
            while stem.ends_with('-') {
                stem.pop();
            }
        }
        let candidate = format!("{stem}{suffix}");
        if !taken.iter().any(|item| item == &candidate) {
            return candidate;
        }
    }
    SLUG_FALLBACK.to_string()
}

/// 词法归一化：折叠 `.` 与 `..`，不访问文件系统、不跟随符号链接。
pub fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if out.parent().is_some() {
                    out.pop();
                } else if !out.has_root() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// 尽可能 canonicalize，失败时退回词法归一化。
pub fn canonical_or_normalized(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| normalize_lexically(path))
}

/// `child` 是否位于 `parent` 内部（`child == parent` 也算）。
pub fn is_inside(child: &Path, parent: &Path) -> bool {
    let child = canonical_or_normalized(child);
    let parent = canonical_or_normalized(parent);
    child == parent || child.starts_with(&parent)
}

/// 校验库目录与所有目标目录不互相包含。
pub fn validate_library_placement(
    library_dir: &Path,
    skills_dirs: impl IntoIterator<Item = PathBuf>,
) -> AppResult<()> {
    let library = canonical_or_normalized(library_dir);
    for skills_dir in skills_dirs {
        let target = canonical_or_normalized(&skills_dir);
        if library == target || library.starts_with(&target) || target.starts_with(&library) {
            return Err(AppError::InvalidInput(format!(
                "库目录 {} 不能与目标目录 {} 互相包含，请修改目标路径",
                library.display(),
                target.display()
            )));
        }
    }
    Ok(())
}

/// 校验目标 skills_dir 是绝对路径并归一化。
pub fn validate_skills_dir(raw: &str) -> AppResult<PathBuf> {
    if raw.trim().is_empty() {
        return Err(AppError::InvalidInput("目标目录不能为空".to_string()));
    }
    if raw.contains('\0') {
        return Err(AppError::InvalidInput("目标目录包含非法字符".to_string()));
    }
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err(AppError::InvalidInput(format!(
            "目标目录必须是绝对路径：{}",
            path.display()
        )));
    }
    Ok(normalize_lexically(&path))
}

/// 由 `skills_dir` 与 slug 得到目标路径，并做静态守卫。
pub fn guarded_target_path(skills_dir: &Path, slug: &str) -> AppResult<PathBuf> {
    validate_slug(slug)?;
    if !skills_dir.is_absolute() {
        return Err(AppError::InvalidInput(format!(
            "目标目录必须是绝对路径：{}",
            skills_dir.display()
        )));
    }
    let dir = normalize_lexically(skills_dir);
    let target = dir.join(slug);
    let Some(parent) = target.parent() else {
        return Err(AppError::PathEscape(target.display().to_string()));
    };
    if parent != dir.as_path() || target.file_name() != Some(std::ffi::OsStr::new(slug)) {
        return Err(AppError::PathEscape(target.display().to_string()));
    }
    Ok(target)
}

/// 实际写入 / 删除前的守卫：父目录必须就是 canonical 后的 skills_dir。
///
/// 要求 `skills_dir` 已存在（调用方先 `create_dir_all`）。
pub fn ensure_write_allowed(skills_dir: &Path, target_path: &Path, slug: &str) -> AppResult<()> {
    validate_slug(slug)?;
    let dir = normalize_lexically(skills_dir);
    let target = normalize_lexically(target_path);
    let Some(parent) = target.parent() else {
        return Err(AppError::PathEscape(target.display().to_string()));
    };
    if parent != dir.as_path() || target.file_name() != Some(std::ffi::OsStr::new(slug)) {
        return Err(AppError::PathEscape(format!(
            "目标路径必须直接位于 {} 下：{}",
            dir.display(),
            target.display()
        )));
    }
    let canonical_dir =
        std::fs::canonicalize(&dir).map_err(|error| AppError::io_at(&dir, error))?;
    let canonical_parent =
        std::fs::canonicalize(parent).map_err(|error| AppError::io_at(parent, error))?;
    if canonical_parent != canonical_dir || !canonical_parent.starts_with(&canonical_dir) {
        return Err(AppError::PathEscape(format!(
            "目标父目录解析后越出 skills_dir：{}",
            canonical_parent.display()
        )));
    }
    Ok(())
}
