use crate::cors::{Priority, Recurrence};
use crate::reminder::Reminder;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Datelike, Local, Timelike};
use std::process::{Command, Output};

/// 苹果提醒事项集成（通过 AppleScript 与 Reminders.app 通信）
pub struct AppleReminders;

impl AppleReminders {
    /// 创建一个新的提醒事项
    pub fn create_reminder(reminder: &Reminder) -> Result<()> {
        let script = build_reminder_script(reminder)?;

        let output = Command::new("osascript").arg("-e").arg(&script).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("创建提醒事项失败: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.to_lowercase().contains("error") {
            return Err(anyhow!("AppleScript 错误: {}", stdout.trim()));
        }

        Ok(())
    }

    /// 检查列表是否存在
    pub fn list_exists(list_name: &str) -> Result<bool> {
        let script = r#"tell application "Reminders"
    set listNames to name of every list
    return listNames
end tell"#;

        // 获取全部列表名称, 不使用AppleScript检测
        let output: Output = Command::new("osascript").arg("-e").arg(script).output()?;
        let list_str = String::from_utf8_lossy(&output.stdout);
        let list_vec: Vec<String> = list_str
            .replace("\n", ",")
            .split(",")
            .map(|s| s.to_string())
            .collect();
        for exist_name in list_vec {
            if exist_name == list_name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 创建新列表（如果不存在）
    pub fn create_list_if_not_exists(list_name: &str) -> Result<()> {
        if !Self::list_exists(list_name)? {
            let script = format!(
                r#"tell application "Reminders"
    make new list with properties {{name:"{}"}}
end tell"#,
                escape_string(list_name)
            );

            let output = Command::new("osascript").arg("-e").arg(&script).output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("创建列表失败: {}", stderr.trim()));
            }
        }

        Ok(())
    }
}

/// 构建创建提醒事项的 AppleScript
fn build_reminder_script(reminder: &Reminder) -> Result<String> {
    // 确保目标列表存在
    AppleReminders::create_list_if_not_exists(&reminder.list)?;

    let apple_priority: u8 = match reminder.priority {
        Priority::None => 0,
        Priority::Low => 1,
        Priority::Medium => 5,
        Priority::High => 9,
    };

    // 构建脚本头部（创建提醒事项，设置标题和优先级）
    let mut script = format!(
        r#"tell application "Reminders"
    set targetList to list "{list}"
    set newReminder to (make new reminder in targetList with properties {{name:"{title}", priority:{priority}}})
"#,
        list = escape_string(&reminder.list),
        title = escape_string(&reminder.title),
        priority = apple_priority,
    );

    // 截止日期（使用逐属性赋值，避免区域设置问题）
    if let Some(due_date) = &reminder.due_date {
        script.push_str(&build_set_date_script("due date", due_date, "newReminder"));
    }

    // 重复规则
    if let Some(rrule) = recurrence_to_rrule(&reminder.recurrence) {
        script.push_str(&format!(
            "    set recurrence of newReminder to \"{}\"\n",
            rrule
        ));
    }

    // 提醒时间（在截止日期前 N 分钟）
    if let (Some(due_date), Some(&minutes)) =
        (&reminder.due_date, reminder.reminder_minutes.first())
        && minutes >= 0
    {
        let alarm_date = *due_date - chrono::Duration::minutes(minutes as i64);
        script.push_str(&build_set_date_script(
            "remind me date",
            &alarm_date,
            "newReminder",
        ));
    }

    script.push_str("end tell");

    Ok(script)
}

/// 构建用于设置日期属性的 AppleScript（区域无关的方式）
fn build_set_date_script(prop: &str, dt: &DateTime<Local>, var: &str) -> String {
    format!(
        r#"    set tmpDate to current date
    set year of tmpDate to {year}
    set month of tmpDate to {month}
    set day of tmpDate to {day}
    set hours of tmpDate to {hour}
    set minutes of tmpDate to {minute}
    set seconds of tmpDate to 0
    set {prop} of {var} to tmpDate
"#,
        year = dt.year(),
        month = dt.month(),
        day = dt.day(),
        hour = dt.hour(),
        minute = dt.minute(),
        prop = prop,
        var = var,
    )
}

/// 将 Recurrence 转换为 RRULE 字符串
fn recurrence_to_rrule(recurrence: &Recurrence) -> Option<String> {
    match recurrence {
        Recurrence::None => None,
        Recurrence::Daily => Some("FREQ=DAILY;INTERVAL=1".to_string()),
        Recurrence::Weekly => Some("FREQ=WEEKLY;INTERVAL=1".to_string()),
        Recurrence::Monthly => Some("FREQ=MONTHLY;INTERVAL=1".to_string()),
        Recurrence::Yearly => Some("FREQ=YEARLY;INTERVAL=1".to_string()),
        Recurrence::Weekdays => Some("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR".to_string()),
        Recurrence::Weekends => Some("FREQ=WEEKLY;BYDAY=SA,SU".to_string()),
        Recurrence::Custom(s) => Some(s.clone()),
    }
}

/// 转义 AppleScript 字符串中的特殊字符
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("Hello \"World\""), "Hello \\\"World\\\"");
        assert_eq!(escape_string("Back\\Slash"), "Back\\\\Slash");
        assert_eq!(escape_string("Normal text"), "Normal text");
    }

    #[test]
    fn test_recurrence_to_rrule() {
        assert_eq!(
            recurrence_to_rrule(&Recurrence::Daily),
            Some("FREQ=DAILY;INTERVAL=1".to_string())
        );
        assert_eq!(
            recurrence_to_rrule(&Recurrence::Weekly),
            Some("FREQ=WEEKLY;INTERVAL=1".to_string())
        );
        assert_eq!(recurrence_to_rrule(&Recurrence::None), None);
    }

    #[test]
    fn test_build_set_date_script() {
        let dt = Local.with_ymd_and_hms(2026, 4, 11, 15, 0, 0).unwrap();
        let script = build_set_date_script("due date", &dt, "newReminder");
        assert!(script.contains("set year of tmpDate to 2026"));
        assert!(script.contains("set month of tmpDate to 4"));
        assert!(script.contains("set hours of tmpDate to 15"));
    }
}
