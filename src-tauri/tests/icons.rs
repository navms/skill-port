mod common;

use std::fs;

use common::Fixture;
use skillport_lib::core::icons;

#[test]
fn seeds_defaults_and_never_overwrites_user_files() {
    let fixture = Fixture::new();
    icons::ensure_defaults(&fixture.paths).expect("seed");

    let file = fixture.paths.icons_dir.join("claude-code.svg");
    assert!(file.is_file(), "首次启动应写入默认图标");

    let custom = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\"><path d=\"M0 0h24v24H0z\"/></svg>";
    fs::write(&file, custom).expect("覆盖为用户版本");
    icons::ensure_defaults(&fixture.paths).expect("reseed");
    assert_eq!(
        fs::read_to_string(&file).expect("read"),
        custom,
        "已存在的图标不能被内置版本覆盖"
    );
}

#[test]
fn lists_svg_files_and_skips_everything_else() {
    let fixture = Fixture::new();
    icons::ensure_defaults(&fixture.paths).expect("seed");

    let dir = &fixture.paths.icons_dir;
    let custom = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\"><path d=\"M0 0h24v24H0z\"/></svg>";
    fs::write(dir.join("my-harness.svg"), custom).expect("custom icon");
    // 不合法文件名、非 SVG、超大文件都必须被跳过。
    fs::write(dir.join("Bad Name.svg"), custom).expect("bad slug");
    fs::write(dir.join("notes.txt"), "hello").expect("not svg");
    fs::write(
        dir.join("not-really.svg"),
        "<html><body>not an svg</body></html>",
    )
    .expect("svg extension but html content");
    fs::write(dir.join("huge.svg"), "x".repeat(300 * 1024)).expect("too big");
    // 合法的 SVG 允许带 XML 声明和前置注释。
    fs::write(
        dir.join("declared.svg"),
        "<?xml version=\"1.0\"?>\n<!-- mark -->\n<svg xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M0 0h24v24H0z\"/></svg>",
    )
    .expect("declared svg");

    let icons = icons::list(&fixture.paths).expect("list");
    let slugs: Vec<&str> = icons.iter().map(|icon| icon.slug.as_str()).collect();
    assert!(slugs.contains(&"claude-code"));
    assert!(slugs.contains(&"my-harness"));
    assert!(slugs.contains(&"declared"));
    assert!(
        !slugs.contains(&"not-really"),
        "扩展名是 .svg 但内容不是 SVG 文档时必须跳过"
    );
    assert_eq!(icons.len(), slugs.len());
    assert_eq!(icons.len(), 10, "8 个默认图标 + 2 个自定义图标");
    assert!(icons.iter().all(|icon| icon.svg.contains("<svg")));
    let custom_entry = icons
        .iter()
        .find(|icon| icon.slug == "my-harness")
        .expect("自定义图标");
    assert_eq!(custom_entry.svg, custom);
}

#[test]
fn list_is_empty_when_directory_is_missing() {
    let fixture = Fixture::new();
    fs::remove_dir_all(&fixture.paths.icons_dir).expect("remove icons dir");
    let icons = icons::list(&fixture.paths).expect("list");
    assert!(icons.is_empty());
}

#[cfg(unix)]
#[test]
fn dangling_symlink_is_never_written_through_and_does_not_break_seeding() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let outside = fixture.temp.path().join("outside-created.txt");
    let link = fixture.paths.icons_dir.join("claude-code.svg");
    symlink(&outside, &link).expect("悬空链接");

    icons::ensure_defaults(&fixture.paths).expect("悬空链接不能让补齐失败");
    assert!(!outside.exists(), "绝不能通过悬空链接写到图标目录之外");
    assert!(
        fs::symlink_metadata(&link)
            .expect("链接还在")
            .file_type()
            .is_symlink(),
        "已有的链接不能被替换"
    );
    assert!(
        fixture.paths.icons_dir.join("codex.svg").is_file(),
        "坏条目不能阻止其余默认图标补齐"
    );
    // 链接本身也不该出现在列表里。
    let icons = icons::list(&fixture.paths).expect("list");
    assert!(icons.iter().all(|icon| icon.slug != "claude-code"));
}
