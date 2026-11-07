mod common;

use std::fs;

use common::Fixture;
use skillport_lib::core::model::{DeleteSkillOptions, ImportErrorCode, SkillPatch};
use skillport_lib::core::{library, repo};

#[test]
fn imports_valid_skill_into_library() {
    let fixture = Fixture::new();
    let dir = fixture.write_skill_source("alpha", "Alpha Skill", "alpha desc");
    let report = fixture.import(&dir);
    assert_eq!(report.imported, 1, "{:?}", report.items);
    assert_eq!(report.skipped, 0);
    assert!(report.items[0].ok);
    assert_eq!(report.items[0].slug.as_deref(), Some("alpha-skill"));

    let skill = repo::list_skills(&fixture.conn).expect("list").remove(0);
    assert_eq!(skill.name, "Alpha Skill");
    assert_eq!(skill.slug, "alpha-skill");
    assert_eq!(skill.source_kind.as_str(), "import");
    assert!(!skill.content_hash.is_empty());
    let library_dir = fixture.paths.library_dir.join(&skill.slug);
    assert!(library_dir.join("SKILL.md").is_file());
    assert!(library_dir.join("scripts/run.sh").is_file());
}

#[test]
fn list_reports_size_and_does_not_take_frontmatter_category() {
    let fixture = Fixture::new();
    let dir = fixture.temp.path().join("sources/categorized");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join("SKILL.md"),
        "---\nname: Categorized\ndescription: d\ncategory: 工具\n---\n",
    )
    .expect("write");
    fixture.import(&dir);

    let skills = library::list(&fixture.conn, &fixture.paths).expect("list");
    assert_eq!(skills.len(), 1);
    // 分类是 SkillPort 自己的口径，导入时不会从别人的 frontmatter 里抄。
    assert_eq!(skills[0].category, None);
    assert!(skills[0].size_bytes > 0);
    let canonical = dir.canonicalize().expect("canonical");
    assert_eq!(
        skills[0].source_ref.as_deref(),
        Some(canonical.display().to_string().as_str())
    );
}

#[test]
fn rejects_missing_skill_md_and_writes_nothing() {
    let fixture = Fixture::new();
    let dir = fixture.temp.path().join("sources/no-skill");
    fs::create_dir_all(&dir).expect("dir");
    let report = fixture.import(&dir);
    assert_eq!(report.imported, 0);
    assert_eq!(report.skipped, 1);
    assert_eq!(
        report.items[0].error_code,
        Some(ImportErrorCode::MissingSkillFile)
    );
    assert!(repo::list_skills(&fixture.conn).expect("list").is_empty());
    assert_eq!(
        fs::read_dir(&fixture.paths.library_dir)
            .expect("library")
            .count(),
        0
    );
}

#[test]
fn rejects_missing_description_strictly() {
    let fixture = Fixture::new();
    let dir = fixture.temp.path().join("sources/no-desc");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(dir.join("SKILL.md"), "---\nname: No Desc\n---\n").expect("write");
    let report = fixture.import(&dir);
    assert_eq!(report.imported, 0);
    assert_eq!(
        report.items[0].error_code,
        Some(ImportErrorCode::MissingDescription)
    );
    assert!(repo::list_skills(&fixture.conn).expect("list").is_empty());
}

#[test]
fn rejects_invalid_yaml() {
    let fixture = Fixture::new();
    let dir = fixture.temp.path().join("sources/bad-yaml");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(dir.join("SKILL.md"), "---\nname: [oops\n---\n").expect("write");
    let report = fixture.import(&dir);
    assert_eq!(report.imported, 0);
    assert_eq!(
        report.items[0].error_code,
        Some(ImportErrorCode::InvalidFrontmatter)
    );
}

#[test]
fn duplicate_slug_is_rejected_instead_of_silently_suffixed() {
    let fixture = Fixture::new();
    let first = fixture.write_skill_source("one", "Same Name", "d");
    let second = fixture.write_skill_source("two", "Same Name", "d");
    fixture.import(&first);
    let report = fixture.import(&second);
    assert_eq!(report.imported, 0);
    assert_eq!(
        report.items[0].error_code,
        Some(ImportErrorCode::DuplicateSlug)
    );
    assert!(report.items[0]
        .detail
        .as_deref()
        .unwrap_or_default()
        .contains("slug 已存在"));
    let slugs: Vec<String> = repo::list_skills(&fixture.conn)
        .expect("list")
        .into_iter()
        .map(|skill| skill.slug)
        .collect();
    assert_eq!(slugs, vec!["same-name".to_string()]);
}

#[test]
fn rejects_unusable_slug() {
    let fixture = Fixture::new();
    let dir = fixture.temp.path().join("sources/技能");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join("SKILL.md"),
        "---\nname: 中文技能\ndescription: d\n---\n",
    )
    .expect("write");
    let report = fixture.import(&dir);
    assert_eq!(report.imported, 0);
    assert_eq!(
        report.items[0].error_code,
        Some(ImportErrorCode::InvalidSlug)
    );
}

#[test]
fn missing_path_reports_path_not_found() {
    let fixture = Fixture::new();
    let report = fixture.import(&fixture.temp.path().join("sources/nope"));
    assert_eq!(report.imported, 0);
    assert_eq!(
        report.items[0].error_code,
        Some(ImportErrorCode::PathNotFound)
    );
}

#[test]
fn batch_import_isolates_failures() {
    let fixture = Fixture::new();
    let good = fixture.write_skill_source("good", "Good Skill", "d");
    let bad = fixture.temp.path().join("sources/bad");
    fs::create_dir_all(&bad).expect("dir");

    let report = library::import(
        &fixture.conn,
        &fixture.paths,
        &[good.display().to_string(), bad.display().to_string()],
    )
    .expect("import");

    assert_eq!(report.imported, 1);
    assert_eq!(report.skipped, 1);
    assert!(report.items.iter().any(|item| item.ok));
    assert!(report.items.iter().any(|item| !item.ok));
    assert_eq!(repo::list_skills(&fixture.conn).expect("list").len(), 1);
}

#[test]
fn imports_subdirectories_when_parent_has_no_skill_md() {
    let fixture = Fixture::new();
    let parent = fixture.temp.path().join("sources/bundle");
    fs::create_dir_all(&parent).expect("dir");
    for name in ["One", "Two"] {
        let child = parent.join(name);
        fs::create_dir_all(&child).expect("child");
        fs::write(
            child.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: d\n---\n"),
        )
        .expect("write");
    }
    let report = fixture.import(&parent);
    assert_eq!(report.imported, 2);
}

#[test]
fn refuses_to_import_from_library_itself() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("origin", "Origin Skill");
    let library_dir = fixture.paths.library_dir.join(&skill.slug);
    let report = fixture.import(&library_dir);
    assert_eq!(report.imported, 0);
    assert!(report.items[0]
        .detail
        .as_deref()
        .unwrap_or_default()
        .contains("库目录"));
}

#[test]
fn update_rewrites_frontmatter_and_hash() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("editable", "Editable");
    let patch = SkillPatch {
        name: Some("Renamed".to_string()),
        description: Some("new desc".to_string()),
        skill_md: None,
        ..Default::default()
    };
    let detail = library::update(&fixture.conn, &fixture.paths, &skill.id, &patch).expect("update");
    assert_eq!(detail.summary.name, "Renamed");
    assert_eq!(detail.summary.description, "new desc");
    assert_ne!(detail.summary.content_hash, skill.content_hash);
    assert_eq!(detail.summary.slug, skill.slug, "slug 不随显示名改变");
    assert!(detail.skill_md.contains("name: Renamed"));
}

#[test]
fn update_accepts_full_skill_md() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("full-md", "Full Md");
    let patch = SkillPatch {
        name: None,
        description: None,
        skill_md: Some("---\nname: Full Md\ndescription: rewritten\n---\n\n# Body\n".to_string()),
        ..Default::default()
    };
    let detail = library::update(&fixture.conn, &fixture.paths, &skill.id, &patch).expect("update");
    assert_eq!(detail.summary.description, "rewritten");
    assert!(detail.skill_md.contains("# Body"));
}

#[test]
fn update_rejects_invalid_skill_md() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("bad-edit", "Bad Edit");
    let patch = SkillPatch {
        name: None,
        description: None,
        skill_md: Some("no frontmatter".to_string()),
        ..Default::default()
    };
    assert!(library::update(&fixture.conn, &fixture.paths, &skill.id, &patch).is_err());
}

#[test]
fn update_category_stays_out_of_skill_md() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("categorized", "Categorized");
    let path = fixture.paths.library_dir.join(&skill.slug).join("SKILL.md");
    let before = fs::read_to_string(&path).expect("read");

    let patch = SkillPatch {
        category: Some(" 工具 ".to_string()),
        ..Default::default()
    };
    let detail = library::update(&fixture.conn, &fixture.paths, &skill.id, &patch).expect("update");
    assert_eq!(detail.summary.category.as_deref(), Some("工具"));
    assert_eq!(
        detail.summary.content_hash, skill.content_hash,
        "分类不改变内容"
    );
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        before,
        "不写 SKILL.md"
    );

    let patch = SkillPatch {
        category: Some(String::new()),
        ..Default::default()
    };
    let detail = library::update(&fixture.conn, &fixture.paths, &skill.id, &patch).expect("clear");
    assert_eq!(detail.summary.category, None, "空串清除分类");
    assert_eq!(fs::read_to_string(&path).expect("read"), before);
}

#[test]
fn delete_removes_library_and_rows_with_undo() {
    let fixture = Fixture::new();
    let skill = fixture.import_ok("deletable", "Deletable");
    let dir = fixture.paths.library_dir.join(&skill.slug);
    assert!(dir.is_dir());
    let result = library::delete(
        &fixture.conn,
        &fixture.paths,
        &skill.id,
        &DeleteSkillOptions::default(),
    )
    .expect("delete");
    assert!(!dir.exists());
    assert_eq!(result.removed_installs, 0);
    assert!(result.undo_id.is_some(), "有备份时应可撤销");
    assert!(repo::list_skills(&fixture.conn).expect("list").is_empty());
    assert!(fixture.backup_count() >= 1);
}
