//! 技能目录的稳定树 hash。
//!
//! 文件顺序无关；排除 `.skillport.json`、`.DS_Store`、`Thumbs.db`；
//! 符号链接只纳入链接目标字符串，不跟随。

use std::path::Path;

use walkdir::WalkDir;

use super::error::{AppError, AppResult};

pub const SIDECAR_NAME: &str = ".skillport.json";
const EXCLUDED_NAMES: [&str; 2] = [".DS_Store", "Thumbs.db"];

/// 拷贝与 hash 都要跳过的文件名。
pub fn is_excluded_name(name: &str) -> bool {
    name == SIDECAR_NAME || EXCLUDED_NAMES.contains(&name)
}

fn walk_error(error: walkdir::Error) -> AppError {
    AppError::Io {
        message: error.to_string(),
        path: error.path().map(|path| path.display().to_string()),
    }
}

/// 计算目录树 hash（小写十六进制 BLAKE3）。
pub fn tree_hash(root: &Path) -> AppResult<String> {
    let walker = WalkDir::new(root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| !is_excluded_name(name))
                .unwrap_or(true)
        });

    let mut entries: Vec<(String, String)> = Vec::new();
    for item in walker {
        let entry = item.map_err(walk_error)?;
        if entry.path() == root {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| AppError::Internal(error.to_string()))?;
        if relative
            .components()
            .any(|component| is_excluded_name(&component.as_os_str().to_string_lossy()))
        {
            continue;
        }
        let key = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let file_type = entry.file_type();
        if file_type.is_dir() {
            continue;
        }
        if file_type.is_symlink() {
            let link = std::fs::read_link(entry.path())
                .map_err(|error| AppError::io_at(entry.path(), error))?;
            entries.push((key, format!("link:{}", link.to_string_lossy())));
        } else if file_type.is_file() {
            let bytes = std::fs::read(entry.path())
                .map_err(|error| AppError::io_at(entry.path(), error))?;
            entries.push((key, format!("file:{}", blake3::hash(&bytes).to_hex())));
        }
    }

    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = blake3::Hasher::new();
    for (path, value) in entries {
        hasher.update(b"entry\0");
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(value.as_bytes());
        hasher.update(b"\0");
    }
    Ok(hasher.finalize().to_hex().to_string())
}
