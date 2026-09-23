# horae — GTD 终端任务管理器

<div align="center">

<p align="center">
  <a href="https://github.com/zhaohang1205/horae/actions/workflows/ci.yml"><img src="https://github.com/zhaohang1205/horae/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/zhaohang1205/horae/actions/workflows/release.yml"><img src="https://github.com/zhaohang1205/horae/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <a href="https://crates.io/crates/horae"><img src="https://img.shields.io/crates/v/horae.svg" alt="Crates.io"></a>
  <a href="https://www.npmjs.com/package/horae-cli"><img src="https://img.shields.io/npm/v/horae-cli.svg" alt="npm"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg" alt="License: GPL-3.0"></a>
</p>

**[ 🇨🇳 简体中文 ](README_cn.md)** &nbsp;•&nbsp; **[ 🇬🇧 English ](README_en.md)** &nbsp;•&nbsp; **[ 📖 文档导航 / Home ](README.md)**

*Rust 单二进制 · 内置 SQLite · CLI 极速秒捕 · 沉浸式 Ratatui TUI · 零云端隐私所有权*

</div>

---

## 目录

- [1. 核心设计哲学与心智模型](#1-核心设计哲学与心智模型)
- [2. 产品核心特性矩阵](#2-产品核心特性矩阵)
- [3. 安装与环境准备](#3-安装与环境准备)
- [4. 快速上手](#4-快速上手)
- [5. 一句话语法与时间引擎规范](#5-一句话语法与时间引擎规范)
- [6. CLI 命令行参考手册](#6-cli-命令行参考手册)
- [7. TUI 交互指南与快捷键参考](#7-tui-交互指南与快捷键参考)
- [8. 深度功能指南](#8-深度功能指南)
  - [8.1 智能专注推荐算法 (`horae do`)](#81-智能专注推荐算法-horae-do)
  - [8.2 专注番茄钟与状态栏集成](#82-专注番茄钟与状态栏集成)
  - [8.3 金句与知识灵感库 (Quotes)](#83-金句与知识灵感库-quotes)
  - [8.4 农历、节气与节假日预警 (`horae calendar`)](#84-农历节气与节假日预警-horae-calendar)
  - [8.5 数据备份与灾难恢复](#85-数据备份与灾难恢复)
- [9. 移动端生态与外部协同](#9-移动端生态与外部协同)
  - [9.1 方案一：Syncthing 零服务器文件桥接](#91-方案一syncthing-零服务器文件桥接)
  - [9.2 方案二：ntfy 极轻量原生手机通知推送](#92-方案二ntfy-极轻量原生手机通知推送)
  - [9.3 方案三：飞书 / Lark 企业级深度协同](#93-方案三飞书--lark-企业级深度协同)
- [10. 开发与参与贡献](#10-开发与参与贡献)
- [11. 作者与开源协议](#11-作者与开源协议)

---

## 1. 核心设计哲学与心智模型

`horae` 是一款纯 Rust 打造的现代化 GTD (Getting Things Done) 终端生产力工具，融合了 **SQLite 数据层 + CLI 命令行 + Ratatui 全屏 TUI** 三合一形态。

### 核心基石：时间数据化 (Time-Datafication)
传统的待办清单工具大多是将任务当作易失的、用完即勾销的便签。而在 `horae` 中，每一次任务状态变更、排期调整、打卡完成都会打上精准的 **UTC 毫秒时间戳**，并以只增方式追加到 `task_events` 事件流水表中：
* **数据不说谎**：通过 `horae show <id>` 可完整复盘该任务从「捕获入收件箱」→「澄清转下一步」→「多次延期排程」→「最终完成」的完整生命周期轨迹；
* **原子事务与审计缝**：所有变更均经由底层 `repo::mutate` 事务闭包，确保变更状态与审计流水共用唯一的 `now` 时间戳；
* **完全私有与数据自主权**：数据全部保存于本地单一 SQLite 文件（`~/.config/horae/horae.db`），零云端依赖、零第三方账号、零数据遥测。

### 状态机生命周期与删除模型
```text
           ┌───────────┐
           │   Inbox   │ (初始收集，未澄清)
           └─────┬─────┘
                 │ 澄清 (Clarify)
    ┌────────────┼────────────┬─────────────┐
    ▼            ▼            ▼             ▼
┌────────┐ ┌───────────┐ ┌─────────┐ ┌─────────────┐
│  Next  │ │ Scheduled │ │ Waiting │ │ Someday /   │
│ (行动) │ │  (排程)   │ │ (等待)  │ │ Reference   │
└───┬────┘ └─────┬─────┘ └────┬────┘ └──────┬──────┘
    │            │            │             │
    └────────────┴─────┬──────┴─────────────┘
                       ▼
                 ┌───────────┐
                 │   Done    │ (完成 / 循环任务自动重排)
                 └───────────┘
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
      ┌─────────────┐     ┌─────────────┐
      │   Archive   │     │    Purge    │
      │ (软删可恢复) │     │ (硬删不可逆) │
      └─────────────┘     └─────────────┘
```

* **双层删除设计**：
  * **归档 (Archive / 软删除)**：保留任务及其全部时间线履历，标记 `archived_at` 与归档原因。可通过 `restore` 随时无损恢复；
  * **粉碎 (Purge / 硬删除)**：仅允许对已归档任务操作，物理执行 `DELETE` 彻底清除级联数据与历史，用于彻底脱敏。

---

## 2. 产品核心特性矩阵

| 核心特性 | 架构实现与技术细节 | 实际用户价值 |
| :--- | :--- | :--- |
| **全生命周期 GTD 方法论** | 严格实现 GTD 五步法闭环；内置分步周回顾向导 (`horae review`)；独立严格的今日 / 明日视图与检查单子任务树 | 告别混乱松散的 Todo 堆积，建立科学高效的个人效能闭环体系 |
| **一句话极速捕获** | Quick-add 语法解析器：单行一并录入 `@情境`、`~排程时间`、`*循环规则`、`!优先级`；支持双模式（语法指南参考卡片 vs 极速盲打）即时补全 | 终端输入只需 2 秒，丝滑录入不打断编码与心流状态 |
| **自研原生 RRULE 循环** | 零外部依赖解析并展开 `DAILY/WEEKLY/MONTHLY/YEARLY` 全频率，支持 `*2w[1,3]`（隔周一三）、`*m[1,-1]`（月初月末）、`*y[jan,jul]` 等高级语义简写；完成自动重排推进锚点 | 周期习惯与重复工作永不失效，列表排序看下一次发生点，不产生冗余垃圾任务 |
| **双形态交互设计** | **CLI 脚本化命令**（支持管道与 JSON 结构化输出）+ **闪念录入模式**（`horae -f` 录入即关）+ **Ratatui 全屏三栏 TUI**（导航/列表/详情） | 满足不同场景：日常极速记录用 CLI/闪念，深度计划与每周复盘用 TUI |
| **动态专注决策算法** | `horae do`（`horae focus`）：基于优先级（!high 赋 10000 绝对统治力）、逾期梯度惩罚、到期临近度与行动状态进行多维动态打分，决胜回退 FIFO | 终结选择困难症，在任务堆积如山时直接告知“当下唯一应该做的事” |
| **深度集成番茄钟** | 原生内建 Pomodoro 计时引擎；全屏倒计时环、每日连击打卡统计、自定义周期配方（`工作;短休;长休[;长休周期]`，如 `25;5;15;4`）；与 Waybar 状态栏深度联动 | 任务管理与深度工作时间分块融为一体，无需在多个计时器软件间反复横跳 |
| **零服务器手机生态** | **Syncthing 文件夹桥接**（手机笔记 App 写 `capture.txt`，电脑常驻回写 `today.md` 与到期提醒）+ **ntfy 原生轻量推送** + **飞书企业应用**（免公网 IP WebSocket 随手记与 Tasks v2 同步） | 零云端中继成本、零第三方 App 绑定，手机与桌面端实现纯本地数据闭环 |
| **天文历法与美学** | 纯内置 1900-2100 农历引擎与二十四节气天体历表（`horae calendar`），重大传统节假日提前 3 天与 1 天智能预警；Catppuccin 双色主题；Nerd Font 矢量图标与 ASCII 优雅降级 | 工具兼具极客生产力美学与中国传统节气文化关怀 |

---

## 3. 安装与环境准备

### 方式一：npm 安装（推荐，无需 Rust 工具链）
适合任意安装了 Node.js 的开发环境，自动下载分发适用于当前 OS/架构的预编译二进制：
```sh
# 免安装瞬时体验
npx horae-cli

# 全局安装（安装后直接使用 horae 命令）
npm install -g horae-cli
```

### 方式二：Cargo 安装（Rust 官方渠道）
系统需具备 Rust 1.89+ 环境，SQLite 已内嵌捆绑，无额外 C 依赖：
```sh
# 从 crates.io 安装最新稳定版
cargo install horae

# 从 GitHub main 分支安装最新开发版
cargo install --git https://github.com/zhaohang1205/horae

# 中国大陆用户可通过 Gitee 镜像加速安装：
cargo install --git https://gitee.com/zhao-hang1205/hora.git
```

### 方式三：从源码本地构建
```sh
git clone https://github.com/zhaohang1205/horae.git
cd horae
cargo build --release
# 生成的可执行文件位于 target/release/horae
```

### 环境与平台适配建议
* **终端字体 (Nerd Font)**：TUI 图标默认采用 Nerd Font 符号字形。若终端未安装对应字体，启动时会自动探测（`fc-list`）并**优雅回退为纯 ASCII 单字符**，绝不会出现界面“豆腐块”乱码。推荐使用 [Nerd Fonts](https://www.nerdfonts.com/)（如 JetBrainsMono Nerd Font）。可通过环境变量 `HORAE_ICONS=nerd|ascii` 强制指定。
* **Windows 用户说明**：
  * 数据默认存储于 `%APPDATA%\horae\`；可用 `HORAE_CONFIG_DIR` 环境变量自定义；
  * Linux 桌面通知依赖 `notify-send`，Windows 上系统级横幅弹窗不触发，但番茄钟计时、TUI 内部提醒及 ntfy 手机推送均正常工作。
* **数据目录**：默认存放于 `~/.config/horae/`（包含 `horae.db` 数据库、`config.json` 配置与 `pomo.json` 番茄钟运行时）。

---

## 4. 快速上手

```sh
# 1. 快速录入任务（quick-add 语法）
horae capture "买牛奶 @home ~今天 18:00 !medium"

# 2. 列出下一步行动
horae list --status next

# 3. 推荐当前最该做的一件事并启动番茄钟
horae do --start

# 4. 完成任务（支持完整 ID、唯一 ID 前缀、或精确标题）
horae done "买牛奶"

# 5. 查看任务生命周期完整时间线
horae show <id>

# 6. 启动全屏交互式 TUI
horae
```

> **任务引用机制**：命令参数接受**完整 ID**、**唯一 ID 前缀**（类似 Git commit hash 体验）或**唯一精确标题**。标题存在歧义时系统会明确报错以防误操作。

---

## 5. 一句话语法与时间引擎规范

`horae` 内置强大的单行 quick-add 语法解析器，在命令行 `capture`、修改 `modify`、TUI 快速录入 (`a` / `Enter`)、手机同步文件以及飞书随手记中均统一生效。

```text
买牛奶 @home ~今天 18:00 *2w[1,3] !high
──┬── ──┬── ─────┬───── ───┬──── ──┬──
  │     │        │         │       └─ 优先级 (!high / !medium / !low)
  │     │        │         └───────── 循环规则 (*d, *w, *2w[1,3], *m[1,-1] 等)
  │     │        └─────────────────── 排程开始时间 (~today, ~+2h, ~2026-07-24 等)
  │     └──────────────────────────── 情境标签 (@home, @work, @calls 等)
  └────────────────────────────────── 任务标题
```

### 1. 时间表达式规范 (`~time` 或 `--due`)
* **排程起点 (`~time`)**：设置任务计划开始执行的时间 (`scheduled_start_at`)，任务自动归入 `Scheduled` (已排程) 状态；
* **截止时间 (`--due`)**：设置任务的软性到期死线 (`due_at`)。

| 语法类型 | 示例 | 解析语义 |
| :--- | :--- | :--- |
| **相对偏移** | `now`, `+15m`, `+30m`, `+1h`, `+2h`, `+4h`, `+1d`, `+3d`, `+1w` | 从当前时刻向后偏移指定分/时/天/周 |
| **相对天数 + 指定时刻** | `+3d 15:30`, `+1w 09:00` | 偏移天数并定位到当日指定时刻 |
| **天词短语** | `today`, `tomorrow`, `今天`, `明天`, `后天 [HH:MM]` | 定位至指定日期（支持拼音缩写 `~td` / `~tm` 快速补全） |
| **星期词短语** | `周一`~`周日`, `下周五`, `mon`~`sun`, `next fri [HH:MM]` | 定位到最近或下周的对应星期几 |
| **当日时刻顺延** | `09:00`, `18:00`, `HH:MM` | 当日时刻；若当前已过该时刻，则智能顺延至明日 |
| **日期时间组合** | `8/20 15:30`, `2026.8.20`, `2026-07-24 14:30` | 绝对公历日期与时刻 |

### 2. 循环规则 (RRULE) 与快捷简写 (`*rrule`)
`horae` 拥有自主研发的原生 RRULE 引擎（零外部库依赖），完全支持 RFC 5545 核心频率 `DAILY`, `WEEKLY`, `MONTHLY`, `YEARLY`，并提供极简输入简写：

| 简写语法 | 对应 RRULE 表达式 | 语义说明 |
| :--- | :--- | :--- |
| `*d` / `*w` / `*m` / `*y` | `FREQ=DAILY`, `WEEKLY`, `MONTHLY`, `YEARLY` | 每天 / 每周 / 每月 / 每年 |
| `*weekday` / `*weekend` | `FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR` / `SA,SU` | 仅工作日 / 仅周末 |
| `*2w[1,3]` | `FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE` | 每两周的周一和周三（1-7 代表周一至周日） |
| `*m[1,-1]` | `FREQ=MONTHLY;BYMONTHDAY=1,-1` | 每月 1 号和最后一天（负数代表月末倒数） |
| `*m[1,15]` | `FREQ=MONTHLY;BYMONTHDAY=1,15` | 每月 1 号和 15 号 |
| `*y[jan,jul]` | `FREQ=YEARLY;BYMONTH=1,7` | 每年 1 月和 7 月（支持英文简写或数字） |
| `*2y[6]` | `FREQ=YEARLY;INTERVAL=2;BYMONTH=6` | 每两年的 6 月 |

> **循环重排机制**：循环任务在打勾完成 (`done`) 时**绝不会消失**，而是推进锚点，自动计算并重排至下一次有效发生点；列表排序与到期计算均看**下一次有效发生时间**。

### 3. 优先级标记 (`!priority`)
* 支持简写：`!high` / `!h` / `!1` / `!高`（设置高优先级）；
* 支持中等：`!medium` / `!m` / `!2` / `!中`；
* 支持低优：`!low` / `!l` / `!3` / `!低`。

---

## 6. CLI 命令行参考手册

`horae` 命令行工具设计精简而强大，提供高频单字母别名，并支持 `--json` 输出以便脚本集成。

### 常用命令总览

| 命令 / Command | 别名 | 功能说明 | 关键选项与示例 |
| :--- | :---: | :--- | :--- |
| `horae` / `horae --normal` | `-n` | 启动 TUI 正常主视图模式 | `horae -n` |
| `horae flash` / `horae -f` | `f` | **闪念输入模式**：秒开即录，回车确认即退出 | `horae -f` / `horae flash` |
| `horae capture [text]` | `c` | 捕获新任务入收件箱（支持 quick-add） | `--clip`（剪贴板录入）, `--high`, `--due <time>` |
| `horae list` | `l` | 查询与检索任务列表 | `--status next`, `--tag work`, `--date 0829`, `--json` |
| `horae show <id>` | `s` | 查看任务详情及完整**事件时间线** | `horae show a1b2 --json` |
| `horae modify <id> [text]` | `m`/`mod`/`edit` | 修改任务（支持一句话 quick-add 批量覆盖） | `--due tomorrow`, `--clear-due`, `--tag`, `--untag` |
| `horae next\|wait\|someday\|done <id>` | `d`=done | 快速流转任务状态（循环任务完成自动重排） | `horae d "买牛奶"` / `horae wait <id>` |
| `horae schedule <id>` | — | 规划排程时间与循环规则 | `--start tomorrow`, `--end +1d 18:00`, `--rrule` |
| `horae archive <id>` | `rm`/`delete` | 软删除任务（存入归档，可恢复） | `horae archive <id>` |
| `horae restore <id>` | — | 从归档中恢复任务 | `horae restore <id>` |
| `horae purge <id>` | — | **彻底物理清除**归档任务（级联清除，不可逆） | `horae purge <id>` |
| `horae focus` / `horae do` | `do` | 智能决策算法：输出当前最该做的一件事 | `--start`（推荐后直接启动番茄钟） |
| `horae log [message]` | — | 记纯时间戳碎碎念/日志（不建任务，隔离流水） | `horae log "完成核心架构重构"` / `horae log` (查历史) |
| `horae stats` | — | 控制台终端看板（Catppuccin 时间女神与效能仪表） | `horae stats` |
| `horae calendar [date]` | `cal` | 农历、节气与重大节假日预警查询 | `--short` (单行摘要), `--json` |
| `horae pomo <action>` | `p` | 专注番茄钟管理 | `start <id>`, `stop`, `daemon`, `waybar` |
| `horae alarm <action>` | — | 近期到期提醒组件 | `waybar [slot]`, `next [slot] --limit 3` |
| `horae export` | — | 全量导出为自包含独立 JSON 文件 | `--file ~/backup.json` |
| `horae import <file>` | — | 导入备份（默认合并跳过已存在，`--replace` 清空还原） | `horae import --replace backup.json` |
| `horae review` | — | GTD 周回顾辅助向导 | `horae review` |
| `horae tags` | — | 列出系统所有标签及分类 | `horae tags` |
| `horae tag` / `untag` | — | 为任务添加或移除标签 | `horae tag <id> work` |
| `horae profile <action>` | — | 多数据集管理（隔离工作/生活不同数据库） | `list`, `new work`, `rename`, `rm`, `set-default` |
| `horae watch` | — | 启动无服务器手机桥守护进程 | `--dir <path>`, `--interval 5`, `--once` |
| `horae ntfy <action>` | — | 手机原生通知推送测试与管理 | `horae ntfy test` |
| `horae feishu <action>` | — | 飞书/Lark 深度集成（自检/长连接/任务同步） | `doctor`, `test`, `due`, `summary`, `listen`, `sync` |
| `horae completions <shell>` | — | 生成 Shell 自动补全脚本 | `horae completions bash > ...` |

> **帮助语言切换**：CLI `--help` 默认显示英文。添加 `--lang zh` 参数（或设置环境变量 `HORAE_LANG=zh`）可无缝切换为完整中文帮助：
> ```sh
> horae --lang zh --help
> HORAE_LANG=zh horae capture --help
> ```

---

## 7. TUI 交互指南与快捷键参考

输入 `horae` 即可进入全屏交互式 TUI。采用响应式三栏设计：
* **左栏 (Navigation)**：视图列表、状态桶计数、情境标签与功能模块；
* **中栏 (Task List)**：当前视图下的活跃任务列表、状态标记、截止期与进度徽标；
* **右栏 (Details & Timeline)**：当前选中任务的详细属性、子检查单、备注以及完整的只写审计时间线。

### 快捷键分类参考表

| 场景分类 | 快捷键 | 功能描述 |
| :--- | :--- | :--- |
| **全局与导航** | `h` / `l` | 左右在导航栏、任务列表与详情面板之间切换焦点 |
| | `j` / `k` 或 `↑` / `↓` | 列表中上下移动光标；`g` / `G` 瞬跳首尾 |
| | `0` - `9` | 切换主视图：`1`收件箱 · `2`下一步 · `3`排程 · `4`等待 · `5`将来 · `6`参考 · `7`完成 · `8`归档 · `9`标签 · `0`金句 |
| | `⇧J` / `⇧K` | 快速切换至 **今日 (Today)** / **明日 (Tomorrow)** 视图 |
| | `/` / `f` | `/` 唤起全量文本搜索；`f` 唤起情境标签即时过滤 |
| | `q` / `Ctrl+c` | 退出 TUI |
| **任务录入与编辑** | `a` | 在任意视图快速捕获新任务（自动聚焦输入条，支持 quick-add） |
| | `Enter` / `e` | 全量编辑当前任务：支持一句话修改标题、标签、时间与优先级 |
| | `n` | 唤起外部编辑器（`$EDITOR`，如 nvim/vim/nano）编辑长文本备注 |
| | `C` | 为当前任务追加一条检查单子项 (Checklist) |
| | `Tab` | **进入检查单子项深度管理模式**（`j/k` 移动，`Space` 勾选，`d` 删除，`J/K` 上下调序，`e` 重命名；`Tab`/`Esc` 退出） |
| | `=` | 快速勾选下一项未完成的检查单项目（列表行徽标实时同步） |
| **状态流转与操作** | `x` | 标记当前任务为已完成（循环任务自动推进并重排至下一次） |
| | `w` | 流转至等待中 (Waiting-for) |
| | `s` | 流转至将来也许 (Someday/Maybe) |
| | `"` | 加入 / 移出金句库（若为行动态任务，自动转为参考资料） |
| | `A` / `D` | 归档任务（软删除，按 y 确认） / 归档视图下 `D` 永久物理清除 |
| | `u` | 恢复已归档任务 (Restore) |
| **可视多选批量操作** | `v` | 开启 / 关闭可视多选模式 (Visual Mode) |
| | `Space` | 单独翻转当前行选中状态（支持非连续离散多选） |
| | `Ctrl+a` / `Ctrl+u` | 全选 / 反选当前视图任务 |
| | `T` | 对选中的多条任务进行**批量打标签** |
| | `A` / `u` | 对选中的多条任务进行批量归档或批量恢复 |
| **专注与复盘工具** | `P` / `S` | 启动 / 停止当前选中任务的番茄钟专注计时 |
| | `[` | 配置番茄钟时长配方（格式 `工作;短休;长休[;长休周期]`，如 `25;5;15;4`） |
| | `r` / `R` | 启动 GTD 周回顾向导 (`r` 开启，`R` 步进下一环节) |
| **系统与主题偏好** | `F5` / `F6` | 一键无缝切换 Catppuccin 主题（深色 Mocha / 浅色 Latte）与中英界面语言 |
| | `F7` | 唤起模块功能与视觉配置（包含金句、图标风格、启动模式、补全风格等 11 项持久化开关） |
| | `M` | 打开 Profile 多数据集管理页面（新建/重命名/删除/设为默认） |
| | `F1` 或 `?` | 弹出上下文敏感的快捷键全量帮助弹窗 |

### 今日 / 明日视图的严格口径规范
* **今日视图 (Today)**：严格展示「计划执行窗口落在今日的任务（含循环任务今日发生点）」+「**逾期任务 (Overdue)**」。循环任务若今日没有发生点，则展示其最近一次已错过的发生点并标为逾期；
* **明日视图 (Tomorrow)**：**仅展示计划排程落在明日窗口内的任务**（含循环任务明日发生点）。今日未完成的任务或历史逾期任务**严禁跨日结转**到明日视图，避免破坏未来的认知聚焦；
* **状态过滤原则**：今日与明日视图仅收录处于 `Next` (下一步) 与 `Scheduled` (已排程) 状态的可执行任务，收件箱、等待中、将来也许和参考资料均保留在各自状态桶中。

---

## 8. 深度功能指南

### 8.1 智能专注推荐算法 (`horae do`)
当你面对堆积如山的项目陷入选择困难时，`horae do`（或 `horae focus`）通过内置的加权评分数学模型实时计算全量活跃任务，选出当下唯一最优行动：

$$\text{Total Score} = \text{Score}_{\text{priority}} + \text{Score}_{\text{due}} + \text{Score}_{\text{status}}$$

* **1. 过滤层 (Filter)**：自动剔除 `Someday`、`Reference`、`Done`、`@quote` 金句，以及未到期且未逾期的 `Scheduled` 与 `Waiting` 任务；
* **2. 优先级加分 ($\text{Score}_{\text{priority}}$)**：
  * `!high`：**+10000 分**（具备压倒性统治力，一旦设置几乎必定首推）；
  * `!medium`：**+5000 分**；
  * `!low`：**+1000 分**；
* **3. 截止期紧迫度加分 ($\text{Score}_{\text{due}}$)**：
  * **已逾期**：**+2000 分**；在此基础上，每多逾期 1 天额外加 **+10 分**（封顶 +500 分，逾期越久越优先救火）；
  * **今日截止**：**+1000 分**；
* **4. 行动状态加分 ($\text{Score}_{\text{status}}$)**：
  * 处于 `Next` (下一步行动)：**+500 分**（鼓励推进已思考清晰的事项）；
* **5. 最终决胜 (Tie-breaker)**：
  * 分数完全相同时，优先推荐创建时间更早 (`created_at`) 的任务，避免底部历史任务永远被忽视；
* **顺带开番茄**：执行 `horae do --start` 可在选出任务的同时，直接拉起专注计时。

### 8.2 专注番茄钟与状态栏集成
`horae pomo` 在后台秒级守护，数据写入 `~/.config/horae/pomo.json`，并支持发送桌面通知：
```sh
# 为特定任务开启 25 分钟专注
horae pomo start <id>

# 停止当前番茄钟
horae pomo stop

# Waybar 状态栏集成：输出 JSON 供 Linux Waybar 模块直接消费
horae pomo waybar
```

### 8.3 金句与知识灵感库 (Quotes)
金句是一个**可选开启的功能**（默认关闭，可通过 TUI `F7` 开启）。用于收藏值得反复品味的名言、灵感闪念与非行动性参考知识：
* **数据实质**：金句是打上了系统标签 `@quote` 且状态为 `reference` 的任务；
* **行动流完全隔离**：开启后，侧边栏出现 `[Library] 0 金句` 视图，普通参考资料视图自动过滤金句；金句绝不会出现在收件箱、下一步或今日待办中；
* **便捷录入**：在任何视图捕获时带有 `@quote`（如 `horae capture "知行合一" --tag quote --status reference`），系统自动将其路由入金句库。

### 8.4 农历、节气与节假日预警 (`horae calendar`)
`horae` 纯原生内置了 1900-2100 年的天文历法算法与二十四节气查找表，无需依赖系统网络即可精准查询：
```sh
# 查看今日公历、农历干支生肖、今日节气及下个节假日倒计时
horae calendar

# 单行紧凑输出（适合终端登录 MOTD 或状态栏）
horae calendar --short

# 输出完整结构化 JSON
horae calendar --json
```
* **节假日预警**：重大传统节日（如春节、中秋、国庆等）提前 3 天与 1 天自动在 TUI 顶部横幅以及后台桌面通知中进行温情提醒与行动提示。

### 8.5 数据备份与灾难恢复
`horae` 的全量备份为一个独立的 JSON 文件，完整包含任务主表、`task_events` 时间线、标签索引、设置项与番茄钟数据：
```sh
# 备份导出到指定文件（默认当前目录自动带时间戳文件名）
horae export --file ~/horae-backup.json

# 合并导入（已存在的相同 ID 任务直接跳过）
horae import ~/horae-backup.json

# 真正精确灾难还原（清空本地数据库，严格按备份文件恢复）
horae import --replace ~/horae-backup.json
```

---

## 9. 移动端生态与外部协同

为了兼顾「本地数据隐私」与「随时随地随手记」，`horae` 提供了三套灵活互补的外部协同方案：

### 9.1 方案一：Syncthing 零服务器文件桥接
通过 Syncthing（或任何私有双向同步盘）将电脑上的 `~/.config/horae/sync` 目录同步至手机，并在电脑常驻后台运行 `horae watch`：
```sh
horae watch                  # 常驻运行（可交由 systemd / tmux / 开机自启）
horae watch --once           # 手动对账一次
```

**同步文件夹交互协议**：
* `capture.txt`（手机写）：每行一条，支持 quick-add 语法，电脑扫描后自动入库并打回执；
* `actions.txt`（手机写）：支持手机发送流转指令（如 `done <id>` 或 `set <id> status next`）；
* `today.md`（电脑生成）：电脑自动回写今日活跃任务与逾期清单快照，手机笔记软件随时可读；
* `reminders/`（电脑生成）：电脑在任务到期时生成提醒 markdown，手机同步时自动触发文件变更弹窗。

### 9.2 方案二：ntfy 极轻量原生手机通知推送
如果你只需要「任务到点时手机弹出系统原生通知」，无需自建服务器，利用基于 Web 开放标准的 [ntfy](https://ntfy.sh) 即可实现：
1. 手机安装 ntfy App，订阅一个私密随机主题（如 `horae-my-secret-uuid`）；
2. 在 `~/.config/horae/config.json` 中配置对应 Profile 的 `ntfy` 节点：
   ```json
   {
     "default_profile": "default",
     "profiles": {
       "default": {
         "db": "horae.db",
         "ntfy": {
           "url": "https://ntfy.sh",
           "topic": "horae-my-secret-uuid",
           "token_env": "HORAE_NTFY_TOKEN",
           "priority": 5,
           "lead_minutes": 10
         }
       }
     }
   }
   ```
   > 🔒 **隐私与安全准则**：访问 Token 仅通过环境变量（`token_env`）传递，绝不落盘；
3. 测试连通性：
   ```sh
   horae ntfy test
   ```
4. 常驻 `horae watch` 后，所有带截止日期的任务在到期前（默认 10 分钟）会自动向手机发送原生横幅通知。

### 9.3 方案三：飞书 / Lark 企业级深度协同
深度支持飞书 (Lark)，涵盖企业自建应用免公网 IP 交互：
* **随时随地随手记 (Capture Everywhere)**：在手机/桌面飞书直接私聊机器人发送自然语言待办（如 `准备周会PPT @work tomorrow 15:00 !high`），本地 `horae` 毫秒级自动入库；
* **交互卡片闭环**：机器人回发富文本交互卡片，在手机端点击【✅ 标记完成】或【📅 顺延1天】直接流转任务；
* **原生任务同步**：与飞书官方 Tasks v2 双向对账。

```sh
# 诊断飞书凭据与网关连通性
horae feishu doctor

# 发送测试卡片验证通道
horae feishu test

# 手动与飞书官方任务 (Tasks v2) 对账
horae feishu sync --push
```

---

## 10. 开发与参与贡献

```sh
# 运行工作区全量单元测试与集成测试
cargo test --workspace

# 静态代码检查（严格要求零告警）
cargo clippy --workspace -- -D warnings

# 格式化校验
cargo fmt --check
```

关于核心领域分层规范、数据库迁移规则（0001-0014）以及不可违背的架构约束，请务必详细阅读：
* 🇬🇧 **[AGENTS.md (权威贡献者规则与架构)](AGENTS.md)**
* 🇨🇳 **[架构设计交接指南 (docs/architecture-handoff.md)](architecture-handoff.md)**

---

## 11. 作者与开源协议

`horae` 由个人独立开发者精心设计与维护。非常欢迎提出宝贵意见、报告缺陷或提交代码合并请求：

* **GitHub 仓库**：[https://github.com/zhaohang1205/horae](https://github.com/zhaohang1205/horae)
* **Gitee 国内镜像**：[https://gitee.com/zhao-hang1205/hora.git](https://gitee.com/zhao-hang1205/hora.git)
* **微信**：`hang_ace`（微信名 `zhaoh`）
* **联系邮箱**：[18658233987@163.com](mailto:18658233987@163.com)

### 开源许可证
本项目遵循 **GPL-3.0** 开源许可证。详细内容参见 [LICENSE](LICENSE)。
