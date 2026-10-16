use crate::tui::app::{App, Mode};
use crate::tui::icons::{Icon, IconStyle};
use crate::tui::status_cn;
use crate::tui::ui;
use chrono::{Local, TimeZone, Utc};
use horae_core::model::event;
use horae_core::repo::tasks::describe_event;
use horae_core::time;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

fn format_compact_time(ms: i64) -> String {
    match Utc.timestamp_millis_opt(ms).single() {
        Some(dt) => dt.with_timezone(&Local).format("%m-%d %H:%M").to_string(),
        None => "-".to_string(),
    }
}

impl<'a> App<'a> {
    /// 详情面板内容行。
    pub(crate) fn detail_lines(
        &self,
        d: &crate::tui::app::DetailData,
        width: u16,
        height: u16,
    ) -> Vec<Line<'static>> {
        let mut lines: Vec<Line> = vec![];

        // 标题
        lines.push(Line::from(vec![
            Span::styled(
                tr!(self.lang, "标题: ", "Title: "),
                Style::default()
                    .fg(self.theme.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                d.task.title.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));

        // 状态 / 归档（归档任务只展示原因与时间，不再显示已完成/逾期）
        if let Some(reason) = &d.task.archive_reason {
            let reason_cn = match reason.as_str() {
                "completed" => tr!(self.lang, "[完成]", "[Done]"),
                "deleted" => tr!(self.lang, "[删除]", "[Deleted]"),
                _ => tr!(self.lang, "[归档]", "[Archived]"),
            };
            lines.push(Line::from(vec![
                Span::styled(
                    tr!(self.lang, "归档: ", "Archived: "),
                    Style::default().fg(self.theme.text_dim),
                ),
                Span::styled(
                    format!("{} {}", reason_cn, time::format_local(d.task.archived_at)),
                    Style::default().fg(self.theme.text_dim),
                ),
            ]));
        } else {
            let st_color = ui::status_color(&d.task.status, self.theme.is_dark);
            lines.push(Line::from(vec![
                Span::styled(
                    tr!(self.lang, "状态: ", "Status: "),
                    Style::default().fg(self.theme.text_dim),
                ),
                Span::styled(
                    status_cn(self.lang, d.task.status),
                    Style::default().fg(st_color).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        // 计划时间：具体时间显示排程起点；循环任务显示最近一次计划执行日期
        let has_schedule = d.task.scheduled_start_at.is_some()
            || d.task.scheduled_end_at.is_some()
            || (d.task.rrule.is_some()
                && (d.task.scheduled_start_at.is_some() || d.task.due_at.is_some()));

        if has_schedule {
            let planned = if d.task.rrule.is_some() {
                // 循环任务：错过 slot 取最近一次已错过（逾期），否则下一次执行
                match horae_core::schedule::effective_due(&d.task) {
                    Some(ms) => time::format_local(Some(ms)),
                    None => "-".to_string(),
                }
            } else {
                match d.task.scheduled_end_at {
                    Some(_) => format!(
                        "{} -> {}",
                        time::format_local(d.task.scheduled_start_at),
                        time::format_local(d.task.scheduled_end_at)
                    ),
                    None => time::format_local(d.task.scheduled_start_at),
                }
            };
            lines.push(Line::from(vec![
                Span::styled(
                    tr!(self.lang, "计划: ", "Planned: "),
                    Style::default().fg(self.theme.text_dim),
                ),
                Span::raw(planned),
            ]));
        }

        // 截止时间：仅在明确指定了 due_at 时展示，避免一般录入任务被误导
        if let Some(due) = d.task.due_at {
            lines.push(Line::from(vec![
                Span::styled(
                    tr!(self.lang, "截止: ", "Due: "),
                    Style::default().fg(self.theme.text_dim),
                ),
                Span::raw(time::format_local(Some(due))),
            ]));
        } else if !has_schedule {
            lines.push(Line::from(vec![
                Span::styled(
                    tr!(self.lang, "时间: ", "Time: "),
                    Style::default().fg(self.theme.text_dim),
                ),
                Span::raw("-"),
            ]));
        }

        // 循环规则
        if let Some(rr) = &d.task.rrule {
            let shown_rr = if self.lang.is_zh() {
                rr.replace("FREQ=DAILY", "每天")
                    .replace("FREQ=WEEKLY", "每周")
                    .replace("FREQ=MONTHLY", "每月")
                    .replace("INTERVAL=", "间隔=")
                    .replace("COUNT=", "次数=")
                    .replace("UNTIL=", "直到=")
            } else {
                rr.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    tr!(self.lang, "循环: ", "Rrule: "),
                    Style::default().fg(self.theme.text_dim),
                ),
                Span::raw(shown_rr),
            ]));
        }

        // 优先级（独立字段，不再作为标签）
        if let Some(ref p) = d.task.priority {
            if let Some((zh, en)) = ui::priority_label(p) {
                let c = ui::priority_color(p).unwrap_or(self.theme.hl_fg);
                lines.push(Line::from(vec![
                    Span::styled(
                        tr!(self.lang, "优先级: ", "Priority: "),
                        Style::default().fg(self.theme.text_dim),
                    ),
                    Span::styled(format!("!{}", self.lang.tr(zh, en)), Style::default().fg(c)),
                ]));
            }
        }

        // 标签
        if !d.tags.is_empty() {
            let mut tag_spans = vec![Span::styled(
                tr!(self.lang, "标签: ", "Tags: "),
                Style::default().fg(self.theme.text_dim),
            )];
            for (i, tg) in d.tags.iter().enumerate() {
                let c = ui::priority_color(&tg.name).unwrap_or(self.theme.hl_fg);
                tag_spans.push(Span::styled(
                    format!("@{}", tg.name),
                    Style::default().fg(c),
                ));
                if i < d.tags.len() - 1 {
                    tag_spans.push(Span::raw(" "));
                }
            }
            lines.push(Line::from(tag_spans));
        }

        // 委派
        if let Some(del) = &d.task.delegated_to {
            lines.push(Line::from(vec![
                Span::styled(
                    tr!(self.lang, "委派: ", "Delegated: "),
                    Style::default().fg(self.theme.text_dim),
                ),
                Span::raw(del.clone()),
            ]));
        }

        // 检查单
        if !d.task.checklist.is_empty() {
            lines.push(Line::from(Span::styled(
                tr!(self.lang, "检查单:", "Checklist:"),
                Style::default().fg(self.theme.text_dim),
            )));
            for (i, item) in d.task.checklist.iter().enumerate() {
                let is_cursor = self.checklist_cursor == Some(i);
                let check = if item.done { "[x]" } else { "[ ]" };
                let c = if item.done {
                    self.theme.text_success
                } else {
                    self.theme.text_dim
                };
                let prefix = if is_cursor { "▶ " } else { "  " };
                lines.push(Line::from(Span::styled(
                    format!("{}{} {}", prefix, check, item.title),
                    Style::default().fg(c).add_modifier(if is_cursor {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                )));
            }
            // 全勾选提示（不自动改任务状态）：引导用户用 x 标记完成。
            if d.task.checklist.iter().all(|i| i.done) {
                lines.push(Line::from(Span::styled(
                    tr!(
                        self.lang,
                        "✓ 所有步骤已完成 — 按 x 标记任务完成",
                        "✓ all steps done — press x to complete the task"
                    ),
                    Style::default().fg(self.theme.text_success),
                )));
            }
        }

        // 番茄钟计数
        let pomo_count = d
            .events
            .iter()
            .filter(|e| e.event_type == event::EV_POMODORO)
            .count();
        if pomo_count > 0 {
            let tomatoes = format!("{} ", self.icon(Icon::Tomato)).repeat(pomo_count);
            lines.push(Line::from(vec![
                Span::styled(
                    tr!(self.lang, "专注: ", "Focus: "),
                    Style::default().fg(self.theme.text_dim),
                ),
                Span::styled(
                    format!("{} ({})", tomatoes, pomo_count),
                    Style::default().fg(self.theme.text_urgent),
                ),
            ]));
        }

        // 分隔线
        lines.push(Line::from("─".repeat((width.saturating_sub(4)) as usize)));

        // 备注
        if d.task.notes.trim().is_empty() {
            lines.push(Line::from(Span::styled(
                tr!(self.lang, "备注: -", "Notes: -"),
                Style::default().fg(self.theme.text_dim),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                tr!(self.lang, "备注:", "Notes:"),
                Style::default().add_modifier(Modifier::BOLD),
            )));
            for ln in d.task.notes.split('\n') {
                lines.push(Line::from(format!("  {}", ln)));
            }
        }

        lines.push(Line::from(""));
        let git_icon = self.icon(Icon::Timeline);
        let header_prefix = if git_icon.is_empty() {
            "".to_string()
        } else {
            format!("{} ", git_icon)
        };
        lines.push(Line::from(vec![
            Span::styled(header_prefix, Style::default().fg(self.theme.accent)),
            Span::styled(
                tr!(self.lang, "时间线 (Git Log)", "Timeline (Git Log)"),
                Style::default()
                    .fg(self.theme.fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        if d.events.is_empty() {
            lines.push(Line::from(Span::styled(
                tr!(self.lang, "  (暂无历史记录)", "  (no history recorded)"),
                Style::default().fg(self.theme.text_dim),
            )));
        } else {
            let is_ascii = self.icon_style == IconStyle::Ascii;
            let node_head = if is_ascii { "*" } else { "●" };
            let node_other = if is_ascii { "o" } else { "○" };
            let rail = if is_ascii { "|" } else { "│" };
            let collapsed_rail = if is_ascii { ":" } else { "┊" };

            // Available vertical space in detail pane
            let used_so_far = lines.len();
            let inner_height = (height.saturating_sub(2)) as usize;
            let available = inner_height.saturating_sub(used_so_far);

            // d.events is chronological: index 0 is root, last is HEAD.
            let total = d.events.len();

            // When total > 6 and space is limited: show newest 3, collapsed summary, and 1 root
            let (display_groups, collapsed_count) = if total > 6 && available < total * 2 {
                let newest = d.events.iter().rev().take(3).cloned().collect::<Vec<_>>();
                let root = d.events.first().cloned().unwrap();
                let collapsed = total - 4;
                (vec![(newest, true), (vec![root], false)], collapsed)
            } else {
                let all_rev = d.events.iter().rev().cloned().collect::<Vec<_>>();
                (vec![(all_rev, true)], 0)
            };

            let total_displayed = if collapsed_count > 0 { 4 } else { total };
            let show_rail_lines = available > total_displayed * 2;

            for (group_idx, (group_events, is_first_group)) in display_groups.iter().enumerate() {
                for (item_idx, event) in group_events.iter().enumerate() {
                    let is_head = *is_first_group && item_idx == 0;
                    let is_root = (!is_first_group && item_idx == group_events.len() - 1)
                        || (*is_first_group
                            && collapsed_count == 0
                            && item_idx == group_events.len() - 1);

                    let entry = describe_event(event, self.lang);
                    let time_str = format_compact_time(entry.at);

                    let (node_glyph, node_color) = if is_head {
                        let color = if entry.event_type == event::EV_COMPLETED
                            || entry.event_type == event::EV_HABIT_COMPLETED
                        {
                            self.theme.text_success
                        } else {
                            self.theme.accent
                        };
                        (node_head, color)
                    } else if entry.event_type == event::EV_COMPLETED
                        || entry.event_type == event::EV_HABIT_COMPLETED
                    {
                        (node_other, self.theme.text_success)
                    } else if entry.event_type == event::EV_POMODORO
                        || entry.event_type == event::EV_POMODORO_STARTED
                    {
                        (node_other, self.theme.rrule_fg)
                    } else {
                        (node_other, self.theme.text_dim)
                    };

                    let (badge_color, badge_bold) = match entry.event_type.as_str() {
                        event::EV_COMPLETED | event::EV_HABIT_COMPLETED => {
                            (self.theme.text_success, true)
                        }
                        event::EV_POMODORO | event::EV_POMODORO_STARTED => {
                            (self.theme.rrule_fg, true)
                        }
                        event::EV_POMODORO_ABORTED | event::EV_ARCHIVED => {
                            (self.theme.text_urgent, false)
                        }
                        event::EV_STATUS_CHANGED | event::EV_SCHEDULED => {
                            (self.theme.accent, false)
                        }
                        event::EV_CHECKLIST => (self.theme.hl_fg, false),
                        _ => (self.theme.text_dim, false),
                    };

                    let mut badge_style = Style::default().fg(badge_color);
                    if badge_bold {
                        badge_style = badge_style.add_modifier(Modifier::BOLD);
                    }

                    let ref_badge = if is_head && is_root {
                        Some((
                            "(HEAD, root)",
                            Style::default()
                                .fg(self.theme.accent)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if is_head {
                        Some((
                            "(HEAD)",
                            Style::default()
                                .fg(self.theme.accent)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if is_root {
                        Some(("(root)", Style::default().fg(self.theme.text_dim)))
                    } else {
                        None
                    };

                    let mut line_spans = vec![
                        Span::styled(
                            format!("  {} ", node_glyph),
                            Style::default().fg(node_color),
                        ),
                        Span::styled(
                            format!("{} ", time_str),
                            Style::default().fg(self.theme.text_dim),
                        ),
                        Span::styled(format!("[{}] ", entry.badge), badge_style),
                    ];

                    if let Some((tag_str, tag_style)) = ref_badge {
                        line_spans.push(Span::styled(format!("{} ", tag_str), tag_style));
                    }

                    line_spans.push(Span::styled(
                        entry.description,
                        Style::default().fg(self.theme.fg),
                    ));

                    lines.push(Line::from(line_spans));

                    let is_last_item_overall =
                        group_idx == display_groups.len() - 1 && item_idx == group_events.len() - 1;
                    if show_rail_lines && !is_last_item_overall {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", rail),
                            Style::default().fg(self.theme.border_inactive),
                        )));
                    }
                }

                if group_idx == 0 && collapsed_count > 0 {
                    if show_rail_lines {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", rail),
                            Style::default().fg(self.theme.border_inactive),
                        )));
                    }
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {} ", collapsed_rail),
                            Style::default().fg(self.theme.border_inactive),
                        ),
                        Span::styled(
                            tr!(
                                self.lang,
                                "... (折叠 {} 项更早记录)",
                                "... (collapsed {} earlier events)",
                                collapsed_count
                            ),
                            Style::default().fg(self.theme.text_dim),
                        ),
                    ]));
                    if show_rail_lines {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", rail),
                            Style::default().fg(self.theme.border_inactive),
                        )));
                    }
                }
            }
        }

        // 检查单管理模式（ChecklistFocus）的操作提示，吸引注意并告知按键。
        if self.mode == Mode::ChecklistFocus {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                tr!(
                    self.lang,
                    "▶ 检查单管理：j/k 移动 · Space 勾选 · d 删除 · J/K 排序 · e 改名 · Tab 退出",
                    "▶ managing: j/k move · Space tick · d delete · J/K reorder · e rename · Tab exit"
                ),
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            )));
        }

        lines
    }
}
