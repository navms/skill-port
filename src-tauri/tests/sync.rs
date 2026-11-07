mod common;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use common::{default_filter, Fixture};
use skillport_lib::core::model::{
    ActualKind, InstallSetInput, InstallStatus, LinkMode, SyncActionKind, SyncFilter,
};
use skillport_lib::core::{repo, sync};

#[test]
fn missing_then_creates_symlink_and_is_idempotent() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("alpha", "Alpha Skill");
    let target = fixture.add_target("claude-code", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);

    let report = fixture.reconcile();
    assert_eq!(report.checked, 1);
    assert_eq!(
        fixture.install(&skill, &target).status,
        InstallStatus::Missing
    );

    let (plan, result) = fixture.plan_and_apply(&default_filter());
    assert_eq!(plan.actions[0].kind, SyncActionKind::Create);
    assert_eq!(result.applied, 1);
    assert_eq!(result.failed, 0);
    assert!(result.undo_id.is_some(), "同步后应可撤销");

    let path = fixture.install_path(&skill, &target);
    let meta = fs::symlink_metadata(&path).expect("链接存在");
    assert!(meta.file_type().is_symlink());
    assert_eq!(
        fs::read_link(&path).expect("read link"),
        fixture.paths.library_dir.join(&skill.slug),
        "链接必须指向库内绝对路径"
    );

    fixture.reconcile();
    let install = fixture.install(&skill, &target);
    assert_eq!(install.status, InstallStatus::Ok);
    assert_eq!(install.actual_kind, Some(ActualKind::Symlink));

    let (second_plan, second_result) = fixture.plan_and_apply(&default_filter());
    assert!(
        second_plan.actions.is_empty(),
        "已同步的条目不应出现在计划中"
    );
    assert_eq!(second_result.applied, 0);
    assert!(second_result.undo_id.is_none());
}

#[test]
fn copy_install_becomes_stale_and_is_replaced_with_backup() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("copy-skill", "Copy Skill");
    let target = fixture.add_target("copy-target", LinkMode::Copy);
    fixture.enable_install(&skill, &target, LinkMode::Copy);

    let (plan, _) = fixture.plan_and_apply(&default_filter());
    assert_eq!(plan.actions[0].kind, SyncActionKind::Create);
    let path = fixture.install_path(&skill, &target);
    assert!(path.join("SKILL.md").is_file());
    assert!(path.join(".skillport.json").is_file());
    let meta = fs::symlink_metadata(&path).expect("copy exists");
    assert!(meta.is_dir() && !meta.file_type().is_symlink());

    fixture.reconcile();
    let install = fixture.install(&skill, &target);
    assert_eq!(install.status, InstallStatus::Ok);
    assert_eq!(install.actual_kind, Some(ActualKind::Copy));

    let library_skill_md = fixture.paths.library_dir.join(&skill.slug).join("SKILL.md");
    fs::write(
        &library_skill_md,
        "---\nname: Copy Skill\ndescription: changed\n---\n\nnew body\n",
    )
    .expect("修改库文件");

    let report = fixture.reconcile();
    assert_eq!(report.changed, 1);
    assert_eq!(
        fixture.install(&skill, &target).status,
        InstallStatus::Stale
    );

    let (plan, result) = fixture.plan_and_apply(&default_filter());
    assert_eq!(plan.actions[0].kind, SyncActionKind::Repair);
    assert_eq!(result.applied, 1);
    assert!(fixture.backup_count() >= 1, "覆盖前必须备份");
    let copied = fs::read_to_string(path.join("SKILL.md")).expect("副本");
    assert!(copied.contains("new body"));

    fixture.reconcile();
    assert_eq!(fixture.install(&skill, &target).status, InstallStatus::Ok);
}

#[test]
fn external_directory_is_conflict_and_never_touched() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("conflict-skill", "Conflict Skill");
    let target = fixture.add_target("conflict-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    let path = fixture.install_path(&skill, &target);
    fs::create_dir_all(&path).expect("外部目录");
    fs::write(path.join("keep.txt"), "user data").expect("write");

    let report = fixture.reconcile();
    assert_eq!(report.issues, 1);
    assert_eq!(
        fixture.install(&skill, &target).status,
        InstallStatus::Conflict
    );

    let (plan, result) = fixture.plan_and_apply(&default_filter());
    assert_eq!(plan.actions[0].kind, SyncActionKind::Conflict);
    assert_eq!(result.applied, 0);
    assert_eq!(result.failed, 0, "冲突动作不会被当作失败执行");
    assert_eq!(
        fs::read_to_string(path.join("keep.txt")).expect("原文件"),
        "user data"
    );
    assert!(!path.join("SKILL.md").exists());
    assert!(fs::symlink_metadata(&path).expect("meta").is_dir());
}

#[test]
fn mode_change_converts_symlink_to_copy() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("mode-skill", "Mode Skill");
    let target = fixture.add_target("mode-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    fixture.plan_and_apply(&default_filter());
    let path = fixture.install_path(&skill, &target);
    assert!(fs::symlink_metadata(&path)
        .expect("meta")
        .file_type()
        .is_symlink());

    fixture.enable_install(&skill, &target, LinkMode::Copy);
    fixture.reconcile();
    let (plan, result) = fixture.plan_and_apply(&default_filter());
    assert_eq!(plan.actions[0].kind, SyncActionKind::Repair);
    assert_eq!(result.applied, 1);
    let meta = fs::symlink_metadata(&path).expect("meta");
    assert!(meta.is_dir() && !meta.file_type().is_symlink());
    assert!(path.join(".skillport.json").is_file());
}

#[test]
fn reconcile_never_modifies_untracked_entries() {
    let fixture = Fixture::new();
    let target = fixture.add_target("untracked-target", LinkMode::Symlink);
    let stray = Path::new(&target.skills_dir).join("stray-skill");
    fs::create_dir_all(&stray).expect("stray");
    fs::write(stray.join("keep.txt"), "mine").expect("write");

    let report = fixture.reconcile();
    assert_eq!(report.checked, 0);
    assert!(stray.join("keep.txt").is_file(), "未托管条目不得被修改");
}

#[test]
fn disabling_install_cleans_owned_symlink() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("disable-skill", "Disable Skill");
    let target = fixture.add_target("disable-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    fixture.plan_and_apply(&default_filter());
    let path = fixture.install_path(&skill, &target);
    assert!(path.exists() || fs::symlink_metadata(&path).is_ok());

    let result = sync::set_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        &InstallSetInput {
            enabled: false,
            mode: None,
        },
    )
    .expect("disable");
    assert!(result.is_none());
    assert!(fs::symlink_metadata(&path).is_err());
    assert!(repo::list_installs_for_skill(&fixture.conn, &skill.id)
        .expect("installs")
        .is_empty());
}

#[test]
fn disabling_install_moves_owned_copy_to_backup() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("copy-disable", "Copy Disable");
    let target = fixture.add_target("copy-disable-target", LinkMode::Copy);
    fixture.enable_install(&skill, &target, LinkMode::Copy);
    fixture.plan_and_apply(&default_filter());
    let path = fixture.install_path(&skill, &target);
    assert!(path.is_dir());

    sync::set_install(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &target.id,
        &InstallSetInput {
            enabled: false,
            mode: Some(LinkMode::Copy),
        },
    )
    .expect("disable");
    assert!(!path.exists());
    assert!(fixture.backup_count() >= 1);
}

#[test]
fn plan_store_handles_cancel() {
    let fixture = Fixture::new();
    let plan =
        sync::build_plan(&fixture.conn, &fixture.paths, &SyncFilter::default()).expect("plan");
    let store = skillport_lib::core::plan::PlanStore::new();
    store.insert(plan.clone()).expect("insert");
    let (loaded, cancel) = store.begin_apply(&plan.id).expect("begin");
    assert_eq!(loaded.id, plan.id);
    assert!(!cancel.load(Ordering::SeqCst));
    store.cancel(&plan.id).expect("cancel");
    assert!(cancel.load(Ordering::SeqCst));
    store.finish(&plan.id).expect("finish");
    assert!(store.begin_apply(&plan.id).is_err());
}

#[test]
fn cancel_flag_stops_apply_between_steps() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("cancel-skill", "Cancel Skill");
    let target = fixture.add_target("cancel-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    let plan =
        sync::build_plan(&fixture.conn, &fixture.paths, &SyncFilter::default()).expect("plan");
    let cancel = AtomicBool::new(true);
    let result = sync::apply_plan(&fixture.conn, &fixture.paths, &plan, &cancel, &mut |_| {})
        .expect("apply");
    assert!(result.cancelled);
    assert_eq!(result.applied, 0);
    assert!(fs::symlink_metadata(fixture.install_path(&skill, &target)).is_err());
}

#[test]
fn symlink_to_other_location_is_conflict_and_preserved() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("foreign-link", "Foreign Link");
    let target = fixture.add_target("foreign-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    let path = fixture.install_path(&skill, &target);
    let outside = fixture.temp.path().join("outside-dir");
    fs::create_dir_all(&outside).expect("outside");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &path).expect("symlink");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&outside, &path).expect("symlink");

    fixture.reconcile();
    assert_eq!(
        fixture.install(&skill, &target).status,
        InstallStatus::Conflict
    );
    let (plan, result) = fixture.plan_and_apply(&default_filter());
    assert_eq!(plan.actions[0].kind, SyncActionKind::Conflict);
    assert_eq!(result.applied, 0);
    assert!(fs::symlink_metadata(&path)
        .expect("链接仍在")
        .file_type()
        .is_symlink());
    assert_eq!(fs::read_link(&path).expect("link"), outside);
}

#[test]
fn remove_distribution_only_deletes_owned_entries() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("owned-skill", "Owned Skill");
    let target = fixture.add_target("owned-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    fixture.plan_and_apply(&default_filter());
    let path = fixture.install_path(&skill, &target);

    let outcome =
        sync::remove_distribution(&fixture.paths, &skill, &target, true, 20).expect("remove");
    assert!(outcome.removed);
    assert!(fs::symlink_metadata(&path).is_err());

    let external = fixture.temp.path().join("external");
    fs::create_dir_all(&external).expect("external");
    fs::write(external.join("keep.txt"), "keep").expect("write");
    #[cfg(unix)]
    let link_result = std::os::unix::fs::symlink(&external, &path);
    #[cfg(windows)]
    let link_result = std::os::windows::fs::symlink_dir(&external, &path);
    link_result.expect("symlink");
    let outcome =
        sync::remove_distribution(&fixture.paths, &skill, &target, true, 20).expect("guard");
    assert!(!outcome.removed);
    assert!(external.join("keep.txt").is_file());
    assert!(fs::symlink_metadata(&path).is_ok());
}
