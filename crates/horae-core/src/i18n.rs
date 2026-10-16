//! Lightweight UI localization for the TUI.
//!
//! Default language is Chinese (`Lang::Zh`); English (`Lang::En`) can be toggled
//! with F6 in the TUI and is persisted in the `settings` table.
//!
//! Strings are localized in place with the `tr!` macro so translations live
//! next to the code and can't drift from keys:
//!
//! ```ignore
//! crate::tr!(self.lang, "收件箱", "Inbox")
//! ```

/// UI language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Zh,
    En,
}

impl Lang {
    pub fn is_zh(self) -> bool {
        matches!(self, Lang::Zh)
    }

    /// Pick the Chinese or English literal.
    pub fn tr(self, zh: &'static str, en: &'static str) -> &'static str {
        match self {
            Lang::Zh => zh,
            Lang::En => en,
        }
    }

    /// Toggle between Chinese and English.
    pub fn toggle(self) -> Self {
        match self {
            Lang::Zh => Lang::En,
            Lang::En => Lang::Zh,
        }
    }

    /// 检测当前语言：优先从 DB `settings.lang` 读取，次之环境变量 `HORAE_LANG`，缺省为 `Lang::Zh`。
    pub fn detect(conn: Option<&rusqlite::Connection>) -> Self {
        if let Some(conn) = conn {
            if let Ok(Some(val)) = crate::repo::settings::get(conn, "lang") {
                if val == "en" {
                    return Lang::En;
                } else if val == "zh" {
                    return Lang::Zh;
                }
            }
        }
        if let Ok(val) = std::env::var("HORAE_LANG") {
            let lower = val.to_ascii_lowercase();
            if lower == "en" || lower == "english" {
                return Lang::En;
            } else if lower == "zh" || lower == "chinese" {
                return Lang::Zh;
            }
        }
        Lang::Zh
    }
}

/// Pick the translation for the current language.
///
/// Expands to `$lang.tr($zh, $en)` so both literals are always visible at the
/// call site. Optional trailing arguments are substituted into `{}` placeholders
/// left-to-right (each once), e.g.:
///
/// ```ignore
/// crate::tr!(lang, "已归档 {} 项", "archived {} items", count)
/// ```
#[macro_export]
macro_rules! tr {
    ($lang:expr, $zh:literal, $en:literal) => {
        $lang.tr($zh, $en)
    };
    ($lang:expr, $zh:literal, $en:literal, $($arg:expr),* $(,)?) => {{
        if $lang.is_zh() {
            format!($zh, $($arg),*)
        } else {
            format!($en, $($arg),*)
        }
    }};
}
