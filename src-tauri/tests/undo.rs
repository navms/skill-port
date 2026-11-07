mod common;

use std::fs;

use common::{default_filter, Fixture};
use skillport_lib::core::model::{DeleteSkillOptions, LinkMode};
use skillport_lib::core::{library, repo, undo};

#[test]
fn undo_delete_restores_library_row_and_installs() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("restorable", "Restorable");
    let target = fixture.add_target("undo-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    fixture.plan_and_apply(&default_filter());
    let library_dir = fixture.paths.library_dir.join(&skill.slug);
    let link = fixture.install_path(&skill, &target);
    assert!(library_dir.is_dir());
    assert!(fs::symlink_metadata(&link).is_ok());

    let result = library::delete(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &DeleteSkillOptions::default(),
    )
    .expect("delete");
    let undo_id = result.undo_id.expect("undo id");
    assert_eq!(result.removed_installs, 1);
    assert!(!library_dir.exists());
    assert!(repo::get_skill(&fixture.conn, &skill.id).is_err());
    assert!(fs::symlink_metadata(&link).is_err());

    let outcome = undo::undo(&fixture.conn, &fixture.paths, &undo_id).expect("undo");
    assert!(outcome.ok, "{:?}", outcome.message);
    assert!(library_dir.join("SKILL.md").is_file());
    assert_eq!(
        repo::get_skill(&fixture.conn, &skill.id)
            .expect("skill")
            .slug,
        skill.slug
    );
    assert!(repo::get_install(&fixture.conn, &skill.id, &target.id)
        .expect("install")
        .is_some());

    // 同一记录不能重复撤销。
    let again = undo::undo(&fixture.conn, &fixture.paths, &undo_id).expect("undo again");
    assert!(!again.ok);
}

#[test]
fn undo_sync_removes_created_symlink() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("sync-undo", "Sync Undo");
    let target = fixture.add_target("sync-undo-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    let (_, result) = fixture.plan_and_apply(&default_filter());
    let undo_id = result.undo_id.expect("undo id");
    let link = fixture.install_path(&skill, &target);
    assert!(fs::symlink_metadata(&link).is_ok());

    let outcome = undo::undo(&fixture.conn, &fixture.paths, &undo_id).expect("undo");
    assert!(outcome.ok, "{:?}", outcome.message);
    assert!(fs::symlink_metadata(&link).is_err());
    assert!(repo::get_install(&fixture.conn, &skill.id, &target.id)
        .expect("install")
        .is_some());
    assert!(fixture.paths.library_dir.join(&skill.slug).is_dir());
}

#[test]
fn undo_sync_restores_replaced_copy_from_backup() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("copy-undo", "Copy Undo");
    let target = fixture.add_target("copy-undo-target", LinkMode::Copy);
    fixture.enable_install(&skill, &target, LinkMode::Copy);
    fixture.plan_and_apply(&default_filter());
    let path = fixture.install_path(&skill, &target);
    assert!(path.join("SKILL.md").is_file());

    let library_skill_md = fixture.paths.library_dir.join(&skill.slug).join("SKILL.md");
    fs::write(
        &library_skill_md,
        "---\nname: Copy Undo\ndescription: changed\n---\n\nnew body\n",
    )
    .expect("edit library");
    fixture.reconcile();
    let (_, result) = fixture.plan_and_apply(&default_filter());
    let undo_id = result.undo_id.expect("undo id");
    assert!(fs::read_to_string(path.join("SKILL.md"))
        .expect("copy")
        .contains("new body"));

    let outcome = undo::undo(&fixture.conn, &fixture.paths, &undo_id).expect("undo");
    assert!(outcome.ok, "{:?}", outcome.message);
    let restored = fs::read_to_string(path.join("SKILL.md")).expect("restored copy");
    assert!(!restored.contains("new body"), "应恢复备份中的旧副本");
    assert!(path.join(".skillport.json").is_file());
}

#[test]
fn activity_entries_carry_names_and_actions() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("activity-skill", "Activity Skill");
    let target = fixture.add_target("activity-target", LinkMode::Symlink);
    fixture.enable_install(&skill, &target, LinkMode::Symlink);
    fixture.plan_and_apply(&default_filter());

    let page = repo::list_activity(&fixture.conn, 20, None).expect("activity");
    assert!(!page.items.is_empty());
    let sync_entry = page
        .items
        .iter()
        .find(|entry| entry.action == "sync")
        .expect("sync activity");
    assert_eq!(sync_entry.skill_name.as_deref(), Some("activity-skill"));
    assert_eq!(sync_entry.target_name.as_deref(), Some("activity-target"));
    assert!(page.next_cursor.is_none());
}
