use std::fs;

use serde_yaml_ng::Value;
use skillport_lib::core::frontmatter::{
    parse_skill_md, parse_skill_md_str, render_skill_md, split_frontmatter,
};

fn write_skill(text: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempfile::tempdir().expect("temp");
    let path = temp.path().join("SKILL.md");
    fs::write(&path, text).expect("write");
    (temp, path)
}

#[test]
fn parses_valid_frontmatter() {
    let (_temp, path) = write_skill(
        "---\nname: My Skill\ndescription: Does things\nextra: keep-me\n---\n\n# Body\n",
    );
    let parsed = parse_skill_md(&path).expect("解析");
    assert_eq!(parsed.name, "My Skill");
    assert_eq!(parsed.description, "Does things");
    assert!(parsed.body.contains("# Body"));
    assert!(parsed
        .frontmatter
        .contains_key(Value::String("extra".to_string())));
}

#[test]
fn rejects_missing_skill_file() {
    let temp = tempfile::tempdir().expect("temp");
    let error = parse_skill_md(&temp.path().join("SKILL.md")).expect_err("应失败");
    assert_eq!(error.code(), "MissingSkillFile");
}

#[test]
fn rejects_missing_frontmatter() {
    let (_temp, path) = write_skill("# Just markdown\n");
    let error = parse_skill_md(&path).expect_err("应失败");
    assert_eq!(error.code(), "InvalidFrontmatter");
}

#[test]
fn rejects_invalid_yaml() {
    let (_temp, path) = write_skill("---\nname: [unclosed\n---\n");
    let error = parse_skill_md(&path).expect_err("应失败");
    assert_eq!(error.code(), "InvalidFrontmatter");
}

#[test]
fn rejects_missing_name() {
    let (_temp, path) = write_skill("---\ndescription: no name\n---\n");
    let error = parse_skill_md(&path).expect_err("应失败");
    assert_eq!(error.code(), "MissingName");
}

#[test]
fn rejects_missing_description() {
    let (_temp, path) = write_skill("---\nname: No Desc\n---\n");
    let error = parse_skill_md(&path).expect_err("应失败");
    assert_eq!(error.code(), "MissingDescription");
}

#[test]
fn rejects_empty_name() {
    let (_temp, path) = write_skill("---\nname: '   '\ndescription: x\n---\n");
    let error = parse_skill_md(&path).expect_err("应失败");
    assert_eq!(error.code(), "MissingName");
}

#[test]
fn split_handles_crlf_and_bom() {
    let text = "\u{feff}---\r\nname: X\r\ndescription: Y\r\n---\r\nbody";
    let (front, body) = split_frontmatter(text).expect("有 frontmatter");
    assert!(front.contains("name: X"));
    assert_eq!(body, "body");
}

#[test]
fn render_round_trips() {
    let parsed = parse_skill_md_str(
        "inline",
        "---\nname: A\ndescription: B\ncustom: 1\n---\n# Body\n",
    )
    .expect("解析");
    let rendered = render_skill_md(&parsed.frontmatter, &parsed.body).expect("渲染");
    let again = parse_skill_md_str("inline", &rendered).expect("再解析");
    assert_eq!(again.name, "A");
    assert_eq!(again.description, "B");
    assert!(again.body.contains("# Body"));
}
