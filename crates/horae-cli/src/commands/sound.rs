use anyhow::Result;
use horae_core::i18n::Lang;
use horae_core::sound::{self, SoundKind};
use horae_core::tr;
use rusqlite::Connection;

pub fn run(conn: &Connection, action: Option<&str>, name: Option<&str>) -> Result<()> {
    let lang = Lang::detect(Some(conn));
    let action_str = action.unwrap_or("status").trim().to_ascii_lowercase();

    match action_str.as_str() {
        "status" => {
            let enabled = sound::is_enabled(Some(conn));
            if enabled {
                println!(
                    "{}",
                    tr!(
                        lang,
                        "🔊 提示音效: 已开启 (默认)",
                        "🔊 Sound effects: Enabled (Default)"
                    )
                );
            } else {
                println!(
                    "{}",
                    tr!(
                        lang,
                        "🔇 提示音效: 已关闭 (静音)",
                        "🔇 Sound effects: Disabled (Muted)"
                    )
                );
            }
        }
        "on" | "enable" | "1" => {
            sound::set_enabled(conn, true)?;
            println!(
                "{}",
                tr!(
                    lang,
                    "✓ 提示音效已开启 (默认)",
                    "✓ Sound effects enabled (default)"
                )
            );
        }
        "off" | "disable" | "0" => {
            sound::set_enabled(conn, false)?;
            println!(
                "{}",
                tr!(
                    lang,
                    "✓ 提示音效已关闭 (静音)",
                    "✓ Sound effects disabled (muted)"
                )
            );
        }
        "toggle" => {
            let current = sound::is_enabled(Some(conn));
            let next = !current;
            sound::set_enabled(conn, next)?;
            if next {
                println!(
                    "{}",
                    tr!(lang, "✓ 提示音效已开启", "✓ Sound effects enabled")
                );
            } else {
                println!(
                    "{}",
                    tr!(
                        lang,
                        "✓ 提示音效已关闭 (静音)",
                        "✓ Sound effects disabled (muted)"
                    )
                );
            }
        }
        "test" => {
            let kind = name
                .and_then(SoundKind::from_str_name)
                .unwrap_or(SoundKind::FocusStart);
            println!(
                "{}",
                tr!(
                    lang,
                    "▶ 正在测试播放音效: {}",
                    "▶ Testing sound playback: {}",
                    kind.name()
                )
            );
            sound::play(kind);
        }
        other => {
            anyhow::bail!("unknown sound action: {}", other);
        }
    }

    Ok(())
}
