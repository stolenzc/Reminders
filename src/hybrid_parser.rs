use crate::config::ConfigManager;
use crate::parser;
use crate::parser::{AIParser, ParsedReminder};
use anyhow::Result;

/// 混合解析器 - 优先使用 AI，失败时回退到正则解析
pub struct HybridParser {
    ai_parser: Option<AIParser>,
    config_manager: ConfigManager,
    quiet: bool,
}

impl HybridParser {
    /// 创建新的混合解析器
    pub fn new(quiet: bool) -> Result<Self> {
        let config_manager = ConfigManager::new()?;

        let ai_parser = if config_manager.is_ai_configured() {
            Some(AIParser::new(config_manager.clone())?)
        } else {
            None
        };

        Ok(Self {
            ai_parser,
            config_manager,
            quiet,
        })
    }

    /// 解析输入 - 优先 AI，失败时回退到正则
    pub async fn parse(&self, input: &str) -> Result<ParsedReminder> {
        let default_list = self.config_manager.get_default_list().to_string();

        if let Some(ref ai_parser) = self.ai_parser {
            if !self.quiet {
                println!("🤖 使用 AI 解析...");
            }
            match ai_parser.parse_with_ai(input).await {
                Ok(result) => {
                    if !self.quiet {
                        println!("✅ AI 解析成功");
                    }
                    return Ok(self.apply_defaults(result));
                }
                Err(e) => {
                    if !self.quiet {
                        println!("⚠️  AI 解析失败：{}，回退到正则解析", e);
                    }
                }
            }
        } else if !self.quiet {
            println!("📝 使用正则解析（未配置 AI）");
        }

        let result = parser::parse_input(input, &default_list)?;
        if !self.quiet {
            println!("✅ 正则解析成功");
        }
        Ok(self.apply_defaults(result))
    }

    fn apply_defaults(&self, mut result: ParsedReminder) -> ParsedReminder {
        // 如果列表为空，使用默认列表
        if result.list.is_empty() {
            result.list = self.config_manager.get_default_list().to_string();
        }

        // 如果没有设置提醒时间，使用默认值
        if result.reminder_minutes.is_empty() {
            result.reminder_minutes = self.config_manager.get_default_reminder_minutes().to_vec();
        }

        result
    }
}

impl Clone for HybridParser {
    fn clone(&self) -> Self {
        Self {
            ai_parser: self.ai_parser.clone(),
            config_manager: self.config_manager.clone(),
            quiet: self.quiet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_parse() {
        let result = parser::parse_input("明天下午3点开会", "reminders");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.due_date.is_some());
    }

    #[tokio::test]
    async fn test_regex_parse_daily() {
        let result = parser::parse_input("每天早上8点吃药", "reminders").unwrap();
        assert!(matches!(result.recurrence, crate::cors::Recurrence::Daily));
    }
}
