# horae — GTD Terminal Task Manager

<div align="center">

<p align="center">
  <a href="https://github.com/zhaohang1205/horae/actions/workflows/ci.yml"><img src="https://github.com/zhaohang1205/horae/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/zhaohang1205/horae/actions/workflows/release.yml"><img src="https://github.com/zhaohang1205/horae/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <a href="https://crates.io/crates/horae"><img src="https://img.shields.io/crates/v/horae.svg" alt="Crates.io"></a>
  <a href="https://www.npmjs.com/package/horae-cli"><img src="https://img.shields.io/npm/v/horae-cli.svg" alt="npm"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg" alt="License: GPL-3.0"></a>
</p>

**[ 🇬🇧 English ](README_en.md)** &nbsp;•&nbsp; **[ 🇨🇳 简体中文 ](README_cn.md)** &nbsp;•&nbsp; **[ 📖 Documentation Home ](README.md)**

*Rust Single Binary · Embedded SQLite · CLI 2-Second Capture · Immersive Ratatui TUI · Zero-Cloud Privacy*

</div>

---

## Table of Contents

- [1. Core Philosophy & Mental Model](#1-core-philosophy--mental-model)
- [2. Core Architecture & Features Matrix](#2-core-architecture--features-matrix)
- [3. Installation & Prerequisites](#3-installation--prerequisites)
- [4. Quick Start](#4-quick-start)
- [5. Quick-Add Syntax & Recurrence Engine Specification](#5-quick-add-syntax--recurrence-engine-specification)
- [6. CLI Command Reference](#6-cli-command-reference)
- [7. TUI User Guide & Keybindings](#7-tui-user-guide--keybindings)
- [8. In-Depth Features & Advanced Workflows](#8-in-depth-features--advanced-workflows)
  - [8.1 Focus Recommendation Engine (`horae do`)](#81-focus-recommendation-engine-horae-do)
  - [8.2 Pomodoro Timer & Waybar Integration](#82-pomodoro-timer--waybar-integration)
  - [8.3 Quotes Knowledge Base (Opt-in)](#83-quotes-knowledge-base-opt-in)
  - [8.4 Astronomical Lunar Calendar & Holiday Warnings (`horae calendar`)](#84-astronomical-lunar-calendar--holiday-warnings-horae-calendar)
  - [8.5 Data Backup & Disaster Recovery](#85-data-backup--disaster-recovery)
- [9. Mobile Ecosystem & Integrations](#9-mobile-ecosystem--integrations)
  - [9.1 Solution 1: Zero-Server Syncthing Folder Bridge](#91-solution-1-zero-server-syncthing-folder-bridge)
  - [9.2 Solution 2: Lightweight Native Push Notifications via ntfy](#92-solution-2-lightweight-native-push-notifications-via-ntfy)
  - [9.3 Solution 3: Feishu / Lark Enterprise Integration](#93-solution-3-feishu--lark-enterprise-integration)
- [10. Development & Contributing](#10-development--contributing)
- [11. Author & License](#11-author--license)

---

## 1. Core Philosophy & Mental Model

`horae` is a modern GTD (Getting Things Done) terminal productivity tool written in pure Rust, integrating a **SQLite data layer + CLI commands + Ratatui full-screen TUI** into a single standalone binary.

### Core Foundation: Time-Datafication
Unlike conventional to-do list managers that treat tasks as ephemeral scratch notes, `horae` is architected on **Time-Datafication**:
* **Data Never Lies**: Every status change, schedule modification, and habit check-in is stamped with an immutable **UTC millisecond timestamp** and appended to an append-only `task_events` timeline. Running `horae show <id>` reveals the complete lifecycle trail from inbox capture to clarification, rescheduling, and eventual completion.
* **Atomic Mutation & Audit Seam**: All mutations go through `repo::mutate`, ensuring that domain updates and audit event logging strictly share a single atomic transaction and an identical `now` timestamp.
* **100% Local & Privacy-First**: Data lives exclusively inside a single local SQLite database (`~/.config/horae/horae.db`). Zero telemetry, zero external cloud servers, and zero account logins.

### Lifecycle State Machine & Deletion Model
```text
           ┌───────────┐
           │   Inbox   │ (Raw capture, unclarified)
           └─────┬─────┘
                 │ Clarify
    ┌────────────┼────────────┬─────────────┐
    ▼            ▼            ▼             ▼
┌────────┐ ┌───────────┐ ┌─────────┐ ┌─────────────┐
│  Next  │ │ Scheduled │ │ Waiting │ │ Someday /   │
│ Action │ │           │ │   For   │ │ Reference   │
└───┬────┘ └─────┬─────┘ └────┬────┘ └──────┬──────┘
    │            │            │             │
    └────────────┴─────┬──────┴─────────────┘
                       ▼
                 ┌───────────┐
                 │   Done    │ (Completed / Habits auto-reschedule)
                 └───────────┘
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
      ┌─────────────┐     ┌─────────────┐
      │   Archive   │     │    Purge    │
      │(Soft-delete)│     │(Hard-delete)│
      └─────────────┘     └─────────────┘
```

* **Two-Tier Deletion Architecture**:
  * **Archive (Soft-Delete)**: Keeps the task and its full historical timeline, marking `archived_at` and an archive reason. Restorable at any time with `horae restore`;
  * **Purge (Hard-Delete)**: Operates only on archived tasks. Physically executes `DELETE` with cascading cleanup of events and checklist items. Irreversible.

---

## 2. Core Architecture & Features Matrix

| Pillar | Architecture & Implementation | Real-World User Value |
| :--- | :--- | :--- |
| **Comprehensive GTD Methodology** | Faithful implementation of David Allen's GTD 5-step method; step-by-step weekly review wizard (`horae review`); strictly defined Today and Tomorrow perspectives; hierarchical checklist subtasks | Eliminates unstructured list sprawl and builds an actionable, reliable productivity closure system |
| **Instant Quick-Add Syntax** | Single-line natural parser: captures `@context`, `~schedule_time`, `*recurrence_rrule`, and `!priority` in one stroke; instant inline autocompletion with dual modes (Syntax Reference Card vs Speed Blind-typing) | Captures tasks in 2 seconds without breaking programming flow or creative thought |
| **Native RRULE Recurrence Engine** | Zero-dependency RFC 5545 engine supporting `DAILY`, `WEEKLY`, `MONTHLY`, and `YEARLY` frequencies; supports shorthands like `*2w[1,3]` (bi-weekly Mon/Wed), `*m[1,-1]` (1st and last day of month), `*y[jan,jul]`; auto-advances schedule anchor upon completion | Recurring tasks and habits never expire or vanish; sort order evaluates the next effective occurrence |
| **Dual-Form Interaction Model** | **CLI Tooling** (supports pipes and structured `--json` output) + **Flash Capture Mode** (`horae -f`, launches to capture and immediately exits) + **Ratatui 3-Pane TUI** (navigation, tasks, detail/timeline) | Pick the right interface for the moment: instantaneous capture from CLI, immersive planning and review in TUI |
| **Dynamic Focus Decision Engine** | `horae do` (`horae focus`): Multi-variable mathematical scoring combining priority (!high 10,000 pts dominance), overdue penalties, deadline proximity, and status weights, with FIFO tie-breaking | Ends decision paralysis; tells you the single most important task right now when overwhelmed |
| **Integrated Pomodoro System** | Built-in Pomodoro runtime; full-screen progress ring, streak tracker, customizable cycles (`work;short;long[;long_interval]`, e.g. `25;5;15;4`), desktop notifications, and Waybar status bar module | Seamlessly bridges task organization with deep time-boxing; no external timer apps needed |
| **Zero-Server Mobile Ecosystem** | **Syncthing Folder Bridge** (phone writes `capture.txt`, desktop generates `today.md` and reminder notes) + **ntfy Native Push Notifications** + **Feishu / Lark Integration** (WebSocket captures without public IP, interactive cards, Tasks v2 sync) | Zero server hosting fees, zero vendor lock-in; achieve true private cross-device sync |
| **Cultural Calendar & Aesthetics** | Pure in-tree 1900-2100 astronomical lunar calendar and 24 solar terms lookup table (`horae calendar`); 3-day and 1-day advance holiday warnings; Catppuccin themes; Nerd Font icons with ASCII fallback | Crafted with geek terminal aesthetics and thoughtful astronomical holiday awareness |

---

## 3. Installation & Prerequisites

### Option 1: npm (Recommended, No Rust Toolchain Required)
Ideal for any environment with Node.js installed; automatically fetches the precompiled binary for your OS and architecture:
```sh
# Try instantly without permanent installation
npx horae-cli

# Or install globally to provide the `horae` command
npm install -g horae-cli
```

### Option 2: Cargo (Official Rust Crates.io)
Requires Rust 1.89+; bundled SQLite requires no external C dependencies:
```sh
# Install latest release from crates.io
cargo install horae

# Or install latest git main branch
cargo install --git https://github.com/zhaohang1205/horae
```

### Option 3: Build from Source
```sh
git clone https://github.com/zhaohang1205/horae.git
cd horae
cargo build --release
# Binary will be located at target/release/horae
```

### Environment Recommendations
* **Terminal Font (Nerd Font)**: UI icons default to Nerd Font glyphs. On startup, `horae` probes `fc-list`. If no Nerd font is found, it **gracefully falls back to pure ASCII single characters**, avoiding "tofu" boxes. You can force the style via `HORAE_ICONS=nerd|ascii` or toggle it inside TUI (`F7`).
* **Windows Users**:
  * Data lives in `%APPDATA%\horae\`; customizable with `HORAE_CONFIG_DIR`;
  * Linux `notify-send` desktop pop-ups are unavailable on Windows, but in-TUI alarms, pomodoro timers, and ntfy mobile notifications function normally.
* **Data Directory**: Defaults to `~/.config/horae/` (contains `horae.db`, `config.json`, and `pomo.json`).

---

## 4. Quick Start

```sh
# 1. Quick capture a task with natural syntax
horae capture "buy milk @home ~today 18:00 !medium"

# 2. List next actions
horae list --status next

# 3. Focus on the top recommendation and launch a pomodoro
horae do --start

# 4. Mark task as done (supports full ID, unique prefix, or exact title)
horae done "buy milk"

# 5. Inspect the immutable audit timeline
horae show <id>

# 6. Launch the full-screen interactive TUI
horae
```

> **Task Reference Semantics**: Subcommands accept a **full ID**, a **unique ID prefix** (Git commit hash semantics), or a **unique exact title**. If a title is ambiguous, the command safely returns an error.

---

## 5. Quick-Add Syntax & Recurrence Engine Specification

`horae` features a unified single-line quick-add parser that works identically across CLI `capture`, `modify`, TUI quick capture (`a` / `Enter`), mobile sync files, and Feishu chat messages.

```text
buy milk @home ~today 18:00 *2w[1,3] !high
───┬──── ──┬── ─────┬────── ───┬──── ──┬──
   │       │        │          │       └─ Priority (!high / !medium / !low)
   │       │        │          └───────── Recurrence rule (*d, *w, *2w[1,3], *m[1,-1])
   │       │        └──────────────────── Scheduled start (~today, ~+2h, ~2026-07-24)
   │       └───────────────────────────── Context tag (@home, @work, @calls)
   └───────────────────────────────────── Task title
```

### 1. Time Expression Specification (`~time` or `--due`)
* **Schedule Start (`~time`)**: Sets the planned start execution time (`scheduled_start_at`); the task automatically transitions into `Scheduled` status;
* **Deadline (`--due`)**: Sets a soft deadline target (`due_at`).

| Type | Examples | Resolution Semantics |
| :--- | :--- | :--- |
| **Relative Offset** | `now`, `+15m`, `+30m`, `+1h`, `+2h`, `+4h`, `+1d`, `+3d`, `+1w` | Offset from current moment |
| **Offset + Clock** | `+3d 15:30`, `+1w 09:00` | Offset by days and anchor to specific clock time |
| **Day Words** | `today`, `tomorrow`, `今天`, `明天`, `后天 [HH:MM]` | Anchor to date (supports `~td` / `~tm` autocomplete) |
| **Weekday Words** | `mon`~`sun`, `next fri`, `周一`~`周日` | Anchor to nearest or next week's weekday |
| **Same-Day Clock Roll** | `09:00`, `18:00`, `HH:MM` | Today's clock time; if past, automatically rolls forward to tomorrow |
| **Absolute Dates** | `8/20 15:30`, `2026.8.20`, `2026-07-24 14:30` | Absolute calendar date and time |

### 2. Recurrence Rules (RRULE) & Shorthands (`*rrule`)
The custom in-tree recurrence engine supports RFC 5545 frequencies `DAILY`, `WEEKLY`, `MONTHLY`, and `YEARLY` with expressive shorthands:

| Shorthand | Evaluated RRULE | Semantics |
| :--- | :--- | :--- |
| `*d` / `*w` / `*m` / `*y` | `FREQ=DAILY`, `WEEKLY`, `MONTHLY`, `YEARLY` | Daily / Weekly / Monthly / Yearly |
| `*weekday` / `*weekend` | `FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR` / `SA,SU` | Workdays only / Weekends only |
| `*2w[1,3]` | `FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE` | Every 2 weeks on Monday and Wednesday (1-7 = Mon-Sun) |
| `*m[1,-1]` | `FREQ=MONTHLY;BYMONTHDAY=1,-1` | The 1st and last day of each month (-1 = last day) |
| `*m[1,15]` | `FREQ=MONTHLY;BYMONTHDAY=1,15` | The 1st and 15th of each month |
| `*y[jan,jul]` | `FREQ=YEARLY;BYMONTH=1,7` | Yearly in January and July |
| `*2y[6]` | `FREQ=YEARLY;INTERVAL=2;BYMONTH=6` | Every 2 years in June |

> **Reschedule Invariant**: When a recurring task is completed (`done`), it is never completed permanently. Instead, it records a check-in event, advances its anchor, and reschedules to the next valid occurrence slot.

### 3. Priority Shorthands (`!priority`)
* High priority: `!high` / `!h` / `!1` / `!高`;
* Medium priority: `!medium` / `!m` / `!2` / `!中`;
* Low priority: `!low` / `!l` / `!3` / `!低`.

---

## 6. CLI Command Reference

### Command Matrix

| Command | Alias | Description | Key Flags & Examples |
| :--- | :---: | :--- | :--- |
| `horae` / `horae --normal` | `-n` | Launch TUI in normal main view mode | `horae -n` |
| `horae flash` / `horae -f` | `f` | **Flash Capture Mode**: instant launch, save on Enter, exit | `horae -f` / `horae flash` |
| `horae capture [text]` | `c` | Capture into inbox (supports quick-add) | `--clip`, `--high`, `--due <time>`, `--tag <t>` |
| `horae list` | `l` | Query and filter active tasks | `--status next`, `--tag work`, `--date 0829`, `--json` |
| `horae show <id>` | `s` | View task details and full **audit timeline** | `horae show a1b2 --json` |
| `horae modify <id> [text]` | `m`/`mod`/`edit` | Update task (quick-add syntax or flags) | `--due tomorrow`, `--clear-due`, `--tag`, `--untag` |
| `horae next\|wait\|someday\|done <id>` | `d`=done | Move task lifecycle status | `horae d "buy milk"` / `horae wait <id>` |
| `horae schedule <id>` | — | Set schedule window and recurrence rules | `--start tomorrow`, `--end +1d 18:00`, `--rrule` |
| `horae archive <id>` | `rm`/`delete` | Soft-delete a task (movable to Archive) | `horae archive <id>` |
| `horae restore <id>` | — | Restore an archived task | `horae restore <id>` |
| `horae purge <id>` | — | **Permanently hard-delete** an archived task | `horae purge <id>` |
| `horae focus` / `horae do` | `do` | Recommendation engine: output top task now | `--start` (immediately starts pomodoro timer) |
| `horae log [message]` | — | Append a pure timestamped note without task | `horae log "deployed v1.0"` / `horae log` (list 50) |
| `horae stats` | — | MOTD terminal dashboard (Catppuccin Goddess of Time) | `horae stats` |
| `horae calendar [date]` | `cal` | Lunar calendar, solar terms, holiday warnings | `--short` (one-line summary), `--json` |
| `horae pomo <action>` | `p` | Pomodoro timer controller | `start <id>`, `stop`, `daemon`, `waybar` |
| `horae alarm <action>` | — | Upcoming-task alarm query & status bar module | `waybar [slot]`, `next [slot] --limit 3` |
| `horae export` | — | Export self-contained single backup JSON | `--file ~/backup.json` |
| `horae import <file>` | — | Import backup (merge by default, `--replace` wipes) | `horae import --replace backup.json` |
| `horae review` | — | Interactive GTD weekly review helper | `horae review` |
| `horae tags` | — | List all system context tags | `horae tags` |
| `horae tag` / `untag` | — | Attach or detach tags | `horae tag <id> work` |
| `horae profile <action>` | — | Multi-dataset profile manager | `list`, `new work`, `rename`, `rm`, `set-default` |
| `horae watch` | — | Launch phone sync daemon | `--dir <path>`, `--interval 5`, `--once` |
| `horae ntfy <action>` | — | Test and manage mobile native notifications | `horae ntfy test` |
| `horae feishu <action>` | — | Feishu / Lark integration manager | `doctor`, `test`, `due`, `summary`, `listen`, `sync` |
| `horae completions <shell>` | — | Generate shell autocompletion script | `horae completions bash > ...` |

> **Bilingual Help Support**: CLI `--help` is English by default. Pass `--lang zh` or set `HORAE_LANG=zh` for Chinese output.

---

## 7. TUI User Guide & Keybindings

Launching `horae` opens the full-screen Ratatui interface with a responsive 3-pane layout:
* **Left Pane (Navigation)**: Views list, status bucket counters, context tags, and system modules;
* **Center Pane (Task List)**: Active tasks in current view, priority indicators, deadlines, and badges;
* **Right Pane (Details & Timeline)**: Selected task metadata, checklist tree, notes, and the immutable event stream.

### Categorized Keybinding Reference

| Category | Key | Action |
| :--- | :--- | :--- |
| **Global & Navigation** | `h` / `l` | Switch focus between sidebar navigation, task list, and details pane |
| | `j` / `k` or `↑` / `↓` | Move selection up and down; `g` / `G` jump to top/bottom |
| | `0` - `9` | Switch view: `1` Inbox · `2` Next Action · `3` Scheduled · `4` Waiting · `5` Someday · `6` Reference · `7` Done · `8` Archive · `9` Tags · `0` Quotes |
| | `⇧J` / `⇧K` | Jump directly to **Today** / **Tomorrow** perspective views |
| | `/` / `f` | `/` triggers global text search; `f` opens tag filter popup |
| | `q` / `Ctrl+c` | Quit TUI |
| **Capture & Editing** | `a` | Quick capture from any view (focuses input with quick-add parser) |
| | `Enter` / `e` | Full edit: single line to update title, tags, schedule, and priority |
| | `n` | Open external editor (`$EDITOR`, e.g. nvim/vim/nano) for multi-line notes |
| | `C` | Append a new checklist item to selected task |
| | `Tab` | **Manage checklist items** (`j/k` move, `Space` toggle, `d` delete, `J/K` reorder, `e` rename; `Tab`/`Esc` exit) |
| | `=` | Check off the next uncompleted checklist item |
| **Lifecycle Actions** | `x` | Mark task as Done (recurring tasks auto-advance to next occurrence) |
| | `w` | Move task to Waiting-for status |
| | `s` | Move task to Someday/Maybe status |
| | `"` | Toggle in/out of Quotes (action tasks auto-transition to Reference) |
| | `A` / `D` | Soft-delete to Archive (y/n confirmation) / `D` in Archive permanently purges |
| | `u` | Restore archived task |
| **Visual Batch Mode** | `v` | Enter / exit Visual Multi-Select Mode |
| | `Space` | Toggle select current row (supports discrete multi-selection) |
| | `Ctrl+a` / `Ctrl+u` | Select all / invert selection |
| | `T` | Bulk apply tags to all selected tasks |
| | `A` / `u` | Bulk archive or bulk restore selected tasks |
| **Focus & Review** | `P` / `S` | Start / stop Pomodoro timer on selected task |
| | `[` | Configure Pomodoro intervals (`work;short;long[;long_interval]`, e.g. `25;5;15;4`) |
| | `r` / `R` | Weekly review wizard (`r` to start, `R` for next step) |
| **Preferences & System** | `F5` / `F6` | Toggle Catppuccin theme (Mocha dark / Latte light) and language (EN / ZH) |
| | `F7` | Module visibility & features popup (11 persistent toggles) |
| | `M` | Settings view (manage profiles: new/rename/delete/set default) |
| | `F1` or `?` | Context-sensitive shortcut help popup |

### Today & Tomorrow Perspective Contract
* **Today Perspective (`⇧J`)**: Strict union of tasks with scheduled start inside today's local window *plus* all **overdue tasks**. Recurring tasks without an occurrence today display their most recent missed occurrence marked overdue.
* **Tomorrow Perspective (`⇧K`)**: **Strictly limited to tasks scheduled inside tomorrow's window**. Unfinished tasks from today and older overdue tasks **never carry over** into tomorrow's view, preventing cognitive clutter.
* **Status Boundary**: Both views filter strictly for `Next` and `Scheduled` statuses. Inbox, Waiting, Someday, and Reference tasks stay in their own views.

---

## 8. In-Depth Features & Advanced Workflows

### 8.1 Focus Recommendation Engine (`horae do`)
When decision fatigue strikes, `horae do` (or `horae focus`) evaluates all active tasks against a weighted scoring algorithm:

$$\text{Total Score} = \text{Score}_{\text{priority}} + \text{Score}_{\text{due}} + \text{Score}_{\text{status}}$$

* **1. Filtering**: Excludes `Someday`, `Reference`, `Done`, `@quote` items, and non-overdue `Scheduled` or `Waiting` tasks whose start time has not arrived.
* **2. Priority Scoring ($\text{Score}_{\text{priority}}$)**:
  * `!high`: **+10,000 points** (absolute dominance);
  * `!medium`: **+5,000 points**;
  * `!low`: **+1,000 points**.
* **3. Deadline Proximity ($\text{Score}_{\text{due}}$)**:
  * **Overdue**: **+2,000 points** base + **10 points per day overdue** (capped at +500 points);
  * **Due Today**: **+1,000 points**.
* **4. Status Weight ($\text{Score}_{\text{status}}$)**:
  * In `Next` status: **+500 points** (promotes clarified actions over raw inbox items).
* **5. Tie-Breaker**:
  * First-In, First-Out (FIFO) on `created_at` ensures neglected older items eventually surface.
* **Immediate Action**: Run `horae do --start` to output the recommendation and immediately spawn a pomodoro session.

### 8.2 Pomodoro Timer & Waybar Integration
`horae pomo` runs a lightweight daemon writing state to `~/.config/horae/pomo.json` and sending desktop notifications:
```sh
# Start 25-minute focus session for a task
horae pomo start <id>

# Stop active session
horae pomo stop

# Emit JSON for Waybar modules
horae pomo waybar
```

### 8.3 Quotes Knowledge Base (Opt-in)
Quotes is an **opt-in feature** (toggle via `F7` in TUI). Designed for memorable quotes, inspirations, and reference fragments:
* Quotes are tasks tagged `@quote` with status `reference`;
* They are strictly isolated in the `[Library] 0 Quotes` view and filtered out of action flows;
* Capture with `@quote` from anywhere (e.g. `horae capture "Knowledge is power" --tag quote --status reference`) automatically routes to Quotes.

### 8.4 Astronomical Lunar Calendar & Holiday Warnings (`horae calendar`)
Features a built-in zero-dependency 1900-2100 lunar calendar and 24 solar terms engine:
```sh
# View today's solar term, lunar date, and holiday countdown
horae calendar

# Single-line compact output for MOTD
horae calendar --short

# Output structured JSON
horae calendar --json
```
Major holidays trigger advance alerts 3 days and 1 day ahead in top banners and background desktop notifications.

### 8.5 Data Backup & Disaster Recovery
The backup file is a self-contained JSON file with schema versioning, capturing all tasks, `task_events` timelines, tags, settings, and pomodoro state:
```sh
# Export full backup
horae export --file ~/horae-backup.json

# Merge import (skips existing task IDs)
horae import ~/horae-backup.json

# True disaster restore (wipes database and restores exactly)
horae import --replace ~/horae-backup.json
```

---

## 9. Mobile Ecosystem & Integrations

`horae` provides three flexible mobile sync solutions balancing privacy with convenience:

### 9.1 Solution 1: Zero-Server Syncthing Folder Bridge
Sync `~/.config/horae/sync` to your phone via Syncthing and run `horae watch` on your computer:
```sh
horae watch                  # Run daemon (systemd / tmux / autostart)
horae watch --once           # Run a single reconciliation cycle
```

**Folder Protocol**:
* `capture.txt` (phone writes): One quick-add line per task; horae ingests and writes receipt;
* `actions.txt` (phone writes): State transitions like `done <id>` or `set <id> status next`;
* `today.md` (computer generates): Snapshot of active tasks and overdue items;
* `reminders/` (computer generates): Markdown files dropped when tasks come due, triggering mobile sync notifications.

### 9.2 Solution 2: Lightweight Native Push Notifications via ntfy
If you only need "phone notifications when tasks come due", use the open-standard [ntfy](https://ntfy.sh):
1. Install ntfy on your phone and subscribe to a private random topic (e.g. `horae-my-random-topic`);
2. Add the `ntfy` block to your profile in `~/.config/horae/config.json`:
   ```json
   {
     "default_profile": "default",
     "profiles": {
       "default": {
         "db": "horae.db",
         "ntfy": {
           "url": "https://ntfy.sh",
           "topic": "horae-my-random-topic",
           "token_env": "HORAE_NTFY_TOKEN",
           "priority": 5,
           "lead_minutes": 10
         }
       }
     }
   }
   ```
   > 🔒 **Security Notice**: Access tokens are read via environment variables (`token_env`) and never stored on disk.
3. Test connectivity:
   ```sh
   horae ntfy test
   ```
4. Run `horae watch`; timed tasks will automatically trigger mobile push notifications 10 minutes prior to due time.

### 9.3 Solution 3: Feishu / Lark Enterprise Integration
Deep bidirectional integration with Feishu (Lark):
* **Capture Everywhere**: Send natural quick-add lines directly to your bot in private chat;
* **Interactive Cards**: Bot replies with interactive cards; tap `[✅ Done]` or `[📅 Postpone 1 Day]` directly on mobile;
* **Native Sync**: Bidirectional sync with official Feishu Tasks v2.

```sh
# Verify credentials and WebSocket gateway
horae feishu doctor

# Send test notification card
horae feishu test

# Synchronize with Feishu Tasks v2
horae feishu sync --push
```

---

## 10. Development & Contributing

```sh
# Run full unit and integration test suite
cargo test --workspace

# Lint code (zero warnings policy)
cargo clippy --workspace -- -D warnings

# Verify formatting
cargo fmt --check
```

Contributors must consult the authoritative guides before opening PRs:
* 🇬🇧 **[AGENTS.md (Contributor Rules & Architecture)](AGENTS.md)**
* 🇨🇳 **[System Architecture Handoff (docs/architecture-handoff.md)](architecture-handoff.md)**

---

## 11. Author & License

`horae` is designed and maintained by an independent developer. Contributions and feedback are warmly welcomed:

* **GitHub Repository**: [https://github.com/zhaohang1205/horae](https://github.com/zhaohang1205/horae)
* **Gitee Mirror**: [https://gitee.com/zhao-hang1205/hora.git](https://gitee.com/zhao-hang1205/hora.git)
* **WeChat**: `hang_ace` (Name: `zhaoh`)
* **Email**: [18658233987@163.com](mailto:18658233987@163.com)

### License
Released under the **GPL-3.0** License. See [LICENSE](LICENSE) for details.
