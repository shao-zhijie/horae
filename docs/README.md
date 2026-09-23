# horae — GTD 终端任务管理器 (Terminal Task Manager)

<div align="center">

```
   ▄█    █▄     ▄██████▄     ▄████████    ▄████████    ▄████████
  ███    ███   ███    ███   ███    ███   ███    ███   ███    ███
  ███    ███   ███    ███   ███    ███   ███    ███   ███    █▀ 
 ▄███▄▄▄▄███▄▄ ███    ███  ▄███▄▄▄▄██▀   ███    ███  ▄███▄▄▄   
▀▀███▀▀▀▀███▀  ███    ███ ▀▀███▀▀▀▀▀   ▀███████████ ▀▀███▀▀▀   
  ███    ███   ███    ███ ▀███████████   ███    ███   ███    █▄ 
  ███    ███   ███    ███   ███    ███   ███    ███   ███    ███
  ███    █▀     ▀██████▀    ███    ███   ███    █▀    ██████████
                            ███    ███
```

**时间数据化的 GTD 终端任务管理器 · Time-Datafied GTD Terminal Task Manager**  
*Rust 单二进制 · 内置 SQLite · CLI 极速秒捕 · 沉浸式 Ratatui TUI · 零云端隐私所有权*

<p align="center">
  <a href="https://github.com/zhaohang1205/horae/actions/workflows/ci.yml"><img src="https://github.com/zhaohang1205/horae/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/zhaohang1205/horae/actions/workflows/release.yml"><img src="https://github.com/zhaohang1205/horae/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <a href="https://crates.io/crates/horae"><img src="https://img.shields.io/crates/v/horae.svg" alt="Crates.io"></a>
  <a href="https://www.npmjs.com/package/horae-cli"><img src="https://img.shields.io/npm/v/horae-cli.svg" alt="npm"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg" alt="License: GPL-3.0"></a>
</p>

### 🌐 文档导航 / Documentation Editions

**[ 🇨🇳 简体中文完整手册 (README_cn.md) ](README_cn.md)** &nbsp;•&nbsp; **[ 🇬🇧 English Full Manual (README_en.md) ](README_en.md)** &nbsp;•&nbsp; **[ 🛠️ Contributor Guide (AGENTS.md) ](AGENTS.md)**

</div>

---

## 🎯 核心设计理念 / Core Philosophy

`horae` 是一款纯 Rust 打造的现代化 GTD 终端生产力工具。与市面上传统的待办清单工具不同，`horae` 的底层基石是 **时间数据化 (Time-Datafication)**：

* 📜 **只写事件流 (Append-Only Event Sourcing)**：每次状态变更、排期修改、打卡完成均打上 UTC 毫秒级时间戳，追加至 `task_events` 审计流水线，永不覆盖。
* 🔍 **全生命周期可回溯**：任务不再是用完即弃的碎片便签，而是一份真实反映拖延周期、决策路径与个人效能的可追溯历史。
* 🔒 **本地私有与数据主权**：内置 SQLite 引擎，所有数据严格存储于本地（`~/.config/horae/`），零云端依赖、零账号注册、零数据遥测。

---

## 🖥️ 终端界面概览 / Terminal Interface

`horae` 采用 Catppuccin 正统极客配色（支持 Mocha 深色与 Latte 浅色主题一键切换），并支持 Nerd Font 矢量图标与纯 ASCII 单字符对齐优雅降级。

```text
┌─ HORAE [Today: 2026-09-23 农历八月十三 · 秋分] ────────────────────────────────────────────────────────┐
│ [Views]             │ [Tasks] Next Action (4)                   │ [Details] 买牛奶                    │
│ 1 Inbox         (2) │ > [!H] 买牛奶                  @home ~18:00 │ ID: a1b2c3d4                        │
│ 2 Next Action   (4) │ > [!M] 整理季度财务报表        @work ~+1d   │ Status: Next Action                 │
│ 3 Scheduled     (3) │ > [!L] 电话跟进客户需求        @calls       │ Tags: @home                         │
│ 4 Waiting       (1) │ > [!M] 阅读 Rust 异步编程专栏   @learning    │ Due: 2026-09-23 18:00 (Today)       │
│ 5 Someday       (5) │                                           │ Checklist: [1/3]                    │
│ 6 Reference     (8) │                                           │ ─────────────────────────────────── │
│ 7 Done         (42) │                                           │ [Timeline]                          │
│ 8 Archived          │                                           │ 10:15 Created (Inbox)               │
│ 9 Tags              │                                           │ 10:30 Clarified -> Next Action      │
│ 0 Quotes            │                                           │ 11:00 Added checklist (3 items)     │
└─────────────────────┴───────────────────────────────────────────┴─────────────────────────────────────┘
  a 快速捕获 · Enter 编辑 · x 完成 · w 等待 · v 可视多选 · P 番茄钟 · r 周回顾 · F5 主题 · F6 语言 · ? 帮助
```

---

## ✨ 核心特性矩阵 / Core Features

| 维度 / Dimension | 功能特性 / Capabilities | 解决的痛点 / Value Solved |
| :--- | :--- | :--- |
| **GTD 闭环流程** | `Inbox` → `Next`/`Scheduled`/`Waiting`/`Someday`/`Reference` → `Done`；双层删除（软删除归档 `Archive` + 永久粉碎 `Purge`）；独立今日/明日视图；周回顾向导 | 彻底告别混乱清单，严格践行 GTD 五步法（收集、厘清、组织、回顾、执行） |
| **一句话极速捕获** | Quick-add 语法：`标题 @tag ~time *rrule !priority`，支持双模式智能补全（语法参考指南卡片 vs 极速盲打） | 终端内 2 秒完成结构化录入，永不打断日常编码与思考心流 |
| **原生 RRULE 循环** | 自研零依赖循环引擎，支持 `DAILY/WEEKLY/MONTHLY/YEARLY` 全周期，支持 `*2w[1,3]`、`*m[1,-1]`、`*y[jan,jul]` 等高级简写 | 循环任务完成后自动按序重排，排序按有效发生期计算，周期习惯永不消失 |
| **双形态交互形态** | **CLI 极速秒捕**（`horae capture` / 别名 `c`）+ **闪念输入模式**（`horae -f` 录入即退）+ **Ratatui 响应式三栏 TUI**（三栏布局、可视多选、检查单层级） | 兼顾命令行脚本化的高效集成与全屏终端界面的沉浸式规划体验 |
| **智能专注推荐** | `horae do`（`horae focus`）动态加权算法：结合优先级（!high 万分统治力）、逾期严惩、截止日临近程度实时打分，输出当下唯一任务 | 终结选择困难症，直接聚焦当下最应该推进的关键核心事务 |
| **专注番茄钟体系** | 原生 Pomodoro：工作/休息倒计时环、连续打卡记录、自定义周期（`25;5;15;4`）、桌面通知与 Waybar 状态栏模块联动 | 任务管理与深度时间分块无缝整合，开箱即用无缝工作 |
| **零服务器手机生态** | **Syncthing 文件夹桥接**（手机写 `capture.txt`，电脑回写 `today.md` 快照与到期提醒）+ **ntfy 极轻量手机原生推送** + **飞书企业自建应用打通**（免公网 IP WebSocket 长连接与 Tasks v2 同步） | 零自建云端服务器、零外部 App 开发，让移动端采集与提醒完全私有闭环 |
| **文化历法与美学** | 纯内置农历引擎（1900-2100）与二十四节气历表（`horae calendar`），传统节假日 3 天与 1 天提前预警；金句灵感独立库；Catppuccin 美学 | 工具每天使用数百次，兼备中国传统文化关怀与极客美学体验 |

---

## 📚 规范化文档地图 / Documentation Index

为了方便不同角色的开发者与用户快速查阅，本项目文档分为以下规范化板块：

### 📖 用户使用手册 (User Manuals)
* 🇨🇳 **[简体中文用户完整手册 (README_cn.md)](README_cn.md)**  
  涵盖环境安装、CLI 命令全集、TUI 快捷键速查表、时间与循环高级语法、每日与每周 GTD 工作流、多 Profile 数据集管理、手机同步与飞书集成指南。
* 🇬🇧 **[English User Manual (README_en.md)](README_en.md)**  
  Complete documentation covering installation, full CLI & TUI command references, time & recurrence syntax, GTD workflows, mobile bridges, and enterprise integrations.

### 🏛️ 架构设计与技术规范 (Architecture & Deep Dive)
* 🇬🇧 **[贡献者与 Agent 权威指南 (AGENTS.md)](AGENTS.md)**  
  架构分层规范（Core / TUI / CLI）、事务与时间线审计缝（`repo::mutate`）、数据库迁移规则（0001-0014）、测试与 Clippy 零容忍规范。
* 🇨🇳 **[系统架构与工程交接 (docs/architecture-handoff.md)](docs/architecture-handoff.md)**  
  系统数据流转模型、事务原子性机制、只写事件时间线设计理念与模块依赖拓扑。
* 🇨🇳 **[领域词汇表与状态模型 (docs/CONTEXT.md)](docs/CONTEXT.md)**  
  任务领域实体、GTD 六大状态生命周期流转图、循环任务推进锚点机制与专业词汇定义。
* 🇨🇳 **[智能专注推荐算法设计 (docs/focus-algorithm.md)](docs/focus-algorithm.md)**  
  `horae do` 评分引擎：优先级加权、逾期梯度惩罚、临近截止期权重、创建时间决胜逻辑。
* 🇨🇳 **[Profile 多库与零服务器设计 (docs/design-profiles-and-cloud.md)](docs/design-profiles-and-cloud.md)**  
  本地工作/生活多数据集隔离方案、无服务器文件桥同步拓扑与移动端扩展设计。
* 🇨🇳 **[历史问题与边界备忘 (docs/issues.md)](docs/issues.md)**  
  已知边界、并发控制策略与长期架构演进规划。

### 🤖 AI 智能助手技能包 (AI Assistant Skill Pack)
* 🇨🇳 **[horae AI 技能包 (.agents/skills/horae/SKILL.md)](.agents/skills/horae/SKILL.md)**  
  专为 Claude、Gemini、Cursor、Antigravity 等 AI 编程助手打造的技能套件，支持系统体检预检、别名脚本自动安装、自然语言任务代管与周回顾辅助。

### 🔄 版本发布与演进 (Releases & Changelog)
* 🌐 **[更新变更日志 (docs/CHANGELOG.md)](docs/CHANGELOG.md)**  
  版本发布记录、新增特性、性能改进与问题修复详细流水。

---

## ⚡ 极速上手 / Quick Start

### 1. 快速安装 (Installation)

#### 方式一：npm 安装（推荐，无需 Rust 环境）
适用于任何安装了 Node.js 的环境，自动分发对应操作系统的预编译二进制文件：
```sh
# 免安装瞬时体验
npx horae-cli

# 全局安装（提供系统级 horae 命令）
npm install -g horae-cli
```

#### 方式二：Cargo 安装（Rust 官方渠道）
需 Rust 1.89+ 环境，SQLite 已捆绑内嵌：
```sh
# 从 crates.io 安装最新稳定版
cargo install horae

# 从 GitHub main 分支安装最新开发版
cargo install --git https://github.com/zhaohang1205/horae
```

### 2. 常用操作三部曲 (30-Second Onboarding)

```sh
# 1. 极速捕获任务（支持 @标签 ~时间 *循环 !优先级）
horae capture "完成系统架构设计报告 @work ~today 18:00 !high"

# 2. 智能推荐当前最该做的一件事并启动番茄钟
horae do --start

# 3. 启动交互式 TUI 沉浸式规划
horae
```

---

## 👤 作者与社区支持 / Author & Community

`horae` 由独立开发者进行持续维护。如果你在配置、使用中遇到任何问题，或有新的功能建议：

* **GitHub 仓库**：[https://github.com/zhaohang1205/horae](https://github.com/zhaohang1205/horae)
* **Gitee 国内镜像**：[https://gitee.com/zhao-hang1205/hora.git](https://gitee.com/zhao-hang1205/hora.git)
* **微信联系**：`hang_ace`（微信名 `zhaoh`）
* **反馈邮箱**：[18658233987@163.com](mailto:18658233987@163.com)

欢迎提交 Issue 与 Pull Request！

---

## 📄 开源许可证 / License

本项目采用 **GPL-3.0** 开源许可证。详情请参阅 [LICENSE](LICENSE) 文件。
