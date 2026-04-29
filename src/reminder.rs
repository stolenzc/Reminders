use crate::cors::{Location, Priority, Recurrence};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 提醒事项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    /// 事项标题
    pub title: String,
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
    pub fn new(title: String) -> Self {
        Self {
            title,
            due_date: None,
            start_date: None,
            completed: false,
            priority: Priority::Medium,
            is_urgent: false,
            recurrence: Recurrence::None,
            location: None,
            reminder_minutes: vec![0],
            tags: Vec::new(),
            list: String::new(),
        }
    }

    /// 设置截止日期
    #[allow(unused)]
    pub fn with_due_date(mut self, due_date: Option<DateTime<Local>>) -> Self {
        self.due_date = due_date;
        self
    }

    /// 设置开始日期时间
    #[allow(unused)]
    pub fn with_start_date(mut self, start_date: Option<DateTime<Local>>) -> Self {
        self.start_date = start_date;
        self
    }

    /// 设置是否已完成
    #[allow(unused)]
    pub fn with_completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }

    /// 设置优先级
    #[allow(unused)]
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置是否紧急
    #[allow(unused)]
    pub fn with_urgent(mut self, is_urgent: bool) -> Self {
        self.is_urgent = is_urgent;
        self
    }

    /// 设置重复模式
    #[allow(unused)]
    pub fn with_recurrence(mut self, recurrence: Recurrence) -> Self {
        self.recurrence = recurrence;
        self
    }

    /// 设置位置信息
    #[allow(unused)]
    pub fn with_location(mut self, location: Option<Location>) -> Self {
        self.location = location;
        self
    }

    /// 设置提醒时间（分钟前）
    #[allow(unused)]
    pub fn with_reminder_minutes(mut self, reminder_minutes: Vec<i32>) -> Self {
        self.reminder_minutes = reminder_minutes;
        self
    }

    /// 添加标签
    #[allow(unused)]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// 设置所属列表
    #[allow(unused)]
    pub fn with_list(mut self, list: String) -> Self {
        self.list = list;
        self
    }
}

impl fmt::Display for Reminder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "提醒事项: {}", self.title)?;

        if let Some(due_date) = &self.due_date {
            write!(f, "\n截止时间: {}", due_date.format("%Y-%m-%d %H:%M"))?;
        }

        if let Some(start_date) = &self.start_date {
            write!(f, "\n开始时间: {}", start_date.format("%Y-%m-%d %H:%M"))?;
        }

        write!(f, "\n优先级: {}", self.priority)?;
        write!(f, "\n紧急: {}", self.is_urgent)?;
        write!(f, "\n重复: {}", self.recurrence)?;
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
        let s = match self {
            Recurrence::None => "不重复",
            Recurrence::Daily => "每天",
            Recurrence::Weekly => "每周",
            Recurrence::Monthly => "每月",
            Recurrence::Yearly => "每年",
            Recurrence::Weekdays => "工作日",
            Recurrence::Weekends => "周末",
            Recurrence::Custom(s) => {
                return write!(f, "自定义: {}", s);
            }
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Priority::None => "不重要",
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_new_reminder() {
        let reminder = Reminder::new("测试事项".to_string()).with_list("默认列表".to_string());
        assert_eq!(reminder.title, "测试事项");
        assert_eq!(reminder.list, "默认列表");
        assert_eq!(reminder.priority, Priority::Medium);
        assert!(!reminder.is_urgent);
        assert_eq!(reminder.reminder_minutes, vec![0]);
    }

    #[test]
    fn test_reminder_builder() {
        let due_date = Local.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let reminder = Reminder::new("重要会议".to_string())
            .with_due_date(Some(due_date))
            .with_priority(Priority::High)
            .with_urgent(true)
            .with_recurrence(Recurrence::Monthly);

        assert_eq!(reminder.title, "重要会议");
        assert_eq!(reminder.priority, Priority::High);
        assert!(reminder.is_urgent);
        assert!(matches!(reminder.recurrence, Recurrence::Monthly));
    }
}
