use super::{rrule_to_shorthand, QuickAdd};

/// 自然语言解析出的结构化任务。
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct NaturalTask {
    /// 净化后的任务标题
    pub title: String,
    /// 标签集合
    pub tags: Vec<String>,
    /// 排程时间（如 `明天 15:30`、`+2h`）
    pub scheduled_str: Option<String>,
    /// 截止时间（如带有“之前”、“前必须”的 deadline）
    pub due_str: Option<String>,
    /// 循环规则（RFC 5545 RRULE 或已验证的标准简写）
    pub rrule: Option<String>,
    /// 优先级（high / medium / low）
    pub priority: Option<String>,
}

impl NaturalTask {
    /// 转换为现有 horae 系统的 QuickAdd 结构（`time_str` 仅承载排程起点 `scheduled_str`，
    /// 软截止 `due_str` 由调用方独立映射到 `due` 字段，避免排程与截止双重写入）。
    pub fn to_quick_add(&self) -> QuickAdd {
        QuickAdd {
            title: self.title.clone(),
            tags: self.tags.clone(),
            time_str: self.scheduled_str.clone(),
            rrule: self.rrule.clone(),
            priority: self.priority.clone(),
        }
    }

    /// 转换为可直接送入 TUI 输入框或快速添加的符号语法
    pub fn to_quick_add_syntax(&self) -> String {
        let mut parts = vec![self.title.clone()];
        if let Some(time) = &self.scheduled_str {
            parts.push(format!("~{time}"));
        } else if let Some(due) = &self.due_str {
            parts.push(format!("~{due}"));
        }
        if let Some(rr) = &self.rrule {
            let shorthand = rrule_to_shorthand(rr);
            parts.push(format!("*{shorthand}"));
        }
        if let Some(p) = &self.priority {
            parts.push(format!("!{p}"));
        }
        for tag in &self.tags {
            parts.push(format!("@{tag}"));
        }
        parts.join(" ")
    }
}

/// 解析自然语言语音/口语输入为结构化任务对象。
pub fn parse_natural_language(input: &str) -> NaturalTask {
    let mut text = input.trim().to_string();
    if text.is_empty() {
        return NaturalTask::default();
    }

    // 0. 基础全角归一化
    text = normalize_text(&text);

    // 1. 抽取优先级（口语情感与程度词）
    let priority = extract_priority(&mut text);

    // 2. 抽取显式标签（如 "标签是工作"、"归类为生活"）
    let mut tags = extract_explicit_tags(&mut text);

    // 3. 抽取周期/循环表达（如 "每天"、"每周一"、"每个工作日"）
    let rrule = extract_recurrence(&mut text);

    // 4. 抽取时间与排程/截止槽位（如 "明天下午三点半"、"两小时后"、"下周五下班前"）
    let (scheduled_str, due_str) = extract_time(&mut text, rrule.as_deref());

    // 5. 若无显式标签，基于剩余文本内容智能推断系统预设标签（如买/超市 -> errands，开会/周报 -> work）
    if tags.is_empty() {
        tags = infer_implicit_tags(&text);
    }

    // 6. 净化剩余文本为干净的任务标题
    let title = clean_title(&text);

    NaturalTask {
        title,
        tags,
        scheduled_str,
        due_str,
        rrule,
        priority,
    }
}

/// 归一化文本（过滤外部工具日志、ANSI 控制码、全角符号转半角）
fn normalize_text(s: &str) -> String {
    // 过滤语音转写工具（如 voxtype）输出可能包含的日志与调试行
    let mut filtered_lines = Vec::new();
    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Loading audio file:")
            || trimmed.starts_with("Audio format:")
            || trimmed.starts_with("Processing ")
            || (trimmed.starts_with("20") && trimmed.contains("INFO"))
            || trimmed.contains("SenseVoice model")
            || trimmed.contains("transcription completed in")
        {
            continue;
        }
        filtered_lines.push(line);
    }
    let text = filtered_lines.join(" ");

    // 剔除可能的 ANSI 控制码（如 \x1b[32m 等）
    let clean_ansi = strip_ansi_escapes(&text);

    clean_ansi
        .replace('：', ":")
        .replace(['，', '。', '！', '？', '；', '、', '\t', '\n', '\r'], " ")
}

fn strip_ansi_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// 中文字符串转数字（支持 0~99）
pub fn parse_cn_number(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    if let Ok(n) = s.parse::<u32>() {
        return Some(n);
    }
    // 剥离口语前导 "零"（如 "零五" -> "五"）
    if s.starts_with('零') && s.len() > '零'.len_utf8() {
        return parse_cn_number(&s['零'.len_utf8()..]);
    }
    match s {
        "零" => Some(0),
        "一" | "壹" => Some(1),
        "二" | "两" | "贰" => Some(2),
        "三" | "叁" => Some(3),
        "四" | "肆" => Some(4),
        "五" | "伍" => Some(5),
        "六" | "陆" => Some(6),
        "七" | "柒" => Some(7),
        "八" | "捌" => Some(8),
        "九" | "玖" => Some(9),
        "十" | "拾" => Some(10),
        "十一" => Some(11),
        "十二" => Some(12),
        "十三" => Some(13),
        "十四" => Some(14),
        "十五" => Some(15),
        "十六" => Some(16),
        "十七" => Some(17),
        "十八" => Some(18),
        "十九" => Some(19),
        "二十" => Some(20),
        "二十一" => Some(21),
        "二十二" => Some(22),
        "二十三" => Some(23),
        "二十四" => Some(24),
        "二十五" => Some(25),
        "二十六" => Some(26),
        "二十七" => Some(27),
        "二十八" => Some(28),
        "二十九" => Some(29),
        "三十" => Some(30),
        "三十一" => Some(31),
        "四十" => Some(40),
        "四十五" => Some(45),
        "五十" => Some(50),
        "五十五" => Some(55),
        _ => {
            // 简单组装十位和个位
            if s.starts_with("二十") && s.len() > "二十".len() {
                let unit = &s["二十".len()..];
                return parse_cn_number(unit).map(|u| 20 + u);
            }
            if s.starts_with("三十") && s.len() > "三十".len() {
                let unit = &s["三十".len()..];
                return parse_cn_number(unit).map(|u| 30 + u);
            }
            if s.starts_with("四十") && s.len() > "四十".len() {
                let unit = &s["四十".len()..];
                return parse_cn_number(unit).map(|u| 40 + u);
            }
            if s.starts_with("五十") && s.len() > "五十".len() {
                let unit = &s["五十".len()..];
                return parse_cn_number(unit).map(|u| 50 + u);
            }
            if s.starts_with('十') && s.len() > '十'.len_utf8() {
                let unit = &s['十'.len_utf8()..];
                return parse_cn_number(unit).map(|u| 10 + u);
            }
            None
        }
    }
}

/// 在文本中查找 pattern，若 pattern 仅含 ASCII 字符，要求模式前后均满足词界约束（非 ASCII 字母数字/下划线）；
/// 若 pattern 含非 ASCII 字符（如中文），则使用标准子串匹配。大小写不敏感。
/// 返回 `Some((start_byte_idx, end_byte_idx))`。
fn find_word_boundary(text: &str, pat: &str) -> Option<(usize, usize)> {
    let is_ascii = pat.is_ascii();
    let pat_len = pat.len();
    if is_ascii {
        let ascii_text: String = text
            .chars()
            .map(|c| {
                if c.is_ascii() {
                    c.to_ascii_lowercase()
                } else {
                    c
                }
            })
            .collect();
        let lower_pat = pat.to_ascii_lowercase();

        let pat_starts_word = pat
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
        let pat_ends_word = pat
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');

        let mut search_from = 0;
        while let Some(rel) = ascii_text[search_from..].find(&lower_pat) {
            let start = search_from + rel;
            let end = start + pat_len;

            let prev_ok = if !pat_starts_word || start == 0 {
                true
            } else {
                let prev_c = text[..start].chars().next_back().unwrap();
                !prev_c.is_ascii_alphanumeric() && prev_c != '_'
            };
            let next_ok = if !pat_ends_word || end >= text.len() {
                true
            } else {
                let next_c = text[end..].chars().next().unwrap();
                !next_c.is_ascii_alphanumeric() && next_c != '_'
            };

            if prev_ok && next_ok {
                return Some((start, end));
            }
            search_from = start + 1;
            if search_from >= text.len() {
                break;
            }
        }
        None
    } else {
        text.find(pat).map(|pos| (pos, pos + pat_len))
    }
}

/// 英文数字/序数词转换（支持 0~99 及 1st~31st 等）
pub fn parse_en_number(s: &str) -> Option<u32> {
    let s = s.trim().to_ascii_lowercase();
    let s = s.trim_end_matches(['.', ',']);
    if let Ok(n) = s.parse::<u32>() {
        return Some(n);
    }
    for suffix in ["st", "nd", "rd", "th"] {
        if let Some(num_str) = s.strip_suffix(suffix) {
            if let Ok(n) = num_str.parse::<u32>() {
                return Some(n);
            }
        }
    }
    match s {
        "zero" => Some(0),
        "one" | "first" => Some(1),
        "two" | "second" => Some(2),
        "three" | "third" => Some(3),
        "four" | "fourth" => Some(4),
        "five" | "fifth" => Some(5),
        "six" | "sixth" => Some(6),
        "seven" | "seventh" => Some(7),
        "eight" | "eighth" => Some(8),
        "nine" | "ninth" => Some(9),
        "ten" | "tenth" => Some(10),
        "eleven" | "eleventh" => Some(11),
        "twelve" | "twelfth" => Some(12),
        "thirteen" | "thirteenth" => Some(13),
        "fourteen" | "fourteenth" => Some(14),
        "fifteen" | "fifteenth" => Some(15),
        "sixteen" | "sixteenth" => Some(16),
        "seventeen" | "seventeenth" => Some(17),
        "eighteen" | "eighteenth" => Some(18),
        "nineteen" | "nineteenth" => Some(19),
        "twenty" | "twentieth" => Some(20),
        "thirty" | "thirtieth" => Some(30),
        "forty" | "fortieth" => Some(40),
        "fifty" | "fiftieth" => Some(50),
        "sixty" | "sixtieth" => Some(60),
        "seventy" | "seventieth" => Some(70),
        "eighty" | "eightieth" => Some(80),
        "ninety" | "ninetieth" => Some(90),
        _ => {
            for (tens_str, tens_val) in [
                ("twenty", 20),
                ("thirty", 30),
                ("forty", 40),
                ("fifty", 50),
                ("sixty", 60),
                ("seventy", 70),
                ("eighty", 80),
                ("ninety", 90),
            ] {
                if let Some(rest) = s.strip_prefix(tens_str) {
                    let rest = rest.trim_start_matches(|c: char| c == '-' || c.is_whitespace());
                    if let Some(unit) = parse_en_number(rest) {
                        if unit < 10 {
                            return Some(tens_val + unit);
                        }
                    }
                }
            }
            None
        }
    }
}

/// 支持中文、英文以及纯数字字符串转换为数字
pub fn parse_any_number(s: &str) -> Option<u32> {
    parse_cn_number(s).or_else(|| parse_en_number(s))
}

/// 抽取优先级，并在文本中擦除匹配片段
fn extract_priority(text: &mut String) -> Option<String> {
    // 高优先级词表
    let high_patterns = [
        "最高优先级",
        "高优先级",
        "非常重要",
        "特别重要",
        "极其重要",
        "十分重要",
        "特别紧急",
        "非常紧急",
        "加急处理",
        "加急",
        "催得紧",
        "立刻马上",
        "马上立刻",
        "立刻",
        "马上",
        "紧急",
        "必须尽快",
        "必须",
        "尽快",
        "务必",
        "重点任务",
        "高优",
        "highest priority",
        "high priority",
        "very urgent",
        "extremely urgent",
        "extremely important",
        "very important",
        "urgent",
        "critical",
        "asap",
        "must do",
        "!high",
        "!1",
        "p1",
    ];

    for pat in high_patterns {
        if let Some((start, end)) = find_word_boundary(text, pat) {
            erase_range(text, start, end);
            return Some("high".to_string());
        }
    }

    // 低优先级词表
    let low_patterns = [
        "最低优先级",
        "低优先级",
        "不着急",
        "不急",
        "有空再做",
        "有空做",
        "有空的时候",
        "有空时",
        "缓办",
        "低优",
        "抽空",
        "lowest priority",
        "low priority",
        "not urgent",
        "when you have time",
        "whenever",
        "someday",
        "!low",
        "!3",
        "p3",
    ];

    for pat in low_patterns {
        if let Some((start, end)) = find_word_boundary(text, pat) {
            erase_range(text, start, end);
            return Some("low".to_string());
        }
    }

    // 中优先级词表
    let med_patterns = [
        "中等优先级",
        "中优先级",
        "一般重要",
        "中优",
        "medium priority",
        "normal priority",
        "moderate",
        "!medium",
        "!2",
        "p2",
    ];

    for pat in med_patterns {
        if let Some((start, end)) = find_word_boundary(text, pat) {
            erase_range(text, start, end);
            return Some("medium".to_string());
        }
    }

    None
}

/// 将指定字节范围替换为空格
fn erase_range(s: &mut String, start: usize, end: usize) {
    let mut new_s = String::with_capacity(s.len());
    new_s.push_str(&s[..start]);
    for _ in 0..(end - start) {
        new_s.push(' ');
    }
    new_s.push_str(&s[end..]);
    *s = new_s;
}

/// 抽取显式标签，如 "标签是工作"、"归类为生活"、"@work"
fn extract_explicit_tags(text: &mut String) -> Vec<String> {
    let mut tags = Vec::new();

    // 匹配类似 "标签[是为: ]xxx" 或 "归类[到为: ]xxx" 或 "tag is xxx"
    let prefixes = [
        "标签是",
        "标签为",
        "标签:",
        "标签 ",
        "分类是",
        "分类为",
        "分类:",
        "分类 ",
        "归类到",
        "归类为",
        "归类:",
        "tagged as: ",
        "tagged as ",
        "tagged: ",
        "tagged ",
        "tag is: ",
        "tag is ",
        "tag: ",
        "tag ",
        "category is: ",
        "category is ",
        "category: ",
        "category ",
    ];

    for prefix in prefixes {
        while let Some((pos, end_prefix)) = find_word_boundary(text, prefix) {
            let rest = &text[end_prefix..];
            // 取到空格或结尾
            let end_offset = rest.find(' ').unwrap_or(rest.len());
            let mut tag_word = rest[..end_offset]
                .trim_matches(|c: char| c.is_whitespace() || ",，。！!？?、:：;；".contains(c));
            // 剥离尾部诸如 "分类"、"标签" 等词缀
            if let Some(stripped) = tag_word.strip_suffix("分类") {
                tag_word = stripped;
            } else if let Some(stripped) = tag_word.strip_suffix("标签") {
                tag_word = stripped;
            }

            if !tag_word.is_empty() {
                let canonical = canonicalize_tag(tag_word);
                if !tags.contains(&canonical) {
                    tags.push(canonical);
                }
            }
            erase_range(text, pos, end_prefix + end_offset);
        }
    }

    // 匹配形如 `@tag` 的标准语法
    while let Some(at_pos) = text.find('@') {
        let rest = &text[at_pos + 1..];
        let end_offset = rest.find(' ').unwrap_or(rest.len());
        let tag_word = rest[..end_offset]
            .trim_matches(|c: char| c.is_whitespace() || ",，。！!？?、:：;；".contains(c));
        if !tag_word.is_empty() {
            let canonical = canonicalize_tag(tag_word);
            if !tags.contains(&canonical) {
                tags.push(canonical);
            }
        }
        erase_range(text, at_pos, at_pos + 1 + end_offset);
    }

    tags
}

/// 将中文日常标签映射为系统规范标签名
fn canonicalize_tag(s: &str) -> String {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "工作" | "上班" | "业务" | "work" => "work".to_string(),
        "生活" | "家里" | "家庭" | "家务" | "个人" | "home" | "personal" | "life" | "house" => {
            "home".to_string()
        }
        "学习" | "阅读" | "读书" | "复习" | "learning" | "study" | "reading" | "read" => {
            "learning".to_string()
        }
        "跑腿" | "采购" | "购物" | "超市" | "办事" | "errands" | "shopping" | "groceries"
        | "grocery" => "errands".to_string(),
        "电话" | "通话" | "联系" | "calls" | "call" | "phone" => "calls".to_string(),
        "电脑" | "技术" | "折腾" | "computer" | "tech" | "coding" => "computer".to_string(),
        "专注" | "深度" | "focus" | "deep" => "focus".to_string(),
        "金句" | "语录" | "摘抄" | "quote" => "quote".to_string(),
        other => other.to_string(),
    }
}

/// 智能推断系统预设标签（仅在用户未显式指定任何标签时触发）
fn infer_implicit_tags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();

    // work 触发词
    let work_words = [
        "开会",
        "会议",
        "周报",
        "月报",
        "财报",
        "报告",
        "分析",
        "材料",
        "总结",
        "设计",
        "文档",
        "项目",
        "客户",
        "需求",
        "发版",
        "上线",
        "代码",
        "评审",
        "bug",
        "pr",
        "commit",
        "汇报",
        "方案",
        "架构",
        "部署",
        "重构",
        "工时",
        "排期",
        "例会",
        "meeting",
        "meetings",
        "standup",
        "sync",
        "presentation",
        "report",
        "sprint",
        "deploy",
        "deployment",
        "release",
        "architecture",
        "refactor",
        "roadmap",
        "quarterly",
        "agenda",
    ];
    for w in work_words {
        if find_word_boundary(text, w).is_some() {
            tags.push("work".to_string());
            break;
        }
    }

    // errands 触发词
    let errands_words = [
        "超市",
        "买",
        "采购",
        "取快递",
        "寄件",
        "还款",
        "交房租",
        "缴费",
        "水电费",
        "银行",
        "办卡",
        "充值",
        "groceries",
        "supermarket",
        "pharmacy",
        "post office",
        "package",
        "parcel",
        "rent",
        "utility",
        "utilities",
        "bill",
        "buy",
        "purchase",
    ];
    for w in errands_words {
        if find_word_boundary(text, w).is_some() {
            tags.push("errands".to_string());
            break;
        }
    }

    // calls 触发词
    let calls_words = [
        "打电话",
        "致电",
        "微信电话",
        "语音电话",
        "电话联系",
        "回电",
        "通个电话",
        "call",
        "phone",
        "zoom",
        "facetime",
    ];
    for w in calls_words {
        if find_word_boundary(text, w).is_some() {
            tags.push("calls".to_string());
            break;
        }
    }

    // learning 触发词
    let learning_words = [
        "背单词",
        "看书",
        "读书",
        "课程",
        "教程",
        "阅读",
        "做题",
        "考研",
        "学习",
        "read",
        "book",
        "course",
        "study",
        "exam",
        "tutorial",
        "homework",
        "lecture",
    ];
    for w in learning_words {
        if find_word_boundary(text, w).is_some() {
            tags.push("learning".to_string());
            break;
        }
    }

    // home 触发词
    let home_words = [
        "打扫",
        "做饭",
        "洗衣",
        "倒垃圾",
        "遛狗",
        "喂猫",
        "家务",
        "整理房间",
        "换灯泡",
        "clean",
        "cook",
        "laundry",
        "trash",
        "garbage",
        "dishes",
        "walk the dog",
        "feed the cat",
        "chore",
        "chores",
    ];
    for w in home_words {
        if find_word_boundary(text, w).is_some() {
            tags.push("home".to_string());
            break;
        }
    }

    tags
}

/// 判断字符是否为可组成时刻/日期的中文数字或阿拉伯数字
fn is_number_char(c: char) -> bool {
    c.is_ascii_digit() || "零一二两三四五六七八九十壹贰叁肆伍陆柒捌玖拾".contains(c)
}

/// 抽取周期/重复规则，并在文本中擦除匹配片段
fn extract_recurrence(text: &mut String) -> Option<String> {
    // 0. 英文每月特定日循环（必须先于 "every month" 基础规则匹配）：every month on the 15th, every month on 1st, monthly on the 15th
    let en_month_prefixes = [
        "every month on the ",
        "every month on ",
        "monthly on the ",
        "monthly on ",
    ];
    for prefix in en_month_prefixes {
        if let Some((pos, end_prefix)) = find_word_boundary(text, prefix) {
            let after = &text[end_prefix..];
            let token: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-')
                .collect();
            if let Some(day_num) = parse_any_number(&token) {
                if (1..=31).contains(&day_num) {
                    let total_len = (end_prefix - pos) + token.len();
                    erase_range(text, pos, pos + total_len);
                    return Some(format!("FREQ=MONTHLY;BYMONTHDAY={day_num}"));
                }
            }
        }
    }

    // 1. 英文每周特定星期循环（必须先于 "every week" 基础规则匹配）：every Monday, every week on Wednesday, every Mon
    let en_week_prefixes = ["every week on ", "every "];
    let en_weekdays = [
        ("monday", "MO"),
        ("mon", "MO"),
        ("tuesday", "TU"),
        ("tues", "TU"),
        ("tue", "TU"),
        ("wednesday", "WE"),
        ("wed", "WE"),
        ("thursday", "TH"),
        ("thur", "TH"),
        ("thurs", "TH"),
        ("thu", "TH"),
        ("friday", "FR"),
        ("fri", "FR"),
        ("saturday", "SA"),
        ("sat", "SA"),
        ("sunday", "SU"),
        ("sun", "SU"),
    ];
    for prefix in en_week_prefixes {
        if let Some((pos, end_prefix)) = find_word_boundary(text, prefix) {
            let after = &text[end_prefix..];
            for (day_name, byday) in en_weekdays {
                if let Some((d_start, d_end)) = find_word_boundary(after, day_name) {
                    if d_start == 0 {
                        let total_len = (end_prefix - pos) + d_end;
                        erase_range(text, pos, pos + total_len);
                        return Some(format!("FREQ=WEEKLY;BYDAY={byday}"));
                    }
                }
            }
        }
    }

    // 2. 英文多天循环：every 2 days, every 3 days, every three days
    if let Some((pos, end_prefix)) = find_word_boundary(text, "every ") {
        let after = &text[end_prefix..];
        let first_c = after.chars().next().unwrap_or(' ');
        let num_str: String = if first_c.is_ascii_digit() {
            after.chars().take_while(|c| c.is_ascii_digit()).collect()
        } else {
            after
                .chars()
                .take_while(|c| c.is_alphabetic() || *c == '-')
                .collect()
        };
        if !num_str.is_empty() {
            let rest_after_num = after[num_str.len()..].trim_start();
            let lower_rest = rest_after_num.to_ascii_lowercase();
            if lower_rest.starts_with("days") || lower_rest.starts_with("day") {
                if let Some(n) = parse_any_number(&num_str) {
                    if (2..=365).contains(&n) {
                        let unit_len = if lower_rest.starts_with("days") { 4 } else { 3 };
                        let spaces = after[num_str.len()..].len() - rest_after_num.len();
                        let total_len = (end_prefix - pos) + num_str.len() + spaces + unit_len;
                        erase_range(text, pos, pos + total_len);
                        return Some(format!("FREQ=DAILY;INTERVAL={n}"));
                    }
                }
            }
        }
    }

    // 3. 基础天/工作日/周末/月/年循环
    let fixed_rules = [
        ("每个工作日", "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"),
        ("每工作日", "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"),
        ("每个周末", "FREQ=WEEKLY;BYDAY=SA,SU"),
        ("每周末", "FREQ=WEEKLY;BYDAY=SA,SU"),
        ("每天", "FREQ=DAILY"),
        ("天天", "FREQ=DAILY"),
        ("每日", "FREQ=DAILY"),
        ("every weekday", "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"),
        ("every work day", "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"),
        ("on weekdays", "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"),
        ("weekdays", "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"),
        ("every weekend", "FREQ=WEEKLY;BYDAY=SA,SU"),
        ("on weekends", "FREQ=WEEKLY;BYDAY=SA,SU"),
        ("weekends", "FREQ=WEEKLY;BYDAY=SA,SU"),
        ("every other day", "FREQ=DAILY;INTERVAL=2"),
        ("every day", "FREQ=DAILY"),
        ("everyday", "FREQ=DAILY"),
        ("daily", "FREQ=DAILY"),
        ("every week", "FREQ=WEEKLY"),
        ("weekly", "FREQ=WEEKLY"),
        ("every month", "FREQ=MONTHLY"),
        ("monthly", "FREQ=MONTHLY"),
        ("every year", "FREQ=YEARLY"),
        ("yearly", "FREQ=YEARLY"),
        ("annually", "FREQ=YEARLY"),
    ];

    for (pat, rrule) in fixed_rules {
        if let Some((start, end)) = find_word_boundary(text, pat) {
            erase_range(text, start, end);
            return Some(rrule.to_string());
        }
    }

    // 裸 "工作日" 仅在非合成词（如非 "工作日记/工作日志/下个工作日"）时才识别为工作日循环
    if let Some(pos) = text.find("工作日") {
        let prev_char = text[..pos].chars().next_back();
        let next_char = text[pos + "工作日".len()..].chars().next();
        let bad_prev = prev_char.is_some_and(|c| "个本下上这那".contains(c));
        let bad_next = next_char.is_some_and(|c| "记志程历休".contains(c));
        if !bad_prev && !bad_next {
            erase_range(text, pos, pos + "工作日".len());
            return Some("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR".to_string());
        }
    }

    // 2. 隔天/多天循环：每隔一天、每隔两天、每两天、每3天
    if let Some(pos) = text.find("每隔") {
        let after = &text[pos + "每隔".len()..];
        let num_str: String = after.chars().take_while(|c| is_number_char(*c)).collect();
        if !num_str.is_empty() && after[num_str.len()..].starts_with('天') {
            if let Some(n) = parse_cn_number(&num_str) {
                if (1..=365).contains(&n) {
                    let total_len = "每隔".len() + num_str.len() + '天'.len_utf8();
                    erase_range(text, pos, pos + total_len);
                    return Some(format!("FREQ=DAILY;INTERVAL={}", n + 1));
                }
            }
        }
    }
    if let Some(pos) = text.find('每') {
        let after = &text[pos + '每'.len_utf8()..];
        let num_str: String = after.chars().take_while(|c| is_number_char(*c)).collect();
        if !num_str.is_empty() && after[num_str.len()..].starts_with('天') {
            if let Some(n) = parse_cn_number(&num_str) {
                if (2..=365).contains(&n) {
                    let total_len = '每'.len_utf8() + num_str.len() + '天'.len_utf8();
                    erase_range(text, pos, pos + total_len);
                    return Some(format!("FREQ=DAILY;INTERVAL={n}"));
                }
            }
        }
    }

    // 3. 每周循环：每周一、每个周三、每周五、每周1等
    let week_prefixes = ["每周", "每个星期", "每星期", "每礼拜", "每个礼拜", "每个周"];
    for prefix in week_prefixes {
        if let Some(pos) = text.find(prefix) {
            let after = &text[pos + prefix.len()..];
            if let Some(wd_char) = after.chars().next() {
                if let Some(byday) = cn_weekday_code(wd_char) {
                    let len = prefix.len() + wd_char.len_utf8();
                    erase_range(text, pos, pos + len);
                    return Some(format!("FREQ=WEEKLY;BYDAY={byday}"));
                }
            }
        }
    }

    // 4. 每月循环：每月15号、每个月十号、每个月1号
    let month_prefixes = ["每个月", "每月"];
    for prefix in month_prefixes {
        if let Some(pos) = text.find(prefix) {
            let after = &text[pos + prefix.len()..];
            // 找随后的数字+号/日
            if let Some(end_idx) = after.find(['号', '日']) {
                let day_str = after[..end_idx].trim();
                if let Some(day_num) = parse_cn_number(day_str) {
                    if (1..=31).contains(&day_num) {
                        let total_len = prefix.len() + end_idx + '号'.len_utf8();
                        erase_range(text, pos, pos + total_len);
                        return Some(format!("FREQ=MONTHLY;BYMONTHDAY={day_num}"));
                    }
                }
            }
        }
    }

    // 5. 每年循环：每年10月24号
    if let Some(pos) = text.find("每年") {
        erase_range(text, pos, pos + "每年".len());
        return Some("FREQ=YEARLY".to_string());
    }

    None
}

/// 映射单字符星期为 RFC 5545 BYDAY 代码
fn cn_weekday_code(c: char) -> Option<&'static str> {
    match c {
        '一' | '1' => Some("MO"),
        '二' | '2' => Some("TU"),
        '三' | '3' => Some("WE"),
        '四' | '4' => Some("TH"),
        '五' | '5' => Some("FR"),
        '六' | '6' => Some("SA"),
        '日' | '天' | '7' | '0' => Some("SU"),
        _ => None,
    }
}

/// 抽取时间表达（如 "明天下午三点半"、"两小时后"、"下周五下班前"）
/// 返回 `(scheduled_str, due_str)`
fn extract_time(text: &mut String, rrule: Option<&str>) -> (Option<String>, Option<String>) {
    // 1. 相对时长表达：两小时后、半小时后、30分钟后、三天后
    if let Some((time_val, is_deadline, pos, len)) = extract_relative_time(text) {
        erase_range(text, pos, pos + len);
        if is_deadline {
            return (None, Some(time_val));
        } else {
            return (Some(time_val), None);
        }
    }

    // 2. 绝对日期/相对日 + 时刻
    if let Some((time_val, is_deadline)) = extract_calendar_time(text, rrule) {
        if is_deadline {
            return (None, Some(time_val));
        } else {
            return (Some(time_val), None);
        }
    }

    (None, None)
}

/// 抽取相对时间（如 "两小时后"、"半小时后"、"in 2 hours"、"in 30 minutes"）
fn extract_relative_time(text: &str) -> Option<(String, bool, usize, usize)> {
    // 1. 英文前置形式：in 2 hours, in 30 minutes, in half an hour, in 3 days
    if let Some((pos, end_prefix)) = find_word_boundary(text, "in ") {
        let after = &text[end_prefix..];
        let lower_after = after.to_ascii_lowercase();

        if lower_after.starts_with("half an hour") {
            let len = (end_prefix - pos) + "half an hour".len();
            return Some(("+30m".to_string(), false, pos, len));
        }
        if lower_after.starts_with("a quarter of an hour") {
            let len = (end_prefix - pos) + "a quarter of an hour".len();
            return Some(("+15m".to_string(), false, pos, len));
        }
        if lower_after.starts_with("a quarter hour") {
            let len = (end_prefix - pos) + "a quarter hour".len();
            return Some(("+15m".to_string(), false, pos, len));
        }
        if lower_after.starts_with("an hour") || lower_after.starts_with("a hour") {
            let unit_len = if lower_after.starts_with("an hour") {
                7
            } else {
                6
            };
            let len = (end_prefix - pos) + unit_len;
            return Some(("+1h".to_string(), false, pos, len));
        }
        if lower_after.starts_with("a day") {
            let len = (end_prefix - pos) + 5;
            return Some(("+1d".to_string(), false, pos, len));
        }
        if lower_after.starts_with("a week") {
            let len = (end_prefix - pos) + 6;
            return Some(("+1w".to_string(), false, pos, len));
        }

        let token: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-')
            .collect();
        if !token.is_empty() {
            if let Some(n) = parse_any_number(&token) {
                let rest_after = after[token.len()..].trim_start();
                let lower_rest = rest_after.to_ascii_lowercase();
                let spaces = after[token.len()..].len() - rest_after.len();

                for (unit_word, code) in [
                    ("hours", 'h'),
                    ("hour", 'h'),
                    ("hrs", 'h'),
                    ("hr", 'h'),
                    ("minutes", 'm'),
                    ("minute", 'm'),
                    ("mins", 'm'),
                    ("min", 'm'),
                    ("days", 'd'),
                    ("day", 'd'),
                    ("weeks", 'w'),
                    ("week", 'w'),
                ] {
                    if lower_rest.starts_with(unit_word) {
                        let total_len = (end_prefix - pos) + token.len() + spaces + unit_word.len();
                        return Some((format!("+{n}{code}"), false, pos, total_len));
                    }
                }
            }
        }
    }

    // 2. 中文后置形式（"两小时后"、"半小时后"、"3天后"）及英文后置形式（"2 hours later"）
    let suffixes = ["后提醒我", "之后", "以后", "后", " later"];

    for suffix in suffixes {
        if let Some(suf_pos) = text.to_ascii_lowercase().find(suffix) {
            let before = &text[..suf_pos];
            let lower_before = before.to_ascii_lowercase();
            // 支持 "半小时"、"半个钟头"、"half an hour"
            if before.ends_with("半小时")
                || before.ends_with("半个钟头")
                || lower_before.ends_with("half an hour")
            {
                let unit_len = if before.ends_with("半小时") {
                    "半小时".len()
                } else if before.ends_with("半个钟头") {
                    "半个钟头".len()
                } else {
                    "half an hour".len()
                };
                let pos = suf_pos - unit_len;
                let len = unit_len + suffix.len();
                return Some(("+30m".to_string(), false, pos, len));
            }

            // 寻找单位：小时 / 分钟 / 分 / 天 / 周 / 星期 / hours / minutes / days / weeks
            for (unit_word, code) in [
                ("小时", 'h'),
                ("个钟头", 'h'),
                ("hours", 'h'),
                ("hour", 'h'),
                ("hrs", 'h'),
                ("hr", 'h'),
                ("分钟", 'm'),
                ("分", 'm'),
                ("minutes", 'm'),
                ("minute", 'm'),
                ("mins", 'm'),
                ("min", 'm'),
                ("天", 'd'),
                ("days", 'd'),
                ("day", 'd'),
                ("周", 'w'),
                ("个星期", 'w'),
                ("weeks", 'w'),
                ("week", 'w'),
            ] {
                if let Some(unit_idx) = lower_before.rfind(unit_word) {
                    let num_candidate = before[..unit_idx].trim_end();
                    // 取前方的中文数字或阿拉伯数字或英文数字
                    let num_start = num_candidate
                        .rfind(|c: char| !c.is_alphanumeric() && c != '-' && !is_number_char(c))
                        .map(|idx| idx + num_candidate[idx..].chars().next().unwrap().len_utf8())
                        .unwrap_or(0);
                    let num_part = &num_candidate[num_start..];
                    if let Some(n) = parse_any_number(num_part) {
                        let pos = num_start;
                        let len = suf_pos + suffix.len() - pos;
                        let res = format!("+{n}{code}");
                        return Some((res, false, pos, len));
                    }
                }
            }
        }
    }

    None
}

/// 抽取英文显式 "Oct 25" / "October 25th" / "25th of October" 结构
fn find_explicit_english_month_day(text: &str) -> Option<(String, usize, usize)> {
    const EN_MONTHS: &[(&str, u32)] = &[
        ("january", 1),
        ("jan", 1),
        ("february", 2),
        ("feb", 2),
        ("march", 3),
        ("mar", 3),
        ("april", 4),
        ("apr", 4),
        ("may", 5),
        ("june", 6),
        ("jun", 6),
        ("july", 7),
        ("jul", 7),
        ("august", 8),
        ("aug", 8),
        ("september", 9),
        ("sept", 9),
        ("sep", 9),
        ("october", 10),
        ("oct", 10),
        ("november", 11),
        ("nov", 11),
        ("december", 12),
        ("dec", 12),
    ];

    // 格式 1: Month Day (如 "Oct 25", "October 25th")
    for (m_name, m_num) in EN_MONTHS {
        if let Some((m_start, m_end)) = find_word_boundary(text, m_name) {
            let after = &text[m_end..];
            let after_trimmed = after.trim_start();
            let spaces = after.len() - after_trimmed.len();
            let day_token: String = after_trimmed
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-')
                .collect();
            if let Some(day_num) = parse_any_number(&day_token) {
                if (1..=31).contains(&day_num) {
                    let mut start = m_start;
                    let mut len = (m_end - m_start) + spaces + day_token.len();
                    // 吸收前置的 "on the " 或 "on "
                    let before = &text[..start];
                    if before.to_ascii_lowercase().ends_with("on the ") {
                        start -= "on the ".len();
                        len += "on the ".len();
                    } else if before.to_ascii_lowercase().ends_with("on ") {
                        start -= "on ".len();
                        len += "on ".len();
                    }
                    return Some((format!("{:02}-{:02}", m_num, day_num), start, len));
                }
            }
        }
    }

    // 格式 2: Day of Month (如 "25th of October")
    for (m_name, m_num) in EN_MONTHS {
        if let Some((m_start, m_end)) = find_word_boundary(text, m_name) {
            let before = &text[..m_start];
            let lower_before = before.to_ascii_lowercase();
            let of_str = if lower_before.ends_with(" of the ") {
                Some(" of the ")
            } else if lower_before.ends_with(" of ") {
                Some(" of ")
            } else {
                None
            };
            if let Some(of_pat) = of_str {
                let before_of = &before[..before.len() - of_pat.len()];
                let day_token: String = before_of
                    .chars()
                    .rev()
                    .take_while(|c| c.is_alphanumeric() || *c == '-')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                if let Some(day_num) = parse_any_number(&day_token) {
                    if (1..=31).contains(&day_num) {
                        let mut start = before_of.len() - day_token.len();
                        let mut len = m_end - start;
                        let before_day = &text[..start];
                        if before_day.to_ascii_lowercase().ends_with("on the ") {
                            start -= "on the ".len();
                            len += "on the ".len();
                        } else if before_day.to_ascii_lowercase().ends_with("on ") {
                            start -= "on ".len();
                            len += "on ".len();
                        }
                        return Some((format!("{:02}-{:02}", m_num, day_num), start, len));
                    }
                }
            }
        }
    }

    None
}

/// 抽取显式 "X月Y号/日" 结构，返回 `(canonical_mm_dd, start_pos, byte_len)`
fn find_explicit_month_day(text: &str) -> Option<(String, usize, usize)> {
    for (m_pos, _) in text.match_indices('月') {
        let before = &text[..m_pos];
        let m_start = before
            .rfind(|c: char| !is_number_char(c))
            .map(|idx| idx + before[idx..].chars().next().unwrap().len_utf8())
            .unwrap_or(0);
        let m_str = &before[m_start..];
        let Some(month) = parse_cn_number(m_str) else {
            continue;
        };
        if !(1..=12).contains(&month) {
            continue;
        }
        let after_m = &text[m_pos + '月'.len_utf8()..];
        let d_str: String = after_m.chars().take_while(|c| is_number_char(*c)).collect();
        if d_str.is_empty() {
            continue;
        }
        let after_d = &after_m[d_str.len()..];
        let suffix_len = if after_d.starts_with('号') {
            '号'.len_utf8()
        } else if after_d.starts_with('日') {
            '日'.len_utf8()
        } else {
            continue;
        };
        let Some(day) = parse_cn_number(&d_str) else {
            continue;
        };
        if (1..=31).contains(&day) {
            let total_len = (m_pos - m_start) + '月'.len_utf8() + d_str.len() + suffix_len;
            return Some((format!("{:02}-{:02}", month, day), m_start, total_len));
        }
    }
    None
}

/// 抽取日历与时刻组合（如 "明天下午三点半"、"今天晚上8点"、"下周六的下午5点"、"tomorrow at 9am"、"by Friday at 6pm"）
fn extract_calendar_time(text: &mut String, _rrule: Option<&str>) -> Option<(String, bool)> {
    // 0. 显式 "X月Y号/日" 或 "Oct 25" / "25th of October" 日期锚点
    let explicit_date =
        find_explicit_month_day(text).or_else(|| find_explicit_english_month_day(text));
    if let Some((canonical_day, pos, pat_len)) = explicit_date {
        let after_pos = pos + pat_len;
        let rest = text[after_pos..].to_string();

        let (clock_str, clock_len, is_deadline) = parse_clock_time(&rest, "");
        if clock_len > 0 {
            let total_len = pat_len + clock_len;
            erase_range(text, pos, pos + total_len);
            let final_time_str = if let Some(clock) = clock_str {
                format!("{canonical_day} {clock}")
            } else {
                canonical_day
            };
            return Some((final_time_str, is_deadline));
        }

        if let Some((clock, is_deadline, rel_pos, clock_len)) = find_clock_time_in_text(&rest) {
            let abs_clock_pos = after_pos + rel_pos;
            erase_range(text, abs_clock_pos, abs_clock_pos + clock_len);
            erase_range(text, pos, pos + pat_len);
            let final_time_str = format!("{canonical_day} {clock}");
            return Some((final_time_str, is_deadline));
        }

        let rest_trimmed = rest.trim_start();
        let is_deadline = rest_trimmed.starts_with("前")
            || rest_trimmed.starts_with("之前")
            || rest_trimmed.to_ascii_lowercase().starts_with("before");
        let deadline_len = if rest_trimmed.starts_with("之前") {
            (rest.len() - rest_trimmed.len()) + "之前".len()
        } else if rest_trimmed.starts_with("前") {
            (rest.len() - rest_trimmed.len()) + "前".len()
        } else if rest_trimmed.to_ascii_lowercase().starts_with("before") {
            (rest.len() - rest_trimmed.len()) + 6
        } else {
            0
        };
        erase_range(text, pos, pos + pat_len + deadline_len);
        return Some((canonical_day, is_deadline));
    }

    // 常见日期前缀（中英双语）
    let date_anchors = [
        ("day after tomorrow", "day after tomorrow", 2),
        ("tomorrow morning", "tomorrow", 1),
        ("tomorrow afternoon", "tomorrow", 1),
        ("tomorrow evening", "tomorrow", 1),
        ("tomorrow night", "tomorrow", 1),
        ("tomorrow", "tomorrow", 1),
        ("tonight", "today", 0),
        ("today", "today", 0),
        ("大后天", "大后天", 3),
        ("后天", "后天", 2),
        ("明早", "明天", 1),
        ("明晚", "明天", 1),
        ("明天", "明天", 1),
        ("今早", "今天", 0),
        ("今晚", "今天", 0),
        ("今日", "今天", 0),
        ("今天", "今天", 0),
        ("next monday", "next mon", 0),
        ("next tuesday", "next tue", 0),
        ("next wednesday", "next wed", 0),
        ("next thursday", "next thu", 0),
        ("next friday", "next fri", 0),
        ("next saturday", "next sat", 0),
        ("next sunday", "next sun", 0),
        ("this monday", "mon", 0),
        ("this tuesday", "tue", 0),
        ("this wednesday", "wed", 0),
        ("this thursday", "thu", 0),
        ("this friday", "fri", 0),
        ("this saturday", "sat", 0),
        ("this sunday", "sun", 0),
        ("monday", "mon", 0),
        ("tuesday", "tue", 0),
        ("wednesday", "wed", 0),
        ("thursday", "thu", 0),
        ("friday", "fri", 0),
        ("saturday", "sat", 0),
        ("sunday", "sun", 0),
        ("下周一", "下周一", 0),
        ("下周二", "下周二", 0),
        ("下周三", "下周三", 0),
        ("下周四", "下周四", 0),
        ("下周五", "下周五", 0),
        ("下周六", "下周六", 0),
        ("下周日", "下周日", 0),
        ("下周天", "下周日", 0),
        ("周一", "周一", 0),
        ("周二", "周二", 0),
        ("周三", "周三", 0),
        ("周四", "周四", 0),
        ("周五", "周五", 0),
        ("周六", "周六", 0),
        ("周日", "周日", 0),
        ("周天", "周日", 0),
        ("星期一", "周一", 0),
        ("星期二", "周二", 0),
        ("星期三", "周三", 0),
        ("星期四", "周四", 0),
        ("星期五", "周五", 0),
        ("星期六", "周六", 0),
        ("星期日", "周日", 0),
        ("星期天", "周日", 0),
    ];

    for (pat, canonical_day, _offset) in date_anchors {
        if let Some((mut pos, end_pos)) = find_word_boundary(text, pat) {
            let mut is_deadline_lead = false;
            // 检查日期前是否有 "due by ", "due ", "before ", "by ", "on "
            let before = &text[..pos];
            let lower_before = before.to_ascii_lowercase();
            if lower_before.ends_with("due by ") {
                pos -= 7;
                is_deadline_lead = true;
            } else if lower_before.ends_with("due ") {
                pos -= 4;
                is_deadline_lead = true;
            } else if lower_before.ends_with("before ") {
                pos -= 7;
                is_deadline_lead = true;
            } else if lower_before.ends_with("by ") {
                pos -= 3;
                is_deadline_lead = true;
            } else if lower_before.ends_with("on ") {
                pos -= 3;
            }

            let after_pos = end_pos;
            let rest = text[after_pos..].to_string();

            // 1. 检查是否有紧随的时刻或时间段（支持 "tomorrow at 9am"、"下周六的下午5点" 等）
            let (clock_str, clock_len, is_deadline_rest) = parse_clock_time(&rest, pat);
            if clock_len > 0 {
                let total_len = (end_pos - pos) + clock_len;
                erase_range(text, pos, pos + total_len);
                let final_time_str = if let Some(clock) = clock_str {
                    format!("{canonical_day} {clock}")
                } else {
                    canonical_day.to_string()
                };
                return Some((final_time_str, is_deadline_lead || is_deadline_rest));
            }

            // 2. 检查 rest 中是否包含隔开的独立时刻
            if let Some((clock, is_deadline_rest, rel_pos, clock_len)) =
                find_clock_time_in_text(&rest)
            {
                let abs_clock_pos = after_pos + rel_pos;
                erase_range(text, abs_clock_pos, abs_clock_pos + clock_len);
                erase_range(text, pos, end_pos);
                let final_time_str = format!("{canonical_day} {clock}");
                return Some((final_time_str, is_deadline_lead || is_deadline_rest));
            }

            // 3. 纯日期前缀
            erase_range(text, pos, end_pos);
            let final_time_str = if pat == "tomorrow morning" || pat == "明早" || pat == "今早"
            {
                format!("{canonical_day} 09:00")
            } else if pat == "tomorrow afternoon" {
                format!("{canonical_day} 14:00")
            } else if pat == "tomorrow evening" {
                format!("{canonical_day} 19:00")
            } else if pat == "tomorrow night" || pat == "tonight" || pat == "明晚" || pat == "今晚"
            {
                format!("{canonical_day} 20:00")
            } else {
                canonical_day.to_string()
            };
            return Some((final_time_str, is_deadline_lead));
        }
    }

    // 若没有显式日期前缀，在整个文本中搜索独立时刻（如 "下午三点半"、"早上九点"、"at 9am"、"15:30"）
    if let Some((clock, is_deadline, pos, clock_len)) = find_clock_time_in_text(text) {
        erase_range(text, pos, pos + clock_len);
        return Some((clock, is_deadline));
    }

    None
}

/// 在整段文本中扫描独立时刻（如 "早上九点"、"下午3点半"、"at 9am"、"by 6pm"、"15:30"、"下班前"）
fn find_clock_time_in_text(text: &str) -> Option<(String, bool, usize, usize)> {
    let triggers = [
        "in the morning",
        "in the afternoon",
        "in the evening",
        "at night",
        "at noon",
        "noon",
        "at midnight",
        "midnight",
        "下班之前",
        "下班前",
        "上班之前",
        "上班前",
        "清晨",
        "早晨",
        "早上",
        "上午",
        "中午",
        "下午",
        "傍晚",
        "黄昏",
        "晚上",
        "夜里",
        "半夜",
        "凌晨",
    ];

    for trig in triggers {
        if let Some((pos, _)) = find_word_boundary(text, trig) {
            let (clock_opt, len, is_deadline) = parse_clock_time(&text[pos..], "");
            if let Some(clock) = clock_opt {
                return Some((clock, is_deadline, pos, len));
            }
        }
    }

    // 扫描形如 "9am", "9:30pm", "8 o'clock" 等带英文时刻后缀的表达
    for suffix in ["am", "pm", "o'clock", "oclock"] {
        let mut search_from = 0;
        let lower = text.to_ascii_lowercase();
        while let Some(rel) = lower[search_from..].find(suffix) {
            let suf_pos = search_from + rel;
            let end_suf = suf_pos + suffix.len();
            let next_ok = end_suf >= text.len()
                || !text[end_suf..]
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_alphanumeric();
            if next_ok {
                let before = &text[..suf_pos];
                let before_trimmed = before.trim_end();
                let num_start = before_trimmed
                    .rfind(|c: char| !c.is_ascii_alphanumeric() && c != ':' && c != '-')
                    .map(|idx| idx + before_trimmed[idx..].chars().next().unwrap().len_utf8())
                    .unwrap_or(0);
                if num_start < suf_pos {
                    let mut start = num_start;
                    let candidate = &text[start..];
                    let (clock_opt, len, mut is_deadline) = parse_clock_time(candidate, "");
                    if let Some(clock) = clock_opt {
                        let mut total_len = len;
                        let before_candidate = &text[..start];
                        let lower_bc = before_candidate.to_ascii_lowercase();
                        if lower_bc.ends_with("due by ") {
                            start -= 7;
                            total_len += 7;
                            is_deadline = true;
                        } else if lower_bc.ends_with("due ") {
                            start -= 4;
                            total_len += 4;
                            is_deadline = true;
                        } else if lower_bc.ends_with("before ") {
                            start -= 7;
                            total_len += 7;
                            is_deadline = true;
                        } else if lower_bc.ends_with("by ") {
                            start -= 3;
                            total_len += 3;
                            is_deadline = true;
                        } else if lower_bc.ends_with("at ") {
                            start -= 3;
                            total_len += 3;
                        }
                        return Some((clock, is_deadline, start, total_len));
                    }
                }
            }
            search_from = suf_pos + suffix.len();
            if search_from >= text.len() {
                break;
            }
        }
    }

    // 扫描以 "at ", "by ", "before " 开头的时刻表达（如 "at 15:30"、"at 9"、"by 5:00"）
    for prep in ["at ", "by ", "before "] {
        if let Some((pos, _)) = find_word_boundary(text, prep) {
            let (clock_opt, len, is_deadline) = parse_clock_time(&text[pos..], "");
            if let Some(clock) = clock_opt {
                return Some((clock, is_deadline, pos, len));
            }
        }
    }

    // 扫描所有形如 "X点" 的结构（跳过 "买一点"、"有一点" 等量词及非时间词）
    for (dot_idx, _) in text.match_indices('点') {
        let before = &text[..dot_idx];
        let start_pos = before
            .rfind(|c: char| !is_number_char(c))
            .map(|idx| idx + before[idx..].chars().next().unwrap().len_utf8())
            .unwrap_or(0);
        if start_pos == dot_idx {
            continue;
        }
        let hour_part = &text[start_pos..dot_idx];
        let after_dot = &text[dot_idx + '点'.len_utf8()..];
        let trimmed_after = after_dot.trim_start();
        let has_minute_suffix = trimmed_after.starts_with('半')
            || trimmed_after.starts_with("一刻")
            || trimmed_after.starts_with("三刻")
            || trimmed_after.starts_with("整")
            || (trimmed_after.starts_with('钟')
                && !trimmed_after['钟'.len_utf8()..].starts_with(|c| "楼表声爱情意馗".contains(c))
                && !trimmed_after['钟'.len_utf8()..].starts_with("点工"))
            || trimmed_after.chars().next().is_some_and(is_number_char);

        // 排除口语量词 "一点"（如 "买一点水果"、"有一点累"、"快一点"）及常见非时刻词（"三点水/三点意见/三点建议"）
        if !has_minute_suffix {
            if hour_part == "一" || hour_part == "1" {
                let prev_char = text[..start_pos].chars().next_back();
                if prev_char
                    .is_some_and(|c| "买有差快慢少多好早晚这那哪吃喝拿带记看写做".contains(c))
                {
                    continue;
                }
            }
            if after_dot.starts_with("水")
                || after_dot.starts_with("意见")
                || after_dot.starts_with("建议")
                || after_dot.starts_with("要求")
            {
                continue;
            }
        }

        let (clock_opt, len, is_deadline) = parse_clock_time(&text[start_pos..], "");
        if let Some(clock) = clock_opt {
            return Some((clock, is_deadline, start_pos, len));
        }
    }

    // 扫描形如 "HH:MM" 的结构
    for (colon_idx, _) in text.match_indices(':') {
        let before = &text[..colon_idx];
        let start_pos = before
            .rfind(|c: char| !c.is_ascii_digit())
            .map(|idx| idx + before[idx..].chars().next().unwrap().len_utf8())
            .unwrap_or(0);
        if start_pos == colon_idx {
            continue;
        }
        let (clock_opt, len, is_deadline) = parse_clock_time(&text[start_pos..], "");
        if let Some(clock) = clock_opt {
            return Some((clock, is_deadline, start_pos, len));
        }
    }

    None
}

/// 解析文本起始处的时刻段，例如：
/// - "下午三点半" -> 15:30
/// - "下班前" -> 18:00 (deadline)
/// - "早上9点" -> 09:00
/// - "9am" / "9:30pm" / "at 3pm" / "by 6pm"
///
/// 返回 `(Option<"HH:MM">, consumed_len, is_deadline)`
fn parse_clock_time(s: &str, prefix_pat: &str) -> (Option<String>, usize, bool) {
    let mut s_trimmed = s.trim_start();
    let mut lead_spaces = s.len() - s_trimmed.len();
    let mut is_deadline_lead = false;

    // 允许日期与时段/时刻之间的连接词，如 "下周六的下午5点"、"明天的晚上"、"at 9am"、"by 6pm"
    for prep in [
        "due by ", "due ", "before ", "by ", "at ", "on ", "的", "在", "于",
    ] {
        if let Some((pos, end)) = find_word_boundary(s_trimmed, prep) {
            if pos == 0 {
                if prep.starts_with("due") || prep.starts_with("before") || prep.starts_with("by") {
                    is_deadline_lead = true;
                }
                let after_prep = s_trimmed[end..].trim_start();
                lead_spaces += s_trimmed.len() - after_prep.len();
                s_trimmed = after_prep;
                break;
            }
        }
    }

    // 预设时段
    if s_trimmed.starts_with("下班前") || s_trimmed.starts_with("下班之前") {
        let len = if s_trimmed.starts_with("下班之前") {
            "下班之前".len()
        } else {
            "下班前".len()
        };
        return (Some("18:00".to_string()), lead_spaces + len, true);
    }
    if s_trimmed.starts_with("上班前") || s_trimmed.starts_with("上班之前") {
        let len = if s_trimmed.starts_with("上班之前") {
            "上班之前".len()
        } else {
            "上班前".len()
        };
        return (Some("09:00".to_string()), lead_spaces + len, true);
    }

    let is_prefix_pm = prefix_pat == "明晚"
        || prefix_pat == "今晚"
        || prefix_pat == "tomorrow night"
        || prefix_pat == "tonight"
        || prefix_pat == "tomorrow evening"
        || prefix_pat == "tomorrow afternoon";

    // 时段修饰词与基础偏移（支持中英文双语）
    let period_modifiers = [
        ("in the morning", 9, false),
        ("in the afternoon", 14, true),
        ("in the evening", 19, true),
        ("at night", 20, true),
        ("at noon", 12, false),
        ("noon", 12, false),
        ("at midnight", 0, false),
        ("midnight", 0, false),
        ("清晨", 6, false),
        ("早晨", 9, false),
        ("早上", 9, false),
        ("上午", 9, false),
        ("中午", 12, false),
        ("下午", 14, true), // pm flag
        ("傍晚", 18, false),
        ("黄昏", 18, false),
        ("晚上", 20, true), // pm flag
        ("夜里", 21, true),
        ("半夜", 23, true),
        ("凌晨", 1, false),
    ];

    for (mod_word, default_hour, is_pm) in period_modifiers {
        if let Some((pos, end)) = find_word_boundary(s_trimmed, mod_word) {
            if pos == 0 {
                let after_mod = &s_trimmed[end..];
                let trimmed_after = after_mod.trim_start();
                let spaces = after_mod.len() - trimmed_after.len();

                // 检查是否有具体的 "X点" 或 "X:MM" 或 "9am"
                if let Some((h, m, parsed_len, is_deadline)) =
                    extract_hour_minute(trimmed_after, is_pm)
                {
                    let total_len = lead_spaces + (end - pos) + spaces + parsed_len;
                    return (
                        Some(format!("{:02}:{:02}", h, m)),
                        total_len,
                        is_deadline || is_deadline_lead,
                    );
                }

                // 无具体时刻，使用该时段的默认小时（如 "上午" -> 09:00, "noon" -> 12:00）
                let is_deadline = trimmed_after.starts_with("前")
                    || trimmed_after.starts_with("之前")
                    || trimmed_after.to_ascii_lowercase().starts_with("before");
                let deadline_len = if trimmed_after.starts_with("之前") {
                    "之前".len()
                } else if trimmed_after.starts_with("前") {
                    "前".len()
                } else if trimmed_after.to_ascii_lowercase().starts_with("before") {
                    6
                } else {
                    0
                };
                let total_len = lead_spaces + (end - pos) + spaces + deadline_len;
                return (
                    Some(format!("{:02}:00", default_hour)),
                    total_len,
                    is_deadline || is_deadline_lead,
                );
            }
        }
    }

    // 直接以时刻开头的形式（如 "15:30"、"3点半"、"9am"、"9:30pm"、"at 8 o'clock"）
    if let Some((h, m, parsed_len, is_deadline)) = extract_hour_minute(s_trimmed, is_prefix_pm) {
        return (
            Some(format!("{:02}:{:02}", h, m)),
            lead_spaces + parsed_len,
            is_deadline || is_deadline_lead,
        );
    }

    (None, 0, false)
}

/// 解析时刻中的时和分（如 "三点半" -> (15, 30), "8点45" -> (8, 45), "9:30am" -> (9, 30), "3pm" -> (15, 0)）
fn extract_hour_minute(s: &str, is_pm: bool) -> Option<(u32, u32, usize, bool)> {
    if s.is_empty() {
        return None;
    }

    // 1. 标准 HH:MM 格式（支持可选 am/pm，如 9:30am, 9:30 pm, 15:30）
    if let Some(colon_pos) = s.find(':') {
        if colon_pos > 0 && colon_pos + 1 < s.len() {
            let h_str = s[..colon_pos].trim();
            if let Ok(mut h) = h_str.parse::<u32>() {
                let after_colon = &s[colon_pos + 1..];
                let m_str: String = after_colon
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(m) = m_str.parse::<u32>() {
                    if h <= 24 && m < 60 {
                        let mut consumed = colon_pos + 1 + m_str.len();
                        let rest_after_m = &s[consumed..];
                        let trimmed_rest = rest_after_m.trim_start();
                        let spaces = rest_after_m.len() - trimmed_rest.len();
                        let lower_rest = trimmed_rest.to_ascii_lowercase();

                        if lower_rest.starts_with("am") {
                            consumed += spaces + 2;
                            if h == 12 {
                                h = 0;
                            }
                        } else if lower_rest.starts_with("pm") {
                            consumed += spaces + 2;
                            if h < 12 {
                                h += 12;
                            }
                        } else if is_pm && h < 12 {
                            h += 12;
                        }

                        let rest = &s[consumed..];
                        let trimmed_rest = rest.trim_start();
                        let space_len = rest.len() - trimmed_rest.len();
                        let is_deadline = trimmed_rest.starts_with("前")
                            || trimmed_rest.starts_with("之前")
                            || trimmed_rest.to_ascii_lowercase().starts_with("before");
                        if trimmed_rest.starts_with("之前") {
                            consumed += space_len + "之前".len();
                        } else if trimmed_rest.starts_with("前") {
                            consumed += space_len + "前".len();
                        } else if trimmed_rest.to_ascii_lowercase().starts_with("before") {
                            consumed += space_len + 6;
                        }

                        return Some((h, m, consumed, is_deadline));
                    }
                }
            }
        }
    }

    // 2. 中文 "X点" 或 "X点半" / "X点Y分" / "X点Y"
    if let Some(dot_pos) = s.find('点') {
        let hour_str = s[..dot_pos].trim();
        if let Some(mut h) = parse_cn_number(hour_str) {
            if h > 24 {
                return None;
            }
            if is_pm && h < 12 {
                h += 12;
            }
            let after_dot = &s[dot_pos + '点'.len_utf8()..];
            let mut m = 0;
            let mut consumed = dot_pos + '点'.len_utf8();

            if after_dot.starts_with('半') {
                m = 30;
                consumed += '半'.len_utf8();
                let rest_half = &s[consumed..];
                let trimmed_half = rest_half.trim_start();
                let space_len = rest_half.len() - trimmed_half.len();
                if trimmed_half.starts_with('钟') {
                    consumed += space_len + '钟'.len_utf8();
                }
            } else if after_dot.starts_with("一刻") {
                m = 15;
                consumed += "一刻".len();
                let rest_ke = &s[consumed..];
                let trimmed_ke = rest_ke.trim_start();
                let space_len = rest_ke.len() - trimmed_ke.len();
                if trimmed_ke.starts_with('钟') {
                    consumed += space_len + '钟'.len_utf8();
                }
            } else if after_dot.starts_with("三刻") {
                m = 45;
                consumed += "三刻".len();
                let rest_ke = &s[consumed..];
                let trimmed_ke = rest_ke.trim_start();
                let space_len = rest_ke.len() - trimmed_ke.len();
                if trimmed_ke.starts_with('钟') {
                    consumed += space_len + '钟'.len_utf8();
                }
            } else {
                // 仅提取连续的数字字符（如 "45分"、"二十去开会" 中的 "二十"），避免吞入后续动词
                let minute_candidate: String = after_dot
                    .chars()
                    .take_while(|c| is_number_char(*c))
                    .collect();
                if !minute_candidate.is_empty() {
                    if let Some(parsed_m) = parse_cn_number(&minute_candidate) {
                        if parsed_m < 60 {
                            m = parsed_m;
                            consumed += minute_candidate.len();
                            // 吸收随后的 "分" 或 "分钟"
                            if s[consumed..].starts_with("分钟") {
                                consumed += "分钟".len();
                            } else if s[consumed..].starts_with('分') {
                                consumed += '分'.len_utf8();
                                if s[consumed..].starts_with('钟') {
                                    consumed += '钟'.len_utf8();
                                }
                            }
                            if s[consumed..].starts_with('整') {
                                consumed += '整'.len_utf8();
                            }
                        }
                    }
                } else {
                    // 没有分钟部分时，检查口语后缀 "整钟"、"钟整"、"整"、"钟"
                    let rest_after_dot = &s[consumed..];
                    let trimmed_rest = rest_after_dot.trim_start();
                    let space_len = rest_after_dot.len() - trimmed_rest.len();
                    if trimmed_rest.starts_with("整钟") {
                        consumed += space_len + "整钟".len();
                    } else if trimmed_rest.starts_with("钟整") {
                        consumed += space_len + "钟整".len();
                    } else if trimmed_rest.starts_with('整') {
                        consumed += space_len + '整'.len_utf8();
                    } else if trimmed_rest.starts_with('钟') {
                        let after_zhong = &trimmed_rest['钟'.len_utf8()..];
                        let is_compound_word = after_zhong.starts_with('楼')
                            || after_zhong.starts_with('表')
                            || after_zhong.starts_with("点工")
                            || after_zhong.starts_with('声')
                            || after_zhong.starts_with('爱')
                            || after_zhong.starts_with('情')
                            || after_zhong.starts_with('意')
                            || after_zhong.starts_with('馗');
                        if !is_compound_word {
                            consumed += space_len + '钟'.len_utf8();
                        }
                    }
                }
            }

            let rest = &s[consumed..];
            let trimmed_rest = rest.trim_start();
            let space_len = rest.len() - trimmed_rest.len();
            let is_deadline = trimmed_rest.starts_with("前") || trimmed_rest.starts_with("之前");
            if trimmed_rest.starts_with("之前") {
                consumed += space_len + "之前".len();
            } else if trimmed_rest.starts_with("前") {
                consumed += space_len + "前".len();
            }

            return Some((h, m, consumed, is_deadline));
        }
    }

    // 3. 英文时刻（如 9am, 9pm, 9 am, 9 o'clock, 8 oclock, at 9）
    {
        let trimmed = s.trim_start();
        let lead_sp = s.len() - trimmed.len();
        let first_char = trimmed.chars().next().unwrap_or(' ');
        let num_token: String = if first_char.is_ascii_digit() {
            trimmed.chars().take_while(|c| c.is_ascii_digit()).collect()
        } else {
            trimmed
                .chars()
                .take_while(|c| c.is_alphabetic() || *c == '-')
                .collect()
        };
        if !num_token.is_empty() {
            if let Some(mut h) = parse_any_number(&num_token) {
                if h <= 24 {
                    let after_num = &trimmed[num_token.len()..];
                    let trimmed_after = after_num.trim_start();
                    let spaces = after_num.len() - trimmed_after.len();
                    let lower_after = trimmed_after.to_ascii_lowercase();

                    let is_am = lower_after.starts_with("am")
                        && (lower_after.len() == 2
                            || !lower_after[2..]
                                .chars()
                                .next()
                                .unwrap()
                                .is_ascii_alphanumeric());
                    let is_pm_word = lower_after.starts_with("pm")
                        && (lower_after.len() == 2
                            || !lower_after[2..]
                                .chars()
                                .next()
                                .unwrap()
                                .is_ascii_alphanumeric());

                    let mut matched = false;
                    let mut consumed = lead_sp + num_token.len();

                    if is_am {
                        if h == 12 {
                            h = 0;
                        }
                        consumed += spaces + 2;
                        matched = true;
                    } else if is_pm_word {
                        if h < 12 {
                            h += 12;
                        }
                        consumed += spaces + 2;
                        matched = true;
                    } else if lower_after.starts_with("o'clock") {
                        if is_pm && h < 12 {
                            h += 12;
                        }
                        consumed += spaces + "o'clock".len();
                        matched = true;
                    } else if lower_after.starts_with("oclock") {
                        if is_pm && h < 12 {
                            h += 12;
                        }
                        consumed += spaces + "oclock".len();
                        matched = true;
                    } else if is_pm {
                        // 处于 pm 修饰语境中，如 "下午 3"
                        let next_is_word = trimmed_after
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_alphabetic());
                        if !next_is_word {
                            if h < 12 {
                                h += 12;
                            }
                            matched = true;
                        }
                    }

                    if matched {
                        let rest = &s[consumed..];
                        let trimmed_rest = rest.trim_start();
                        let space_len = rest.len() - trimmed_rest.len();
                        let is_deadline = trimmed_rest.starts_with("前")
                            || trimmed_rest.starts_with("之前")
                            || trimmed_rest.to_ascii_lowercase().starts_with("before");
                        if trimmed_rest.starts_with("之前") {
                            consumed += space_len + "之前".len();
                        } else if trimmed_rest.starts_with("前") {
                            consumed += space_len + "前".len();
                        } else if trimmed_rest.to_ascii_lowercase().starts_with("before") {
                            consumed += space_len + 6;
                        }
                        return Some((h, 0, consumed, is_deadline));
                    }
                }
            }
        }
    }

    None
}

/// 净化任务标题：剥离语音引导词、多余语气助词及标点（支持中英双语）
fn clean_title(text: &str) -> String {
    let mut s = text.trim().to_string();

    // 剥离句首语音触发词 / 礼貌词（支持中英双语）
    let prefixes = [
        "请提醒我",
        "请帮我提醒我",
        "请帮我记一下",
        "请帮我记录",
        "请帮我",
        "帮我记一下",
        "帮我记录一下",
        "帮我记录",
        "帮我记",
        "提醒我",
        "提醒一下",
        "千万别忘了",
        "千万不要忘",
        "不要忘了",
        "别忘了",
        "记得要",
        "记得",
        "需要去",
        "需要在",
        "需要",
        "打算去",
        "打算",
        "计划去",
        "计划",
        "准备去",
        "准备",
        "要去",
        "要在",
        "的",
        "please remind me to",
        "remind me to",
        "please remember to",
        "remember to",
        "don't forget to",
        "dont forget to",
        "i need to",
        "need to",
        "i have to",
        "have to",
        "i plan to",
        "plan to",
        "gotta",
        "schedule",
        "please",
    ];

    let mut changed = true;
    while changed {
        changed = false;
        s = s
            .trim_matches(|c: char| c.is_whitespace() || ",，。！!？?、:：;；-".contains(c))
            .to_string();

        for prefix in prefixes {
            let lower = s.to_ascii_lowercase();
            if lower.starts_with(prefix) {
                let is_ascii = prefix.is_ascii();
                if is_ascii {
                    let next_c = s[prefix.len()..].chars().next();
                    if next_c.is_none()
                        || next_c.unwrap().is_whitespace()
                        || !next_c.unwrap().is_ascii_alphanumeric()
                    {
                        s = s[prefix.len()..].to_string();
                        changed = true;
                        break;
                    }
                } else {
                    s = s[prefix.len()..].to_string();
                    changed = true;
                    break;
                }
            }
        }

        // 剥离因时间抽取遗留在句首的孤立英文介词（如 "to buy milk", "at supermarket", "for meeting"）
        for prep in ["to ", "at ", "on ", "by ", "for "] {
            if s.to_ascii_lowercase().starts_with(prep) {
                s = s[prep.len()..].to_string();
                changed = true;
                break;
            }
        }
    }

    // 剥离句尾语气词或杂质短语（中英双语）
    let suffixes = [
        "千万别忘了",
        "千万不要忘",
        "不要忘了",
        "别忘了",
        "这个任务",
        "的事情",
        "的事",
        "一下吧",
        "好吧",
        "去吧",
        "一下",
        "啊",
        "呀",
        "哦",
        "呢",
        "please",
        "asap",
        "thank you",
        "thanks",
    ];

    changed = true;
    while changed {
        changed = false;
        s = s
            .trim_matches(|c: char| c.is_whitespace() || ",，。！!？?、:：;；-".contains(c))
            .to_string();
        for suffix in suffixes {
            let lower = s.to_ascii_lowercase();
            if lower.ends_with(suffix) {
                let is_ascii = suffix.is_ascii();
                if is_ascii {
                    let before_suf = &s[..s.len() - suffix.len()];
                    let prev_c = before_suf.chars().next_back();
                    if prev_c.is_none()
                        || prev_c.unwrap().is_whitespace()
                        || !prev_c.unwrap().is_ascii_alphanumeric()
                    {
                        s = before_suf.to_string();
                        changed = true;
                        break;
                    }
                } else {
                    s = s[..s.len() - suffix.len()].to_string();
                    changed = true;
                    break;
                }
            }
        }
    }

    // 剥离多余标点与空白
    let cleaned: String = s
        .trim_matches(|c: char| c.is_whitespace() || ",，。！!？?、:：;；-".contains(c))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.is_empty() {
        "未命名语音任务".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complex_voice_inputs() {
        // 案例 1: 复杂口语：时间 + 优先级 + 显式标签 + 引导词
        let res1 =
            parse_natural_language("提醒我明天下午三点半去参加架构评审会，非常重要，标签是工作");
        assert_eq!(res1.title, "去参加架构评审会");
        assert_eq!(res1.scheduled_str.as_deref(), Some("明天 15:30"));
        assert_eq!(res1.priority.as_deref(), Some("high"));
        assert_eq!(res1.tags, vec!["work"]);
        assert_eq!(
            res1.to_quick_add_syntax(),
            "去参加架构评审会 ~明天 15:30 !high @work"
        );

        // 案例 2: 周期 + 隐式标签识别
        let res2 = parse_natural_language("提醒我每周一早上九点开例会");
        assert_eq!(res2.title, "开例会");
        assert_eq!(res2.rrule.as_deref(), Some("FREQ=WEEKLY;BYDAY=MO"));
        assert_eq!(res2.scheduled_str.as_deref(), Some("09:00"));
        assert_eq!(res2.tags, vec!["work"]);

        // 案例 3: 相对时间 + 引导词
        let res3 = parse_natural_language("两小时后提醒我喝水");
        assert_eq!(res3.title, "喝水");
        assert_eq!(res3.scheduled_str.as_deref(), Some("+2h"));

        // 案例 4: 截止时间 (deadline) + 优先级 + 隐式标签
        let res4 = parse_natural_language("下周五下班前必须提交财报分析");
        assert_eq!(res4.title, "提交财报分析");
        assert_eq!(res4.due_str.as_deref(), Some("下周五 18:00"));
        assert_eq!(res4.scheduled_str, None);
        assert_eq!(res4.to_quick_add().time_str, None);
        assert_eq!(res4.priority.as_deref(), Some("high"));
        assert_eq!(res4.tags, vec!["work"]);

        // 案例 5: 月循环 + 低优先级 + 跑腿标签
        let res5 = parse_natural_language("每个月十号交房租，顺便去超市买点水果，不着急");
        assert_eq!(res5.title, "交房租 顺便去超市买点水果");
        assert_eq!(res5.rrule.as_deref(), Some("FREQ=MONTHLY;BYMONTHDAY=10"));
        assert_eq!(res5.priority.as_deref(), Some("low"));
        assert_eq!(res5.tags, vec!["errands"]);

        // 案例 6: 纯简单任务
        let res6 = parse_natural_language("买牛奶");
        assert_eq!(res6.title, "买牛奶");
        assert_eq!(res6.tags, vec!["errands"]);

        // 案例 7: 标点丰富口语与医院预约
        let res7 = parse_natural_language("提醒我：后天上午十点半约医生体检，不要忘了。");
        assert_eq!(res7.title, "约医生体检");
        assert_eq!(res7.scheduled_str.as_deref(), Some("后天 10:30"));

        // 案例 8: 工作日循环 + 晨跑
        let res8 = parse_natural_language("工作日早上八点晨跑");
        assert_eq!(res8.title, "晨跑");
        assert_eq!(
            res8.rrule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR")
        );
        assert_eq!(res8.scheduled_str.as_deref(), Some("08:00"));

        // 案例 9: 紧急前缀与周报
        let res9 = parse_natural_language("紧急：下周一下午三点交周报");
        assert_eq!(res9.title, "交周报");
        assert_eq!(res9.priority.as_deref(), Some("high"));
        assert_eq!(res9.scheduled_str.as_deref(), Some("下周一 15:00"));
        assert_eq!(res9.tags, vec!["work"]);

        // 案例 10: 电话标签推断与晚上时间
        let res10 = parse_natural_language("周日晚上九点给妈妈打电话");
        assert_eq!(res10.title, "给妈妈打电话");
        assert_eq!(res10.scheduled_str.as_deref(), Some("周日 21:00"));
        assert_eq!(res10.tags, vec!["calls"]);

        // 案例 11: 每天循环
        let res11 = parse_natural_language("每天喝八杯水");
        assert_eq!(res11.title, "喝八杯水");
        assert_eq!(res11.rrule.as_deref(), Some("FREQ=DAILY"));

        // 案例 12: 空输入
        let res12 = parse_natural_language("   ");
        assert_eq!(res12.title, "");

        // 案例 13: 包含 voxtype 调试日志与 ANSI 控制码的原始输出
        let raw_vox = "Loading audio file: \"/tmp/horae_voice.wav\"\nAudio format: 16000 Hz, 1 channel(s), Int\nProcessing 129338 samples (8.08s)...\n\x1b[2m2026-10-23T09:17:18.194734Z\x1b[0m \x1b[32m INFO\x1b[0m Loading SenseVoice model from \"/home/alex/.local/share/voxtype/models/sensevoice-small\"\n\x1b[2m2026-10-23T09:17:19.033586Z\x1b[0m \x1b[32m INFO\x1b[0m SenseVoice model loaded in 0.84s (language=zh, use_itn=true)\n\x1b[2m2026-10-23T09:17:19.165959Z\x1b[0m \x1b[32m INFO\x1b[0m SenseVoice transcription completed in 0.13s: \"提醒我 去跑步 \"\n\n提醒我明天下午6点去跑步 非常重要";
        let res13 = parse_natural_language(raw_vox);
        assert_eq!(res13.title, "去跑步");
        assert_eq!(res13.scheduled_str.as_deref(), Some("明天 18:00"));
        assert_eq!(res13.priority.as_deref(), Some("high"));

        // 案例 14: 带 "的" 连接词的日期与时刻，如 "下周六的下午5点去锻炼，标签是室外"
        let res14 = parse_natural_language("下周六的下午5点去锻炼，标签是室外");
        assert_eq!(res14.title, "去锻炼");
        assert_eq!(res14.scheduled_str.as_deref(), Some("下周六 17:00"));
        assert_eq!(res14.tags, vec!["室外"]);

        // 案例 15: 隔开的日期与时刻，如 "下周六去健身，下午5点"
        let res15 = parse_natural_language("下周六去健身，下午5点");
        assert_eq!(res15.title, "去健身");
        assert_eq!(res15.scheduled_str.as_deref(), Some("下周六 17:00"));
    }

    #[test]
    fn test_voice_edge_cases_and_false_positives() {
        // 1. 不带 "分" 的中文分钟口语 ("三点二十去开会")
        let r1 = parse_natural_language("明天下午三点二十去开会");
        assert_eq!(r1.title, "去开会");
        assert_eq!(r1.scheduled_str.as_deref(), Some("明天 15:20"));

        // 2. 句中含有量词 "一点"，后跟真实时刻 ("买一点水果，三点半去接孩子")
        let r2 = parse_natural_language("买一点水果，三点半去接孩子");
        assert_eq!(r2.title, "买一点水果 去接孩子");
        assert_eq!(r2.scheduled_str.as_deref(), Some("03:30"));

        // 3. 首尾包含正常词汇 "要求"、"在线"、"于老师"、"网吧"、"工作日记" 不被误剥离
        let r3 = parse_natural_language("要求供应商重发合同");
        assert_eq!(r3.title, "要求供应商重发合同");

        let r4 = parse_natural_language("在线办理签证");
        assert_eq!(r4.title, "在线办理签证");

        let r5 = parse_natural_language("周末去网吧");
        assert_eq!(r5.title, "周末去网吧");

        let r6 = parse_natural_language("整理工作日记");
        assert_eq!(r6.title, "整理工作日记");
        assert_eq!(r6.rrule, None);

        // 4. "大后天" 与显式 "X月Y号" 日期口语
        let r7 = parse_natural_language("大后天下午两点去牙科复诊");
        assert_eq!(r7.title, "去牙科复诊");
        assert_eq!(r7.scheduled_str.as_deref(), Some("大后天 14:00"));

        let r8 = parse_natural_language("10月25号下午3点参加技术分享");
        assert_eq!(r8.title, "参加技术分享");
        assert_eq!(r8.scheduled_str.as_deref(), Some("10-25 15:00"));

        // 5. "每两天" / "每隔两天" 循环
        let r9 = parse_natural_language("每两天给花浇水");
        assert_eq!(r9.title, "给花浇水");
        assert_eq!(r9.rrule.as_deref(), Some("FREQ=DAILY;INTERVAL=2"));

        let r10 = parse_natural_language("每隔两天备份一次数据");
        assert_eq!(r10.title, "备份一次数据");
        assert_eq!(r10.rrule.as_deref(), Some("FREQ=DAILY;INTERVAL=3"));

        // 6. 口语 "点钟"、"点整"、"零X分" 与循环组合（修复 "每天早晨九点钟吃饭" 解析为 "钟吃饭" 的 bug）
        let r11 = parse_natural_language("每天早晨九点钟吃饭");
        assert_eq!(r11.title, "吃饭");
        assert_eq!(r11.scheduled_str.as_deref(), Some("09:00"));
        assert_eq!(r11.rrule.as_deref(), Some("FREQ=DAILY"));

        let r12 = parse_natural_language("每天九点钟吃饭");
        assert_eq!(r12.title, "吃饭");
        assert_eq!(r12.scheduled_str.as_deref(), Some("09:00"));
        assert_eq!(r12.rrule.as_deref(), Some("FREQ=DAILY"));

        let r13 = parse_natural_language("明天早晨九点钟开会");
        assert_eq!(r13.title, "开会");
        assert_eq!(r13.scheduled_str.as_deref(), Some("明天 09:00"));

        let r14 = parse_natural_language("九点整开会");
        assert_eq!(r14.title, "开会");
        assert_eq!(r14.scheduled_str.as_deref(), Some("09:00"));

        let r15 = parse_natural_language("九点钟整开会");
        assert_eq!(r15.title, "开会");
        assert_eq!(r15.scheduled_str.as_deref(), Some("09:00"));

        let r16 = parse_natural_language("九点半钟去游泳");
        assert_eq!(r16.title, "去游泳");
        assert_eq!(r16.scheduled_str.as_deref(), Some("09:30"));

        let r17 = parse_natural_language("九点一刻钟集合");
        assert_eq!(r17.title, "集合");
        assert_eq!(r17.scheduled_str.as_deref(), Some("09:15"));

        let r18 = parse_natural_language("九点钟之前交报告");
        assert_eq!(r18.title, "交报告");
        assert_eq!(r18.due_str.as_deref(), Some("09:00"));

        let r19 = parse_natural_language("九点零五分去开会");
        assert_eq!(r19.title, "去开会");
        assert_eq!(r19.scheduled_str.as_deref(), Some("09:05"));

        // 7. 避免将属于地名/名词的 "钟" 误吞（如 "钟楼集合"、"钟表店买表"、"钟点工"）
        let r20 = parse_natural_language("九点钟楼集合");
        assert_eq!(r20.title, "钟楼集合");
        assert_eq!(r20.scheduled_str.as_deref(), Some("09:00"));

        let r21 = parse_natural_language("九点钟表店买表");
        assert_eq!(r21.title, "钟表店买表");
        assert_eq!(r21.scheduled_str.as_deref(), Some("09:00"));
    }

    #[test]
    fn test_parse_english_voice_inputs() {
        // 1. 基础提醒 + 相对时间
        let r1 = parse_natural_language("Remind me to drink water in 30 minutes");
        assert_eq!(r1.title, "drink water");
        assert_eq!(r1.scheduled_str.as_deref(), Some("+30m"));
        assert_eq!(r1.to_quick_add_syntax(), "drink water ~+30m");

        // 2. 复合口语：引导词 + 时间 + 优先级 + 显式标签
        let r2 = parse_natural_language(
            "Please remind me to submit architecture proposal tomorrow at 3:30pm, very urgent, tag is work",
        );
        assert_eq!(r2.title, "submit architecture proposal");
        assert_eq!(r2.scheduled_str.as_deref(), Some("tomorrow 15:30"));
        assert_eq!(r2.priority.as_deref(), Some("high"));
        assert_eq!(r2.tags, vec!["work"]);
        assert_eq!(
            r2.to_quick_add_syntax(),
            "submit architecture proposal ~tomorrow 15:30 !high @work"
        );

        // 3. 截止期限 (Deadline)
        let r3 = parse_natural_language("Finish quarterly financial report by Friday at 6pm");
        assert_eq!(r3.title, "Finish quarterly financial report");
        assert_eq!(r3.due_str.as_deref(), Some("fri 18:00"));
        assert_eq!(r3.scheduled_str, None);
        assert_eq!(r3.tags, vec!["work"]);

        // 4. 每日循环 + 时刻
        let r4 = parse_natural_language("Workout every day at 7am");
        assert_eq!(r4.title, "Workout");
        assert_eq!(r4.rrule.as_deref(), Some("FREQ=DAILY"));
        assert_eq!(r4.scheduled_str.as_deref(), Some("07:00"));

        // 5. 工作日循环
        let r5 = parse_natural_language("Team standup every weekday at 10am");
        assert_eq!(r5.title, "Team standup");
        assert_eq!(
            r5.rrule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR")
        );
        assert_eq!(r5.scheduled_str.as_deref(), Some("10:00"));
        assert_eq!(r5.tags, vec!["work"]);

        // 6. 每周特定星期
        let r6 = parse_natural_language("Review metrics every Monday at 9:30am");
        assert_eq!(r6.title, "Review metrics");
        assert_eq!(r6.rrule.as_deref(), Some("FREQ=WEEKLY;BYDAY=MO"));
        assert_eq!(r6.scheduled_str.as_deref(), Some("09:30"));

        // 7. 每月特定日 + 跑腿标签推断
        let r7 = parse_natural_language("Pay apartment rent every month on the 1st");
        assert_eq!(r7.title, "Pay apartment rent");
        assert_eq!(r7.rrule.as_deref(), Some("FREQ=MONTHLY;BYMONTHDAY=1"));
        assert_eq!(r7.tags, vec!["errands"]);

        // 8. 特殊时段：tonight + o'clock + 电话标签推断
        let r8 = parse_natural_language("Call Mom tonight at 8 o'clock");
        assert_eq!(r8.title, "Call Mom");
        assert_eq!(r8.scheduled_str.as_deref(), Some("today 20:00"));
        assert_eq!(r8.tags, vec!["calls"]);

        // 9. 简短口语：tomorrow morning
        let r9 = parse_natural_language("Buy milk tomorrow morning");
        assert_eq!(r9.title, "Buy milk");
        assert_eq!(r9.scheduled_str.as_deref(), Some("tomorrow 09:00"));
        assert_eq!(r9.tags, vec!["errands"]);

        // 10. 显式月日：Oct 25 at 3pm
        let r10 = parse_natural_language("Doctor appointment on Oct 25 at 3pm");
        assert_eq!(r10.title, "Doctor appointment");
        assert_eq!(r10.scheduled_str.as_deref(), Some("10-25 15:00"));
    }

    #[test]
    fn test_english_edge_cases_and_word_boundaries() {
        // 1. 词界防呆：passport details 不被误识别为 p1，divergent 不被误识别为 urgent
        let r1 = parse_natural_language("Read passport details, urgent");
        assert_eq!(r1.title, "Read passport details");
        assert_eq!(r1.priority.as_deref(), Some("high"));
        assert_eq!(r1.tags, vec!["learning"]);

        // 2. 词界防呆：already 不触发 read 标签
        let r2 = parse_natural_language("Already finished the report");
        assert_eq!(r2.title, "Already finished the report");
        assert_eq!(r2.tags, vec!["work"]);

        // 3. 特殊时刻：noon / midnight
        let r3 = parse_natural_language("Lunch with Alice at noon");
        assert_eq!(r3.title, "Lunch with Alice");
        assert_eq!(r3.scheduled_str.as_deref(), Some("12:00"));

        let r4 = parse_natural_language("Server backup at midnight");
        assert_eq!(r4.title, "Server backup");
        assert_eq!(r4.scheduled_str.as_deref(), Some("00:00"));

        // 4. 多天循环与周数相对时间
        let r5 = parse_natural_language("Water plants every 3 days");
        assert_eq!(r5.title, "Water plants");
        assert_eq!(r5.rrule.as_deref(), Some("FREQ=DAILY;INTERVAL=3"));

        let r6 = parse_natural_language("Doctor checkup in 2 weeks");
        assert_eq!(r6.title, "Doctor checkup");
        assert_eq!(r6.scheduled_str.as_deref(), Some("+2w"));

        // 5. 中英混合自然语音
        let r7 = parse_natural_language("明天下午 3pm 和 Tom 开会");
        assert_eq!(r7.title, "和 Tom 开会");
        assert_eq!(r7.scheduled_str.as_deref(), Some("明天 15:00"));
        assert_eq!(r7.tags, vec!["work"]);
    }
}
