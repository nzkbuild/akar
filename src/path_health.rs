/// PATH version health — detect, report, and optionally repair the `akar`
/// binary on the system PATH so hooks can find the correct version.
///
/// v0.53: Fresh-session dogfood surfaced a stale v0.35.0 on PATH while the
/// running binary was v0.52.0. This module detects that mismatch and offers
/// safe copy-to-PATH repair with explicit confirmation.
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Full health snapshot comparing the running akar binary to PATH akar.
#[derive(Debug, Clone)]
pub struct PathHealth {
    pub running_path: PathBuf,
    pub running_version: String,
    pub path_akar: Option<PathBuf>,
    pub path_version: Option<String>,
    pub status: PathHealthStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathHealthStatus {
    /// PATH akar matches the running binary (same version or same path).
    Healthy,
    /// `akar` not found on PATH (`where`/`which` returned nothing).
    Missing,
    /// Found on PATH but version differs from running binary.
    Mismatch,
    /// Found on PATH but could not determine its version.
    UnknownVersion,
}

/// Result of a PATH repair operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PathRepairResult {
    pub action: String, // "copied", "skipped", "cancelled", "failed"
    pub source: PathBuf,
    pub dest: PathBuf,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Read-only: assess whether the `akar` on PATH matches the running binary.
///
/// Never writes, creates, or modifies files or directories.
pub fn check_path_health() -> PathHealth {
    let running_path = match env::current_exe() {
        Ok(p) => p,
        Err(_) => {
            return PathHealth {
                running_path: PathBuf::new(),
                running_version: env!("CARGO_PKG_VERSION").to_string(),
                path_akar: None,
                path_version: None,
                status: PathHealthStatus::UnknownVersion,
            };
        }
    };

    // Use CARGO_PKG_VERSION for the running binary's version (same binary).
    let running_version = env!("CARGO_PKG_VERSION").to_string();

    let path_akar = find_akar_on_path();

    let path_version = path_akar.as_ref().and_then(|p| extract_version(p));

    let status = match &path_akar {
        None => PathHealthStatus::Missing,
        Some(path) => {
            // If it's the same path as the running binary, it's healthy.
            if paths_equal(path, &running_path) {
                PathHealthStatus::Healthy
            } else {
                match &path_version {
                    Some(v) if v == &running_version => PathHealthStatus::Healthy,
                    Some(_) => PathHealthStatus::Mismatch,
                    None => PathHealthStatus::UnknownVersion,
                }
            }
        }
    };

    PathHealth {
        running_path,
        running_version,
        path_akar,
        path_version,
        status,
    }
}

/// Copy the running binary to the canonical PATH location.
///
/// When `confirmed` is false, returns a would-do result without copying.
///
/// Safety:
/// - Never overwrites a file that doesn't look like an akar binary.
/// - No-op if the running binary is already at the target location.
/// - Uses `std::fs::copy` (not atomic, but safe for this use case).
pub fn repair_path(health: &PathHealth, confirmed: bool) -> PathRepairResult {
    // Skip if already healthy.
    if health.status == PathHealthStatus::Healthy {
        return PathRepairResult {
            action: "skipped".to_string(),
            source: health.running_path.clone(),
            dest: PathBuf::new(),
            detail: "akar on PATH is already healthy — no repair needed".to_string(),
        };
    }

    let source = &health.running_path;

    let dest = match determine_dest(health) {
        Some(d) => d,
        None => {
            return PathRepairResult {
                action: "failed".to_string(),
                source: source.clone(),
                dest: PathBuf::new(),
                detail: "could not find a writable directory on PATH".to_string(),
            };
        }
    };

    // No-op if source == dest.
    if paths_equal(source, &dest) {
        return PathRepairResult {
            action: "skipped".to_string(),
            source: source.clone(),
            dest: dest.clone(),
            detail: "running binary is already at the PATH location".to_string(),
        };
    }

    // If dest exists, verify it looks like an akar binary before overwriting.
    if dest.exists() {
        if !is_akar_binary(&dest) {
            return PathRepairResult {
                action: "skipped".to_string(),
                source: source.clone(),
                dest: dest.clone(),
                detail: "target exists but is not an akar binary — refusing to overwrite"
                    .to_string(),
            };
        }
    }

    if !confirmed {
        return PathRepairResult {
            action: "cancelled".to_string(),
            source: source.clone(),
            dest: dest.clone(),
            detail: format!("would copy {} to {}", source.display(), dest.display()),
        };
    }

    match std::fs::copy(source, &dest) {
        Ok(_) => PathRepairResult {
            action: "copied".to_string(),
            source: source.clone(),
            dest,
            detail: "akar binary copied to PATH location".to_string(),
        },
        Err(e) => PathRepairResult {
            action: "failed".to_string(),
            source: source.clone(),
            dest,
            detail: format!("copy failed: {}", e),
        },
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Find `akar` on the system PATH using `where.exe` (Windows) or `which` (Unix).
fn find_akar_on_path() -> Option<PathBuf> {
    let exe_name = if cfg!(windows) { "akar.exe" } else { "akar" };

    // On Windows, try `where.exe akar` first.
    if cfg!(windows) {
        if let Ok(output) = Command::new("where.exe").arg(exe_name).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        return Some(PathBuf::from(trimmed));
                    }
                }
            }
        }
    }

    // On Unix (or fallback on Windows), try `which akar`.
    if let Ok(output) = Command::new("which").arg(exe_name).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }

    // Final fallback: walk PATH ourselves.
    if let Ok(path_var) = env::var("PATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for dir_str in path_var.split(sep) {
            let candidate = PathBuf::from(dir_str).join(exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Extract version string by running `<binary> --version`.
///
/// Parses output like "akar 0.52.0" → "0.52.0". Returns `None` on any failure.
fn extract_version(binary_path: &Path) -> Option<String> {
    let output = Command::new(binary_path).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for word in stdout.split_whitespace() {
        // Look for a token that looks like a semver: digits.digits.digits
        if word.chars().filter(|c| *c == '.').count() == 2
            && word.chars().all(|c| c.is_ascii_digit() || c == '.')
        {
            return Some(word.to_string());
        }
    }

    None
}

/// Determine where to install the akar binary on PATH.
///
/// Prefers an existing akar location on PATH, then falls back to the first
/// writable PATH directory. Prefers `~/.cargo/bin` if it's on PATH.
fn determine_dest(health: &PathHealth) -> Option<PathBuf> {
    let exe_name: &str = if cfg!(windows) { "akar.exe" } else { "akar" };

    // If PATH akar exists, install to that location.
    if let Some(ref path_akar) = health.path_akar {
        if let Some(parent) = path_akar.parent() {
            if is_dir_writable(parent) {
                return Some(parent.join(exe_name));
            }
        }
    }

    // Walk PATH directories, prefer ~/.cargo/bin.
    let path_var = env::var("PATH").ok()?;
    let sep = if cfg!(windows) { ';' } else { ':' };

    let mut cargo_bin: Option<PathBuf> = None;
    let mut first_writable: Option<PathBuf> = None;

    for dir_str in path_var.split(sep) {
        if dir_str.is_empty() {
            continue;
        }
        let dir = PathBuf::from(dir_str);
        if !is_dir_writable(&dir) {
            continue;
        }
        if cargo_bin.is_none() && is_cargo_bin(&dir) {
            cargo_bin = Some(dir.clone());
        }
        if first_writable.is_none() {
            first_writable = Some(dir);
        }
    }

    let dir = cargo_bin.or(first_writable)?;
    Some(dir.join(exe_name))
}

/// Check whether a directory is writable by probing with a temp file.
fn is_dir_writable(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    let probe = dir.join(format!(".akar_write_probe_{}", std::process::id()));
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Check if a directory path looks like `~/.cargo/bin` (or ends with `/.cargo/bin`).
fn is_cargo_bin(dir: &Path) -> bool {
    let dir_str = dir.to_string_lossy();
    dir_str.ends_with("/.cargo/bin") || dir_str.ends_with("\\.cargo\\bin")
}

/// Check whether a binary file is likely an akar binary by running `--version`.
fn is_akar_binary(path: &Path) -> bool {
    extract_version(path).is_some()
}

/// Compare two paths, accounting for canonicalization differences.
fn paths_equal(a: &Path, b: &Path) -> bool {
    // Fast path: direct string compare.
    if a == b {
        return true;
    }
    // Try canonicalize. If that fails for either, fall back to string compare.
    if let (Ok(ca), Ok(cb)) = (a.canonicalize(), b.canonicalize()) {
        return ca == cb;
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_path_health_running_path_exists() {
        let health = check_path_health();
        assert!(
            health.running_path.exists(),
            "running akar binary should exist"
        );
    }

    #[test]
    fn check_path_health_running_version_matches_cargo() {
        let health = check_path_health();
        assert_eq!(
            health.running_version,
            env!("CARGO_PKG_VERSION"),
            "running version should match Cargo.toml version"
        );
    }

    #[test]
    fn check_path_health_does_not_panic() {
        // Must return a valid result regardless of system state.
        let health = check_path_health();
        assert!(!health.running_version.is_empty());
        // status should be a valid enum variant (no partial/unknown states).
    }

    #[test]
    fn extract_version_parses_valid_output() {
        let version = extract_version_from_str("akar 0.52.0\n");
        assert_eq!(version, Some("0.52.0".to_string()));

        let version2 = extract_version_from_str("akar 0.53.0 (abc123)\n");
        assert_eq!(version2, Some("0.53.0".to_string()));
    }

    #[test]
    fn extract_version_rejects_garbage() {
        assert_eq!(extract_version_from_str("not a version\n"), None);
        assert_eq!(extract_version_from_str(""), None);
        assert_eq!(extract_version_from_str("akar 0.52\n"), None);
        assert_eq!(extract_version_from_str("v1.2.3.4\n"), None);
    }

    #[test]
    fn paths_equal_same_path() {
        let a = PathBuf::from("/usr/local/bin/akar");
        assert!(paths_equal(&a, &a));
    }

    #[test]
    fn paths_equal_different_paths() {
        let a = PathBuf::from("/usr/local/bin/akar");
        let b = PathBuf::from("/usr/bin/akar");
        assert!(!paths_equal(&a, &b));
    }

    #[test]
    fn repair_path_cancelled_when_not_confirmed() {
        let health = check_path_health();
        let result = repair_path(&health, false);
        assert!(
            result.action == "cancelled" || result.action == "skipped",
            "should cancel or skip when not confirmed: {}",
            result.action
        );
    }

    #[test]
    fn repair_path_skips_when_same_path() {
        let running_path = std::env::current_exe().unwrap();
        let health = PathHealth {
            running_path: running_path.clone(),
            running_version: "0.53.0".to_string(),
            path_akar: Some(running_path.clone()),
            path_version: Some("0.53.0".to_string()),
            status: PathHealthStatus::Healthy,
        };
        let result = repair_path(&health, true);
        assert_eq!(result.action, "skipped");
        assert!(result.detail.contains("already"));
    }

    #[test]
    fn test_is_cargo_bin() {
        assert!(is_cargo_bin(Path::new("/home/user/.cargo/bin")));
        assert!(is_cargo_bin(Path::new("C:\\Users\\user\\.cargo\\bin")));
        assert!(!is_cargo_bin(Path::new("/usr/local/bin")));
        assert!(!is_cargo_bin(Path::new("C:\\Windows\\System32")));
    }

    // Helper: extract version from a string (bypasses subprocess for testing).
    fn extract_version_from_str(output: &str) -> Option<String> {
        for word in output.split_whitespace() {
            if word.chars().filter(|c| *c == '.').count() == 2
                && word.chars().all(|c| c.is_ascii_digit() || c == '.')
            {
                return Some(word.to_string());
            }
        }
        None
    }
}
