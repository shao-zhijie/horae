use std::path::{Path, PathBuf};

/// 确保目录存在且具有私有访问权限 (Unix 0700: rwx------)。
/// 若目录不存在则递归创建并设置 0700 权限。
/// 若目录已存在，则收紧其权限，去除 group 和 other 的访问权限。
pub fn ensure_private_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        use std::os::unix::fs::PermissionsExt;

        if !dir.exists() {
            let mut builder = std::fs::DirBuilder::new();
            builder.recursive(true);
            builder.mode(0o700);
            builder.create(dir)?;
        }
        if let Ok(metadata) = std::fs::metadata(dir) {
            let mut perms = metadata.permissions();
            if perms.mode() & 0o077 != 0 {
                perms.set_mode(perms.mode() & !0o077);
                let _ = std::fs::set_permissions(dir, perms);
            }
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)
    }
}

/// 确保已有文件具有私有访问权限 (Unix 0600: rw-------)。
pub fn ensure_private_file(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut perms = metadata.permissions();
                if perms.mode() & 0o077 != 0 {
                    perms.set_mode(perms.mode() & !0o077);
                    let _ = std::fs::set_permissions(path, perms);
                }
            }
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// 写入私有文件，确保新建文件以 0600 模式打开创建并刷新。
pub fn write_private_file(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(contents.as_ref())?;
        file.flush()?;
        drop(file);
        let _ = ensure_private_file(path);
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, contents)
    }
}

/// 针对 SQLite 数据库文件及其 WAL/SHM 辅助文件加固权限 (0600)。
pub fn ensure_sqlite_files_private(path: &Path) -> std::io::Result<()> {
    ensure_private_file(path)?;
    let p_str = path.to_string_lossy();
    let wal = PathBuf::from(format!("{p_str}-wal"));
    let shm = PathBuf::from(format!("{p_str}-shm"));
    let _ = ensure_private_file(&wal);
    let _ = ensure_private_file(&shm);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[cfg(unix)]
    fn test_ensure_private_dir_and_file() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempdir().unwrap();
        let target_dir = temp.path().join("sub/private_dir");

        ensure_private_dir(&target_dir).unwrap();
        assert!(target_dir.is_dir());
        let meta = std::fs::metadata(&target_dir).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o700);

        // Relax permissions to 0755, then re-ensure
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target_dir, perms).unwrap();
        ensure_private_dir(&target_dir).unwrap();
        assert_eq!(
            std::fs::metadata(&target_dir).unwrap().permissions().mode() & 0o777,
            0o700
        );

        // Test private file
        let file_path = target_dir.join("test.txt");
        write_private_file(&file_path, b"secret data").unwrap();
        assert_eq!(
            std::fs::metadata(&file_path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        // Relax file permissions to 0644, then re-ensure
        let mut fperms = std::fs::metadata(&file_path).unwrap().permissions();
        fperms.set_mode(0o644);
        std::fs::set_permissions(&file_path, fperms).unwrap();
        ensure_private_file(&file_path).unwrap();
        assert_eq!(
            std::fs::metadata(&file_path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        // Relax file permissions to 0644, then overwrite via write_private_file
        let mut fperms2 = std::fs::metadata(&file_path).unwrap().permissions();
        fperms2.set_mode(0o644);
        std::fs::set_permissions(&file_path, fperms2).unwrap();
        write_private_file(&file_path, b"updated secret").unwrap();
        assert_eq!(
            std::fs::metadata(&file_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
