//! 集成测试共享夹具：临时数据目录 + 临时目标目录，绝不碰真实 HOME。

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use rusqlite::Connection;
use skillport_lib::core::library;
use skillport_lib::core::model::{ImportReport, LinkMode, Skill, SyncFilter, SyncResult, Target};
use skillport_lib::core::paths::{self, AppPaths};
use skillport_lib::core::{db, repo, sync, util};
use tempfile::TempDir;

pub struct Fixture {
    pub temp: TempDir,
    pub paths: AppPaths,
    pub conn: Connection,
}

impl Fixture {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let paths = AppPaths::new(temp.path().join("data"));
        paths.ensure_dirs().expect("创建数据目录");
        let conn = db::open(&paths.db_path).expect("打开数据库");
        Self { temp, paths, conn }
    }

    pub fn write_skill_source(&self, dir_name: &str, name: &str, description: &str) -> PathBuf {
        let dir = self.temp.path().join("sources").join(dir_name);
        fs::create_dir_all(dir.join("scripts")).expect("创建源目录");
        fs::write(
            dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n"),
        )
        .expect("写 SKILL.md");
        fs::write(dir.join("scripts/run.sh"), "echo hi\n").expect("写脚本");
        dir
    }

    pub fn import(&self, dir: &Path) -> ImportReport {
        library::import(&self.conn, &self.paths, &[dir.display().to_string()]).expect("导入")
    }

    pub fn import_ok(&self, dir_name: &str, name: &str) -> Skill {
        let dir = self.write_skill_source(dir_name, name, &format!("{name} 的描述"));
        let report = self.import(&dir);
        assert_eq!(report.imported, 1, "导入失败：{:?}", report.items);
        let slug = report.items[0].slug.clone().expect("slug");
        repo::find_skill_by_slug(&self.conn, &slug)
            .expect("查询技能")
            .expect("技能存在")
    }

    pub fn target_dir(&self, name: &str) -> PathBuf {
        self.temp.path().join("targets").join(name)
    }

    pub fn add_target(&self, slug: &str, mode: LinkMode) -> Target {
        let dir = self.target_dir(slug);
        fs::create_dir_all(&dir).expect("创建目标目录");
        let now = util::now_iso();
        let target = Target {
            id: util::new_id(),
            slug: slug.to_string(),
            name: slug.to_string(),
            kind: "skills_dir".to_string(),
            skills_dir: dir.display().to_string(),
            link_mode: mode,
            enabled: true,
            detected: false,
            created_at: now.clone(),
            updated_at: now,
        };
        repo::insert_target(&self.conn, &target).expect("写入目标");
        target
    }

    pub fn install_path(&self, skill: &Skill, target: &Target) -> PathBuf {
        paths::guarded_target_path(Path::new(&target.skills_dir), &skill.slug).expect("目标路径")
    }

    pub fn enable_install(&self, skill: &Skill, target: &Target, mode: LinkMode) {
        let path = self.install_path(skill, target);
        repo::upsert_install(
            &self.conn,
            &skill.id,
            &target.id,
            mode,
            &path.to_string_lossy(),
        )
        .expect("写入 Install");
    }

    pub fn plan_and_apply(
        &self,
        filter: &SyncFilter,
    ) -> (skillport_lib::core::model::SyncPlan, SyncResult) {
        let plan = sync::build_plan(&self.conn, &self.paths, filter).expect("生成计划");
        let cancel = AtomicBool::new(false);
        let mut events = 0usize;
        let result = sync::apply_plan(&self.conn, &self.paths, &plan, &cancel, &mut |_| {
            events += 1;
        })
        .expect("执行计划");
        assert_eq!(events, result.items.len());
        (plan, result)
    }

    pub fn install(&self, skill: &Skill, target: &Target) -> skillport_lib::core::model::Install {
        repo::get_install(&self.conn, &skill.id, &target.id)
            .expect("查询 Install")
            .expect("Install 存在")
    }

    pub fn reconcile(&self) -> skillport_lib::core::model::ReconcileReport {
        sync::reconcile(&self.conn, &self.paths).expect("核对")
    }

    pub fn backup_count(&self) -> usize {
        fs::read_dir(&self.paths.backups_dir)
            .map(|entries| entries.count())
            .unwrap_or(0)
    }
}

pub fn default_filter() -> SyncFilter {
    SyncFilter::default()
}
