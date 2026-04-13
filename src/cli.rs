use clap::{Parser, Subcommand};
use std::io::{self, Write};
use crate::cors::{Priority, Recurrence, Location};

/// 提醒事项命令行工具
/// command 不声明name，自动读取cargo.toml 中的name
/// command 声明 version,不设置值，自动读取cargo.toml中的version
#[derive(Parser, Debug)]
#[command(
    version,
    about = "智能提醒事项管理工具 - 通过自然语言添加提醒事项到苹果提醒事项.app",
    long_about = "通过自然语言输入创建提醒事项并同步到苹果提醒事项.app。支持中文输入，自动识别时间、优先级等信息。支持正则解析和 AI 解析两种模式。"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 静默模式（减少输出信息）
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

/// 子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 添加新的提醒事项（支持自然语言）
    Add {
        /// 自然语言描述，例如：明天下午3点开会
        description: Vec<String>,

        /// 强制模式（不询问确认，直接添加）
        #[arg(short, long)]
        force: bool,

        /// 指定目标列表名称
        #[arg(short, long)]
        list: Option<String>,

        /// 测试模式（只解析不实际创建）
        #[arg(short, long)]
        test: bool,
    },

    /// 解析自然语言输入并显示结果（用于调试）
    Parse {
        /// 自然语言描述
        description: Vec<String>,
    },
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
    println!("⚠️  {}", message);
}

/// 显示信息（安静模式下不显示）
#[allow(unused)]
pub fn show_info(message: &str, quiet: bool) {
    if !quiet {
        println!("ℹ️  {}", message);
    }
}

/// 显示进度信息
pub fn show_progress(message: &str) {
    println!("⏳ {}", message);
}

/// 显示解析结果摘要
pub fn show_parsed_summary(
    title: &str,
    due_date: Option<&chrono::DateTime<chrono::Local>>,
    priority: &Priority,
    is_urgent: bool,
    recurrence: &Recurrence,
    list: &str,
    tags: &[String],
    location: Option<&Location>,
) {
    println!("\n📋 解析结果:");
    println!("  ┌─────────────────────────────────────────────┐");
    println!("  │ 标题: {}", title);

    if let Some(due_date) = due_date {
        println!("  │ 截止时间: {}", due_date.format("%Y-%m-%d %H:%M"));
    }

    println!("  │ 优先级: {}", priority);

    if is_urgent {
        println!("  │ 紧急: 是");
    }

    if !matches!(recurrence, Recurrence::None) {
        println!("  │ 重复: {}", recurrence);
    }

    println!("  │ 列表: {}", list);

    if !tags.is_empty() {
        println!("  │ 标签: {}", tags.join(", "));
    }

    if let Some(location) = location {
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
