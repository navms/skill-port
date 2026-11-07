//! `SKILL.md` 的严格解析与重写。
//!
//! 规则：必须有 YAML frontmatter，`name` 与 `description` 必须是非空字符串；
//! 未知字段原样保留；不自动补字段。

use std::path::Path;

use serde_yaml_ng::{Mapping, Value};

use super::error::{AppError, AppResult};

/// 解析后的技能元数据。
#[derive(Debug, Clone)]
pub struct ParsedSkill {
    pub name: String,
    pub description: String,
    pub frontmatter: Mapping,
    /// 关闭 `---` 之后的内容（原样保留）。
    pub body: String,
}

/// 读取并严格解析 `SKILL.md`。
pub fn parse_skill_md(path: &Path) -> AppResult<ParsedSkill> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::MissingSkillFile(path.display().to_string()));
        }
        Err(error) => return Err(AppError::io_at(path, error)),
    };
    parse_skill_md_str(&path.display().to_string(), &text)
}

/// 从字符串严格解析（用于编辑器中校验后写入）。
pub fn parse_skill_md_str(label: &str, text: &str) -> AppResult<ParsedSkill> {
    let (front, body) = split_frontmatter(text).ok_or_else(|| AppError::InvalidFrontmatter {
        path: label.to_string(),
        detail: "缺少由 --- 包围的 YAML frontmatter".to_string(),
    })?;
    let value: Value =
        serde_yaml_ng::from_str(front).map_err(|error| AppError::InvalidFrontmatter {
            path: label.to_string(),
            detail: error.to_string(),
        })?;
    let frontmatter = match value {
        Value::Mapping(mapping) => mapping,
        Value::Null => Mapping::new(),
        _ => {
            return Err(AppError::InvalidFrontmatter {
                path: label.to_string(),
                detail: "frontmatter 必须是 YAML 映射（key: value）".to_string(),
            })
        }
    };
    let name = required_string(&frontmatter, "name")
        .ok_or_else(|| AppError::MissingName(label.to_string()))?;
    let description = required_string(&frontmatter, "description")
        .ok_or_else(|| AppError::MissingDescription(label.to_string()))?;
    Ok(ParsedSkill {
        name,
        description,
        frontmatter,
        body: body.to_string(),
    })
}

fn required_string(mapping: &Mapping, key: &str) -> Option<String> {
    optional_string(mapping, key)
}

fn optional_string(mapping: &Mapping, key: &str) -> Option<String> {
    let key = Value::String(key.to_string());
    mapping
        .get(&key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// 切出 frontmatter 与正文；容忍 BOM 与 CRLF。
pub fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end_matches(['\r', '\n']).trim() != "---" {
        return None;
    }
    let front_start = first.len();
    let mut consumed = front_start;
    for line in lines {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed == "---" || trimmed == "..." {
            let front = &text[front_start..consumed];
            let body_start = consumed + line.len();
            return Some((front, &text[body_start..]));
        }
        consumed += line.len();
    }
    None
}

/// 设置或覆盖一个 frontmatter 字段。
pub fn set_field(mapping: &mut Mapping, key: &str, value: &str) {
    mapping.insert(
        Value::String(key.to_string()),
        Value::String(value.to_string()),
    );
}

/// 按 frontmatter + 正文重新渲染整篇 `SKILL.md`。
pub fn render_skill_md(frontmatter: &Mapping, body: &str) -> AppResult<String> {
    let yaml = serde_yaml_ng::to_string(frontmatter)?;
    let mut out = String::with_capacity(yaml.len() + body.len() + 8);
    out.push_str("---\n");
    out.push_str(&yaml);
    if !yaml.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n");
    out.push_str(body);
    Ok(out)
}
