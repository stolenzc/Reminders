use crate::{cors::Recurrence, parser::ParsedReminder};
use clap::{Arg, Command};
use std::io::{self, Write};

/// Build the CLI application with localized strings.
pub fn build_cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("智能提醒事项管理工具 - 通过自然语言添加提醒事项到苹果提醒事项.app")
        .long_about("通过自然语言输入创建提醒事项并同步到苹果提醒事项.app。支持中文输入，自动识别时间、优先级等信息。支持正则解析和 AI 解析两种模式。")
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("静默模式（减少输出信息）")
                .global(true)
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("add")
                .about("添加新的提醒事项（支持自然语言）")
                .arg(
                    Arg::new("description")
                        .help("自然语言描述，例如：明天下午3点开会")
                        .required(true)
                        .num_args(1..)
                )
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .help("强制模式（不询问确认，直接添加）")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("list")
                        .short('l')
                        .long("list")
                        .help("指定目标列表名称")
                        .action(clap::ArgAction::Set),
                )
                .arg(
                    Arg::new("test")
                        .short('t')
                        .long("test")
                        .help("测试模式（只解析不实际创建）")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("parse")
                .about("解析自然语言输入并显示结果（用于调试）")
                .arg(
                    Arg::new("description")
                        .help("自然语言描述，例如：明天下午3点开会")
                        .required(true)
                        .num_args(1..)
                ),
        )
}

/// Parse CLI arguments and return the matched command.
pub fn get_matches() -> clap::ArgMatches {
    build_cli().get_matches()
}

/// Extract subcommand info from ArgMatches
pub enum ParsedCommand {
    Add {
        description: Vec<String>,
        force: bool,
        list: Option<String>,
        test: bool,
        quiet: bool,
    },
    Parse {
        description: Vec<String>,
        quiet: bool,
    },
}

pub fn parse_command(matches: clap::ArgMatches) -> ParsedCommand {
    let quiet = matches.get_flag("quiet");
    match matches.subcommand() {
        Some(("add", sub)) => ParsedCommand::Add {
            description: sub
                .get_many::<String>("description")
                .unwrap_or_default()
                .cloned()
                .collect(),
            force: sub.get_flag("force"),
            list: sub.get_one::<String>("list").cloned(),
            test: sub.get_flag("test"),
            quiet,
        },
        Some(("parse", sub)) => ParsedCommand::Parse {
            description: sub
                .get_many::<String>("description")
                .unwrap_or_default()
                .cloned()
                .collect(),
            quiet,
        },
        _ => unreachable!("unreachable subcommand"),
    }
}

/// 用户确认
pub fn confirm(prompt: &str, default_yes: bool) -> bool {
    if default_yes {
        print!("{} [Y/n]: ", prompt);
    } else {
        print!("{} [y/N]: ", prompt);
    }
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    if default_yes {
        input.is_empty() || input == "y" || input == "yes"
    } else {
        input == "y" || input == "yes"
    }
}

/// 显示成功信息
#[allow(unused)]
pub fn show_success(message: &str) {
    println!("✅ {}", message);
}

/// 显示错误信息
#[allow(unused)]
pub fn show_error(message: &str) {
    eprintln!("❌ {}", message);
}

/// 显示警告信息
#[allow(unused)]
pub fn show_warning(message: &str) {
    println!("⚠️ {}", message);
}

/// 显示信息（安静模式下不显示）
#[allow(unused)]
pub fn show_info(message: &str, quiet: bool) {
    if !quiet {
        println!("ℹ️ {}", message);
    }
}

/// 显示进度信息
pub fn show_progress(message: &str) {
    println!("⏳ {}", message);
}

/// 显示解析结果摘要
pub fn show_parsed_summary(parsed: &ParsedReminder) {
    println!("\n📋 解析结果:");
    println!("  ┌─────────────────────────────────────────────┐");
    println!("  │ 标题: {}", parsed.title);

    if let Some(due_date) = parsed.due_date {
        println!("  │ 截止时间: {}", due_date.format("%Y-%m-%d %H:%M"));
    }

    println!("  │ 优先级: {}", parsed.priority);

    if parsed.is_urgent {
        println!("  │ 是否紧急: 是");
    }

    if !matches!(parsed.recurrence, Recurrence::None) {
        println!("  │ 重复: {}", parsed.recurrence);
    }

    println!("  │ 列表: {}", parsed.list);

    if !parsed.tags.is_empty() {
        println!("  │ 标签: {}", parsed.tags.join(", "));
    }

    if let Some(location) = &parsed.location {
        println!("  │ 位置: {}", location.name);
    }

    println!("  └─────────────────────────────────────────────┘");
}

/// 显示添加成功信息
pub fn show_add_success(title: &str, list: &str) {
    show_success(&format!("已添加 '{}' 到列表 '{}'", title, list));
}

/// 解析描述参数（处理多个单词的情况）
pub fn parse_description_args(args: &[String]) -> String {
    args.join(" ")
}
