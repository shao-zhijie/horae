//! Timeline semantic decoding for task events.
//!
//! Translates raw `TaskEvent` records into human-readable, Git-commit-like
//! representations with localized badges and descriptions.

use crate::i18n::Lang;
use crate::model::event;
use crate::model::event::TaskEvent;
use crate::model::task::Status;
use crate::tr;

/// A formatted, human-readable timeline entry for Git-like representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineEntry {
    pub id: i64,
    pub at: i64,
    pub event_type: String,
    pub badge: String,
    pub description: String,
}

/// Describe a `TaskEvent` into a `TimelineEntry` for the given language.
pub fn describe_event(event: &TaskEvent, lang: Lang) -> TimelineEntry {
    let (badge, desc) = match event.event_type.as_str() {
        event::EV_CAPTURED => {
            let b = tr!(lang, "创建", "Init");
            let d = match event.to_status.as_deref() {
                Some("inbox") | None => {
                    tr!(lang, "捕获入收件箱", "Captured into inbox").to_string()
                }
                Some(st) => {
                    let st_label = parse_status_label(st, lang);
                    tr!(lang, "捕获并进入 {}", "Captured into {}", st_label)
                }
            };
            (b.to_string(), d)
        }
        event::EV_CLARIFIED => {
            let b = tr!(lang, "理清", "Clarify");
            let d = match event.to_status.as_deref() {
                Some(to) => {
                    let to_lbl = parse_status_label(to, lang);
                    let inbox_lbl = Status::Inbox.label(lang);
                    format!("{} → {}", inbox_lbl, to_lbl)
                }
                None => tr!(lang, "理清并移出收件箱", "Clarified out of inbox").to_string(),
            };
            (b.to_string(), d)
        }
        event::EV_STATUS_CHANGED => {
            let b = tr!(lang, "流转", "Status");
            let from_lbl = event
                .from_status
                .as_deref()
                .map(|s| parse_status_label(s, lang))
                .unwrap_or_else(|| "-".to_string());
            let to_lbl = event
                .to_status
                .as_deref()
                .map(|s| parse_status_label(s, lang))
                .unwrap_or_else(|| "-".to_string());
            (b.to_string(), format!("{} → {}", from_lbl, to_lbl))
        }
        event::EV_COMPLETED => {
            let b = tr!(lang, "完成", "Done");
            let d = match event.from_status.as_deref() {
                Some(from) => {
                    let from_lbl = parse_status_label(from, lang);
                    let done_lbl = Status::Done.label(lang);
                    format!("{} → {}", from_lbl, done_lbl)
                }
                None => tr!(lang, "标记为已完成", "Marked as completed").to_string(),
            };
            (b.to_string(), d)
        }
        event::EV_HABIT_COMPLETED => {
            let b = tr!(lang, "习惯", "Habit");
            let d = tr!(
                lang,
                "习惯打卡完成，已排程至下个周期",
                "Habit completed, rescheduled to next cycle"
            )
            .to_string();
            (b.to_string(), d)
        }
        event::EV_SCHEDULED => {
            let b = tr!(lang, "排期", "Sched");
            let d = if let Some(meta) = event.meta.as_deref() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(meta) {
                    if let Some(rrule) = v.get("rrule").and_then(|r| r.as_str()) {
                        tr!(lang, "循环规则: {}", "Rrule: {}", rrule)
                    } else {
                        tr!(lang, "排期已更新", "Schedule updated").to_string()
                    }
                } else {
                    tr!(lang, "排期已更新", "Schedule updated").to_string()
                }
            } else {
                tr!(lang, "排期已更新", "Schedule updated").to_string()
            };
            (b.to_string(), d)
        }
        event::EV_DUE => {
            let b = tr!(lang, "截止", "Due");
            let d = tr!(lang, "截止日期变更", "Deadline updated").to_string();
            (b.to_string(), d)
        }
        event::EV_PRIORITY => {
            let b = tr!(lang, "优级", "Prio");
            let d = tr!(lang, "优先级变更", "Priority updated").to_string();
            (b.to_string(), d)
        }
        event::EV_TAG_ADDED => {
            let b = tr!(lang, "标签", "+Tag");
            let tag = parse_json_field(event.meta.as_deref(), "name")
                .unwrap_or_else(|| event.meta.clone().unwrap_or_default());
            let d = format!("+@{}", tag);
            (b.to_string(), d)
        }
        event::EV_TAG_REMOVED => {
            let b = tr!(lang, "标签", "-Tag");
            let tag = parse_json_field(event.meta.as_deref(), "name")
                .unwrap_or_else(|| event.meta.clone().unwrap_or_default());
            let d = format!("-@{}", tag);
            (b.to_string(), d)
        }
        event::EV_POMODORO => {
            let b = tr!(lang, "专注", "Focus");
            let duration = format_duration_secs(event.meta.as_deref());
            let d = tr!(lang, "专注完成 ({})", "Focus session done ({})", duration);
            (b.to_string(), d)
        }
        event::EV_POMODORO_STARTED => {
            let b = tr!(lang, "专注", "Focus");
            let d = tr!(lang, "开始专注", "Started focus session").to_string();
            (b.to_string(), d)
        }
        event::EV_POMODORO_ABORTED => {
            let b = tr!(lang, "中断", "Abort");
            let duration = format_duration_secs(event.meta.as_deref());
            let d = tr!(lang, "专注中断 ({})", "Focus aborted ({})", duration);
            (b.to_string(), d)
        }
        event::EV_CHECKLIST => {
            let b = tr!(lang, "清单", "Check");
            let meta = event.meta.as_deref().unwrap_or("");
            let d = if let Some(title) = meta.strip_prefix("toggle:") {
                format!("✓ {}", title)
            } else if let Some(title) = meta.strip_prefix("add:") {
                format!("+ {}", title)
            } else if let Some(title) = meta.strip_prefix("delete:") {
                format!("- {}", title)
            } else if let Some(title) = meta.strip_prefix("rename:") {
                tr!(lang, "重命名步骤: {}", "Renamed item: {}", title)
            } else {
                meta.to_string()
            };
            (b.to_string(), d)
        }
        event::EV_ARCHIVED => {
            let b = tr!(lang, "归档", "Arch");
            let reason = parse_json_field(event.meta.as_deref(), "reason");
            let d = match reason.as_deref() {
                Some("completed") => tr!(lang, "已完成归档", "Archived (completed)").to_string(),
                Some("deleted") => tr!(lang, "已删除归档", "Archived (deleted)").to_string(),
                _ => tr!(lang, "任务归档", "Task archived").to_string(),
            };
            (b.to_string(), d)
        }
        event::EV_RESTORED => {
            let b = tr!(lang, "恢复", "Restore");
            let d = tr!(lang, "从归档箱恢复", "Restored from archive").to_string();
            (b.to_string(), d)
        }
        event::EV_LOGGED => {
            let b = tr!(lang, "日志", "Log");
            let d = event.meta.clone().unwrap_or_default();
            (b.to_string(), d)
        }
        other => {
            let b = other.to_string();
            let d = if let (Some(f), Some(t)) =
                (event.from_status.as_deref(), event.to_status.as_deref())
            {
                format!(
                    "{} → {}",
                    parse_status_label(f, lang),
                    parse_status_label(t, lang)
                )
            } else {
                event.meta.clone().unwrap_or_default()
            };
            (b, d)
        }
    };

    TimelineEntry {
        id: event.id,
        at: event.at,
        event_type: event.event_type.clone(),
        badge,
        description: desc,
    }
}

fn parse_status_label(s: &str, lang: Lang) -> String {
    s.parse::<Status>()
        .map(|st| st.label(lang).to_string())
        .unwrap_or_else(|_| s.to_string())
}

fn parse_json_field(meta: Option<&str>, field: &str) -> Option<String> {
    meta.and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .and_then(|v| {
            v.get(field)
                .and_then(|val| val.as_str())
                .map(str::to_string)
        })
}

fn format_duration_secs(raw: Option<&str>) -> String {
    if let Some(s) = raw {
        if let Ok(secs) = s.parse::<i64>() {
            let mins = (secs + 30) / 60;
            return format!("{}m", mins.max(1));
        }
    }
    "25m".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_captured() {
        let e = TaskEvent {
            id: 1,
            task_id: "t1".into(),
            event_type: event::EV_CAPTURED.into(),
            from_status: None,
            to_status: Some("inbox".into()),
            at: 1000,
            meta: None,
        };
        let zh = describe_event(&e, Lang::Zh);
        assert_eq!(zh.badge, "创建");
        assert_eq!(zh.description, "捕获入收件箱");

        let en = describe_event(&e, Lang::En);
        assert_eq!(en.badge, "Init");
        assert_eq!(en.description, "Captured into inbox");
    }

    #[test]
    fn test_describe_status_changed() {
        let e = TaskEvent {
            id: 2,
            task_id: "t1".into(),
            event_type: event::EV_STATUS_CHANGED.into(),
            from_status: Some("inbox".into()),
            to_status: Some("next".into()),
            at: 2000,
            meta: None,
        };
        let zh = describe_event(&e, Lang::Zh);
        assert_eq!(zh.badge, "流转");
        assert_eq!(zh.description, "收件箱 → 下一步");

        let en = describe_event(&e, Lang::En);
        assert_eq!(en.badge, "Status");
        assert_eq!(en.description, "Inbox → Next");
    }

    #[test]
    fn test_describe_pomodoro() {
        let e = TaskEvent {
            id: 3,
            task_id: "t1".into(),
            event_type: event::EV_POMODORO.into(),
            from_status: None,
            to_status: None,
            at: 3000,
            meta: Some("1500".into()),
        };
        let zh = describe_event(&e, Lang::Zh);
        assert_eq!(zh.badge, "专注");
        assert_eq!(zh.description, "专注完成 (25m)");
    }

    #[test]
    fn test_describe_checklist() {
        let e = TaskEvent {
            id: 4,
            task_id: "t1".into(),
            event_type: event::EV_CHECKLIST.into(),
            from_status: None,
            to_status: None,
            at: 4000,
            meta: Some("toggle:写单测".into()),
        };
        let zh = describe_event(&e, Lang::Zh);
        assert_eq!(zh.badge, "清单");
        assert_eq!(zh.description, "✓ 写单测");
    }

    #[test]
    fn test_describe_archived() {
        let e = TaskEvent {
            id: 5,
            task_id: "t1".into(),
            event_type: event::EV_ARCHIVED.into(),
            from_status: None,
            to_status: None,
            at: 5000,
            meta: Some("{\"reason\":\"completed\"}".into()),
        };
        let zh = describe_event(&e, Lang::Zh);
        assert_eq!(zh.badge, "归档");
        assert_eq!(zh.description, "已完成归档");
    }
}
