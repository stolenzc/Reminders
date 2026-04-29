mod apple_reminders;
mod cli;
mod config;
mod cors;
mod hybrid_parser;
mod parser;
mod reminder;

use anyhow::Result;
use cli::{ParsedCommand, parse_command};

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::ConfigManager::new().ok();

    let matches = cli::get_matches();
    let command = parse_command(matches);

    match command {
        ParsedCommand::Add {
            ref description,
            force,
            ref list,
            test,
            quiet,
        } => handle_add(description, force, list.as_deref(), test, quiet, &config).await,
        ParsedCommand::Parse {
            ref description,
            quiet,
        } => handle_parse(description, quiet, &config).await,
    }
}

async fn handle_add(
    description: &[String],
    force: bool,
    list: Option<&str>,
    test: bool,
    quiet: bool,
    config: &Option<config::ConfigManager>,
) -> Result<()> {
    if description.is_empty() {
        cli::show_error("请提供提醒事项的描述");
        return Ok(());
    }

    let input = cli::parse_description_args(description);
    cli::show_info(&format!("ℹ️ 正在解析: '{}'", input), quiet);

    let parser = match config {
        Some(cm) => match hybrid_parser::HybridParser::from_config(cm.clone(), quiet) {
            Ok(p) => p,
            Err(e) => {
                cli::show_error(&format!("❌ 初始化解析器失败: {}", e));
                return Ok(());
            }
        },
        Option::None => {
            cli::show_error("❌ 初始化解析器失败: config");
            return Ok(());
        }
    };

    let mut reminder = match parser.parse(&input).await {
        Ok(p) => p,
        Err(e) => {
            cli::show_error(&format!("❌ 解析失败: {}", e));
            return Ok(());
        }
    };

    if let Some(list_name) = list {
        reminder = reminder.with_list(list_name.to_string());
    }

    cli::show_parsed_summary(&reminder);

    if !force && !test && !cli::confirm("确认添加？", true) {
        cli::show_info("已取消", quiet);
        return Ok(());
    }

    if test {
        cli::show_success("测试模式 - 解析成功，不会实际创建提醒事项");
        return Ok(());
    }

    cli::show_progress("⏳ 正在创建提醒事项...");

    match apple_reminders::AppleReminders::create_reminder(&reminder) {
        Ok(_) => {
            cli::show_add_success(&reminder.title, &reminder.list);
        }
        Err(e) => {
            cli::show_error(&format!("❌ 创建失败: {}", e));
        }
    }

    Ok(())
}

async fn handle_parse(
    description: &[String],
    quiet: bool,
    config: &Option<config::ConfigManager>,
) -> Result<()> {
    if description.is_empty() {
        cli::show_error("请提供提醒事项的描述");
        return Ok(());
    }

    let input = cli::parse_description_args(description);
    cli::show_info(&format!("ℹ️ 正在解析: '{}'", input), quiet);

    let parser = match config {
        Some(cm) => match hybrid_parser::HybridParser::from_config(cm.clone(), quiet) {
            Ok(p) => p,
            Err(e) => {
                cli::show_error(&format!("❌ 初始化解析器失败: {}", e));
                return Ok(());
            }
        },
        Option::None => {
            cli::show_error("❌ 初始化解析器失败: config");
            return Ok(());
        }
    };

    match parser.parse(&input).await {
        Ok(parsed) => {
            println!("\n📋 详细解析结果:");
            println!("{:#?}", parsed);
        }
        Err(e) => {
            cli::show_error(&format!("❌ 解析失败: {}", e));
        }
    }

    Ok(())
}
