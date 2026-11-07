use std::fs;

use skillport_lib::core::hash::{is_excluded_name, tree_hash};

fn write(path: &std::path::Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent");
    }
    fs::write(path, text).expect("write");
}

#[test]
fn hash_is_independent_of_file_creation_order() {
    let left = tempfile::tempdir().expect("temp");
    let right = tempfile::tempdir().expect("temp");
    write(&left.path().join("a.md"), "alpha");
    write(&left.path().join("nested/b.md"), "beta");
    write(&right.path().join("nested/b.md"), "beta");
    write(&right.path().join("a.md"), "alpha");
    assert_eq!(
        tree_hash(left.path()).expect("hash"),
        tree_hash(right.path()).expect("hash")
    );
}

#[test]
fn hash_changes_when_content_changes() {
    let dir = tempfile::tempdir().expect("temp");
    write(&dir.path().join("SKILL.md"), "one");
    let first = tree_hash(dir.path()).expect("hash");
    write(&dir.path().join("SKILL.md"), "two");
    let second = tree_hash(dir.path()).expect("hash");
    assert_ne!(first, second);
}

#[test]
fn hash_ignores_excluded_files() {
    let dir = tempfile::tempdir().expect("temp");
    write(&dir.path().join("SKILL.md"), "content");
    let clean = tree_hash(dir.path()).expect("hash");
    write(&dir.path().join(".DS_Store"), "junk");
    write(&dir.path().join("Thumbs.db"), "junk");
    write(
        &dir.path().join(".skillport.json"),
        r#"{"schema":1,"managed_by":"skillport","skill_id":"x","source_hash":"y"}"#,
    );
    assert_eq!(clean, tree_hash(dir.path()).expect("hash"));
}

#[test]
fn excluded_names_are_exact() {
    assert!(is_excluded_name(".skillport.json"));
    assert!(is_excluded_name(".DS_Store"));
    assert!(is_excluded_name("Thumbs.db"));
    assert!(!is_excluded_name("notes.md"));
}

#[cfg(unix)]
#[test]
fn hash_records_symlink_target() {
    let dir = tempfile::tempdir().expect("temp");
    write(&dir.path().join("real.md"), "real");
    std::os::unix::fs::symlink("real.md", dir.path().join("link.md")).expect("symlink");
    let with_link = tree_hash(dir.path()).expect("hash");
    fs::remove_file(dir.path().join("link.md")).expect("remove");
    let without_link = tree_hash(dir.path()).expect("hash");
    assert_ne!(with_link, without_link);
}
