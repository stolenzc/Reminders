use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// AI 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// API 地址
    pub api_url: String,
    /// API 密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// AI 配置
    pub ai: AIConfig,
    /// 默认列表
    pub default_list: String,
    /// 默认提醒时间（分钟）
    pub default_reminder_minutes: Vec<i32>,
    /// 是否使用 AI 解析（默认 true）
    #[serde(default = "default_true")]
    pub use_ai: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AIConfig {
                api_url: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: "".to_string(),
                model: "gpt-4-turbo-preview".to_string(),
            },
            default_list: "提醒事项".to_string(),
            default_reminder_minutes: vec![15],
            use_ai: false,
        }
    }
}

/// 配置管理器
#[derive(Clone)]
pub struct ConfigManager {
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("无法获取配置目录"))?
            .join("reminders");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config_path = config_dir.join("config.json").to_string_lossy().to_string();
        let config = Self::load_config(&config_path)?;

        Ok(Self { config })
    }

    /// 加载配置
    fn load_config(path: &str) -> Result<AppConfig> {
        if Path::new(path).exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let default_config = AppConfig::default();
            Self::save_config(path, &default_config)?;
            Ok(default_config)
        }
    }

    /// 保存配置
    fn save_config(path: &str, config: &AppConfig) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 检查 AI 配置是否有效
    pub fn is_ai_configured(&self) -> bool {
        !self.config.ai.api_key.is_empty() && self.config.use_ai
    }

    /// 获取 AI 配置
    pub fn get_ai_config(&self) -> &AIConfig {
        &self.config.ai
    }

    /// 获取默认列表
    pub fn get_default_list(&self) -> &str {
        &self.config.default_list
    }

    /// 获取默认提醒时间
    pub fn get_default_reminder_minutes(&self) -> &[i32] {
        &self.config.default_reminder_minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.default_list, "提醒事项");
        assert_eq!(config.default_reminder_minutes, vec![15]);
        assert!(!config.use_ai);
    }
}
