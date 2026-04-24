use super::cors::ParsedReminder;
use crate::cors::{Priority, Recurrence};
use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Timelike};
use regex::Regex;
use std::sync::OnceLock;

// ── Hardcoded Chinese parsing keywords (was loaded from locales/parsing/zh.json) ──

fn regex_from_strs(strings: &[&str]) -> Regex {
    Regex::new(
        &strings
            .iter()
            .map(|s| regex::escape(s))
            .collect::<Vec<_>>()
            .join("|"),
    )
    .unwrap()
}

fn strs_to_alt(strings: &[&str]) -> String {
    if strings.len() == 1 {
        regex::escape(strings[0])
    } else {
        let escaped: Vec<String> = strings.iter().map(|s| regex::escape(s)).collect();
        format!("({})", escaped.join("|"))
    }
}

fn build_suffix_pattern(suffixes: &[&str], capture: &str) -> Regex {
    let alt = strs_to_alt(suffixes);
    Regex::new(&format!(r"{}{}", capture, alt)).unwrap()
}

fn build_hour_pattern(hour_marker: &str) -> Regex {
    let escaped_marker = regex::escape(hour_marker);
    Regex::new(&format!(r"(\d+){}", escaped_marker)).unwrap()
}

fn build_next_week_pattern(prefixes: &[&str], weekdays: &[(&str, &[&str])]) -> Regex {
    let prefix_alt = strs_to_alt(prefixes);
    let all_days: Vec<String> = weekdays
        .iter()
        .flat_map(|(_, days)| days.iter().map(|d| d.to_string()))
        .collect();
    Regex::new(&format!(r"(?:{})([{}])", prefix_alt, all_days.join("|"))).unwrap()
}

fn build_this_week_pattern(prefixes: &[&str], weekdays: &[(&str, &[&str])]) -> Regex {
    let prefix_alt = strs_to_alt(prefixes);
    let all_days: Vec<String> = weekdays
        .iter()
        .flat_map(|(_, days)| days.iter().map(|d| d.to_string()))
        .collect();
    Regex::new(&format!(r"(?:{}[{}])", prefix_alt, all_days.join("|"))).unwrap()
}

fn time_patterns() -> &'static Vec<(Regex, &'static str)> {
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let weekdays: &[(&str, &[&str])] = &[
            ("mon", &["一"]),
            ("tue", &["二"]),
            ("wed", &["三"]),
            ("thu", &["四"]),
            ("fri", &["五"]),
            ("sat", &["六"]),
            ("sun", &["日", "天"]),
        ];
        vec![
            (regex_from_strs(&["明天", "明日"]), "tomorrow"),
            (regex_from_strs(&["后天", "後天"]), "day_after_tomorrow"),
            (regex_from_strs(&["今天", "今日"]), "today"),
            (build_next_week_pattern(&["下周"], weekdays), "next_week"),
            (build_this_week_pattern(&["周"], weekdays), "this_week"),
            (build_suffix_pattern(&["天后"], r"(\d+)"), "days_from_now"),
            (
                build_suffix_pattern(&["小时后"], r"(\d+)"),
                "hours_from_now",
            ),
            (
                Regex::new(r"(\d{4})[-/.](\d{1,2})[-/.](\d{1,2})").unwrap(),
                "date_ymd",
            ),
            (Regex::new(r"(\d{1,2})[-/.](\d{1,2})").unwrap(), "date_md"),
            (Regex::new(r"(\d{1,2}):(\d{2})").unwrap(), "time_hm"),
            (build_hour_pattern("点"), "time_h"),
            (Regex::new(r"(\d{1,2})点(\d{1,2})").unwrap(), "time_h_cn_m"),
            (Regex::new(r"(\d{1,2})点(\d{1,2})分").unwrap(), "time_hm_cn"),
            (regex_from_strs(&["早上", "早晨"]), "morning"),
            (regex_from_strs(&["上午"]), "forenoon"),
            (regex_from_strs(&["下午"]), "afternoon"),
            (regex_from_strs(&["晚上", "晚间"]), "evening"),
            (regex_from_strs(&["中午"]), "noon"),
            (regex_from_strs(&["凌晨"]), "early_morning"),
        ]
    })
}

fn recurrence_patterns() -> &'static Vec<(Regex, Recurrence)> {
    static PATTERNS: OnceLock<Vec<(Regex, Recurrence)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            (regex_from_strs(&["每天", "每日"]), Recurrence::Daily),
            (regex_from_strs(&["每周", "每星期"]), Recurrence::Weekly),
            (regex_from_strs(&["每月", "每个月"]), Recurrence::Monthly),
            (regex_from_strs(&["每年"]), Recurrence::Yearly),
            (
                regex_from_strs(&["每个工作日", "工作日"]),
                Recurrence::Weekdays,
            ),
            (regex_from_strs(&["每个周末", "周末"]), Recurrence::Weekends),
        ]
    })
}

fn list_patterns() -> &'static Vec<(Regex, &'static str)> {
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let verbs = strs_to_alt(&["加到", "放到", "存到", "添加到"]);
        let suffixes = strs_to_alt(&["列表", "清单"]);
        vec![
            (
                Regex::new(&format!(r"(?:{})\s*([\S]+?)(?:{})?$", verbs, suffixes)).unwrap(),
                "dynamic",
            ),
            (Regex::new(r"列表[:：]\s*(\S+)").unwrap(), "dynamic"),
        ]
    })
}

fn reminder_time_patterns() -> &'static Vec<(Regex, &'static str)> {
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            (Regex::new(r"提前(\d+)分钟").unwrap(), "minutes"),
            (Regex::new(r"提前(\d+)小时").unwrap(), "hours"),
            (Regex::new(r"提前(\d+)天").unwrap(), "days"),
        ]
    })
}

fn title_patterns() -> &'static Vec<String> {
    static PATTERNS: OnceLock<Vec<String>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            strs_to_alt(&["明天", "明日", "后天", "今天", "今日"]),
            "下周[一二三四五六日天]|(?:下周)?周[一二三四五六日天]".to_string(),
            "\\d+天后|\\d+小时后".to_string(),
            "\\d{4}[-/.]\\d{1,2}[-/.]\\d{1,2}".to_string(),
            "\\d{1,2}[-/.]\\d{1,2}".to_string(),
            "\\d{1,2}:\\d{2}".to_string(),
            "\\d{1,2}点\\d{1,2}分|\\d{1,2}点\\d{1,2}|\\d{1,2}点".to_string(),
            strs_to_alt(&[
                "早上", "早晨", "上午", "下午", "晚上", "晚间", "中午", "凌晨",
            ]),
            strs_to_alt(&[
                "每天",
                "每日",
                "每周",
                "每星期",
                "每月",
                "每个月",
                "每年",
                "每个工作日",
                "工作日",
                "每个周末",
                "周末",
            ]),
            strs_to_alt(&[
                "紧急",
                "马上",
                "立刻",
                "ASAP",
                "asap",
                "火烧眉毛",
                "十万火急",
            ]),
            strs_to_alt(&["重要", "优先", "高优先级", "不重要", "低优先级", "有空再做"]),
            "(?:加到|放到|存到|添加到)\\s*[\\S]+?(?:列表|清单)?".to_string(),
            "列表[:：]\\s*\\S+".to_string(),
            "提前\\d+(?:分钟|小时|天)".to_string(),
            strs_to_alt(&["，", "。", "！", "？", ",", ".", "!", "?"]),
        ]
    })
}

fn high_priority_keywords() -> &'static [&'static str] {
    &["重要", "优先", "高优先级", "critical", "high priority"]
}

fn low_priority_keywords() -> &'static [&'static str] {
    &["不重要", "低优先级", "有空再做", "low priority"]
}

fn urgent_keywords() -> &'static [&'static str] {
    &[
        "紧急",
        "急",
        "马上",
        "立刻",
        "ASAP",
        "asap",
        "火烧眉毛",
        "十万火急",
        "urgent",
    ]
}

fn parse_weekday_char(ch: &str) -> Option<chrono::Weekday> {
    let weekdays: &[(&str, chrono::Weekday)] = &[
        ("一", chrono::Weekday::Mon),
        ("二", chrono::Weekday::Tue),
        ("三", chrono::Weekday::Wed),
        ("四", chrono::Weekday::Thu),
        ("五", chrono::Weekday::Fri),
        ("六", chrono::Weekday::Sat),
        ("日", chrono::Weekday::Sun),
        ("天", chrono::Weekday::Sun),
    ];
    for (c, wd) in weekdays {
        if c.contains(ch) {
            return Some(*wd);
        }
    }
    None
}

const DEFAULT_FALLBACK_TITLE: &str = "新提醒";

pub fn parse_input(input: &str, default_list: &str) -> Result<ParsedReminder> {
    let input = input.trim();
    let now = Local::now();

    let mut result = ParsedReminder {
        title: String::new(),
        due_date: None,
        start_date: None,
        priority: Priority::None,
        is_urgent: false,
        recurrence: Recurrence::None,
        location: None,
        reminder_minutes: vec![0],
        tags: Vec::new(),
        list: default_list.to_string(),
    };

    // 提取标签 #tag
    let tag_pattern = Regex::new(r"#(\w+)").unwrap();
    for cap in tag_pattern.captures_iter(input) {
        if let Some(tag) = cap.get(1) {
            result.tags.push(tag.as_str().to_string());
        }
    }

    let text = tag_pattern.replace_all(input, "").to_string();

    // 解析目标列表
    for (pattern, kind) in list_patterns().iter() {
        if let Some(caps) = pattern.captures(&text) {
            if *kind == "dynamic"
                && let Some(m) = caps.get(1)
            {
                result.list = m.as_str().trim().to_string();
            }
            break;
        }
    }

    // 解析重复模式
    for (pattern, recurrence) in recurrence_patterns().iter() {
        if pattern.is_match(&text) {
            result.recurrence = recurrence.clone();
            break;
        }
    }

    // 解析优先级
    let text_lower = text.to_lowercase();
    for keyword in high_priority_keywords() {
        if text.contains(*keyword) || text_lower.contains(&keyword.to_lowercase()) {
            result.priority = Priority::High;
            break;
        }
    }
    for keyword in low_priority_keywords() {
        if text.contains(*keyword) || text_lower.contains(&keyword.to_lowercase()) {
            result.priority = Priority::Low;
            break;
        }
    }

    // 解析紧急程度
    for keyword in urgent_keywords() {
        if text.contains(*keyword) || text_lower.contains(&keyword.to_lowercase()) {
            result.is_urgent = true;
            if result.priority != Priority::High {
                result.priority = Priority::High;
            }
            break;
        }
    }

    // 解析提醒时间
    for (pattern, kind) in reminder_time_patterns().iter() {
        if let Some(caps) = pattern.captures(&text)
            && let Some(m) = caps.get(1)
        {
            let value: i32 = m.as_str().parse().unwrap_or(0);
            let minutes = match *kind {
                "hours" => value * 60,
                "days" => value * 24 * 60,
                _ => value,
            };
            result.reminder_minutes = vec![minutes];
            break;
        }
    }

    // 解析日期时间
    result.due_date = parse_datetime(&text, now)?;

    // 提取标题
    result.title = extract_title(&text);

    Ok(result)
}

/// 解析日期时间
fn parse_datetime(text: &str, now: DateTime<Local>) -> Result<Option<DateTime<Local>>> {
    let mut day_offset: Option<i32> = None;
    let mut explicit_date: Option<DateTime<Local>> = None;
    let mut time: Option<NaiveTime> = None;
    let mut time_period: Option<&str> = None;

    for (pattern, pattern_type) in time_patterns().iter() {
        if let Some(caps) = pattern.captures(text) {
            match *pattern_type {
                "tomorrow" => {
                    day_offset = Some(1);
                }
                "day_after_tomorrow" => {
                    day_offset = Some(2);
                }
                "today" => {
                    day_offset = Some(0);
                }
                "this_week" => {
                    if let Some(day) = caps.get(1) {
                        let weekday = parse_weekday(day.as_str());
                        let current_weekday = now.weekday().num_days_from_monday() as i32;
                        let target_weekday = weekday.num_days_from_monday() as i32;
                        let mut offset = target_weekday - current_weekday;
                        if offset <= 0 {
                            offset += 7;
                        }
                        day_offset = Some(offset);
                    }
                }
                "next_week" => {
                    if let Some(day) = caps.get(1) {
                        let weekday = parse_weekday(day.as_str());
                        let current_weekday = now.weekday().num_days_from_monday() as i32;
                        let target_weekday = weekday.num_days_from_monday() as i32;
                        let mut offset = target_weekday - current_weekday;
                        if offset <= 0 {
                            offset += 7;
                        }
                        day_offset = Some(offset + 7);
                    }
                }
                "days_from_now" => {
                    if let Some(days) = caps.get(1) {
                        day_offset = Some(days.as_str().parse().unwrap_or(0));
                    }
                }
                "hours_from_now" => {
                    if let Some(hours) = caps.get(1) {
                        let hours: i32 = hours.as_str().parse().unwrap_or(0);
                        explicit_date = Some(now + Duration::hours(hours as i64));
                    }
                }
                "date_ymd" => {
                    if let (Some(year), Some(month), Some(day)) =
                        (caps.get(1), caps.get(2), caps.get(3))
                    {
                        let year: i32 = year.as_str().parse().unwrap_or(now.year());
                        let month: u32 = month.as_str().parse().unwrap_or(1);
                        let day: u32 = day.as_str().parse().unwrap_or(1);
                        if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
                            explicit_date = Some(
                                Local
                                    .from_local_datetime(
                                        &d.and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
                                    )
                                    .unwrap(),
                            );
                        }
                    }
                }
                "date_md" => {
                    if let (Some(month), Some(day)) = (caps.get(1), caps.get(2)) {
                        let year = now.year();
                        let month: u32 = month.as_str().parse().unwrap_or(1);
                        let day: u32 = day.as_str().parse().unwrap_or(1);
                        if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
                            explicit_date = Some(
                                Local
                                    .from_local_datetime(
                                        &d.and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
                                    )
                                    .unwrap(),
                            );
                        }
                    }
                }
                "time_hm" => {
                    if let (Some(hour), Some(minute)) = (caps.get(1), caps.get(2)) {
                        let hour: u32 = hour.as_str().parse().unwrap_or(9);
                        let minute: u32 = minute.as_str().parse().unwrap_or(0);
                        time = NaiveTime::from_hms_opt(hour, minute, 0);
                    }
                }
                "time_hm_cn" => {
                    if let (Some(hour), Some(minute)) = (caps.get(1), caps.get(2)) {
                        let hour: u32 = hour.as_str().parse().unwrap_or(9);
                        let minute: u32 = minute.as_str().parse().unwrap_or(0);
                        time = NaiveTime::from_hms_opt(hour, minute, 0);
                    }
                }
                "time_h_cn_m" => {
                    if let (Some(hour), Some(minute)) = (caps.get(1), caps.get(2)) {
                        let hour: u32 = hour.as_str().parse().unwrap_or(9);
                        let minute: u32 = minute.as_str().parse().unwrap_or(0);
                        time = NaiveTime::from_hms_opt(hour, minute, 0);
                    }
                }
                "time_h" => {
                    if let Some(hour) = caps.get(1) {
                        let mut hour: u32 = hour.as_str().parse().unwrap_or(9);
                        if (time_period == Some("afternoon") || time_period == Some("evening"))
                            && hour < 12
                        {
                            hour += 12;
                        }
                        time = NaiveTime::from_hms_opt(hour, 0, 0);
                    }
                }
                "morning" => time_period = Some("morning"),
                "forenoon" => time_period = Some("forenoon"),
                "afternoon" => time_period = Some("afternoon"),
                "evening" => time_period = Some("evening"),
                "noon" => time_period = Some("noon"),
                "early_morning" => time_period = Some("early_morning"),
                _ => {}
            }
        }
    }

    // 如果只有时间段没有具体时间，设置默认时间
    if time.is_none() {
        time = match time_period {
            Some("morning") => NaiveTime::from_hms_opt(8, 0, 0),
            Some("forenoon") => NaiveTime::from_hms_opt(10, 0, 0),
            Some("afternoon") => NaiveTime::from_hms_opt(14, 0, 0),
            Some("evening") => NaiveTime::from_hms_opt(19, 0, 0),
            Some("noon") => NaiveTime::from_hms_opt(12, 0, 0),
            Some("early_morning") => NaiveTime::from_hms_opt(5, 0, 0),
            _ => None,
        };
    } else if let (Some(t), Some(period)) = (time, time_period) {
        let hour = t.hour();
        let corrected_hour = match period {
            "afternoon" | "evening" if hour < 12 => hour + 12,
            _ => hour,
        };
        time = NaiveTime::from_hms_opt(corrected_hour, t.minute(), 0);
    }

    // 没有任何时间信息，返回 None（不强制设置今天）
    if explicit_date.is_none() && day_offset.is_none() && time.is_none() {
        return Ok(None);
    }

    // 构建基准日期
    let base_date = if let Some(d) = explicit_date {
        d
    } else {
        let offset = day_offset.unwrap_or(0);
        let base = now.date_naive().and_hms_opt(9, 0, 0).unwrap();
        Local.from_local_datetime(&base).unwrap() + Duration::days(offset as i64)
    };

    // 合并日期和时间
    let result = if let Some(t) = time {
        Local
            .from_local_datetime(&base_date.naive_local().date().and_time(t))
            .unwrap()
    } else {
        base_date
    };

    Ok(Some(result))
}

/// 解析中文星期
fn parse_weekday(day: &str) -> chrono::Weekday {
    if let Some(weekday) = parse_weekday_char(day) {
        return weekday;
    }
    // Fallback for single-char locales like zh
    match day {
        "一" => chrono::Weekday::Mon,
        "二" => chrono::Weekday::Tue,
        "三" => chrono::Weekday::Wed,
        "四" => chrono::Weekday::Thu,
        "五" => chrono::Weekday::Fri,
        "六" => chrono::Weekday::Sat,
        "日" | "天" => chrono::Weekday::Sun,
        _ => chrono::Weekday::Mon,
    }
}

/// 提取标题（移除时间、优先级等关键词后的剩余文本）
fn extract_title(text: &str) -> String {
    let mut title = text.to_string();

    for pattern in title_patterns().iter() {
        if let Ok(re) = Regex::new(pattern) {
            title = re.replace_all(&title, "").to_string();
        }
    }

    let title = title.trim().to_string();

    if title.is_empty() {
        DEFAULT_FALLBACK_TITLE.to_string()
    } else {
        title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_reminder() {
        let result = parse_input("明天下午3点开会", "reminders").unwrap();
        assert!(result.due_date.is_some());
        let due = result.due_date.unwrap();
        assert_eq!(due.hour(), 15);
    }

    #[test]
    fn test_parse_urgent_reminder() {
        let result = parse_input("紧急今天下午5点前提交报告", "reminders").unwrap();
        assert!(result.is_urgent);
    }

    #[test]
    fn test_parse_title_reminder() {
        let result = parse_input("下午19点30分提交报告", "reminders").unwrap();
        assert_eq!(result.title, "提交报告");
        assert_eq!(result.due_date.unwrap().hour(), 19);
        assert_eq!(result.due_date.unwrap().minute(), 30);
        let result = parse_input("下午19点30提交报告", "reminders").unwrap();
        assert_eq!(result.title, "提交报告");
        assert_eq!(result.due_date.unwrap().hour(), 19);
        assert_eq!(result.due_date.unwrap().minute(), 30);
        let result = parse_input("下午19点提交报告", "reminders").unwrap();
        assert_eq!(result.title, "提交报告");
        assert_eq!(result.due_date.unwrap().hour(), 19);
    }

    #[test]
    fn test_parse_recurring_reminder() {
        let result = parse_input("每天早上8点吃药", "reminders").unwrap();
        assert!(matches!(result.recurrence, Recurrence::Daily));
    }

    #[test]
    fn test_parse_with_tags() {
        let result = parse_input("明天 #工作 提交项目报告", "reminders").unwrap();
        assert!(result.tags.contains(&"工作".to_string()));
    }

    #[test]
    fn test_no_time_returns_none() {
        let result = parse_input("买牛奶", "reminders").unwrap();
        assert!(result.due_date.is_none());
    }
}
