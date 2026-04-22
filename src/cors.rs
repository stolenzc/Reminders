use serde::{Deserialize, Serialize};

/// 重复模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Recurrence {
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Weekdays,
    Weekends,
    Custom(String),
}

/// 优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    None,
    Low,
    Medium,
    High,
}

/// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
}
