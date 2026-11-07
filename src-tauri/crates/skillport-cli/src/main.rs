//! SkillPort CLI：把技能目录导入 SkillPort 库，供其他 agent 调用。
//!
//! 复用 `skillport-core`，与桌面端读写同一份库、DB 与路径守卫；
//! 默认输出人类可读文本，`--json` 输出稳定结构供程序解析。

mod args;
mod output;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use skillport_core::error::{AppError, AppResult};
use skillport_core::paths::AppPaths;
use skillport_core::{library, paths, workspace};

use args::Command;

const EXIT_OK: u8 = 0;
const EXIT_REJECTED: u8 = 1;
const EXIT_FAILED: u8 = 2;

fn main() -> ExitCode {
    let command = match args::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{}\n\n{}", error.0, args::USAGE);
            return ExitCode::from(EXIT_FAILED);
        }
    };

    match command {
        Command::Help => {
            println!("{}", args::USAGE);
            ExitCode::SUCCESS
        }
        Command::Version => {
            println!("skillport-cli {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Import {
            sources,
            data_dir,
            json,
        } => import(&sources, data_dir, json),
    }
}

/// 打开数据目录并导入。数据目录或数据库打不开是致命错误（2）；
/// 单个路径被严格校验拒绝不算致命（1），报告里逐条给出原因。
fn import(sources: &[String], data_dir: Option<PathBuf>, json: bool) -> ExitCode {
    let opened = match resolve_data_dir(data_dir).and_then(workspace::open) {
        Ok(opened) => opened,
        Err(error) => {
            output::print_error(&error, json);
            return ExitCode::from(EXIT_FAILED);
        }
    };

    match library::import(&opened.conn, &opened.paths, sources) {
        Ok(report) => {
            output::print_report(&opened.paths, &report, json);
            if report.skipped == 0 {
                ExitCode::from(EXIT_OK)
            } else {
                ExitCode::from(EXIT_REJECTED)
            }
        }
        Err(error) => {
            output::print_error(&error, json);
            ExitCode::from(EXIT_FAILED)
        }
    }
}

/// 数据目录优先级：`--data-dir` > `$SKILLPORT_DATA_DIR` > 平台默认。
fn resolve_data_dir(explicit: Option<PathBuf>) -> AppResult<PathBuf> {
    if let Some(path) = explicit {
        return absolute(&path);
    }
    if let Some(dir) = AppPaths::env_data_dir() {
        return Ok(dir);
    }
    paths::default_data_dir()
}

/// 把相对路径变成基于当前目录的绝对路径，避免库落在一个含糊的位置。
fn absolute(path: &Path) -> AppResult<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(AppError::InvalidInput("--data-dir 不能为空".to_string()));
    }
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::path::absolute(path).map_err(|error| AppError::io_at(path, error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_data_dir_becomes_absolute() {
        let resolved = absolute(Path::new("relative/data")).expect("absolute");
        assert!(resolved.is_absolute());
    }

    #[test]
    fn empty_data_dir_is_rejected() {
        assert!(absolute(Path::new("")).is_err());
    }
}
