use crate::cors::{Location, Priority, Recurrence};
use crate::reminder::Reminder;
use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct ParsedReminder {
    pub title: String,
    pub due_date: Option<DateTime<Local>>,
    pub start_date: Option<DateTime<Local>>,
    pub priority: Priority,
    pub is_urgent: bool,
    pub recurrence: Recurrence,
    pub location: Option<Location>,
    pub reminder_minutes: Vec<i32>,
    pub tags: Vec<String>,
    pub list: String,
}

impl ParsedReminder {
    pub fn into_reminder(self) -> Reminder {
        let mut reminder = Reminder::new(self.title, self.list);

        if let Some(due_date) = self.due_date {
            reminder = reminder.with_due_date(due_date);
        }
        if let Some(start_date) = self.start_date {
            reminder.start_date = Some(start_date);
        }
        reminder = reminder.with_priority(self.priority);
        reminder = reminder.with_urgent(self.is_urgent);
        reminder = reminder.with_recurrence(self.recurrence);
        reminder.add_tag(self.tags);

        reminder.location = self.location;
        reminder.reminder_minutes = self.reminder_minutes;

        reminder
    }
}
