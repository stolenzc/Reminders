use crate::cors::{Location, Priority, Recurrence};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 提醒事项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    /// 事项标题
    pub title: String,
    /// 事项描述（可选）
    pub description: Option<String>,
    /// 截止日期时间
    pub due_date: Option<DateTime<Local>>,
    /// 开始日期时间（可选）
    pub start_date: Option<DateTime<Local>>,
    /// 是否已完成
    pub completed: bool,
    /// 优先级
    pub priority: Priority,
    /// 是否紧急
    pub is_urgent: bool,
    /// 重复模式
    pub recurrence: Recurrence,
    /// 位置信息（可选）
    pub location: Option<Location>,
    /// 提醒时间（分钟前）
    pub reminder_minutes: Vec<i32>,
    /// 标签
    pub tags: Vec<String>,
    /// 所属列表
    pub list: String,
}

impl Reminder {
    /// 创建一个新的提醒事项
    pub fn new(title: String, list: String) -> Self {
        Self {
            title,
            description: None,
            due_date: None,
            start_date: None,
            completed: false,
            priority: Priority::Medium,
            is_urgent: false,
            recurrence: Recurrence::None,
            location: None,
            reminder_minutes: vec![15, 5], // 默认提前15分钟和5分钟提醒
            tags: Vec::new(),
            list,
        }
    }

    /// 设置截止日期
    pub fn with_due_date(mut self, due_date: DateTime<Local>) -> Self {
        self.due_date = Some(due_date);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置是否紧急
    pub fn with_urgent(mut self, is_urgent: bool) -> Self {
        self.is_urgent = is_urgent;
        self
    }

    /// 设置重复模式
    pub fn with_recurrence(mut self, recurrence: Recurrence) -> Self {
        self.recurrence = recurrence;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 添加标签
    pub fn add_tag(&mut self, tags: Vec<String>) {
        for tag in tags {
            self.tags.push(tag);
        }
    }

    /// 添加提醒时间
    pub fn add_reminder(&mut self, minutes_before: i32) {
        self.reminder_minutes.push(minutes_before);
        self.reminder_minutes.sort_unstable();
        self.reminder_minutes.dedup();
    }

    /// 检查是否过期
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            due_date < Local::now()
        } else {
            false
        }
    }
}

impl fmt::Display for Reminder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "提醒事项: {}", self.title)?;

        if let Some(desc) = &self.description {
            write!(f, "\n描述: {}", desc)?;
        }

        if let Some(due_date) = &self.due_date {
            write!(f, "\n截止时间: {}", due_date.format("%Y-%m-%d %H:%M"))?;
        }

        if let Some(start_date) = &self.start_date {
            write!(f, "\n开始时间: {}", start_date.format("%Y-%m-%d %H:%M"))?;
        }

        write!(f, "\n优先级: {:?}", self.priority)?;
        write!(f, "\n紧急: {}", self.is_urgent)?;
        write!(f, "\n重复: {:?}", self.recurrence)?;
        write!(f, "\n列表: {}", self.list)?;

        if !self.tags.is_empty() {
            write!(f, "\n标签: {}", self.tags.join(", "))?;
        }

        if !self.reminder_minutes.is_empty() {
            write!(f, "\n提醒时间（提前分钟）: {:?}", self.reminder_minutes)?;
        }

        if let Some(location) = &self.location {
            write!(f, "\n位置: {}", location.name)?;
            if let Some(addr) = &location.address {
                write!(f, " ({})", addr)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Recurrence::None => write!(f, "不重复"),
            Recurrence::Daily => write!(f, "每天"),
            Recurrence::Weekly => write!(f, "每周"),
            Recurrence::Monthly => write!(f, "每月"),
            Recurrence::Yearly => write!(f, "每年"),
            Recurrence::Weekdays => write!(f, "工作日"),
            Recurrence::Weekends => write!(f, "周末"),
            Recurrence::Custom(s) => write!(f, "自定义: {}", s),
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::None => write!(f, "不重要"),
            Priority::Low => write!(f, "低"),
            Priority::Medium => write!(f, "中"),
            Priority::High => write!(f, "高"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_new_reminder() {
        let reminder = Reminder::new("测试事项".to_string(), "默认列表".to_string());
        assert_eq!(reminder.title, "测试事项");
        assert_eq!(reminder.list, "默认列表");
        assert_eq!(reminder.priority, Priority::Medium);
        assert!(!reminder.is_urgent);
        assert_eq!(reminder.reminder_minutes, vec![15, 5]);
    }

    #[test]
    fn test_reminder_builder() {
        let due_date = Local.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let reminder = Reminder::new("重要会议".to_string(), "工作".to_string())
            .with_due_date(due_date)
            .with_priority(Priority::High)
            .with_urgent(true)
            .with_description("与客户的季度回顾会议".to_string())
            .with_recurrence(Recurrence::Monthly);

        assert_eq!(reminder.title, "重要会议");
        assert_eq!(reminder.priority, Priority::High);
        assert!(reminder.is_urgent);
        assert!(matches!(reminder.recurrence, Recurrence::Monthly));
        assert_eq!(
            reminder.description,
            Some("与客户的季度回顾会议".to_string())
        );
    }

    #[test]
    fn test_is_overdue() {
        let past_date = Local.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let future_date = Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();

        let overdue_reminder =
            Reminder::new("过期事项".to_string(), "测试".to_string()).with_due_date(past_date);

        let future_reminder =
            Reminder::new("未来事项".to_string(), "测试".to_string()).with_due_date(future_date);

        assert!(overdue_reminder.is_overdue());
        assert!(!future_reminder.is_overdue());
    }
}
