//! Harness 图标：从应用数据目录读取，首次启动时补齐内置默认图标。
//!
//! 图标外置在 `<data_dir>/icons/<slug>.svg`，用户可以直接增删，不需要重新打包。
//! 默认图标只是种子：已存在的文件永远不覆盖。

use std::io::Write;
use std::path::{Path, PathBuf};

use super::error::{AppError, AppResult};
use super::model::TargetIconAsset;
use super::paths::{self, AppPaths};

/// 内置默认图标，文件名即 target slug。
const DEFAULTS: &[(&str, &str)] = &[
    (
        "claude-code",
        include_str!("../../../src/assets/targets/claude-code.svg"),
    ),
    (
        "codex",
        include_str!("../../../src/assets/targets/codex.svg"),
    ),
    (
        "cursor",
        include_str!("../../../src/assets/targets/cursor.svg"),
    ),
    (
        "deepseek",
        include_str!("../../../src/assets/targets/deepseek.svg"),
    ),
    (
        "gemini-cli",
        include_str!("../../../src/assets/targets/gemini-cli.svg"),
    ),
    (
        "opencode",
        include_str!("../../../src/assets/targets/opencode.svg"),
    ),
    (
        "qoder",
        include_str!("../../../src/assets/targets/qoder.svg"),
    ),
    (
        "workbuddy",
        include_str!("../../../src/assets/targets/workbuddy.svg"),
    ),
];

/// 单个图标文件的读取上限；超出视为用户放错了文件。
const MAX_ICON_BYTES: u64 = 256 * 1024;

/// 补齐缺失的默认图标。已有文件（包括用户替换的版本）一律不动。
///
/// 用 `create_new` 打开：路径上已有任何东西（普通文件、目录、符号链接，
/// 包括悬空链接）都会返回 `AlreadyExists` 并跳过，绝不跟随链接写到目录之外。
pub fn ensure_defaults(paths: &AppPaths) -> AppResult<()> {
    for (slug, svg) in DEFAULTS {
        let file = paths.icons_dir.join(format!("{slug}.svg"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)
        {
            Ok(mut handle) => handle
                .write_all(svg.as_bytes())
                .map_err(|error| AppError::io_at(&file, error))?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(AppError::io_at(&file, error)),
        }
    }
    Ok(())
}

/// 读取 icons 目录下的全部 SVG。
///
/// 只接受普通文件（不跟随符号链接）、合法 slug 文件名、UTF-8 文本、上限内的体积，
/// 且内容确实是一个 SVG 文档；其余条目静默跳过，让一个坏文件不会影响其它图标。
pub fn list(paths: &AppPaths) -> AppResult<Vec<TargetIconAsset>> {
    let dir = &paths.icons_dir;
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(AppError::io_at(dir, error)),
    };

    let mut icons = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(slug) = icon_slug(&path) else {
            continue;
        };
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if !meta.file_type().is_file() || meta.len() > MAX_ICON_BYTES {
            continue;
        }
        let Ok(svg) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !is_svg_document(&svg) {
            continue;
        }
        icons.push(TargetIconAsset {
            slug: slug.to_string(),
            svg,
        });
    }
    icons.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(icons)
}

/// 图标目录本身；用于「打开图标目录」。
pub fn dir(paths: &AppPaths) -> PathBuf {
    paths.icons_dir.clone()
}

/// 内容是不是一个 SVG 文档：跳过 BOM、空白、XML 声明、注释与 DOCTYPE 后，
/// 根元素必须是 `<svg`。只看开头，不引入 XML 解析器。
fn is_svg_document(svg: &str) -> bool {
    let mut rest = svg.trim_start_matches('\u{feff}').trim_start();
    loop {
        if let Some(after) = rest.strip_prefix("<?") {
            let Some(end) = after.find("?>") else {
                return false;
            };
            rest = after[end + 2..].trim_start();
        } else if let Some(after) = rest.strip_prefix("<!--") {
            let Some(end) = after.find("-->") else {
                return false;
            };
            rest = after[end + 3..].trim_start();
        } else if rest.starts_with("<!DOCTYPE") || rest.starts_with("<!doctype") {
            let Some(end) = rest.find('>') else {
                return false;
            };
            rest = rest[end + 1..].trim_start();
        } else {
            break;
        }
    }
    rest.strip_prefix("<svg").is_some_and(|tail| {
        tail.chars().next().map_or(true, |next| {
            next.is_whitespace() || next == '>' || next == '/'
        })
    })
}

/// 文件名必须是 `<合法 slug>.svg`。
fn icon_slug(path: &Path) -> Option<&str> {
    if path.extension().and_then(|value| value.to_str()) != Some("svg") {
        return None;
    }
    let slug = path.file_stem().and_then(|value| value.to_str())?;
    paths::validate_slug(slug).ok()?;
    Some(slug)
}
