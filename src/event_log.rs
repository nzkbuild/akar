//! Append-only event log backed by a JSONL file.
//!
//! Each entry is a single JSON object on one line (no pretty-printing).
//! No external dependencies — uses std only.

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single event log entry.
#[derive(Debug, Clone, PartialEq)]
pub struct EventEntry {
    /// ISO 8601 timestamp, e.g. `"2026-07-03T22:09:31Z"`.
    pub ts: String,
    /// Project name or identifier.
    pub project: String,
    /// Model id that produced this event.
    pub model: String,
    /// Severity: `"failure"` | `"success"` | `"warning"` | `"info"`.
    pub event: String,
    /// Machine-readable event kind, e.g. `"test_run"`, `"bootstrap"`.
    pub event_type: String,
    /// Human-readable one-line description.
    pub summary: String,
    /// How the event was resolved (empty string if not yet resolved).
    pub resolution: String,
    /// Confidence in the outcome: `"low"` | `"medium"` | `"high"`.
    pub confidence: String,
}

// ---------------------------------------------------------------------------
// JSON serialisation (std-only, manual)
// ---------------------------------------------------------------------------

/// Escape a string for use inside a JSON double-quoted value.
/// Covers the characters required by RFC 8259 §7.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                // Other control characters → \uXXXX
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Serialise an `EventEntry` to a single JSON line (no trailing newline).
fn to_json_line(e: &EventEntry) -> String {
    format!(
        r#"{{"ts":"{ts}","project":"{project}","model":"{model}","event":"{event}","event_type":"{event_type}","summary":"{summary}","resolution":"{resolution}","confidence":"{confidence}"}}"#,
        ts         = json_escape(&e.ts),
        project    = json_escape(&e.project),
        model      = json_escape(&e.model),
        event      = json_escape(&e.event),
        event_type = json_escape(&e.event_type),
        summary    = json_escape(&e.summary),
        resolution = json_escape(&e.resolution),
        confidence = json_escape(&e.confidence),
    )
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Append a single event entry to the JSONL log at `log_path`.
///
/// Creates the file if it does not exist.  Never panics — returns
/// `Err(String)` on any I/O failure.
pub fn append_event(log_path: &Path, entry: &EventEntry) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(log_path)
        .map_err(|e| format!("event_log: cannot open {}: {}", log_path.display(), e))?;

    let line = to_json_line(entry);
    writeln!(file, "{}", line)
        .map_err(|e| format!("event_log: write failed on {}: {}", log_path.display(), e))?;

    Ok(())
}

/// Return the last `n` lines from `log_path` as raw strings.
///
/// Returns an empty `Vec` if the file does not exist or is empty.
/// Never panics.
pub fn read_recent(log_path: &Path, n: usize) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let file = match std::fs::File::open(log_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let reader = BufReader::new(file);
    // Collect all lines then take the tail — log files are small enough that
    // reading everything into memory is fine for the sizes AKAR targets.
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.trim().is_empty())
        .collect();

    let start = lines.len().saturating_sub(n);
    lines[start..].to_vec()
}

/// Rotate the log file if it exceeds `max_bytes`.
///
/// Renames `log_path` to `log_path` + `".bak"`, overwriting any existing `.bak`.
/// Returns `true` if a rotation happened, `false` otherwise.
/// Never panics.
#[allow(dead_code)]
pub fn rotate_if_needed(log_path: &Path, max_bytes: u64) -> bool {
    let meta = match std::fs::metadata(log_path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if meta.len() <= max_bytes {
        return false;
    }
    let mut bak = log_path.as_os_str().to_owned();
    bak.push(".bak");
    let bak_path = Path::new(&bak);
    // Best-effort: ignore errors so this never panics.
    std::fs::rename(log_path, bak_path).is_ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sample_entry(summary: &str) -> EventEntry {
        EventEntry {
            ts: "2026-07-03T22:09:31Z".to_string(),
            project: "akar".to_string(),
            model: "claude-opus-4".to_string(),
            event: "info".to_string(),
            event_type: "test_run".to_string(),
            summary: summary.to_string(),
            resolution: "".to_string(),
            confidence: "high".to_string(),
        }
    }

    /// append_event creates the file when it does not exist.
    #[test]
    fn test_append_creates_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("akar_test_append_creates.jsonl");
        let _ = fs::remove_file(&path); // clean slate

        let entry = sample_entry("first entry");
        append_event(&path, &entry).expect("append should succeed");

        assert!(path.exists(), "log file should have been created");
        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("first entry"));

        let _ = fs::remove_file(&path);
    }

    /// append_event is append-only: multiple calls produce multiple lines.
    #[test]
    fn test_append_is_append_only() {
        let dir = std::env::temp_dir();
        let path = dir.join("akar_test_append_only.jsonl");
        let _ = fs::remove_file(&path);

        append_event(&path, &sample_entry("line one")).unwrap();
        append_event(&path, &sample_entry("line two")).unwrap();
        append_event(&path, &sample_entry("line three")).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 3, "expected 3 lines, got {}", lines.len());
        assert!(lines[0].contains("line one"));
        assert!(lines[1].contains("line two"));
        assert!(lines[2].contains("line three"));

        let _ = fs::remove_file(&path);
    }

    /// rotate_if_needed renames the file when it exceeds max_bytes.
    #[test]
    fn test_rotate_if_needed_triggers_at_threshold() {
        let dir = std::env::temp_dir();
        let path = dir.join("akar_test_rotate.jsonl");
        let bak_path = dir.join("akar_test_rotate.jsonl.bak");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&bak_path);

        // Write enough data to exceed a tiny threshold.
        append_event(&path, &sample_entry("entry to trigger rotation")).unwrap();
        let size = fs::metadata(&path).unwrap().len();
        assert!(size > 0);

        // Threshold below current size — should rotate.
        let rotated = rotate_if_needed(&path, size - 1);
        assert!(rotated, "should have rotated");
        assert!(!path.exists(), "original file should be gone after rotation");
        assert!(bak_path.exists(), "backup file should exist after rotation");

        let _ = fs::remove_file(&bak_path);
    }

    /// rotate_if_needed does nothing when the file is within the limit.
    #[test]
    fn test_rotate_if_needed_no_op_below_threshold() {
        let dir = std::env::temp_dir();
        let path = dir.join("akar_test_no_rotate.jsonl");
        let _ = fs::remove_file(&path);

        append_event(&path, &sample_entry("small entry")).unwrap();
        let size = fs::metadata(&path).unwrap().len();

        // Threshold at or above current size — should NOT rotate.
        let rotated = rotate_if_needed(&path, size);
        assert!(!rotated, "should not have rotated");
        assert!(path.exists(), "file should still exist");

        let _ = fs::remove_file(&path);
    }

    /// read_recent returns the correct last n lines.
    #[test]
    fn test_read_recent_returns_correct_lines() {
        let dir = std::env::temp_dir();
        let path = dir.join("akar_test_read_recent.jsonl");
        let _ = fs::remove_file(&path);

        for i in 1..=5 {
            append_event(&path, &sample_entry(&format!("entry {}", i))).unwrap();
        }

        let recent = read_recent(&path, 3);
        assert_eq!(recent.len(), 3, "expected 3 lines back");
        assert!(recent[0].contains("entry 3"));
        assert!(recent[1].contains("entry 4"));
        assert!(recent[2].contains("entry 5"));

        let _ = fs::remove_file(&path);
    }

    /// read_recent on a missing file returns an empty vec (no panic).
    #[test]
    fn test_read_recent_missing_file() {
        let path = std::env::temp_dir().join("akar_test_nonexistent_log.jsonl");
        let _ = std::fs::remove_file(&path);
        let result = read_recent(&path, 10);
        assert!(result.is_empty());
    }

    /// json_escape handles special characters correctly.
    #[test]
    fn test_json_escape_special_chars() {
        assert_eq!(json_escape(r#"say "hello""#), r#"say \"hello\""#);
        assert_eq!(json_escape("line\nnewline"), r"line\nnewline");
        assert_eq!(json_escape("tab\there"), r"tab\there");
        assert_eq!(json_escape("back\\slash"), r"back\\slash");
    }
}
