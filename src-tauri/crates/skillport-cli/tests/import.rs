//! 端到端：真正启动 CLI 进程，用临时数据目录导入技能并检查 JSON 报告。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

fn write_skill(root: &Path, dir_name: &str, skill_md: &str) -> PathBuf {
    let dir = root.join(dir_name);
    fs::create_dir_all(&dir).expect("create skill dir");
    fs::write(dir.join("SKILL.md"), skill_md).expect("write SKILL.md");
    dir
}

fn run_import(data_dir: &Path, source: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_skillport-cli"))
        .arg("import")
        .arg(source)
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--json")
        .env_remove("SKILLPORT_DATA_DIR")
        .output()
        .expect("run skillport-cli")
}

fn parse_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout 不是 JSON：{error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn imports_skill_and_creates_library_entry() {
    let temp = TempDir::new().expect("temp");
    let source = write_skill(
        temp.path(),
        "alpha",
        "---\nname: Alpha Skill\ndescription: does alpha things\n---\n# Alpha\n",
    );
    let data_dir = temp.path().join("data");

    let output = run_import(&data_dir, &source);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json(&output);
    assert_eq!(json["schema"], 1);
    assert_eq!(json["ok"], true);
    assert_eq!(json["imported"], 1);
    assert_eq!(json["skipped"], 0);
    assert_eq!(json["items"][0]["slug"], "alpha-skill");
    assert_eq!(json["items"][0]["ok"], true);
    assert_eq!(
        json["libraryDir"],
        data_dir.join("library").display().to_string()
    );
    assert!(data_dir.join("library/alpha-skill/SKILL.md").is_file());
    assert!(data_dir.join("db.sqlite3").is_file());
}

#[test]
fn rejects_invalid_skill_without_writing_library() {
    let temp = TempDir::new().expect("temp");
    let source = write_skill(temp.path(), "bad", "---\nname: Bad\n---\n# Bad\n");
    let data_dir = temp.path().join("data");

    let output = run_import(&data_dir, &source);
    assert_eq!(output.status.code(), Some(1));

    let json = parse_json(&output);
    assert_eq!(json["ok"], false);
    assert_eq!(json["imported"], 0);
    assert_eq!(json["skipped"], 1);
    assert_eq!(json["items"][0]["ok"], false);
    assert_eq!(json["items"][0]["errorCode"], "MissingDescription");
    assert_eq!(
        fs::read_dir(data_dir.join("library"))
            .expect("library dir")
            .count(),
        0
    );
}

#[test]
fn imports_all_skill_subdirectories_of_a_parent_directory() {
    let temp = TempDir::new().expect("temp");
    let parent = temp.path().join("bundle");
    fs::create_dir_all(&parent).expect("bundle dir");
    write_skill(
        &parent,
        "one",
        "---\nname: One\ndescription: first skill\n---\n",
    );
    write_skill(
        &parent,
        "two",
        "---\nname: Two\ndescription: second skill\n---\n",
    );
    // 不含 SKILL.md 的兄弟目录应被忽略。
    fs::create_dir_all(parent.join("assets")).expect("assets dir");
    let data_dir = temp.path().join("data");

    let output = run_import(&data_dir, &parent);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json(&output);
    assert_eq!(json["imported"], 2);
    let mut slugs: Vec<String> = json["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["slug"].as_str().unwrap_or_default().to_string())
        .collect();
    slugs.sort();
    assert_eq!(slugs, vec!["one".to_string(), "two".to_string()]);
    assert!(data_dir.join("library/one/SKILL.md").is_file());
    assert!(data_dir.join("library/two/SKILL.md").is_file());
}

#[test]
fn usage_error_exits_with_2() {
    let output = Command::new(env!("CARGO_BIN_EXE_skillport-cli"))
        .output()
        .expect("run skillport-cli");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("缺少子命令"));
}
