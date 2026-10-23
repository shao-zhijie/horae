use std::f32::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Stdio};

/// 内置合成音效算法版本号。升级此版本号会自动刷新 `~/.config/horae/sounds/` 下旧版内置合成缓存。
const SYNTH_CACHE_VERSION: &str = "v3";

/// 语义化音效类型：每个番茄钟阶段与 GTD 核心动作均对应独有的听觉反馈。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundKind {
    /// 开启专注 / 番茄开始：温润饱满的红木马林巴/电钢琴上行大七和弦 (C4/E4-G4-B4-E5)，舒缓入定
    FocusStart,
    /// 专注达成 / 休息开始：温暖醇厚的大七度治愈和弦 (F3/F4-A4-C5-E5)，愉悦释然
    BreakStart,
    /// 休息结束 / 开启新一轮：清润通透的木质风铃泛音 (D4-A4-D5)，温和唤醒不惊扰
    BreakEnd,
    /// 专注终止 / 中途收起：温厚内敛的毛毡下行四度 (A4-E4)，无压力收尾
    FocusStop,
    /// 任务到期 / 闹钟提醒：圆润立体的木琴/颂钵四连音韵律 (G4-B4-D5-G5)，低音量清晰、高音量不刺耳
    TaskDue,
    /// 任务打卡完成：清脆温润的木槌微反馈 (E4-B4-E5)，兼具颗粒质感与柔和听感
    TaskCompleted,
}

impl SoundKind {
    pub const ALL: [Self; 6] = [
        Self::FocusStart,
        Self::BreakStart,
        Self::BreakEnd,
        Self::FocusStop,
        Self::TaskDue,
        Self::TaskCompleted,
    ];

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

    /// 每种音效的目标峰值裕度（相对满刻度 1.0 的比例）。
    /// 兼顾低音量（如 25%~30% 桌面音箱音量）下的清晰可闻度与大音量下的温润不炸耳（约 -3.5dBFS ~ -5.5dBFS）。
    fn master_gain(&self) -> f32 {
        match self {
            Self::FocusStart => 0.62,
            Self::BreakStart => 0.65,
            Self::BreakEnd => 0.62,
            Self::FocusStop => 0.52,
            Self::TaskDue => 0.68,
            Self::TaskCompleted => 0.56,
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

/// 音符定义：(起始偏移秒数, 频率Hz, 相对振幅, 基础衰减速率)
struct NoteDef {
    start_sec: f32,
    freq: f32,
    amp: f32,
    decay: f32,
}

/// 纯 Rust 物理模态合成：模拟红木马林巴 (Rosewood Marimba) 与毛毡电钢琴 (Felt Rhodes) 的温润声学质感。
/// - 音域下移至人耳最舒适的温暖中低频区 (174Hz ~ 659Hz)，避开 1kHz~4kHz 耳道刺耳敏感区；
/// - 高次谐波采用物理模态快衰减（仅在起音前 40~80ms 提供木槌/毛毡触感，随后迅速化为纯净基频与低八度共鸣）；
/// - 叠加早期房间声学漫反射、二阶温暖低通滤波 (1.9kHz) 与模拟磁带软削波 (tanh)，大音量下依然温润耐听。
///
/// 采样率 44.1kHz，16-bit PCM 单声道。
pub fn synth_pcm_wav(kind: SoundKind) -> Vec<u8> {
    let (notes, duration) = match kind {
        SoundKind::FocusStart => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 261.63, // C4 温暖根音
                    amp: 0.55,
                    decay: 2.6,
                },
                NoteDef {
                    start_sec: 0.0,
                    freq: 329.63, // E4
                    amp: 0.72,
                    decay: 2.7,
                },
                NoteDef {
                    start_sec: 0.11,
                    freq: 392.00, // G4
                    amp: 0.74,
                    decay: 2.6,
                },
                NoteDef {
                    start_sec: 0.22,
                    freq: 493.88, // B4
                    amp: 0.70,
                    decay: 2.4,
                },
                NoteDef {
                    start_sec: 0.34,
                    freq: 659.25, // E5
                    amp: 0.64,
                    decay: 2.0,
                },
            ],
            1.65f32,
        ),
        SoundKind::BreakStart => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 174.61, // F3 温暖低音根音衬底
                    amp: 0.45,
                    decay: 2.2,
                },
                NoteDef {
                    start_sec: 0.0,
                    freq: 349.23, // F4
                    amp: 0.74,
                    decay: 2.4,
                },
                NoteDef {
                    start_sec: 0.11,
                    freq: 440.00, // A4
                    amp: 0.72,
                    decay: 2.4,
                },
                NoteDef {
                    start_sec: 0.22,
                    freq: 523.25, // C5
                    amp: 0.68,
                    decay: 2.2,
                },
                NoteDef {
                    start_sec: 0.34,
                    freq: 659.25, // E5 (温润大七度)
                    amp: 0.60,
                    decay: 1.9,
                },
            ],
            1.75f32,
        ),
        SoundKind::BreakEnd => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 293.66, // D4
                    amp: 0.74,
                    decay: 3.0,
                },
                NoteDef {
                    start_sec: 0.14,
                    freq: 440.00, // A4
                    amp: 0.76,
                    decay: 2.6,
                },
                NoteDef {
                    start_sec: 0.28,
                    freq: 587.33, // D5
                    amp: 0.70,
                    decay: 2.2,
                },
            ],
            1.40f32,
        ),
        SoundKind::FocusStop => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 440.00, // A4
                    amp: 0.72,
                    decay: 3.8,
                },
                NoteDef {
                    start_sec: 0.14,
                    freq: 329.63, // E4
                    amp: 0.68,
                    decay: 3.2,
                },
            ],
            1.00f32,
        ),
        SoundKind::TaskDue => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 392.00, // G4
                    amp: 0.76,
                    decay: 3.2,
                },
                NoteDef {
                    start_sec: 0.13,
                    freq: 493.88, // B4
                    amp: 0.74,
                    decay: 3.0,
                },
                NoteDef {
                    start_sec: 0.26,
                    freq: 587.33, // D5
                    amp: 0.76,
                    decay: 2.5,
                },
                NoteDef {
                    start_sec: 0.40,
                    freq: 783.99, // G5 柔和木泛音尾韵
                    amp: 0.62,
                    decay: 2.1,
                },
            ],
            1.45f32,
        ),
        SoundKind::TaskCompleted => (
            vec![
                NoteDef {
                    start_sec: 0.0,
                    freq: 329.63, // E4
                    amp: 0.68,
                    decay: 5.0,
                },
                NoteDef {
                    start_sec: 0.06,
                    freq: 493.88, // B4
                    amp: 0.72,
                    decay: 4.4,
                },
                NoteDef {
                    start_sec: 0.12,
                    freq: 659.25, // E5
                    amp: 0.68,
                    decay: 3.6,
                },
            ],
            0.85f32,
        ),
    };

    let sample_rate = 44100u32;
    let sr = sample_rate as f32;
    // 预留 30ms 前置静音唤醒窗，防止桌面 USB 声卡/音箱休眠唤醒瞬间吞掉第一个音符的起音
    let pre_roll_sec = 0.030f32;
    let total_samples = ((duration + pre_roll_sec) * sr) as usize;
    let mut samples = vec![0.0f32; total_samples];

    for note in &notes {
        let start_idx = ((note.start_sec + pre_roll_sec) * sr) as usize;
        // 等响度高频衰减补偿：基频越高，高次泛音自动收敛，确保高音通透而不刺耳
        let hf_atten = (380.0 / note.freq).clamp(0.28, 1.0);
        let pitch_scale = (400.0 / note.freq).sqrt().clamp(0.75, 1.08);

        for (idx, sample) in samples.iter_mut().enumerate().skip(start_idx) {
            let t = (idx - start_idx) as f32 / sr;

            // 14ms 升余弦 (Raised-Cosine) 丝滑起振窗，零斜率切入，彻底消除数码爆音 (pop/click)
            let attack_sec = 0.014;
            let attack = if t < attack_sec {
                0.5 * (1.0 - (PI * t / attack_sec).cos())
            } else {
                1.0
            };

            // 物理模态包络：高次泛音比基频快 2.5~5.5 倍衰减，既保留前 80ms 清润的木琴/电钢颗粒质感，又避免持续嗡鸣
            let env_sub = attack * (-note.decay * 0.9 * t).exp();
            let env_fund = attack * (-note.decay * t).exp();
            let env_h2 = attack * (-note.decay * 2.4 * t).exp();
            let env_h3 = attack * (-note.decay * 5.2 * t).exp();

            // 起音微幅频率沉降包络，模拟真实琴槌/木片受击瞬间的物理张力变化
            let f = note.freq * (1.0 + 0.005 * (-t * 95.0).exp());

            // 双振子微失谐 (±0.07%) 营造天然声学律动感，告别死板的纯正弦电子味
            let fund =
                ((2.0 * PI * (f * 0.9993) * t).sin() + (2.0 * PI * (f * 1.0007) * t).sin()) * 0.5;
            // 低八度温厚木腔共鸣
            let sub = (2.0 * PI * (f * 0.5) * t).sin();
            // 二次与三次泛音（赋予低音量下的穿透度与高音量下的温润木质感）
            let h2 = (2.0 * PI * (f * 2.0) * t).sin();
            let h3 = (2.0 * PI * (f * 3.0) * t).sin();
            // 毛毡/木槌软触瞬态（220Hz 柔木共振，仅持续约 20ms）
            let wood_touch = (2.0 * PI * 220.0 * t).sin() * attack * (-t * 80.0).exp();

            let val = fund * env_fund * 0.68
                + sub * env_sub * 0.14
                + h2 * env_h2 * 0.18 * hf_atten
                + h3 * env_h3 * 0.05 * hf_atten
                + wood_touch * 0.05;

            *sample += val * note.amp * pitch_scale;
        }
    }

    // 早期声学空间漫反射（29ms 与 63ms 双抽头微混响），赋予声音真实房间的空气感与立体温度
    let delay1 = (0.029 * sr) as usize;
    let delay2 = (0.063 * sr) as usize;
    let dry = samples.clone();
    for i in 0..total_samples {
        if i >= delay1 {
            samples[i] += dry[i - delay1] * 0.16;
        }
        if i >= delay2 {
            samples[i] += dry[i - delay2] * 0.08;
        }
    }

    // 温暖低通滤波器 (截止频率约 2800Hz)：保留中频段清晰度，仅滤除 >3kHz 的尖锐数码毛刺
    let cutoff_hz = 2800.0f32;
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    let dt = 1.0 / sr;
    let alpha = dt / (rc + dt);
    let mut lp = 0.0f32;
    for s in &mut samples {
        lp += alpha * (*s - lp);
        *s = lp;
    }

    // 尾音 90ms 升余弦平滑渐隐窗，确保音频结束于绝对零点，防止播放器截断尾音爆音
    let fade_samples = ((0.090 * sr) as usize).min(total_samples / 4);
    let fade_start = total_samples.saturating_sub(fade_samples);
    for (i, s) in samples[fade_start..].iter_mut().enumerate() {
        let progress = i as f32 / fade_samples.max(1) as f32;
        let window = 0.5 * (1.0 + (PI * progress).cos());
        *s *= window;
    }

    encode_wav(&samples, sample_rate, kind.master_gain())
}

/// 编码 16-bit PCM 单声道 WAV 字节流（带模拟软饱和与舒适动态裕度控制）
fn encode_wav(samples: &[f32], sample_rate: u32, master_gain: f32) -> Vec<u8> {
    let mut peak: f32 = 0.001;
    for &s in samples {
        let abs = s.abs();
        if abs > peak {
            peak = abs;
        }
    }

    let drive = 1.25f32;
    let drive_norm = drive.tanh();
    let ceiling = (32767.0 * master_gain.clamp(0.05, 0.85)).round();

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
        let normalized = s / peak;
        // 模拟磁带软膝饱和 (soft-knee tanh)，压缩突兀波峰并增添偶次温暖听感
        let shaped = (normalized * drive).tanh() / drive_norm;
        let v = (shaped * ceiling).clamp(-32767.0, 32767.0) as i16;
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

/// 获取自定义音效目录：`~/.config/horae/sounds/`
pub fn sound_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("horae").join("sounds"))
}

/// 判断某个 `.wav` 文件是否为旧版 (v1) 自动生成的刺耳合成缓存（峰值固定为 28000 且长度精确等于旧版时长）。
fn is_legacy_v1_synth_wav(path: &Path) -> bool {
    const LEGACY_SIZES: [u64; 6] = [
        44 + 66150 * 2, // focus_start (1.5s) = 132344
        44 + 70560 * 2, // break_start (1.6s) = 141164
        44 + 57330 * 2, // break_end   (1.3s) = 114704
        44 + 35280 * 2, // focus_stop  (0.8s) = 70604
        44 + 44100 * 2, // task_due    (1.0s) = 88244
        44 + 26460 * 2, // task_complete (0.6s) = 52964
    ];
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !LEGACY_SIZES.contains(&meta.len()) {
        return false;
    }
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return false;
    }
    let max_sample = bytes[44..]
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]).unsigned_abs())
        .max()
        .unwrap_or(0);
    (27990..=28005).contains(&max_sample)
}

/// 清理 `~/.config/horae/sounds/` 中遗留的旧版自动合成缓存，确保升级后新音效立即生效。
fn migrate_legacy_sound_cache(dir: &Path) {
    let marker = dir.join(".synth_version");
    let current = fs::read_to_string(&marker).unwrap_or_default();
    if current.trim() == SYNTH_CACHE_VERSION {
        return;
    }
    for kind in SoundKind::ALL {
        let wav_path = dir.join(format!("{}.wav", kind.name()));
        if wav_path.is_file() && is_legacy_v1_synth_wav(&wav_path) {
            let _ = fs::remove_file(&wav_path);
        }
        for old_ver in ["v1", "v2"] {
            let old_builtin = dir.join(format!(".builtin_{}_{}.wav", old_ver, kind.name()));
            if old_builtin.is_file() {
                let _ = fs::remove_file(&old_builtin);
            }
            let old_tmp =
                std::env::temp_dir().join(format!("horae_sound_{}_{}.wav", old_ver, kind.name()));
            if old_tmp.is_file() {
                let _ = fs::remove_file(&old_tmp);
            }
        }
        let legacy_tmp = std::env::temp_dir().join(format!("horae_sound_{}.wav", kind.name()));
        if legacy_tmp.is_file() {
            let _ = fs::remove_file(&legacy_tmp);
        }
    }
    if fs::create_dir_all(dir).is_ok() {
        let _ = fs::write(&marker, SYNTH_CACHE_VERSION);
    }
}

/// 解析指定类型的音效文件路径。优先读取用户自定义文件，未命中时缓存并返回内置高品质合成音。
pub fn resolve_sound_file(kind: SoundKind) -> Option<PathBuf> {
    let name = kind.name();
    if let Some(dir) = sound_dir() {
        migrate_legacy_sound_cache(&dir);

        // 1. 用户自定义音频（支持 wav / oga / ogg / mp3）
        for ext in &["wav", "oga", "ogg", "mp3"] {
            let p = dir.join(format!("{}.{}", name, ext));
            if p.is_file() {
                return Some(p);
            }
        }

        // 2. 尝试从内置版本化缓存读取或自动生成
        let cached_path = dir.join(format!(".builtin_{}_{}.wav", SYNTH_CACHE_VERSION, name));
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
    let tmp_path =
        std::env::temp_dir().join(format!("horae_sound_{}_{}.wav", SYNTH_CACHE_VERSION, name));
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
/// 同步拉起底层播放器子进程后再交由后台线程回收状态，确保：
/// 1. 短生命周期 CLI 命令（如 `horae done` / `horae pomo start`）退出时不会提前掐断音频线程；
/// 2. 长生命周期 TUI / daemon 进程中不会残留僵尸子进程 (`<defunct>`)，且静默屏蔽 stderr 防止破坏 TUI 界面。
pub fn play_with_conn(kind: SoundKind, conn: Option<&rusqlite::Connection>) {
    if !is_enabled(conn) {
        return;
    }

    let Some(path) = resolve_sound_file(kind) else {
        print!("\x07");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        return;
    };

    if let Some(mut child) = spawn_player(&path) {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    } else {
        print!("\x07");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
}

/// 同步阻塞播放指定语义音效（供 `horae sound test` 使用，确保试听完整播放完毕再退出）。
pub fn play_blocking(kind: SoundKind) -> bool {
    if crate::repo::state::is_test_override() || cfg!(test) {
        return false;
    }
    play_internal(kind)
}

fn spawn_player(path: &Path) -> Option<std::process::Child> {
    let p_str = path.to_str()?;
    let candidates: &[(&str, &[&str])] = &[
        ("pw-play", &[p_str]),
        ("paplay", &[p_str]),
        ("aplay", &["-q", p_str]),
        ("mpv", &["--no-terminal", "--really-quiet", p_str]),
        ("canberra-gtk-play", &["-f", p_str]),
    ];

    for &(bin, args) in candidates {
        if let Ok(child) = StdCommand::new(bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            return Some(child);
        }
    }
    None
}

fn play_internal(kind: SoundKind) -> bool {
    let Some(path) = resolve_sound_file(kind) else {
        print!("\x07");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        return false;
    };

    if let Some(mut child) = spawn_player(&path) {
        if child.wait().map(|s| s.success()).unwrap_or(false) {
            return true;
        }
    }

    print!("\x07");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    false
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

    #[test]
    fn test_synth_wav_headroom_and_zero_click_envelope() {
        for kind in SoundKind::ALL {
            let wav = synth_pcm_wav(kind);
            let samples: Vec<i16> = wav[44..]
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                .collect();

            assert!(!samples.is_empty());
            // 首尾采样点必须严格归零，杜绝播放开关瞬间的爆音 (pop/click)
            assert_eq!(samples[0], 0, "{:?} 起始采样点未归零", kind);
            assert_eq!(*samples.last().unwrap(), 0, "{:?} 结尾采样点未归零", kind);

            // 峰值必须受控在清晰且温和的舒适裕度内（低于旧版 28000 满刻度轰炸，同时保证低音量音箱清晰可闻）
            let peak = samples.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
            assert!(
                (15000..=23000).contains(&peak),
                "{:?} 峰值 {} 超出舒适动态区间 [15000, 23000]",
                kind,
                peak
            );
        }
    }

    #[test]
    fn test_migrate_legacy_v1_sound_cache_removes_old_wav_only() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // 构造一个模拟旧版 v1 focus_start.wav (长度 132344，含 28000 峰值)
        let mut fake_v1 = vec![0u8; 132344];
        fake_v1[0..4].copy_from_slice(b"RIFF");
        fake_v1[8..12].copy_from_slice(b"WAVE");
        fake_v1[44..46].copy_from_slice(&28000i16.to_le_bytes());
        let legacy_path = dir.join("focus_start.wav");
        fs::write(&legacy_path, &fake_v1).unwrap();

        // 构造一个用户自定义的 task_due.wav（非旧版特征）
        let custom_path = dir.join("task_due.wav");
        fs::write(&custom_path, b"RIFFcustomWAVEfmt ").unwrap();

        assert!(is_legacy_v1_synth_wav(&legacy_path));
        assert!(!is_legacy_v1_synth_wav(&custom_path));

        migrate_legacy_sound_cache(dir);

        assert!(!legacy_path.exists(), "旧版 v1 刺耳合成缓存应被自动清理");
        assert!(custom_path.exists(), "用户自定义音频文件必须完整保留");
        assert_eq!(
            fs::read_to_string(dir.join(".synth_version"))
                .unwrap()
                .trim(),
            SYNTH_CACHE_VERSION
        );
    }
}
