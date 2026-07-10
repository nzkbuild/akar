/// Backup and restore utilities for safe_fix.
///
/// Timestamp format: seconds since UNIX epoch (no external deps).
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Copy `path` to `<path>.bak.<seconds_since_epoch>` and return the backup path.
///
/// Returns `Err` if the source file cannot be read or the backup cannot be written.
pub fn backup_file(path: &Path) -> Result<PathBuf, String> {
    let ts = seconds_since_epoch();
    let backup_name = format!(
        "{}.bak.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("invalid file name: {}", path.display()))?,
        ts
    );

    let backup_path = match path.parent() {
        Some(parent) => parent.join(&backup_name),
        None => PathBuf::from(&backup_name),
    };

    std::fs::copy(path, &backup_path).map_err(|e| {
        format!(
            "backup_file: failed to copy {} -> {}: {}",
            path.display(),
            backup_path.display(),
            e
        )
    })?;

    Ok(backup_path)
}

/// Copy `backup_path` back to `target`, overwriting it.
///
/// Returns `Err` if the copy fails.
pub fn restore_backup(backup_path: &Path, target: &Path) -> Result<(), String> {
    std::fs::copy(backup_path, target).map_err(|e| {
        format!(
            "restore_backup: failed to copy {} -> {}: {}",
            backup_path.display(),
            target.display(),
            e
        )
    })?;
    Ok(())
}

/// Find the most recent `.bak.*` file for `path` in the same directory.
///
/// "Most recent" is determined by the numeric suffix (largest seconds value).
/// Returns `None` if no backup files are found.
#[allow(dead_code)]
pub fn find_latest_backup(path: &Path) -> Option<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name()?.to_str()?;
    let prefix = format!("{}.bak.", file_name);

    let entries = std::fs::read_dir(parent).ok()?;

    let mut best: Option<(u64, PathBuf)> = None;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        if let Some(suffix) = name_str.strip_prefix(&prefix) {
            // suffix should be the timestamp (numeric seconds)
            if let Ok(ts) = suffix.parse::<u64>() {
                let is_better = best.as_ref().map_or(true, |(best_ts, _)| ts > *best_ts);
                if is_better {
                    best = Some((ts, entry.path()));
                }
            }
        }
    }

    best.map(|(_, p)| p)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn seconds_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    /// Create a temp file with the given content. Returns (dir, file_path).
    fn temp_file(name: &str, content: &[u8]) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "akar_backup_test_{}_{}",
            name,
            seconds_since_epoch()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        let file_path = dir.join(name);
        let mut f = fs::File::create(&file_path).expect("create temp file");
        f.write_all(content).expect("write temp file");
        (dir, file_path)
    }

    #[test]
    fn backup_file_creates_bak_file() {
        let (_dir, path) = temp_file("target.txt", b"hello backup");
        let backup = backup_file(&path).expect("backup_file should succeed");

        assert!(backup.exists(), "backup file should exist");
        let backup_name = backup.file_name().unwrap().to_str().unwrap();
        assert!(
            backup_name.starts_with("target.txt.bak."),
            "backup name should have .bak. suffix, got: {}",
            backup_name
        );

        // Content should match
        let original = fs::read(&path).unwrap();
        let backed_up = fs::read(&backup).unwrap();
        assert_eq!(original, backed_up, "backup content should match original");
    }

    #[test]
    fn backup_file_returns_err_for_missing_source() {
        let missing = PathBuf::from("/nonexistent/__akar_test__/missing.txt");
        assert!(
            backup_file(&missing).is_err(),
            "should fail for non-existent source"
        );
    }

    #[test]
    fn restore_backup_writes_content_back() {
        let (_dir, path) = temp_file("restore_me.txt", b"original content");
        let backup = backup_file(&path).expect("backup");

        // Overwrite the original
        fs::write(&path, b"modified content").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"modified content");

        // Restore
        restore_backup(&backup, &path).expect("restore should succeed");
        assert_eq!(
            fs::read(&path).unwrap(),
            b"original content",
            "content should be restored"
        );
    }

    #[test]
    fn restore_backup_returns_err_for_missing_backup() {
        let target = std::env::temp_dir().join("akar_restore_target_dummy.txt");
        let missing_backup = PathBuf::from("/nonexistent/__akar_test__/missing.bak.0");
        assert!(
            restore_backup(&missing_backup, &target).is_err(),
            "should fail when backup doesn't exist"
        );
    }

    #[test]
    fn find_latest_backup_returns_most_recent() {
        let (_dir, path) = temp_file("versioned.txt", b"v1");

        // Create two backups with a small sleep-free trick: write files directly
        // with synthetic timestamps to guarantee ordering.
        let parent = path.parent().unwrap();
        let old_bak = parent.join("versioned.txt.bak.1000");
        let new_bak = parent.join("versioned.txt.bak.9999");
        fs::write(&old_bak, b"old").unwrap();
        fs::write(&new_bak, b"new").unwrap();

        let latest = find_latest_backup(&path).expect("should find a backup");
        assert_eq!(
            latest.file_name().unwrap().to_str().unwrap(),
            "versioned.txt.bak.9999",
            "should return the backup with the highest timestamp"
        );
    }

    #[test]
    fn find_latest_backup_returns_none_when_no_backups() {
        let (_dir, path) = temp_file("no_backups.txt", b"data");
        // No backups were created for this file
        assert!(
            find_latest_backup(&path).is_none(),
            "should return None when no .bak.* files exist"
        );
    }
}
