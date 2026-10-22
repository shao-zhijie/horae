//! 密码保护模式 (Password Protection Mode)。
//!
//! 提供基于 PBKDF2-HMAC-SHA256 的密码哈希与加盐验证，保护 horae 数据安全。
//! 默认无密码模式；在设置中开启后，进入应用与访问任务必须验证密码。

use anyhow::{bail, Result};
use hmac::{Hmac, Mac};
use rusqlite::Connection;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub const KEY_PASSWORD_ENABLED: &str = "password_enabled";
pub const KEY_PASSWORD_SALT: &str = "password_salt";
pub const KEY_PASSWORD_HASH: &str = "password_hash";

/// 迭代次数（平衡安全性与启动性能）
pub const PBKDF2_ITERATIONS: u32 = 50_000;

/// 检查当前数据库是否已启用密码保护模式。
pub fn is_password_enabled(conn: &Connection) -> Result<bool> {
    let val = crate::repo::settings::get(conn, KEY_PASSWORD_ENABLED)?;
    Ok(matches!(val.as_deref(), Some("1")))
}

/// 检查当前数据库是否已设置过密码（无论当前开关状态）。
pub fn has_password(conn: &Connection) -> Result<bool> {
    let hash = crate::repo::settings::get(conn, KEY_PASSWORD_HASH)?;
    Ok(hash.is_some_and(|h| !h.is_empty()))
}

/// 设置新密码并启用密码保护模式。
pub fn set_password(conn: &Connection, password: &str) -> Result<()> {
    if password.trim().is_empty() {
        bail!("密码不能为空");
    }

    // 生成 16 字节随机盐
    let salt_uuid = uuid::Uuid::new_v4();
    let salt_bytes = salt_uuid.as_bytes();
    let salt_hex = hex_encode(salt_bytes);

    let hash_bytes = pbkdf2_sha256(password.as_bytes(), salt_bytes, PBKDF2_ITERATIONS);
    let hash_hex = hex_encode(&hash_bytes);

    crate::repo::settings::set(conn, KEY_PASSWORD_SALT, &salt_hex)?;
    crate::repo::settings::set(conn, KEY_PASSWORD_HASH, &hash_hex)?;
    crate::repo::settings::set(conn, KEY_PASSWORD_ENABLED, "1")?;

    Ok(())
}

/// 验证密码是否正确。
pub fn verify_password(conn: &Connection, password: &str) -> Result<bool> {
    let salt_hex = match crate::repo::settings::get(conn, KEY_PASSWORD_SALT)? {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(false),
    };
    let expected_hash_hex = match crate::repo::settings::get(conn, KEY_PASSWORD_HASH)? {
        Some(h) if !h.is_empty() => h,
        _ => return Ok(false),
    };

    let salt_bytes = match hex_decode(&salt_hex) {
        Some(b) => b,
        None => return Ok(false),
    };
    let expected_hash_bytes = match hex_decode(&expected_hash_hex) {
        Some(b) => b,
        None => return Ok(false),
    };

    let computed_hash = pbkdf2_sha256(password.as_bytes(), &salt_bytes, PBKDF2_ITERATIONS);
    Ok(constant_time_eq(&computed_hash, &expected_hash_bytes))
}

/// 禁用密码保护模式。
pub fn disable_password(conn: &Connection) -> Result<()> {
    crate::repo::settings::set(conn, KEY_PASSWORD_ENABLED, "0")?;
    Ok(())
}

/// 修改密码：需提供原密码验证。原密码正确返回 Ok(true)，错误返回 Ok(false)。
pub fn change_password(conn: &Connection, old_pass: &str, new_pass: &str) -> Result<bool> {
    if !verify_password(conn, old_pass)? {
        return Ok(false);
    }
    set_password(conn, new_pass)?;
    Ok(true)
}

/// PBKDF2-HMAC-SHA256 单块算法 (RFC 8018, dkLen = 32)。
pub fn pbkdf2_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(password).expect("HMAC accepts any key size");
    mac.update(salt);
    mac.update(&1u32.to_be_bytes());
    let mut u = mac.finalize().into_bytes();
    let mut result = u;

    for _ in 1..iterations {
        let mut mac = HmacSha256::new_from_slice(password).expect("HMAC key");
        mac.update(&u);
        u = mac.finalize().into_bytes();
        for (r, byte) in result.iter_mut().zip(u.iter()) {
            *r ^= byte;
        }
    }
    result.into()
}

/// 恒定时间字节比较，防止时序侧信道攻击。
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc6070_pbkdf2_sha256_vector_1() {
        // P = "password", S = "salt", c = 1, dkLen = 32
        let p = b"password";
        let s = b"salt";
        let out = pbkdf2_sha256(p, s, 1);
        let expected = "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b";
        assert_eq!(hex_encode(&out), expected);
    }

    #[test]
    fn rfc6070_pbkdf2_sha256_vector_2() {
        // P = "password", S = "salt", c = 2, dkLen = 32
        let p = b"password";
        let s = b"salt";
        let out = pbkdf2_sha256(p, s, 2);
        let expected = "ae4d0c95af6b46d32d0adff928f06dd02a303f8ef3c251dfd6e2d85a95474c43";
        assert_eq!(hex_encode(&out), expected);
    }

    #[test]
    fn password_lifecycle() {
        let mut conn = Connection::open(":memory:").unwrap();
        crate::db::migrate::run(&mut conn).unwrap();

        // 默认未开启
        assert!(!is_password_enabled(&conn).unwrap());
        assert!(!has_password(&conn).unwrap());

        // 设置密码
        set_password(&conn, "my_secret_123").unwrap();
        assert!(is_password_enabled(&conn).unwrap());
        assert!(has_password(&conn).unwrap());

        // 验证密码
        assert!(verify_password(&conn, "my_secret_123").unwrap());
        assert!(!verify_password(&conn, "wrong_password").unwrap());
        assert!(!verify_password(&conn, "").unwrap());

        // 修改密码
        assert!(!change_password(&conn, "wrong", "new_secret").unwrap());
        assert!(change_password(&conn, "my_secret_123", "new_secret").unwrap());
        assert!(verify_password(&conn, "new_secret").unwrap());
        assert!(!verify_password(&conn, "my_secret_123").unwrap());

        // 禁用密码模式
        disable_password(&conn).unwrap();
        assert!(!is_password_enabled(&conn).unwrap());
        // 依然保留了历史设置密码
        assert!(has_password(&conn).unwrap());
    }
}
