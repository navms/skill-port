use std::path::{Path, PathBuf};

use skillport_lib::core::paths::{
    self, normalize_lexically, slugify, unique_slug, validate_library_placement, validate_slug,
};

#[test]
fn accepts_valid_slugs() {
    for slug in ["ab", "a-b", "test-skill-2", "x9", "a1-b2-c3"] {
        assert!(validate_slug(slug).is_ok(), "{slug} 应合法");
    }
}

#[test]
fn rejects_invalid_slugs() {
    let cases = [
        ("a", "太短"),
        ("-ab", "前导连字符"),
        ("ab-", "尾随连字符"),
        ("a--b", "连续连字符"),
        ("Ab", "大写"),
        ("a_b", "下划线"),
        ("../x", "路径穿越"),
        ("/abs", "绝对路径"),
        ("skillport", "保留名"),
        ("node_modules", "保留名"),
        ("中文", "非 ASCII"),
        ("a b", "空格"),
    ];
    for (slug, reason) in cases {
        assert!(validate_slug(slug).is_err(), "{slug} 应被拒绝（{reason}）");
    }
    let too_long = "a".repeat(64);
    assert!(validate_slug(&too_long).is_err());
}

#[test]
fn slugify_cleans_names() {
    assert_eq!(slugify("Test Skill"), "test-skill");
    assert_eq!(slugify("  --Hello,, World!!  "), "hello-world");
    assert_eq!(slugify("A"), "skill");
    assert_eq!(slugify("技能"), "skill");
    assert!(validate_slug(&slugify("A Very Long Name With Spaces")).is_ok());
}

#[test]
fn unique_slug_appends_suffix() {
    let taken = vec!["test-skill".to_string(), "test-skill-2".to_string()];
    assert_eq!(unique_slug("test-skill", &taken), "test-skill-3");
    assert_eq!(unique_slug("other", &taken), "other");
}

#[test]
fn rejects_library_inside_target_and_vice_versa() {
    let base = PathBuf::from("/tmp/skillport-test");
    let library = base.join("library");
    assert!(validate_library_placement(&library, [base.join("skills")]).is_ok());
    assert!(validate_library_placement(&library, [base.join("library/nested")]).is_err());
    assert!(validate_library_placement(&library, [base.join("library")]).is_err());
    assert!(validate_library_placement(&library, [base]).is_err());
}

#[test]
fn guarded_path_requires_direct_child() {
    // 必须用平台自己的绝对路径：Windows 上 `/tmp/...` 没有盘符，不是绝对路径。
    let dir = std::env::temp_dir().join("skillport-test").join("skills");
    assert!(dir.is_absolute(), "测试需要绝对路径");
    let path = paths::guarded_target_path(&dir, "my-skill").expect("路径");
    assert_eq!(path, dir.join("my-skill"));
    assert!(paths::guarded_target_path(&dir, "../escape").is_err());
    assert!(paths::guarded_target_path(Path::new("relative"), "my-skill").is_err());
}

#[test]
fn normalize_folds_dot_segments() {
    assert_eq!(
        normalize_lexically(Path::new("/a/./b/../c")),
        PathBuf::from("/a/c")
    );
}

#[test]
fn write_guard_rejects_parent_outside_skills_dir() {
    let temp = tempfile::tempdir().expect("temp");
    let skills_dir = temp.path().join("skills");
    std::fs::create_dir_all(&skills_dir).expect("dir");
    let inside = skills_dir.join("ok-skill");
    assert!(paths::ensure_write_allowed(&skills_dir, &inside, "ok-skill").is_ok());
    let outside = temp.path().join("other").join("ok-skill");
    assert!(paths::ensure_write_allowed(&skills_dir, &outside, "ok-skill").is_err());
}
