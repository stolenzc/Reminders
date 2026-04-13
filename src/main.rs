mod apple_reminders;
mod cli;
mod config;
mod cors;
mod hybrid_parser;
mod parser;
mod reminder;

use anyhow::Result;
use clap::Parser as ClapParser;
use cli::{Cli, Commands};
use hybrid_parser::HybridParser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add {
            description,
            force,
            list,
            test,
        } => handle_add(description, *force, list.as_deref(), *test, cli.quiet).await,

        Commands::Parse { description } => handle_parse(description, cli.quiet).await,
    }
}

/// 处理添加命令
async fn handle_add(
    description: &[String],
    force: bool,
    list: Option<&str>,
    test: bool,
    quiet: bool,
) -> Result<()> {
    if description.is_empty() {
        cli::show_error("请提供提醒事项的描述，例如：reminders add 明天下午3点开会");
        return Ok(());
    }

    let input = cli::parse_description_args(description);
    cli::show_info(&format!("正在解析: '{}'", input), quiet);

    let parser = match HybridParser::new(quiet) {
        Ok(p) => p,
        Err(e) => {
            cli::show_error(&format!("初始化解析器失败: {}", e));
            return Ok(());
        }
    };

    let parsed = match parser.parse(&input).await {
        Ok(parsed) => parsed,
        Err(e) => {
            cli::show_error(&format!("解析失败: {}", e));
            return Ok(());
        }
    };

    let mut parsed = parsed;
    if let Some(list_name) = list {
        parsed.list = list_name.to_string();
    }

    cli::show_parsed_summary(
        &parsed.title,
        parsed.due_date.as_ref(),
        &parsed.priority,
        parsed.is_urgent,
        &parsed.recurrence,
        &parsed.list,
        &parsed.tags,
        parsed.location.as_ref(),
    );

    if !force && !test {
        if !cli::confirm("确认添加？", true) {
            cli::show_info("已取消", quiet);
            return Ok(());
        }
    }

    if test {
        cli::show_success("测试模式 - 解析成功，不会实际创建提醒事项");
        return Ok(());
    }

    cli::show_progress("正在创建提醒事项...");
    let reminder = parsed.into_reminder();

    match apple_reminders::AppleReminders::create_reminder(&reminder) {
        Ok(_) => {
            cli::show_add_success(&reminder.title, &reminder.list);
        }
        Err(e) => {
            cli::show_error(&format!("创建失败: {}", e));
        }
    }

    Ok(())
}

/// 处理解析命令（调试用）
async fn handle_parse(description: &[String], quiet: bool) -> Result<()> {
    if description.is_empty() {
        cli::show_error("请提供提醒事项的描述");
        return Ok(());
    }

    let input = cli::parse_description_args(description);
    cli::show_info(&format!("正在解析: '{}'", input), quiet);

    let parser = match HybridParser::new(quiet) {
        Ok(p) => p,
        Err(e) => {
            cli::show_error(&format!("初始化解析器失败: {}", e));
            return Ok(());
        }
    };

    match parser.parse(&input).await {
        Ok(parsed) => {
            println!("\n📋 详细解析结果:");
            println!("{:#?}", parsed);
        }
        Err(e) => {
            cli::show_error(&format!("解析失败: {}", e));
        }
    }

    Ok(())
}
