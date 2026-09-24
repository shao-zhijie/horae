use rusqlite::Connection;

use anyhow::Result;
use horae_core::model::event;
use horae_core::model::tag::Tag;
use horae_core::model::task::Task;
use horae_core::repo::{tags, tasks};
use horae_core::time;
use serde::Serialize;

#[derive(Serialize)]
struct ShowOutput {
    task: Task,
    tags: Vec<Tag>,
    events: Vec<event::TaskEvent>,
}

pub fn run(conn: &Connection, id: &str, json: bool) -> Result<()> {
    let id = tasks::resolve_id(conn, id)?;
    let task = tasks::get(conn, &id)?;
    let tags = tags::get_task_tags(conn, &task.id)?;
    let events = tasks::events(conn, &task.id)?;

    if json {
        let out = ShowOutput { task, tags, events };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("Task {}", task.id);
    println!("  title     : {}", task.title);
    println!("  status    : {}", task.status);
    println!(
        "  notes     : {}",
        if task.notes.trim().is_empty() {
            "-".to_string()
        } else {
            task.notes.clone()
        }
    );
    if let Some(rr) = &task.rrule {
        println!("  rrule     : {}", rr);
    }
    println!(
        "  captured  : {}",
        time::format_local(Some(task.created_at))
    );
    println!("  clarified : {}", time::format_local(task.clarified_at));
    println!("  due       : {}", time::format_local(task.due_at));
    println!(
        "  scheduled : {} -> {}",
        time::format_local(task.scheduled_start_at),
        time::format_local(task.scheduled_end_at)
    );
    println!("  completed : {}", time::format_local(task.completed_at));
    println!(
        "  tags      : {}",
        tags.iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    let lang = crate::cli_i18n::detect_lang();
    println!("Timeline (Git Log):");
    if events.is_empty() {
        println!("  (no events)");
    } else {
        let total = events.len();
        for (idx, e) in events.iter().rev().enumerate() {
            let is_head = idx == 0;
            let is_root = idx == total - 1;
            let entry = horae_core::repo::tasks::describe_event(e, lang);
            let time_str = time::format_local(Some(entry.at));

            let ref_tag = if is_head && is_root {
                " \x1b[1;38;2;137;180;250m(HEAD, root)\x1b[0m"
            } else if is_head {
                " \x1b[1;38;2;137;180;250m(HEAD)\x1b[0m"
            } else if is_root {
                " \x1b[38;2;108;112;134m(root)\x1b[0m"
            } else {
                ""
            };

            let node_color = if is_head {
                "\x1b[1;38;2;137;180;250m"
            } else if entry.event_type == event::EV_COMPLETED
                || entry.event_type == event::EV_HABIT_COMPLETED
            {
                "\x1b[38;2;166;227;161m"
            } else if entry.event_type == event::EV_POMODORO
                || entry.event_type == event::EV_POMODORO_STARTED
            {
                "\x1b[38;2;250;179;135m"
            } else {
                "\x1b[38;2;108;112;134m"
            };

            let badge_color = match entry.event_type.as_str() {
                event::EV_COMPLETED | event::EV_HABIT_COMPLETED => "\x1b[1;38;2;166;227;161m",
                event::EV_POMODORO | event::EV_POMODORO_STARTED => "\x1b[1;38;2;250;179;135m",
                event::EV_POMODORO_ABORTED | event::EV_ARCHIVED => "\x1b[38;2;243;139;168m",
                event::EV_STATUS_CHANGED | event::EV_SCHEDULED => "\x1b[38;2;137;180;250m",
                _ => "\x1b[38;2;108;112;134m",
            };

            println!(
                "  {}* \x1b[0m \x1b[38;2;108;112;134m{}\x1b[0m  {}[{}]\x1b[0m{}  {}",
                node_color, time_str, badge_color, entry.badge, ref_tag, entry.description,
            );

            if !is_root {
                println!("  \x1b[38;2;69;71;90m|\x1b[0m");
            }
        }
    }
    Ok(())
}
