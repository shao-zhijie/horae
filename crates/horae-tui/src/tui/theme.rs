use horae_core::model::task::Status;
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    CatppuccinMocha,
    CatppuccinLatte,
    TokyoNight,
    Gruvbox,
    Retro82,
}

impl ThemeId {
    pub fn key(&self) -> &'static str {
        match self {
            Self::CatppuccinMocha => "mocha",
            Self::CatppuccinLatte => "latte",
            Self::TokyoNight => "tokyo-night",
            Self::Gruvbox => "gruvbox",
            Self::Retro82 => "retro-82",
        }
    }

    pub fn display_name(&self, lang: horae_core::i18n::Lang) -> &'static str {
        match (self, lang) {
            (Self::CatppuccinMocha, horae_core::i18n::Lang::Zh) => "Catppuccin 摩卡 (深色)",
            (Self::CatppuccinMocha, horae_core::i18n::Lang::En) => "Catppuccin Mocha (Dark)",
            (Self::CatppuccinLatte, horae_core::i18n::Lang::Zh) => "Catppuccin 拿铁 (亮色)",
            (Self::CatppuccinLatte, horae_core::i18n::Lang::En) => "Catppuccin Latte (Light)",
            (Self::TokyoNight, horae_core::i18n::Lang::Zh) => "Tokyo Night 东京之夜 (深色)",
            (Self::TokyoNight, horae_core::i18n::Lang::En) => "Tokyo Night (Dark)",
            (Self::Gruvbox, horae_core::i18n::Lang::Zh) => "Gruvbox 复古暖色 (深色)",
            (Self::Gruvbox, horae_core::i18n::Lang::En) => "Gruvbox (Dark)",
            (Self::Retro82, horae_core::i18n::Lang::Zh) => "Retro 82 复古未来 (深色)",
            (Self::Retro82, horae_core::i18n::Lang::En) => "Retro 82 (Dark)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::CatppuccinMocha => Self::CatppuccinLatte,
            Self::CatppuccinLatte => Self::TokyoNight,
            Self::TokyoNight => Self::Gruvbox,
            Self::Gruvbox => Self::Retro82,
            Self::Retro82 => Self::CatppuccinMocha,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: ThemeId,
    pub is_dark: bool,
    pub bg: Color,
    pub fg: Color,
    pub text_dim: Color,
    pub text_urgent: Color,
    pub text_warn: Color,
    pub text_success: Color,
    pub rrule_fg: Color,

    // Borders
    pub border_active: Color,
    pub border_inactive: Color,

    // Highlight
    pub hl_bg: Color,
    pub hl_fg: Color,
    pub row_active_bg: Color,

    // Status Bar
    pub status_bg: Color,
    pub status_fg: Color,

    // Specific elements
    pub accent: Color,

    // Semantic status colors
    pub status_inbox: Color,
    pub status_next: Color,
    pub status_waiting: Color,
    pub status_scheduled: Color,
    pub status_someday: Color,
    pub status_reference: Color,
    pub status_done: Color,

    // Priority colors
    pub priority_high: Color,
    pub priority_medium: Color,
    pub priority_low: Color,
}

impl Theme {
    pub fn catppuccin_mocha() -> Self {
        Self {
            id: ThemeId::CatppuccinMocha,
            is_dark: true,
            bg: Color::Rgb(30, 30, 46),              // Base #1e1e2e
            fg: Color::Rgb(205, 214, 244),           // Text #cdd6f4
            text_dim: Color::Rgb(108, 112, 134),     // Overlay0 #6c7086
            text_urgent: Color::Rgb(243, 139, 168),  // Red #f38ba8
            text_warn: Color::Rgb(249, 226, 175),    // Yellow #f9e2af
            text_success: Color::Rgb(166, 227, 161), // Green #a6e3a1
            rrule_fg: Color::Rgb(250, 179, 135),     // Peach #fab387

            border_active: Color::Rgb(137, 180, 250), // Blue #89b4fa
            border_inactive: Color::Rgb(69, 71, 90),  // Surface1 #45475a

            hl_bg: Color::Rgb(49, 50, 68),          // Surface0 #313244
            hl_fg: Color::Rgb(137, 180, 250),       // Blue #89b4fa
            row_active_bg: Color::Rgb(88, 91, 112), // Surface2 #585b70

            status_bg: Color::Rgb(24, 24, 37),    // Mantle #181825
            status_fg: Color::Rgb(186, 194, 222), // Subtext1 #bac2de

            accent: Color::Rgb(137, 180, 250), // Blue #89b4fa

            // Status colors
            status_inbox: Color::Rgb(147, 153, 178), // Overlay2 #9399b2
            status_next: Color::Rgb(166, 227, 161),  // Green #a6e3a1
            status_waiting: Color::Rgb(249, 226, 175), // Yellow #f9e2af
            status_scheduled: Color::Rgb(137, 180, 250), // Blue #89b4fa
            status_someday: Color::Rgb(203, 166, 247), // Mauve #cba6f7
            status_reference: Color::Rgb(148, 226, 213), // Teal #94e2d5
            status_done: Color::Rgb(127, 132, 156),  // Overlay1 #7f849c

            // Priority colors
            priority_high: Color::Rgb(243, 139, 168), // Red #f38ba8
            priority_medium: Color::Rgb(249, 226, 175), // Yellow #f9e2af
            priority_low: Color::Rgb(137, 180, 250),  // Blue #89b4fa
        }
    }

    pub fn catppuccin_latte() -> Self {
        Self {
            id: ThemeId::CatppuccinLatte,
            is_dark: false,
            bg: Color::Rgb(239, 241, 245),         // Base #eff1f5
            fg: Color::Rgb(76, 79, 105),           // Text #4c4f69
            text_dim: Color::Rgb(108, 111, 133),   // Subtext0 #6c6f85
            text_urgent: Color::Rgb(210, 15, 57),  // Red #d20f39
            text_warn: Color::Rgb(223, 142, 29),   // Yellow #df8e1d
            text_success: Color::Rgb(64, 160, 43), // Green #40a02b
            rrule_fg: Color::Rgb(254, 100, 11),    // Peach #fe640b

            border_active: Color::Rgb(30, 102, 245), // Blue #1e66f5
            border_inactive: Color::Rgb(156, 160, 176), // Overlay0 #9ca0b0

            hl_bg: Color::Rgb(204, 208, 218), // Surface0 #ccd0da
            hl_fg: Color::Rgb(30, 102, 245),  // Blue #1e66f5
            row_active_bg: Color::Rgb(172, 176, 190), // Surface2 #acb0be

            status_bg: Color::Rgb(230, 233, 239), // Mantle #e6e9ef
            status_fg: Color::Rgb(76, 79, 105),   // Text #4c4f69

            accent: Color::Rgb(30, 102, 245), // Blue #1e66f5

            // Status colors
            status_inbox: Color::Rgb(138, 143, 161), // Overlay1 #8a8fa1
            status_next: Color::Rgb(64, 160, 43),    // Green #40a02b
            status_waiting: Color::Rgb(223, 142, 29), // Yellow #df8e1d
            status_scheduled: Color::Rgb(30, 102, 245), // Blue #1e66f5
            status_someday: Color::Rgb(136, 57, 239), // Mauve #8839ef
            status_reference: Color::Rgb(23, 146, 153), // Teal #179299
            status_done: Color::Rgb(138, 143, 161),  // Overlay1 #8a8fa1

            // Priority colors
            priority_high: Color::Rgb(210, 15, 57), // Red #d20f39
            priority_medium: Color::Rgb(223, 142, 29), // Yellow #df8e1d
            priority_low: Color::Rgb(30, 102, 245), // Blue #1e66f5
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            id: ThemeId::TokyoNight,
            is_dark: true,
            bg: Color::Rgb(26, 27, 38),              // #1a1b26
            fg: Color::Rgb(192, 202, 245),           // #c0caf5
            text_dim: Color::Rgb(86, 95, 137),       // #565f89
            text_urgent: Color::Rgb(247, 118, 142),  // Red #f7768e
            text_warn: Color::Rgb(224, 175, 104),    // Yellow #e0af68
            text_success: Color::Rgb(158, 206, 106), // Green #9ece6a
            rrule_fg: Color::Rgb(255, 158, 100),     // Orange #ff9e64

            border_active: Color::Rgb(122, 162, 247), // Blue #7aa2f7
            border_inactive: Color::Rgb(65, 72, 104), // Muted #414868

            hl_bg: Color::Rgb(41, 46, 66),         // Selection #292e42
            hl_fg: Color::Rgb(122, 162, 247),      // Blue #7aa2f7
            row_active_bg: Color::Rgb(54, 60, 86), // #363c56

            status_bg: Color::Rgb(19, 20, 28), // Dark background #13141c
            status_fg: Color::Rgb(169, 177, 214), // Light fg #a9b1d6

            accent: Color::Rgb(122, 162, 247), // Blue #7aa2f7

            // Status colors
            status_inbox: Color::Rgb(169, 177, 214), // Light fg #a9b1d6
            status_next: Color::Rgb(158, 206, 106),  // Green #9ece6a
            status_waiting: Color::Rgb(224, 175, 104), // Yellow #e0af68
            status_scheduled: Color::Rgb(122, 162, 247), // Blue #7aa2f7
            status_someday: Color::Rgb(187, 154, 247), // Bright magenta #bb9af7
            status_reference: Color::Rgb(13, 185, 215), // Bright cyan #0db9d7
            status_done: Color::Rgb(86, 95, 137),    // Dark fg #565f89

            // Priority colors
            priority_high: Color::Rgb(247, 118, 142), // Red #f7768e
            priority_medium: Color::Rgb(224, 175, 104), // Yellow #e0af68
            priority_low: Color::Rgb(122, 162, 247),  // Blue #7aa2f7
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            id: ThemeId::Gruvbox,
            is_dark: true,
            bg: Color::Rgb(40, 40, 40),              // Background #282828
            fg: Color::Rgb(212, 190, 152),           // Foreground #d4be98
            text_dim: Color::Rgb(124, 111, 100),     // Dark fg #7c6f64
            text_urgent: Color::Rgb(234, 105, 98),   // Red #ea6962
            text_warn: Color::Rgb(216, 166, 87),     // Yellow #d8a657
            text_success: Color::Rgb(169, 182, 101), // Green #a9b665
            rrule_fg: Color::Rgb(225, 135, 92),      // Orange #e1875c

            border_active: Color::Rgb(125, 174, 163), // Aqua/Blue #7daea3
            border_inactive: Color::Rgb(102, 92, 84), // Muted #665c54

            hl_bg: Color::Rgb(60, 56, 54),         // Lighter bg #3c3836
            hl_fg: Color::Rgb(216, 166, 87),       // Yellow #d8a657
            row_active_bg: Color::Rgb(80, 73, 69), // Selection #504945

            status_bg: Color::Rgb(30, 30, 30),    // Dark bg #1e1e1e
            status_fg: Color::Rgb(189, 174, 147), // Light fg #bdae93

            accent: Color::Rgb(125, 174, 163), // Aqua #7daea3

            // Status colors
            status_inbox: Color::Rgb(189, 174, 147), // Light fg #bdae93
            status_next: Color::Rgb(169, 182, 101),  // Green #a9b665
            status_waiting: Color::Rgb(216, 166, 87), // Yellow #d8a657
            status_scheduled: Color::Rgb(125, 174, 163), // Blue #7daea3
            status_someday: Color::Rgb(211, 134, 155), // Magenta #d3869b
            status_reference: Color::Rgb(137, 180, 130), // Cyan/Green #89b482
            status_done: Color::Rgb(124, 111, 100),  // Muted #7c6f64

            // Priority colors
            priority_high: Color::Rgb(234, 105, 98), // Red #ea6962
            priority_medium: Color::Rgb(216, 166, 87), // Yellow #d8a657
            priority_low: Color::Rgb(125, 174, 163), // Aqua #7daea3
        }
    }

    pub fn retro_82() -> Self {
        Self {
            id: ThemeId::Retro82,
            is_dark: true,
            bg: Color::Rgb(5, 24, 46),              // Deep navy #05182e
            fg: Color::Rgb(246, 220, 172),          // Amber cream #f6dcac
            text_dim: Color::Rgb(63, 143, 138),     // Teal #3f8f8a
            text_urgent: Color::Rgb(248, 85, 37),   // Neon red/orange #f85525
            text_warn: Color::Rgb(233, 123, 60),    // Warm orange #e97b3c
            text_success: Color::Rgb(53, 162, 167), // Bright teal #35a2a7
            rrule_fg: Color::Rgb(250, 169, 104),    // Amber accent #faa968

            border_active: Color::Rgb(250, 169, 104), // Amber accent #faa968
            border_inactive: Color::Rgb(42, 107, 120), // Muted teal #2a6b78

            hl_bg: Color::Rgb(19, 78, 90),          // Selection #134e5a
            hl_fg: Color::Rgb(250, 169, 104),       // Amber #faa968
            row_active_bg: Color::Rgb(30, 95, 108), // #1e5f6c

            status_bg: Color::Rgb(3, 18, 34),     // Dark bg #031222
            status_fg: Color::Rgb(167, 201, 198), // Light fg #a7c9c6

            accent: Color::Rgb(250, 169, 104), // Warm amber #faa968

            // Status colors
            status_inbox: Color::Rgb(167, 201, 198), // Light fg #a7c9c6
            status_next: Color::Rgb(53, 162, 167),   // Bright teal #35a2a7
            status_waiting: Color::Rgb(233, 123, 60), // Orange #e97b3c
            status_scheduled: Color::Rgb(250, 169, 104), // Amber #faa968
            status_someday: Color::Rgb(101, 165, 161), // Bright magenta/cyan #65a5a1
            status_reference: Color::Rgb(140, 191, 184), // Bright cyan #8cbfb8
            status_done: Color::Rgb(63, 143, 138),   // Dim teal #3f8f8a

            // Priority colors
            priority_high: Color::Rgb(248, 85, 37), // Neon red #f85525
            priority_medium: Color::Rgb(233, 123, 60), // Amber #e97b3c
            priority_low: Color::Rgb(53, 162, 167), // Teal #35a2a7
        }
    }

    pub fn from_id(id: ThemeId) -> Self {
        match id {
            ThemeId::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemeId::CatppuccinLatte => Self::catppuccin_latte(),
            ThemeId::TokyoNight => Self::tokyo_night(),
            ThemeId::Gruvbox => Self::gruvbox(),
            ThemeId::Retro82 => Self::retro_82(),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().replace('_', "-").as_str() {
            "catppuccin" | "catppuccin-mocha" | "mocha" => Some(Self::catppuccin_mocha()),
            "catppuccin-latte" | "latte" => Some(Self::catppuccin_latte()),
            "tokyo-night" | "tokyonight" => Some(Self::tokyo_night()),
            "gruvbox" => Some(Self::gruvbox()),
            "retro-82" | "retro82" => Some(Self::retro_82()),
            _ => None,
        }
    }

    pub fn status_color(&self, s: &Status) -> Color {
        match s {
            Status::Inbox => self.status_inbox,
            Status::Next => self.status_next,
            Status::Waiting => self.status_waiting,
            Status::Scheduled => self.status_scheduled,
            Status::Someday => self.status_someday,
            Status::Reference => self.status_reference,
            Status::Done => self.status_done,
        }
    }

    pub fn priority_color(&self, p: &str) -> Option<Color> {
        match p {
            "high" | "h" | "1" | "高" => Some(self.priority_high),
            "medium" | "m" | "2" | "中" => Some(self.priority_medium),
            "low" | "l" | "3" | "低" => Some(self.priority_low),
            _ => None,
        }
    }

    pub fn next(&self) -> Self {
        Self::from_id(self.id.next())
    }

    pub fn toggle(&self) -> Self {
        self.next()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}

/// 解析 6 位十六进制 RGB 颜色 (如 "#7aa2f7")。
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let clean = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

/// 检查当前环境是否为 Omarchy 桌面系统。
pub fn is_omarchy_system() -> bool {
    if horae_core::repo::state::is_test_override() {
        return false;
    }
    let Some(home) = horae_core::config::home_dir() else {
        return false;
    };
    home.join(".local/state/omarchy").exists() || home.join(".config/omarchy").exists()
}

/// 从 Omarchy 当前激活的主题探测并返回对应主题配置。
pub fn detect_omarchy_theme() -> Option<Theme> {
    if horae_core::repo::state::is_test_override() {
        return None;
    }
    let home = horae_core::config::home_dir()?;

    // 1. 读取 theme.name 文本文件
    let name_paths = [
        home.join(".local/state/omarchy/current/theme.name"),
        home.join(".config/omarchy/current/theme.name"),
    ];
    for p in &name_paths {
        if let Ok(content) = std::fs::read_to_string(p) {
            let name = content.trim();
            if let Some(t) = Theme::from_name(name) {
                return Some(t);
            }
        }
    }

    // 2. 检查软链接指向的主题目录名称
    let symlink_paths = [
        home.join(".local/state/omarchy/current/theme"),
        home.join(".config/omarchy/current/theme"),
    ];
    for p in &symlink_paths {
        if let Ok(target) = std::fs::read_link(p) {
            if let Some(name) = target.file_name().and_then(|n| n.to_str()) {
                if let Some(t) = Theme::from_name(name) {
                    return Some(t);
                }
            }
        }
    }

    None
}
