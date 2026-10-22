use std::f32::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

/// 语义化音效类型：每个番茄钟阶段与 GTD 核心动作均对应独有的听觉反馈。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundKind {
    /// 开启专注 / 番茄开始：宁静上扬的和弦 (C5-E5-G5-C6)，舒缓杂念，沉淀心神进入心流
    FocusStart,
    /// 专注达成 / 休息开始：温暖圆润的成就感大七度扩展和弦 (F5-A5-C6-E6)，愉悦释放
    BreakStart,
    /// 休息结束 / 开启新一轮：清脆提神的双重钟声 (E5-B5)，温和唤醒重聚专注
    BreakEnd,
    /// 专注终止 / 中途放弃：轻柔低调的下行渐隐音 (E5-B4)，无压力收尾
    FocusStop,
    /// 任务到期 / 闹钟响铃：优雅清晰的双音提示钟声 (A5-E6)，醒目而不刺耳惊悚
    TaskDue,
    /// 任务打卡完成：晶莹剔透的高音叮咚微反馈 (G5-D6)，即时正向多巴胺激励
    TaskCompleted,
}

impl SoundKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FocusStart => "focus_start",
            Self::BreakStart => "break_start",
            Self::BreakEnd => "break_end",
            Self::FocusStop => "focus_stop",
            Self::TaskDue => "task_due",
            Self::TaskCompleted => "task_complete",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "focus_start" | "start" | "focus" => Some(Self::FocusStart),
            "break_start" | "break" | "done" | "work_done" => Some(Self::BreakStart),
            "break_end" | "resume" => Some(Self::BreakEnd),
            "focus_stop" | "stop" | "abort" => Some(Self::FocusStop),
            "task_due" | "due" | "alarm" => Some(Self::TaskDue),
            "task_complete" | "task_completed" | "complete" | "check" => Some(Self::TaskCompleted),
            _ => None,
        }
    }

    /// 对应的系统 freedesktop 备选音效（按语义精准匹配）
    pub fn freedesktop_fallback(&self) -> &'static [&'static str] {
        match self {
            Self::FocusStart => &[
                "/usr/share/sounds/freedesktop/stereo/service-login.oga",
                "/usr/share/sounds/freedesktop/stereo/dialog-information.oga",
            ],
            Self::BreakStart => &["/usr/share/sounds/freedesktop/stereo/complete.oga"],
            Self::BreakEnd => &[
                "/usr/share/sounds/freedesktop/stereo/bell.oga",
                "/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga",
            ],
            Self::FocusStop => &["/usr/share/sounds/freedesktop/stereo/service-logout.oga"],
            Self::TaskDue => &[
                "/usr/share/sounds/freedesktop/stereo/message-new-instant.oga",
                "/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga",
            ],
            Self::TaskCompleted => &[
                "/usr/share/sounds/freedesktop/stereo/complete.oga",
                "/usr/share/sounds/freedesktop/stereo/message-new-instant.oga",
            ],
        }
    }
}

/// 音符定义：(起始偏移秒数, 频率Hz, 相对振幅, 衰减速率)
struct NoteDef {
    start_sec: f32,
    freq: f32,
    amp: f32,
    decay: f32,
}

/// 纯 Rust 合成符合声学声场特征的温暖泛音 WAV 音频。
/// 采样率 44.1kHz，16-bit PCM 单声道。
pub fn synth_pcm_wav(kind: SoundKind) -> Vec<u8> {
    let (notes, duration) = match kind {
        SoundKind::FocusStart => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 523.25, // C5
                    amp: 0.6,
                    decay: 3.2,
                },
                NoteDef {
                    start_sec: 0.12,
                    freq: 659.25, // E5
                    amp: 0.7,
                    decay: 3.2,
                },
                NoteDef {
                    start_sec: 0.24,
                    freq: 783.99, // G5
                    amp: 0.8,
                    decay: 3.0,
                },
                NoteDef {
                    start_sec: 0.36,
                    freq: 1046.50, // C6
                    amp: 0.9,
                    decay: 2.2,
                },
            ],
            1.5f32,
        ),
        SoundKind::BreakStart => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 698.46, // F5
                    amp: 0.6,
                    decay: 2.8,
                },
                NoteDef {
                    start_sec: 0.10,
                    freq: 880.00, // A5
                    amp: 0.7,
                    decay: 2.8,
                },
                NoteDef {
                    start_sec: 0.20,
                    freq: 1046.50, // C6
                    amp: 0.8,
                    decay: 2.5,
                },
                NoteDef {
                    start_sec: 0.30,
                    freq: 1318.51, // E6
                    amp: 0.9,
                    decay: 2.0,
                },
            ],
            1.6f32,
        ),
        SoundKind::BreakEnd => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 659.25, // E5
                    amp: 0.7,
                    decay: 3.5,
                },
                NoteDef {
                    start_sec: 0.18,
                    freq: 987.77, // B5
                    amp: 0.9,
                    decay: 2.8,
                },
            ],
            1.3f32,
        ),
        SoundKind::FocusStop => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 659.25, // E5
                    amp: 0.7,
                    decay: 4.5,
                },
                NoteDef {
                    start_sec: 0.14,
                    freq: 493.88, // B4
                    amp: 0.6,
                    decay: 4.0,
                },
            ],
            0.8f32,
        ),
        SoundKind::TaskDue => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 880.00, // A5
                    amp: 0.8,
                    decay: 4.0,
                },
                NoteDef {
                    start_sec: 0.12,
                    freq: 1318.51, // E6
                    amp: 0.9,
                    decay: 3.0,
                },
            ],
            1.0f32,
        ),
        SoundKind::TaskCompleted => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 783.99, // G5
                    amp: 0.6,
                    decay: 5.0,
                },
                NoteDef {
                    start_sec: 0.08,
                    freq: 1174.66, // D6
                    amp: 0.9,
                    decay: 3.8,
                },
            ],
            0.6f32,
        ),
    };

    let sample_rate = 44100u32;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = vec![0.0f32; total_samples];

    for note in &notes {
        let start_idx = (note.start_sec * sample_rate as f32) as usize;
        for (idx, sample) in samples.iter_mut().enumerate().skip(start_idx) {
            let t = (idx - start_idx) as f32 / sample_rate as f32;
            // 12ms 平滑起振窗，彻底避免开关造成的爆音 (pop/click)
            let attack = (t / 0.012).min(1.0);
            let env = attack * (-note.decay * t).exp();

            // 天然有机声学谐波 (基频 + 八度 + 五度 + 双八度)，富有温暖空间感
            let val = (2.0 * PI * note.freq * t).sin() * 0.65
                + (2.0 * PI * note.freq * 2.0 * t).sin() * 0.22
                + (2.0 * PI * note.freq * 3.0 * t).sin() * 0.09
                + (2.0 * PI * note.freq * 4.0 * t).sin() * 0.04;

            *sample += val * env * note.amp;
        }
    }

    encode_wav(&samples, sample_rate)
}

/// 编码 16-bit PCM 单声道 WAV 字节流
fn encode_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let mut peak: f32 = 0.001;
    for &s in samples {
        let abs = s.abs();
        if abs > peak {
            peak = abs;
        }
    }
    let scale = 28000.0 / peak;

    let num_samples = samples.len() as u32;
    let data_len = num_samples * 2;
    let file_len = data_len + 36;

    let mut buf = Vec::with_capacity((44 + data_len) as usize);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_len.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // subchunk 1 size (16 for PCM)
    buf.extend_from_slice(&1u16.to_le_bytes()); // audio format 1 (PCM)
    buf.extend_from_slice(&1u16.to_le_bytes()); // num channels (1 = mono)
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());

    for &s in samples {
        let v = (s * scale).clamp(-32767.0, 32767.0) as i16;
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

/// 获取自定义音效目录：`~/.config/horae/sounds/`
pub fn sound_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("horae").join("sounds"))
}

/// 解析指定类型的音效文件路径。优先读取用户自定义文件，未命中时缓存并返回内置高品质合成音。
pub fn resolve_sound_file(kind: SoundKind) -> Option<PathBuf> {
    let name = kind.name();
    if let Some(dir) = sound_dir() {
        // 1. 用户自定义音频（支持 wav / oga / ogg / mp3）
        for ext in &["wav", "oga", "ogg", "mp3"] {
            let p = dir.join(format!("{}.{}", name, ext));
            if p.is_file() {
                return Some(p);
            }
        }

        // 2. 尝试从本地缓存或自动生成并缓存内置高品质音频
        let cached_path = dir.join(format!("{}.wav", name));
        if cached_path.is_file() {
            return Some(cached_path);
        }
        if fs::create_dir_all(&dir).is_ok() {
            let wav_bytes = synth_pcm_wav(kind);
            if fs::write(&cached_path, &wav_bytes).is_ok() {
                return Some(cached_path);
            }
        }
    }

    // 3. 若自定义目录不可写，尝试写入 /tmp 临时缓存
    let tmp_path = std::env::temp_dir().join(format!("horae_sound_{}.wav", name));
    if tmp_path.is_file() {
        return Some(tmp_path);
    }
    let wav_bytes = synth_pcm_wav(kind);
    if fs::write(&tmp_path, &wav_bytes).is_ok() {
        return Some(tmp_path);
    }

    // 4. 回退到对应语义的系统 freedesktop 音效
    for &fb in kind.freedesktop_fallback() {
        let p = Path::new(fb);
        if p.is_file() {
            return Some(p.to_path_buf());
        }
    }

    None
}

/// 检查音效是否启用。
/// - 测试环境静音；
/// - 环境变量 HORAE_SOUND=0 / off / false 时静音；
/// - 数据库 settings 表中的 sound_enabled="0" 时静音；
/// - 缺省为 true (默认开启)。
pub fn is_enabled(conn: Option<&rusqlite::Connection>) -> bool {
    if crate::repo::state::is_test_override() || cfg!(test) {
        return false;
    }
    if let Ok(val) = std::env::var("HORAE_SOUND") {
        let v = val.trim().to_ascii_lowercase();
        if v == "0" || v == "false" || v == "off" || v == "none" {
            return false;
        }
    }
    if let Some(conn) = conn {
        if let Ok(Some(val)) = crate::repo::settings::get(conn, "sound_enabled") {
            if val.trim() == "0" {
                return false;
            }
        }
    } else if let Ok(conn) = crate::db::conn::open(None) {
        if let Ok(Some(val)) = crate::repo::settings::get(&conn, "sound_enabled") {
            if val.trim() == "0" {
                return false;
            }
        }
    }
    true
}

/// 设置音效启用状态并持久化到 settings 表。
pub fn set_enabled(conn: &rusqlite::Connection, enabled: bool) -> anyhow::Result<()> {
    crate::repo::settings::set(conn, "sound_enabled", if enabled { "1" } else { "0" })?;
    Ok(())
}

/// 异步播放指定语义音效。
pub fn play(kind: SoundKind) {
    play_with_conn(kind, None);
}

/// 异步播放指定语义音效（显式传入 DB 连接用于校验设置项）。
pub fn play_with_conn(kind: SoundKind, conn: Option<&rusqlite::Connection>) {
    if !is_enabled(conn) {
        return;
    }

    std::thread::spawn(move || {
        let _ = play_internal(kind);
    });
}

fn play_internal(kind: SoundKind) -> bool {
    let Some(path) = resolve_sound_file(kind) else {
        print!("\x07");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        return false;
    };

    let p_str = match path.to_str() {
        Some(s) => s,
        None => return false,
    };

    let success = StdCommand::new("pw-play")
        .arg(p_str)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        || StdCommand::new("paplay")
            .arg(p_str)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        || StdCommand::new("aplay")
            .args(["-q", p_str])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        || StdCommand::new("mpv")
            .args(["--no-terminal", "--really-quiet", p_str])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        || StdCommand::new("canberra-gtk-play")
            .args(["-f", p_str])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

    if !success {
        print!("\x07");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
    success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_kinds_have_valid_wav_headers() {
        let kinds = [
            SoundKind::FocusStart,
            SoundKind::BreakStart,
            SoundKind::BreakEnd,
            SoundKind::FocusStop,
            SoundKind::TaskDue,
            SoundKind::TaskCompleted,
        ];

        for kind in kinds {
            let wav = synth_pcm_wav(kind);
            assert!(wav.len() > 44, "{:?} 生成的 wav 长度过短", kind);
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
            assert_eq!(&wav[12..16], b"fmt ");
            assert_eq!(&wav[36..40], b"data");

            // 采样率必须为 44100
            let sr = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
            assert_eq!(sr, 44100);
            // 单声道 16-bit
            let channels = u16::from_le_bytes([wav[22], wav[23]]);
            let bits = u16::from_le_bytes([wav[34], wav[35]]);
            assert_eq!(channels, 1);
            assert_eq!(bits, 16);
        }
    }

    #[test]
    fn test_resolve_sound_file_fallback_or_synth() {
        let path = resolve_sound_file(SoundKind::FocusStart);
        assert!(path.is_some(), "应能解析或生成音效路径");
        let p = path.unwrap();
        assert!(p.exists(), "音效文件路径必须存在");
    }

    #[test]
    fn test_sound_enabled_toggle_and_defaults() {
        let mut conn = rusqlite::Connection::open(":memory:").unwrap();
        crate::db::migrate::run(&mut conn).unwrap();

        // 默认情况下 sound_enabled 未设置，逻辑缺省为开启 (true)
        // 注意在测试模式下 is_enabled 会因为 test_mode 返回 false，但针对非 test_mode 的逻辑：
        let val = crate::repo::settings::get(&conn, "sound_enabled").unwrap();
        assert_eq!(val, None, "未设置前应为 None (即缺省开启)");

        // 显式关闭
        set_enabled(&conn, false).unwrap();
        assert_eq!(
            crate::repo::settings::get(&conn, "sound_enabled")
                .unwrap()
                .as_deref(),
            Some("0")
        );

        // 显式开启
        set_enabled(&conn, true).unwrap();
        assert_eq!(
            crate::repo::settings::get(&conn, "sound_enabled")
                .unwrap()
                .as_deref(),
            Some("1")
        );
    }

    #[test]
    fn test_sound_kind_from_str() {
        assert_eq!(
            SoundKind::from_str_name("focus_start"),
            Some(SoundKind::FocusStart)
        );
        assert_eq!(
            SoundKind::from_str_name("break_start"),
            Some(SoundKind::BreakStart)
        );
        assert_eq!(
            SoundKind::from_str_name("break_end"),
            Some(SoundKind::BreakEnd)
        );
        assert_eq!(
            SoundKind::from_str_name("focus_stop"),
            Some(SoundKind::FocusStop)
        );
        assert_eq!(
            SoundKind::from_str_name("task_due"),
            Some(SoundKind::TaskDue)
        );
        assert_eq!(
            SoundKind::from_str_name("task_complete"),
            Some(SoundKind::TaskCompleted)
        );
        assert_eq!(SoundKind::from_str_name("unknown"), None);
    }
}
