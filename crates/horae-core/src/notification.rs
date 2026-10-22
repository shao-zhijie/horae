use crate::i18n::Lang;
use crate::repo::{state::JsonStateStore, tasks};
use crate::time;
use crate::tr;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// 已发送提醒的持久化状态，存入 `notify_tui.json`。
/// key 格式：`"type:id:due_ms"`，通过提取 due_ms 判断是否过期。
/// Bug3 修复：持久化使 TUI 重启后不重复发送相同提醒。
#[derive(Debug, Default, Serialize, Deserialize)]
struct NotifyTuiState {
    #[serde(default)]
    notified: Vec<String>,
}

/// 过期窗口：due_ms 早于 now 超过 2 小时的 key 可以丢弃。
/// Bug2 修复：按时间精确过期，替代原来超过 1024 条暴力 clear() 的策略。
const PRUNE_MS: i64 = 2 * 3600 * 1000;

fn key_fresh(key: &str, now: i64) -> bool {
    // key 格式 "type:id:due_ms"，取最后一段为 due_ms
    key.rsplit(':')
        .next()
        .and_then(|s| s.parse::<i64>().ok())
        .map(|due_ms| due_ms + PRUNE_MS > now)
        .unwrap_or(true) // 格式不识别时保守保留
}

fn make_key(kind: &str, id: &str, due: i64) -> String {
    format!("{}:{}:{}", kind, id, due)
}

pub use crate::model::task::TaskTimingKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum NotificationEvent {
    InOneHour {
        id: String,
        title: String,
        kind: TaskTimingKind,
        due_ms: i64,
        priority: Option<String>,
        tags: Vec<String>,
    },
    InTenMins {
        id: String,
        title: String,
        kind: TaskTimingKind,
        due_ms: i64,
        priority: Option<String>,
        tags: Vec<String>,
    },
    Now {
        id: String,
        title: String,
        kind: TaskTimingKind,
        due_ms: i64,
        is_overdue: bool,
        priority: Option<String>,
        tags: Vec<String>,
    },
}

impl NotificationEvent {
    pub fn id(&self) -> &str {
        match self {
            Self::InOneHour { id, .. } | Self::InTenMins { id, .. } | Self::Now { id, .. } => id,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::InOneHour { title, .. }
            | Self::InTenMins { title, .. }
            | Self::Now { title, .. } => title,
        }
    }

    pub fn kind(&self) -> TaskTimingKind {
        match self {
            Self::InOneHour { kind, .. }
            | Self::InTenMins { kind, .. }
            | Self::Now { kind, .. } => *kind,
        }
    }

    pub fn due_ms(&self) -> i64 {
        match self {
            Self::InOneHour { due_ms, .. }
            | Self::InTenMins { due_ms, .. }
            | Self::Now { due_ms, .. } => *due_ms,
        }
    }

    /// 结构化、逻辑严密的标题，区分截止、计划开始与已逾期，避免误导。
    pub fn summary(&self, lang: Lang) -> String {
        match self {
            Self::InOneHour { kind, .. } => match kind {
                TaskTimingKind::Deadline => {
                    tr!(lang, "⏰ 任务即将在 1 小时后截止", "⏰ Task due in 1 hour").into()
                }
                TaskTimingKind::ScheduledStart => tr!(
                    lang,
                    "📅 任务即将在 1 小时后开始",
                    "📅 Task starting in 1 hour"
                )
                .into(),
                TaskTimingKind::Recurring => tr!(
                    lang,
                    "🔄 循环任务即将在 1 小时后到期",
                    "🔄 Recurring task due in 1 hour"
                )
                .into(),
            },
            Self::InTenMins { kind, .. } => match kind {
                TaskTimingKind::Deadline => tr!(
                    lang,
                    "⏰ 任务即将在 10 分钟后截止",
                    "⏰ Task due in 10 minutes"
                )
                .into(),
                TaskTimingKind::ScheduledStart => tr!(
                    lang,
                    "📅 任务即将在 10 分钟后开始",
                    "📅 Task starting in 10 minutes"
                )
                .into(),
                TaskTimingKind::Recurring => tr!(
                    lang,
                    "🔄 循环任务即将在 10 分钟后到期",
                    "🔄 Recurring task due in 10 minutes"
                )
                .into(),
            },
            Self::Now {
                kind, is_overdue, ..
            } => {
                if *is_overdue {
                    match kind {
                        TaskTimingKind::Deadline => {
                            tr!(lang, "⚠️ 任务已逾期！", "⚠️ Task is overdue!").into()
                        }
                        TaskTimingKind::ScheduledStart => tr!(
                            lang,
                            "⏳ 任务已过计划开始时间！",
                            "⏳ Task scheduled start time passed!"
                        )
                        .into(),
                        TaskTimingKind::Recurring => {
                            tr!(lang, "⚠️ 循环任务已逾期！", "⚠️ Recurring task is overdue!").into()
                        }
                    }
                } else {
                    match kind {
                        TaskTimingKind::Deadline => {
                            tr!(lang, "⏰ 任务现已到截止时间！", "⏰ Task deadline reached!").into()
                        }
                        TaskTimingKind::ScheduledStart => {
                            tr!(lang, "🚀 任务计划现在开始！", "🚀 Task starts now!").into()
                        }
                        TaskTimingKind::Recurring => tr!(
                            lang,
                            "🔄 循环任务时间已到！",
                            "🔄 Recurring task time reached!"
                        )
                        .into(),
                    }
                }
            }
        }
    }

    /// 包含任务标题、具体时间上下文、相对时间、优先级与标签的完整正文。
    pub fn body(&self, lang: Lang) -> String {
        let (title, kind, due_ms, is_overdue, priority, tags) = match self {
            Self::InOneHour {
                title,
                kind,
                due_ms,
                priority,
                tags,
                ..
            } => (title, *kind, *due_ms, false, priority, tags),
            Self::InTenMins {
                title,
                kind,
                due_ms,
                priority,
                tags,
                ..
            } => (title, *kind, *due_ms, false, priority, tags),
            Self::Now {
                title,
                kind,
                due_ms,
                is_overdue,
                priority,
                tags,
                ..
            } => (title, *kind, *due_ms, *is_overdue, priority, tags),
        };

        let formatted_time = time::format_local(Some(due_ms));
        let time_label = match kind {
            TaskTimingKind::Deadline => tr!(lang, "截止", "Due"),
            TaskTimingKind::ScheduledStart => tr!(lang, "计划开始", "Start"),
            TaskTimingKind::Recurring => tr!(lang, "周期发生", "Slot"),
        };

        let mut lines = vec![title.clone()];

        let mut meta_parts = Vec::new();
        if is_overdue {
            if let Some(rel) = time::relative_due(lang, Some(due_ms)) {
                meta_parts.push(format!("⚠️ {}: {} ({})", time_label, formatted_time, rel));
            } else {
                meta_parts.push(format!("⚠️ {}: {}", time_label, formatted_time));
            }
        } else {
            meta_parts.push(format!("⏰ {}: {}", time_label, formatted_time));
        }

        if let Some(pri) = priority {
            let pri_str = match pri.as_str() {
                "high" => tr!(lang, "🔥 高优先级", "🔥 High priority"),
                "medium" => tr!(lang, "⚡ 中优先级", "⚡ Medium priority"),
                "low" => tr!(lang, "🌱 低优先级", "🌱 Low priority"),
                _ => "",
            };
            if !pri_str.is_empty() {
                meta_parts.push(pri_str.to_string());
            }
        }

        if !tags.is_empty() {
            let tags_str = tags
                .iter()
                .map(|t| format!("#{}", t))
                .collect::<Vec<_>>()
                .join(" ");
            meta_parts.push(format!("🏷 {}", tags_str));
        }

        lines.push(meta_parts.join(" | "));
        lines.join("\n")
    }

    pub fn urgency(&self) -> &'static str {
        match self {
            Self::InOneHour { .. } => "low",
            Self::InTenMins { .. } => "normal",
            Self::Now {
                is_overdue,
                priority,
                ..
            } => {
                if *is_overdue || priority.as_deref() == Some("high") {
                    "critical"
                } else {
                    "normal"
                }
            }
        }
    }

    pub fn icon(&self) -> Option<&'static str> {
        match self {
            Self::InOneHour { .. } => None,
            Self::InTenMins { .. } => None,
            Self::Now { is_overdue, .. } => {
                if *is_overdue {
                    Some("dialog-warning")
                } else {
                    Some("alarm-clock")
                }
            }
        }
    }
}

pub struct NotificationEngine {
    last_tick_ms: i64,
    /// 已通知过的 key 集合（内存缓存，与 state_file 同步）。
    notified: std::collections::HashSet<String>,
    /// 持久化状态文件名（`notify_tui.json`）。
    state_file: &'static str,
}

impl Default for NotificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationEngine {
    /// 默认构造：使用 `notify_tui.json` 状态文件。
    pub fn new() -> Self {
        Self::with_store("notify_tui.json")
    }

    fn with_store(state_file: &'static str) -> Self {
        // Bug3 修复：从持久化文件恢复状态，避免重启后重复通知。
        let saved: NotifyTuiState = JsonStateStore::new(state_file).load().unwrap_or_default();
        let test_mode = cfg!(test) || crate::repo::state::is_test_override();
        Self {
            // Delay first background scan by 1 second so it is off the critical startup path.
            last_tick_ms: if test_mode {
                0
            } else {
                crate::time::now_ms() - 9_000
            },
            notified: saved.notified.into_iter().collect(),
            state_file,
        }
    }

    /// 重置节流计时（测试辅助）。
    pub fn reset_throttle(&mut self) {
        self.last_tick_ms = 0;
    }

    /// Evaluates due tasks and returns a list of events to be notified.
    pub fn tick(&mut self, conn: &Connection) -> Vec<NotificationEvent> {
        let now_ms = crate::time::now_ms();
        if now_ms - self.last_tick_ms < 10_000 {
            return vec![];
        }
        self.last_tick_ms = now_ms;

        // Bug2 修复：按时间过期清理，不再暴力 clear()。
        // 只丢弃 due_ms 早于 now 超过 PRUNE_MS 的 key，保留近期去重记录。
        let before_len = self.notified.len();
        self.notified.retain(|k| key_fresh(k, now_ms));
        let pruned = before_len != self.notified.len();

        let mut events = Vec::new();

        // 加载全部未归档任务，通过 effective_due() 计算有效截止时间。
        // 正确支持带 rrule 的循环任务（每日站会、每周回顾等）。
        let all = match tasks::list(
            conn,
            &tasks::ListFilter {
                status: None,
                tags: vec![],
                query: None,
                review_stale: false,
            },
        ) {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let ids: Vec<&str> = all.iter().map(|t| t.id.as_str()).collect();
        let mut tag_map = crate::repo::tags::get_tags_for_tasks(conn, &ids).unwrap_or_default();

        let mut dirty = pruned;

        for task in all {
            // 已完成/已归档的任务不需要提醒
            if task.status == crate::model::task::Status::Done || task.archived_at.is_some() {
                continue;
            }

            let Some(due) = crate::schedule::effective_due(&task) else {
                continue;
            };

            let diff_ms = due - now_ms;

            // 仅处理接下来 1 小时内、或过期未超过 PRUNE_MS 的任务。
            // 下界必须与 PRUNE_MS 对齐：更早的过期任务其去重 key 已被 prune，
            // 若仍处理会导致每个 tick 重复触发通知（key_fresh 保留条件为
            // due + PRUNE_MS > now，即 diff > -PRUNE_MS）。
            if diff_ms > 3_600_000 || diff_ms <= -PRUNE_MS {
                continue;
            }

            let tags = tag_map.remove(&task.id).unwrap_or_default();
            let kind = TaskTimingKind::from_task(&task);

            if diff_ms > 3_540_000 && diff_ms <= 3_600_000 {
                let key = make_key("1h", &task.id, due);
                if self.notified.insert(key) {
                    dirty = true;
                    events.push(NotificationEvent::InOneHour {
                        id: task.id,
                        title: task.title,
                        kind,
                        due_ms: due,
                        priority: task.priority,
                        tags,
                    });
                }
            } else if diff_ms > 540_000 && diff_ms <= 600_000 {
                let key = make_key("10m", &task.id, due);
                if self.notified.insert(key) {
                    dirty = true;
                    events.push(NotificationEvent::InTenMins {
                        id: task.id,
                        title: task.title,
                        kind,
                        due_ms: due,
                        priority: task.priority,
                        tags,
                    });
                }
            } else if diff_ms <= 0 {
                let key = make_key("now", &task.id, due);
                if self.notified.insert(key) {
                    dirty = true;
                    let is_overdue = diff_ms < -60_000;
                    events.push(NotificationEvent::Now {
                        id: task.id,
                        title: task.title,
                        kind,
                        due_ms: due,
                        is_overdue,
                        priority: task.priority,
                        tags,
                    });
                }
            }
        }

        // Bug3 修复：有变化时才落盘，保证重启后不重复通知。
        if dirty {
            let state = NotifyTuiState {
                notified: self.notified.iter().cloned().collect(),
            };
            let _ = JsonStateStore::new(self.state_file).save(&state);
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::task::{Status, Task};
    use crate::repo::tasks::CaptureInput;
    use crate::testutil::test_conn;

    fn mk_task_with_rrule(
        conn: &Connection,
        title: &str,
        rrule: &str,
        scheduled_start_at: Option<i64>,
        due_at: Option<i64>,
    ) -> Task {
        let mut t = crate::repo::tasks::create_capture(
            conn,
            &CaptureInput {
                title: title.into(),
                status: Status::Next,
                ..Default::default()
            },
        )
        .unwrap();
        conn.execute(
            "UPDATE tasks SET rrule = ?1, scheduled_start_at = ?2, due_at = ?3 WHERE id = ?4",
            rusqlite::params![rrule, scheduled_start_at, due_at, &t.id],
        )
        .unwrap();
        t.rrule = Some(rrule.to_string());
        t.scheduled_start_at = scheduled_start_at;
        t.due_at = due_at;
        t
    }

    #[test]
    fn tick_fires_for_recurring_task_due_now() {
        let (_dir, conn) = test_conn();
        let now = crate::time::now_ms();
        // 循环任务：锚点在 30 秒前（刚过期，在 -60s～0 窗口内）
        let anchor = now - 30_000;
        mk_task_with_rrule(&conn, "每日站会", "FREQ=DAILY", Some(anchor), None);

        let mut engine = NotificationEngine::new();
        // 强制 last_tick_ms 为 0，跳过节流
        let events = engine.tick(&conn);
        // 应当触发 Now 事件
        let has_now = events
            .iter()
            .any(|e| matches!(e, NotificationEvent::Now { .. }));
        assert!(has_now, "循环任务到点后应触发 Now 提醒，但未触发");
    }

    #[test]
    fn tick_does_not_fire_for_recurring_task_far_future() {
        let (_dir, conn) = test_conn();
        let now = crate::time::now_ms();
        // 循环任务：锚点在 2 小时后，超出 1 小时窗口
        let anchor = now + 2 * 3600 * 1000;
        mk_task_with_rrule(&conn, "远期任务", "FREQ=DAILY", Some(anchor), None);

        let mut engine = NotificationEngine::new();
        let events = engine.tick(&conn);
        assert!(events.is_empty(), "2 小时后的任务不应触发任何提醒");
    }

    #[test]
    fn tick_does_not_spam_for_long_overdue_task() {
        let (_dir, conn) = test_conn();
        let now = crate::time::now_ms();
        // 过期超过 PRUNE_MS(2h)：去重 key 已被 prune，不应再触发任何提醒，
        // 否则会每个 tick（~10s）重复弹出通知。
        let long_overdue = now - 3 * 3600 * 1000;
        crate::repo::tasks::create_capture(
            &conn,
            &CaptureInput {
                title: "逾期已久".into(),
                status: Status::Next,
                due_at: Some(long_overdue),
                ..Default::default()
            },
        )
        .unwrap();

        let mut engine = NotificationEngine::new();
        let events = engine.tick(&conn);
        assert!(
            events.is_empty(),
            "过期超过 PRUNE_MS 的任务不应触发提醒，实际 {:?}",
            events
        );
    }

    #[test]
    fn tick_deduplicates_recurring_task() {
        let (_dir, conn) = test_conn();
        let now = crate::time::now_ms();
        let anchor = now - 30_000;
        mk_task_with_rrule(&conn, "每日站会", "FREQ=DAILY", Some(anchor), None);

        let mut engine = NotificationEngine::new();
        let first = engine.tick(&conn);
        assert!(!first.is_empty(), "首次应触发");

        // 重置节流计时，模拟 10s 后再次 tick
        engine.last_tick_ms = 0;
        let second = engine.tick(&conn);
        assert!(second.is_empty(), "相同 occurrence 不应重复触发");
    }

    #[test]
    fn summary_and_body_distinguish_deadline_and_start() {
        let deadline_event = NotificationEvent::InOneHour {
            id: "t1".into(),
            title: "提交财务报表".into(),
            kind: TaskTimingKind::Deadline,
            due_ms: 1_700_000_000_000,
            priority: Some("high".into()),
            tags: vec!["work".into(), "finance".into()],
        };
        assert_eq!(
            deadline_event.summary(Lang::Zh),
            "⏰ 任务即将在 1 小时后截止"
        );
        let body = deadline_event.body(Lang::Zh);
        assert!(body.contains("提交财务报表"));
        assert!(body.contains("截止:"));
        assert!(body.contains("🔥 高优先级"));
        assert!(body.contains("#work #finance"));

        let start_event = NotificationEvent::InOneHour {
            id: "t2".into(),
            title: "季度规划会".into(),
            kind: TaskTimingKind::ScheduledStart,
            due_ms: 1_700_000_000_000,
            priority: None,
            tags: vec![],
        };
        assert_eq!(start_event.summary(Lang::Zh), "📅 任务即将在 1 小时后开始");
        let start_body = start_event.body(Lang::Zh);
        assert!(start_body.contains("计划开始:"));

        // 逾期任务
        let overdue_event = NotificationEvent::Now {
            id: "t3".into(),
            title: "逾期申报".into(),
            kind: TaskTimingKind::Deadline,
            due_ms: 1_700_000_000_000,
            is_overdue: true,
            priority: Some("medium".into()),
            tags: vec![],
        };
        assert_eq!(overdue_event.summary(Lang::Zh), "⚠️ 任务已逾期！");
        assert_eq!(overdue_event.urgency(), "critical");
        assert_eq!(overdue_event.icon(), Some("dialog-warning"));
        let overdue_body = overdue_event.body(Lang::Zh);
        assert!(overdue_body.contains("⚠️ 截止:"));
        assert!(overdue_body.contains("⚡ 中优先级"));

        // 到期（刚到）任务
        let now_event = NotificationEvent::Now {
            id: "t4".into(),
            title: "准时打卡".into(),
            kind: TaskTimingKind::Deadline,
            due_ms: 1_700_000_000_000,
            is_overdue: false,
            priority: None,
            tags: vec![],
        };
        assert_eq!(now_event.summary(Lang::Zh), "⏰ 任务现已到截止时间！");
    }
}
