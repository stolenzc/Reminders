use super::cors::ParsedReminder;
use crate::config::ConfigManager;
use crate::cors::{Location, Priority, Recurrence};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// AI API 请求结构
#[derive(Debug, Serialize)]
struct AIRequest {
    model: String,
    messages: Vec<AIMessage>,
    temperature: f32,
    response_format: Option<AIResponseFormat>,
}

#[derive(Debug, Serialize)]
struct AIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct AIResponseFormat {
    #[serde(rename = "type")]
    type_: String,
}

/// AI API 响应结构
#[derive(Debug, Deserialize)]
struct AIResponse {
    choices: Vec<AIChoice>,
    error: Option<AIError>,
}

#[derive(Debug, Deserialize)]
struct AIChoice {
    message: AIMessageResponse,
}

#[derive(Debug, Deserialize)]
struct AIMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct AIError {
    message: String,
}

/// AI 解析结果结构（需要与提示词中的 JSON 格式匹配）
#[derive(Debug, Deserialize)]
struct AIParsedResult {
    title: String,
    due_date: Option<String>,
    start_date: Option<String>,
    priority: String,
    is_urgent: bool,
    recurrence: String,
    location: Option<AILocation>,
    reminder_minutes: Vec<i32>,
    tags: Vec<String>,
    list: String,
}

#[derive(Debug, Deserialize)]
struct AILocation {
    name: String,
    address: Option<String>,
}

/// AI 解析器
#[derive(Clone)]
pub struct AIParser {
    client: Client,
    config_manager: ConfigManager,
}

impl AIParser {
    /// 创建新的 AI 解析器
    pub fn new(config_manager: ConfigManager) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            config_manager,
        })
    }

    /// 使用 AI 解析输入
    pub async fn parse_with_ai(&self, input: &str) -> Result<ParsedReminder> {
        if !self.config_manager.is_ai_configured() {
            return Err(anyhow!("AI 未配置"));
        }

        let ai_config = self.config_manager.get_ai_config();
        let prompt = self.generate_prompt(input);

        let request = AIRequest {
            model: ai_config.model.clone(),
            messages: vec![AIMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.1,
            response_format: Some(AIResponseFormat {
                type_: "json_object".to_string(),
            }),
        };

        let response = self
            .client
            .post(&ai_config.api_url)
            .header("Authorization", format!("Bearer {}", ai_config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("AI API 请求失败: {}", error_text));
        }

        let response_text = response.text().await?;
        let ai_response: AIResponse = serde_json::from_str(&response_text)?;

        if let Some(error) = ai_response.error {
            return Err(anyhow!("AI API 错误: {}", error.message));
        }

        if ai_response.choices.is_empty() {
            return Err(anyhow!("AI 未返回有效结果"));
        }

        let content = &ai_response.choices[0].message.content;
        self.parse_ai_response(content)
    }

    /// 生成提示词
    fn generate_prompt(&self, input: &str) -> String {
        let current_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let default_list = self.config_manager.get_default_list();

        let template = "你是一个智能提醒事项解析器。请将用户输入的自然语言解析为结构化的提醒事项信息。\n\n输入：{input}\n\n请按照以下 JSON 格式返回解析结果：\n\n{{\n  \"title\": \"提醒事项标题\",\n  \"description\": \"可选描述\",\n  \"due_date\": \"截止日期时间，格式：YYYY-MM-DD HH:MM:SS，如果无法确定则为 null\",\n  \"start_date\": \"开始日期时间，格式：YYYY-MM-DD HH:MM:SS，可选\",\n  \"priority\": \"优先级，可选值：none, low, medium, high\",\n  \"is_urgent\": \"是否紧急，布尔值\",\n  \"recurrence\": \"重复模式，可选：none, daily, weekly, monthly, yearly, weekdays, weekends\",\n  \"location\": {{\n    \"name\": \"位置名称\",\n    \"address\": \"详细地址，可选\"\n  }},\n  \"reminder_minutes\": [0],\n  \"tags\": [\"标签1\", \"标签2\"],\n  \"list\": \"列表名称\"\n}}\n\n注意：\n1. 当前时间：{current_time}\n2. 如果用户没有指定时间，请根据上下文推断合理的时间\n3. 标题应该简洁明了\n4. 优先使用中文标签\n5. 如果用户指定了列表，使用用户指定的列表，否则使用 \"{default_list}\"\n6. 日期时间请使用 24 小时制\n7. 如果无法确定某些字段，请使用合理的默认值\n8. 请确保返回的是有效的 JSON 格式";
        template
            .replace("{input}", input)
            .replace("{current_time}", &current_time)
            .replace("{default_list}", default_list)
    }

    fn parse_ai_response(&self, content: &str) -> Result<ParsedReminder> {
        let json_content = if content.trim().starts_with('{') && content.trim().ends_with('}') {
            content.trim().to_string()
        } else {
            let start = content.find('{').unwrap_or(0);
            let end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
            content[start..end].to_string()
        };

        let ai_result: AIParsedResult = serde_json::from_str(&json_content)?;

        Ok(ParsedReminder {
            title: ai_result.title,
            due_date: parse_datetime_string(&ai_result.due_date),
            start_date: parse_datetime_string(&ai_result.start_date),
            priority: parse_priority(&ai_result.priority),
            is_urgent: ai_result.is_urgent,
            recurrence: parse_recurrence(&ai_result.recurrence),
            location: ai_result.location.map(|loc| Location {
                name: loc.name,
                latitude: None,
                longitude: None,
                address: loc.address,
            }),
            reminder_minutes: ai_result.reminder_minutes,
            tags: ai_result.tags,
            list: ai_result.list,
        })
    }
}

/// 解析日期时间字符串
fn parse_datetime_string(datetime_str: &Option<String>) -> Option<DateTime<Local>> {
    datetime_str.as_ref().and_then(|s| {
        if s.to_lowercase() == "null" || s.is_empty() {
            return None;
        }

        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d %H:%M",
        ];

        for format in &formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, format) {
                return Some(Local.from_local_datetime(&dt).unwrap());
            }
        }

        None
    })
}

/// 解析优先级字符串
fn parse_priority(priority_str: &str) -> Priority {
    match priority_str.to_lowercase().as_str() {
        "none" => Priority::None,
        "low" => Priority::Low,
        "medium" => Priority::Medium,
        "high" => Priority::High,
        _ => Priority::Medium,
    }
}

/// 解析重复模式
fn parse_recurrence(recurrence_str: &str) -> Recurrence {
    match recurrence_str.to_lowercase().as_str() {
        "daily" => Recurrence::Daily,
        "weekly" => Recurrence::Weekly,
        "monthly" => Recurrence::Monthly,
        "yearly" => Recurrence::Yearly,
        "weekdays" => Recurrence::Weekdays,
        "weekends" => Recurrence::Weekends,
        "custom" => Recurrence::Custom("custom".to_string()),
        _ => Recurrence::None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::config::ConfigManager;

    #[test]
    fn test_parse_datetime_string() {
        assert!(parse_datetime_string(&Some("2024-01-01 12:00:00".to_string())).is_some());
        assert!(parse_datetime_string(&Some("null".to_string())).is_none());
        assert!(parse_datetime_string(&Some("".to_string())).is_none());
        assert!(parse_datetime_string(&None).is_none());
    }

    #[test]
    fn test_parse_priority() {
        assert_eq!(parse_priority("none"), Priority::None);
        assert_eq!(parse_priority("low"), Priority::Low);
        assert_eq!(parse_priority("medium"), Priority::Medium);
        assert_eq!(parse_priority("high"), Priority::High);
        assert_eq!(parse_priority("unknown"), Priority::Medium);
    }

    #[test]
    fn test_parse_recurrence() {
        assert!(matches!(parse_recurrence("daily"), Recurrence::Daily));
        assert!(matches!(parse_recurrence("weekly"), Recurrence::Weekly));
        assert!(matches!(parse_recurrence("none"), Recurrence::None));
        assert!(matches!(parse_recurrence("unknown"), Recurrence::None));
    }

    #[test]
    fn test_generate_prompt() {
        let config_manager = ConfigManager::new().unwrap();
        let ai_parser = AIParser::new(config_manager).unwrap();

        let prompt = ai_parser.generate_prompt("明天下午3点开会");
        assert!(prompt.contains("明天下午3点开会"));
        assert!(prompt.contains("当前时间：") || prompt.contains("Current time:"));
    }
}
