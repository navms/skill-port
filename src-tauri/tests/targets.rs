mod common;

use std::fs;

use common::Fixture;
use skillport_lib::core::model::{LinkMode, TargetCreateInput, TargetUpdatePatch};
use skillport_lib::core::{repo, target};

fn input(slug: &str, skills_dir: &str) -> TargetCreateInput {
    TargetCreateInput {
        slug: slug.to_string(),
        name: slug.to_string(),
        skills_dir: skills_dir.to_string(),
        link_mode: LinkMode::Symlink,
        enabled: true,
    }
}

#[test]
fn creates_and_lists_targets() {
    let fixture = Fixture::new();
    let dir = fixture.temp.path().join("harness/skills");
    fs::create_dir_all(&dir).expect("dir");
    let created = target::create(
        &fixture.conn,
        &fixture.paths,
        &input("claude-code", &dir.display().to_string()),
    )
    .expect("create");
    assert_eq!(created.slug, "claude-code");
    assert_eq!(target::list(&fixture.conn).expect("list").len(), 1);
}

#[test]
fn rejects_relative_path() {
    let fixture = Fixture::new();
    let error = target::create(
        &fixture.conn,
        &fixture.paths,
        &input("relative", "relative/dir"),
    )
    .expect_err("相对路径应被拒绝");
    assert_eq!(error.code(), "InvalidInput");
}

#[test]
fn rejects_duplicate_slug_and_path() {
    let fixture = Fixture::new();
    let dir = fixture.temp.path().join("a/skills");
    fs::create_dir_all(&dir).expect("dir");
    target::create(
        &fixture.conn,
        &fixture.paths,
        &input("alpha", &dir.display().to_string()),
    )
    .expect("first");

    let other = fixture.temp.path().join("b/skills");
    fs::create_dir_all(&other).expect("dir");
    let slug_conflict = target::create(
        &fixture.conn,
        &fixture.paths,
        &input("alpha", &other.display().to_string()),
    )
    .expect_err("slug 冲突");
    assert_eq!(slug_conflict.code(), "Conflict");

    let path_conflict = target::create(
        &fixture.conn,
        &fixture.paths,
        &input("beta", &dir.display().to_string()),
    )
    .expect_err("路径冲突");
    assert_eq!(path_conflict.code(), "Conflict");
}

#[test]
fn rejects_target_containing_library() {
    let fixture = Fixture::new();
    let parent = fixture.paths.library_dir.parent().expect("parent");
    let error = target::create(
        &fixture.conn,
        &fixture.paths,
        &input("bad", &parent.display().to_string()),
    )
    .expect_err("库目录必须独立");
    assert_eq!(error.code(), "InvalidInput");
}

#[test]
fn update_moves_install_paths() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("moving", "Moving Skill");
    let original = fixture.temp.path().join("original/skills");
    fs::create_dir_all(&original).expect("dir");
    let created = target::create(
        &fixture.conn,
        &fixture.paths,
        &input("mover", &original.display().to_string()),
    )
    .expect("create");
    fixture.enable_install(&skill, &created, LinkMode::Symlink);
    let before = repo::list_installs_for_target(&fixture.conn, &created.id).expect("installs");

    let moved = fixture.temp.path().join("moved/skills");
    fs::create_dir_all(&moved).expect("dir");
    target::update(
        &fixture.conn,
        &fixture.paths,
        &created.id,
        &TargetUpdatePatch {
            skills_dir: Some(moved.display().to_string()),
            ..Default::default()
        },
    )
    .expect("update");

    let after = repo::list_installs_for_target(&fixture.conn, &created.id).expect("installs");
    assert_ne!(before[0].target_path, after[0].target_path);
    // 按路径组件比较：Windows 上 `display()` 的分隔符可能与入库时的归一化结果不同。
    assert!(std::path::Path::new(&after[0].target_path).starts_with(&moved));
}

#[test]
fn detect_suggestions_reports_existence() {
    let fixture = Fixture::new();
    let home = fixture.temp.path().join("home");
    fs::create_dir_all(home.join(".claude/skills")).expect("claude");
    let suggestions = target::detect_suggestions(&fixture.conn, &home).expect("suggest");
    let claude = suggestions
        .iter()
        .find(|item| item.slug == "claude-code")
        .expect("claude 候选");
    assert!(claude.exists);
    let cursor = suggestions
        .iter()
        .find(|item| item.slug == "cursor")
        .expect("cursor 候选");
    assert!(!cursor.exists);
}

#[test]
fn delete_target_cleans_owned_links() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("target-delete", "Target Delete");
    let target_dir = fixture.temp.path().join("delete/skills");
    fs::create_dir_all(&target_dir).expect("dir");
    let created = target::create(
        &fixture.conn,
        &fixture.paths,
        &input("deleter", &target_dir.display().to_string()),
    )
    .expect("create");
    fixture.enable_install(&skill, &created, LinkMode::Symlink);
    let plan =
        skillport_lib::core::sync::build_plan(&fixture.conn, &fixture.paths, &Default::default())
            .expect("plan");
    let _ = skillport_lib::core::sync::apply_plan(
        &fixture.conn,
        &fixture.paths,
        &plan,
        &std::sync::atomic::AtomicBool::new(false),
        &mut |_| {},
    )
    .expect("apply");
    let path = fixture.install_path(&skill, &created);
    assert!(fs::symlink_metadata(&path).is_ok());

    target::delete(
        &fixture.conn,
        &fixture.paths,
        &created.id,
        &Default::default(),
    )
    .expect("delete target");
    assert!(fs::symlink_metadata(&path).is_err());
    assert!(target::list(&fixture.conn).expect("list").is_empty());
}
