//! 参数解析。只有 import 一个子命令，薄薄一层模式匹配就够，不引入 clap。

use std::path::PathBuf;

pub const USAGE: &str = "\
SkillPort CLI — 将技能目录导入 SkillPort 库

用法:
  skillport-cli import <PATH>... [--data-dir <DIR>] [--json]
  skillport-cli --help
  skillport-cli --version

子命令:
  import   导入一个或多个技能目录。目录本身含 SKILL.md 就导入它；
           否则导入其下所有含 SKILL.md 的直接子目录。严格校验，失败项不写库。

选项:
  --data-dir <DIR>  SkillPort 数据目录。默认依次取 $SKILLPORT_DATA_DIR、
                    平台应用数据目录（macOS: ~/Library/Application Support/com.skillport.desktop）
  --json            输出机器可读的 JSON 报告
  -h, --help        显示帮助
  -V, --version     显示版本
  --                其后的参数一律按路径处理（用于以 - 开头的路径）

退出码:
  0  全部导入成功
  1  有路径被拒绝（严格校验失败或 slug 冲突）
  2  参数错误、数据目录打不开或数据库写入失败
";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Import {
        sources: Vec<String>,
        data_dir: Option<PathBuf>,
        json: bool,
    },
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArgsError(pub String);

/// 解析 `std::env::args().skip(1)`。
pub fn parse<I>(args: I) -> Result<Command, ArgsError>
where
    I: IntoIterator<Item = String>,
{
    let mut positional: Vec<String> = Vec::new();
    let mut data_dir = None;
    let mut json = false;
    let mut literal = false;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if literal {
            positional.push(arg);
            continue;
        }
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            "--json" => json = true,
            "--" => literal = true,
            "--data-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| ArgsError("--data-dir 缺少参数".to_string()))?;
                data_dir = Some(PathBuf::from(value));
            }
            _ if arg.starts_with("--data-dir=") => {
                let value = arg.trim_start_matches("--data-dir=");
                if value.is_empty() {
                    return Err(ArgsError("--data-dir 缺少参数".to_string()));
                }
                data_dir = Some(PathBuf::from(value));
            }
            _ if arg.starts_with('-') && arg != "-" => {
                return Err(ArgsError(format!("未知选项：{arg}")));
            }
            _ => positional.push(arg),
        }
    }

    match positional.split_first() {
        Some((command, rest)) if command == "import" => {
            if rest.is_empty() {
                return Err(ArgsError("import 至少需要一个技能目录".to_string()));
            }
            Ok(Command::Import {
                sources: rest.to_vec(),
                data_dir,
                json,
            })
        }
        Some((command, _)) => Err(ArgsError(format!("未知子命令：{command}"))),
        None => Err(ArgsError("缺少子命令".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_vec(items: &[&str]) -> Result<Command, ArgsError> {
        parse(items.iter().map(|item| item.to_string()))
    }

    #[test]
    fn parses_import_with_multiple_sources() {
        let command = parse_vec(&["import", "a", "b"]).expect("parse");
        assert_eq!(
            command,
            Command::Import {
                sources: vec!["a".to_string(), "b".to_string()],
                data_dir: None,
                json: false,
            }
        );
    }

    #[test]
    fn accepts_flags_before_or_after_sources() {
        let command = parse_vec(&["--data-dir=/tmp/data", "import", "a", "--json"]).expect("parse");
        assert_eq!(
            command,
            Command::Import {
                sources: vec!["a".to_string()],
                data_dir: Some(PathBuf::from("/tmp/data")),
                json: true,
            }
        );
    }

    #[test]
    fn treats_arguments_after_double_dash_as_paths() {
        let command = parse_vec(&["import", "--", "-weird-path"]).expect("parse");
        assert_eq!(
            command,
            Command::Import {
                sources: vec!["-weird-path".to_string()],
                data_dir: None,
                json: false,
            }
        );
    }

    #[test]
    fn help_and_version_win_over_subcommand() {
        assert_eq!(parse_vec(&["import", "--help"]), Ok(Command::Help));
        assert_eq!(parse_vec(&["import", "-V"]), Ok(Command::Version));
    }

    #[test]
    fn rejects_missing_sources_and_unknown_options() {
        assert!(parse_vec(&["import"]).is_err());
        assert!(parse_vec(&["import", "--bogus", "a"]).is_err());
        assert!(parse_vec(&["copy", "a"]).is_err());
        assert!(parse_vec(&[]).is_err());
        assert!(parse_vec(&["import", "--data-dir"]).is_err());
    }
}
