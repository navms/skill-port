//! 受控文件系统操作：sidecar、符号链接、拷贝、备份、临时目录清理。
//!
//! 所有删除入口都假定调用方已经通过 `sync::delete_owned_*` 的归属校验。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use super::error::{AppError, AppResult};
use super::hash::{is_excluded_name, SIDECAR_NAME};
use super::paths::normalize_lexically;
use super::util::compact_stamp;

pub const SIDECAR_SCHEMA: u32 = 1;
pub const MANAGED_BY: &str = "skillport";
pub const TMP_PREFIX: &str = ".skillport-tmp-";

/// 拷贝到目标目录时写入的归属标记。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sidecar {
    pub schema: u32,
    pub managed_by: String,
    pub skill_id: String,
    pub source_hash: String,
}

impl Sidecar {
    pub fn new(skill_id: &str, source_hash: &str) -> Self {
        Self {
            schema: SIDECAR_SCHEMA,
            managed_by: MANAGED_BY.to_string(),
            skill_id: skill_id.to_string(),
            source_hash: source_hash.to_string(),
        }
    }
}

/// 读取并校验 sidecar；缺失或不属于本工具时返回 `None`。
pub fn read_sidecar(dir: &Path) -> Option<Sidecar> {
    let text = std::fs::read_to_string(dir.join(SIDECAR_NAME)).ok()?;
    let sidecar: Sidecar = serde_json::from_str(&text).ok()?;
    if sidecar.schema == SIDECAR_SCHEMA && sidecar.managed_by == MANAGED_BY {
        Some(sidecar)
    } else {
        None
    }
}

pub fn write_sidecar(dir: &Path, sidecar: &Sidecar) -> AppResult<()> {
    let path = dir.join(SIDECAR_NAME);
    let json = serde_json::to_vec_pretty(sidecar)?;
    std::fs::write(&path, json).map_err(|error| AppError::io_at(&path, error))
}

fn map_symlink_error(link: &Path, error: std::io::Error) -> AppError {
    let privileged =
        error.kind() == std::io::ErrorKind::PermissionDenied || error.raw_os_error() == Some(1314);
    if privileged {
        AppError::SymlinkPrivilege {
            path: link.display().to_string(),
            detail: "创建符号链接需要权限：Windows 可开启开发者模式或以管理员身份运行，\
                     也可以把该技能改为 copy 模式。"
                .to_string(),
        }
    } else {
        AppError::io_at(link, error)
    }
}

/// 创建目录符号链接。
pub fn create_dir_symlink(link: &Path, target: &Path) -> AppResult<()> {
    #[cfg(unix)]
    let result = std::os::unix::fs::symlink(target, link);
    #[cfg(windows)]
    let result = std::os::windows::fs::symlink_dir(target, link);
    #[cfg(not(any(unix, windows)))]
    let result: std::io::Result<()> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "当前平台不支持符号链接",
    ));
    result.map_err(|error| map_symlink_error(link, error))
}

/// 创建文件符号链接（仅用于复制技能目录内部的相对链接）。
pub fn create_file_symlink(link: &Path, target: &Path) -> AppResult<()> {
    #[cfg(unix)]
    let result = std::os::unix::fs::symlink(target, link);
    #[cfg(windows)]
    let result = std::os::windows::fs::symlink_file(target, link);
    #[cfg(not(any(unix, windows)))]
    let result: std::io::Result<()> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "当前平台不支持符号链接",
    ));
    result.map_err(|error| map_symlink_error(link, error))
}

/// 删除链接本身（不跟随）；不存在视为成功。
pub fn remove_symlink(link: &Path) -> AppResult<()> {
    #[cfg(unix)]
    {
        match std::fs::remove_file(link) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(AppError::io_at(link, error)),
        }
    }
    #[cfg(windows)]
    {
        match std::fs::remove_dir(link) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(dir_error) => match std::fs::remove_file(link) {
                Ok(()) => Ok(()),
                Err(file_error) if file_error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(_) => Err(AppError::io_at(link, dir_error)),
            },
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = link;
        Err(AppError::Internal("当前平台不支持符号链接".to_string()))
    }
}

/// 解析链接目标为绝对、词法归一化路径（不要求目标存在）。
pub fn resolve_link(link: &Path) -> AppResult<PathBuf> {
    let target = std::fs::read_link(link).map_err(|error| AppError::io_at(link, error))?;
    let resolved = if target.is_absolute() {
        target
    } else {
        match link.parent() {
            Some(parent) => parent.join(target),
            None => target,
        }
    };
    Ok(normalize_lexically(&resolved))
}

/// 链接是否指向 `root` 内部（符号链接本身不被跟随）。
pub fn symlink_points_into(link: &Path, root: &Path) -> AppResult<bool> {
    let resolved = resolve_link(link)?;
    let root_lex = normalize_lexically(root);
    if resolved.starts_with(&root_lex) {
        return Ok(true);
    }
    let real = std::fs::canonicalize(&resolved).unwrap_or(resolved);
    let root_real = std::fs::canonicalize(root).unwrap_or(root_lex);
    Ok(real.starts_with(root_real))
}

fn walk_error(error: walkdir::Error) -> AppError {
    AppError::Io {
        message: error.to_string(),
        path: error.path().map(|path| path.display().to_string()),
    }
}

fn relative_has_excluded(relative: &Path) -> bool {
    relative
        .components()
        .any(|component| is_excluded_name(&component.as_os_str().to_string_lossy()))
}

/// 复制目录树；跳过 sidecar / 系统垃圾文件，拒绝指向技能目录外部的符号链接。
pub fn copy_tree(src: &Path, dst: &Path, label: &str) -> AppResult<()> {
    std::fs::create_dir_all(dst).map_err(|error| AppError::io_at(dst, error))?;
    let src_real = std::fs::canonicalize(src).unwrap_or_else(|_| normalize_lexically(src));
    for item in WalkDir::new(src)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| !is_excluded_name(&entry.file_name().to_string_lossy()))
    {
        let entry = item.map_err(walk_error)?;
        if entry.path() == src {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(src)
            .map_err(|error| AppError::Internal(error.to_string()))?;
        if relative_has_excluded(relative) {
            continue;
        }
        let target = dst.join(relative);
        let file_type = entry.file_type();
        if file_type.is_dir() {
            std::fs::create_dir_all(&target).map_err(|error| AppError::io_at(&target, error))?;
        } else if file_type.is_symlink() {
            let resolved = resolve_link(entry.path())?;
            let inside = resolved.starts_with(&src_real) || resolved.starts_with(src);
            if !inside {
                return Err(AppError::Validation(format!(
                    "{label} 含指向技能目录外部的符号链接：{} → {}",
                    relative.display(),
                    resolved.display()
                )));
            }
            let link_target = std::fs::read_link(entry.path())
                .map_err(|error| AppError::io_at(entry.path(), error))?;
            match std::fs::metadata(entry.path()) {
                Ok(meta) if meta.is_dir() => create_dir_symlink(&target, &link_target)?,
                Ok(_) => create_file_symlink(&target, &link_target)?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Err(AppError::Validation(format!(
                        "{label} 含悬空符号链接：{}",
                        relative.display()
                    )))
                }
                Err(error) => return Err(AppError::io_at(entry.path(), error)),
            }
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &target)
                .map_err(|error| AppError::io_at(entry.path(), error))?;
        }
    }
    Ok(())
}

/// 把已确认归属本工具的目录移入备份目录；跨卷时退回复制 + 删除。
pub fn backup_dir(
    backups_root: &Path,
    skill_slug: &str,
    target_slug: &str,
    source: &Path,
    keep: usize,
) -> AppResult<Option<PathBuf>> {
    if !source.exists() && std::fs::symlink_metadata(source).is_err() {
        return Ok(None);
    }
    std::fs::create_dir_all(backups_root).map_err(|error| AppError::io_at(backups_root, error))?;
    let stamp = compact_stamp();
    let mut destination = backups_root.join(format!("{stamp}-{skill_slug}-{target_slug}"));
    let mut attempt = 2u32;
    while destination.exists() {
        destination = backups_root.join(format!("{stamp}-{skill_slug}-{target_slug}-{attempt}"));
        attempt += 1;
        if attempt > 999 {
            return Err(AppError::Internal("备份名称冲突过多".to_string()));
        }
    }
    if std::fs::rename(source, &destination).is_err() {
        copy_tree(source, &destination, "备份")?;
        std::fs::remove_dir_all(source).map_err(|error| AppError::io_at(source, error))?;
    }
    prune_backups(backups_root, skill_slug, target_slug, keep)?;
    Ok(Some(destination))
}

fn is_backup_for(name: &str, skill_slug: &str, target_slug: &str) -> bool {
    let marker = format!("-{skill_slug}-{target_slug}");
    let Some(position) = name.find(&marker) else {
        return false;
    };
    let tail = &name[position + marker.len()..];
    if tail.is_empty() {
        return true;
    }
    tail.strip_prefix('-')
        .map(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .unwrap_or(false)
}

/// 每个技能 × 目标只保留最近 `keep` 份备份。
pub fn prune_backups(
    backups_root: &Path,
    skill_slug: &str,
    target_slug: &str,
    keep: usize,
) -> AppResult<()> {
    let Ok(entries) = std::fs::read_dir(backups_root) else {
        return Ok(());
    };
    let mut matches: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            is_backup_for(&name, skill_slug, target_slug).then(|| (name, entry.path()))
        })
        .collect();
    if matches.len() <= keep {
        return Ok(());
    }
    // 名称以时间戳开头，字典序即时间序；保留末尾（最新）的 keep 份。
    matches.sort_by(|left, right| left.0.cmp(&right.0));
    let remove_count = matches.len().saturating_sub(keep);
    for (_, path) in matches.into_iter().take(remove_count) {
        if std::fs::symlink_metadata(&path)
            .map(|meta| meta.is_dir())
            .unwrap_or(false)
        {
            std::fs::remove_dir_all(&path).map_err(|error| AppError::io_at(&path, error))?;
        }
    }
    Ok(())
}

/// 启动时清理本工具命名的临时目录；只匹配 `.skillport-tmp-*`。
pub fn cleanup_temp_dirs(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.file_name().to_string_lossy().starts_with(TMP_PREFIX) {
            continue;
        }
        let path = entry.path();
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            let _ = remove_symlink(&path);
        } else if meta.is_dir() {
            let _ = std::fs::remove_dir_all(&path);
        } else if meta.is_file() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// 列出目录下的条目名（用于未托管检测）。
pub fn list_entry_names(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}
