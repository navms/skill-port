mod common;

use std::fs;

use common::{default_filter, Fixture};
use skillport_lib::core::model::{ConflictResolution, InstallStatus, LinkMode, SyncActionKind};
use skillport_lib::core::{repo, sync};

fn conflicting_install(
    fixture: &Fixture,
) -> (
    skillport_lib::core::model::Skill,
    skillport_lib::core::model::Target,
    std::path::PathBuf,
) {
    let skill = fixture.import_ok("conflicted", "Conflicted");
    let target = fixture.add_target("resolve-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    let path = fixture.install_path(&skill, &target);
    fs::create_dir_all(&path).expect("external dir");
    fs::write(path.join("keep.txt"), "user data").expect("write");
    fixture.reconcile();
    (skill, target, path)
}

#[test]
fn ignore_marks_conflict_and_plan_skips_it() {
    let fixture = Fixture::new();
    let (skill, target, path) = conflicting_install(&fixture);
    let install = sync::resolve_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        ConflictResolution::Ignore,
    )
    .expect("ignore")
    .expect("install");
    assert!(install.ignored);
    assert_eq!(fixture.reconcile().issues, 0, "已忽略的冲突不再计入问题");

    let plan = sync::build_plan(&fixture.conn, &fixture.paths, &Default::default()).expect("plan");
    assert_eq!(plan.actions[0].kind, SyncActionKind::Skip);
    assert_eq!(
        fs::read_to_string(path.join("keep.txt")).expect("keep"),
        "user data"
    );
}

#[test]
fn unignore_restores_conflict_issue() {
    let fixture = Fixture::new();
    let (skill, target, _) = conflicting_install(&fixture);
    sync::resolve_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        ConflictResolution::Ignore,
    )
    .expect("ignore");
    let install = sync::resolve_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        ConflictResolution::Unignore,
    )
    .expect("unignore")
    .expect("install");
    assert!(!install.ignored);
}

#[test]
fn cancel_removes_install_but_keeps_external_dir() {
    let fixture = Fixture::new();
    let (skill, target, path) = conflicting_install(&fixture);
    let result = sync::resolve_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        ConflictResolution::Cancel,
    )
    .expect("cancel");
    assert!(result.is_none());
    assert!(repo::get_install(&fixture.conn, &skill.id, &target.id)
        .expect("query")
        .is_none());
    assert_eq!(
        fs::read_to_string(path.join("keep.txt")).expect("keep"),
        "user data"
    );
}

#[test]
fn adopt_directory_takes_ownership_as_stale_copy() {
    let fixture = Fixture::new();
    let (skill, target, path) = conflicting_install(&fixture);
    let install = sync::resolve_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        ConflictResolution::Adopt,
    )
    .expect("adopt")
    .expect("install");
    assert_eq!(install.mode, LinkMode::Copy);
    assert_eq!(
        install.status,
        InstallStatus::Stale,
        "外部内容与库不同，应等待同步"
    );
    assert!(path.join(".skillport.json").is_file());
    assert!(path.join("keep.txt").is_file(), "接管不得删除用户内容");

    // 下一次同步会把旧内容备份后替换为库副本。
    let (plan, result) = fixture.plan_and_apply(&default_filter());
    assert_eq!(plan.actions[0].kind, SyncActionKind::Repair);
    assert_eq!(result.applied, 1);
    assert!(path.join("SKILL.md").is_file());
    assert!(!path.join("keep.txt").exists());
}

#[test]
fn adopt_symlink_copies_external_content() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("adopt-link", "Adopt Link");
    let target = fixture.add_target("adopt-link-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    let path = fixture.install_path(&skill, &target);
    let outside = fixture.temp.path().join("outside");
    fs::create_dir_all(&outside).expect("outside");
    fs::write(outside.join("keep.txt"), "outside data").expect("write");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &path).expect("symlink");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&outside, &path).expect("symlink");
    fixture.reconcile();

    sync::resolve_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        ConflictResolution::Adopt,
    )
    .expect("adopt");

    let meta = fs::symlink_metadata(&path).expect("meta");
    assert!(meta.is_dir() && !meta.file_type().is_symlink());
    assert_eq!(
        fs::read_to_string(path.join("keep.txt")).expect("keep"),
        "outside data"
    );
    assert!(path.join(".skillport.json").is_file());
    assert!(outside.join("keep.txt").is_file(), "原链接目标不应被修改");
}

#[test]
fn adopt_rejects_non_conflict_install() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("healthy", "Healthy");
    let target = fixture.add_target("healthy-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    fixture.plan_and_apply(&default_filter());
    fixture.reconcile();

    let error = sync::resolve_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        ConflictResolution::Adopt,
    )
    .expect_err("健康的分发不能接管");
    assert_eq!(error.code(), "InvalidInput");
}
