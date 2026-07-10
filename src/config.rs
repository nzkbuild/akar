use std::path::PathBuf;

/// Core configuration paths for an AKAR session.
#[derive(Debug, Clone)]
pub struct Config {
    /// Absolute path to the project root (cwd at discovery time).
    pub project_root: PathBuf,
    /// Project-local `.akar/` directory.
    pub akar_dir: PathBuf,
    /// Global AKAR config at `~/.claude/akar/`.
    pub global_dir: PathBuf,
    /// Derived from the last component of `project_root`.
    pub project_name: String,
}

impl Config {
    /// Discover paths from the current working directory and the user home dir.
    /// Never panics; falls back to reasonable defaults when env vars are absent.
    pub fn discover() -> Self {
        let project_root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .canonicalize()
            .unwrap_or_else(|e| {
                // canonicalize can fail on Windows if the path doesn't exist yet
                let _ = e;
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            });

        let project_name = project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let akar_dir = project_root.join(".akar");

        let global_dir = home_dir().join(".claude").join("akar");

        Config {
            project_root,
            akar_dir,
            global_dir,
            project_name,
        }
    }

    /// Check whether key directories exist. Returns a list of human-readable issues.
    /// An empty list means everything looks healthy.
    ///
    /// Note: the v0.23 doctor now performs its own granular environment checks
    /// (`doctor::check_environment`) rather than calling this. This method is
    /// retained as a small public Config API exercised by unit tests.
    #[allow(dead_code)]
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !self.project_root.exists() {
            issues.push(format!(
                "project_root does not exist: {}",
                self.project_root.display()
            ));
        }

        if !self.akar_dir.exists() {
            issues.push(format!(
                "project .akar/ dir missing: {}",
                self.akar_dir.display()
            ));
        }

        if !self.global_dir.exists() {
            issues.push(format!(
                "global akar dir missing: {}",
                self.global_dir.display()
            ));
        }

        issues
    }
}

// ---------------------------------------------------------------------------
// Home-dir resolution (std-only, Windows-friendly)
// ---------------------------------------------------------------------------

/// Returns the user home directory.
/// Tries `USERPROFILE` first (Windows), then `HOME` (Unix/macOS), then falls
/// back to the current directory so the binary never hard-crashes.
pub fn home_dir() -> PathBuf {
    if let Ok(p) = std::env::var("USERPROFILE") {
        let pb = PathBuf::from(p);
        if pb.is_absolute() {
            return pb;
        }
    }
    if let Ok(p) = std::env::var("HOME") {
        let pb = PathBuf::from(p);
        if pb.is_absolute() {
            return pb;
        }
    }
    // Last-resort: use cwd
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

// ---------------------------------------------------------------------------
// Secret redaction
// ---------------------------------------------------------------------------

/// Replace common secret patterns in `s` with `[REDACTED]`.
///
/// Patterns covered:
/// - OpenAI / Anthropic-style API keys: `sk-[A-Za-z0-9]{16,}`
/// - `token=<value>` / `key=<value>` / `secret=<value>` (case-insensitive)
/// - `password=<value>`, `api_key=<value>`
/// - `bearer <value>` (case-insensitive)
/// - `authorization: <value>` (case-insensitive)
/// - Long hex strings (32+ hex chars)
#[allow(dead_code)]
pub fn redact(s: &str) -> String {
    // We implement a simple state-machine redactor without regex (no external deps).
    // Each rule is a (prefix_fn, value_end_fn) pair applied left-to-right.
    let s = redact_sk_keys(s);
    let s = redact_kv_secrets(&s);
    let s = redact_long_hex(&s);
    let s = redact_bearer(&s);
    let s = redact_authorization(&s);
    s
}

/// Redact `sk-` prefixed keys.
fn redact_sk_keys(s: &str) -> String {
    redact_pattern(
        s,
        "sk-",
        |c: char| c.is_alphanumeric() || c == '_' || c == '-',
        16,
    )
}

/// Redact `Bearer <token>` (case-insensitive via lowercase pre-scan in redact_pattern_ci).
fn redact_bearer(s: &str) -> String {
    redact_pattern_ci(
        s,
        "bearer ",
        |c: char| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '+' | '/'),
        8,
    )
}

/// Redact `Authorization: <value>` (case-insensitive).
fn redact_authorization(s: &str) -> String {
    redact_pattern_ci(s, "authorization:", |c: char| !matches!(c, '\n' | '\r'), 4)
}

/// Redact `token=X`, `key=X`, `secret=X`, `password=X`, `api_key=X` (case-insensitive).
fn redact_kv_secrets(s: &str) -> String {
    let prefixes = [
        "token=",
        "key=",
        "secret=",
        "password=",
        "api_key=",
        "apikey=",
    ];
    let mut result = s.to_string();
    for prefix in &prefixes {
        // Case-insensitive scan: find lower-case version of prefix
        let lower = result.to_lowercase();
        let mut out = String::with_capacity(result.len());
        let mut pos = 0usize;
        let prefix_bytes = prefix.len();
        while pos < result.len() {
            if lower[pos..].starts_with(prefix) {
                // Emit the prefix as-is (from original case)
                out.push_str(&result[pos..pos + prefix_bytes]);
                pos += prefix_bytes;
                // Consume value chars (non-whitespace, non-quote, non-ampersand)
                let value_start = pos;
                while pos < result.len() {
                    let c = result[pos..].chars().next().unwrap();
                    if c.is_whitespace() || matches!(c, '"' | '\'' | '&' | ';' | '\n' | '\r') {
                        break;
                    }
                    pos += c.len_utf8();
                }
                let value = &result[value_start..pos];
                if value.len() >= 4 {
                    out.push_str("[REDACTED]");
                } else {
                    out.push_str(value);
                }
            } else {
                let c = result[pos..].chars().next().unwrap();
                out.push(c);
                pos += c.len_utf8();
            }
        }
        result = out;
    }
    result
}

/// Redact runs of 32+ consecutive hex characters (looks like a raw token/hash).
#[allow(dead_code)]
fn redact_long_hex(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_hexdigit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_hexdigit() {
                i += 1;
            }
            let run = &chars[start..i];
            if run.len() >= 32 {
                out.push_str("[REDACTED]");
            } else {
                out.extend(run.iter());
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Case-insensitive variant of `redact_pattern`.
/// Matches `prefix` case-insensitively, emits the original-case prefix, then redacts the value.
fn redact_pattern_ci<F>(s: &str, prefix: &str, value_char: F, min_len: usize) -> String
where
    F: Fn(char) -> bool,
{
    let lower_s = s.to_lowercase();
    let lower_prefix = prefix.to_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut pos = 0usize;
    while pos < s.len() {
        if lower_s[pos..].starts_with(&lower_prefix) {
            out.push_str(&s[pos..pos + prefix.len()]);
            pos += prefix.len();
            let value_start = pos;
            while pos < s.len() {
                let c = s[pos..].chars().next().unwrap();
                if !(value_char)(c) {
                    break;
                }
                pos += c.len_utf8();
            }
            let value_len = pos - value_start;
            if value_len >= min_len {
                out.push_str("[REDACTED]");
            } else {
                out.push_str(&s[value_start..pos]);
            }
        } else {
            let c = s[pos..].chars().next().unwrap();
            out.push(c);
            pos += c.len_utf8();
        }
    }
    out
}

/// Generic prefix-based redactor.
/// Scans `s` for occurrences of `prefix`, then consumes chars matching `value_char`
/// and replaces the value portion with `[REDACTED]` if at least `min_len` chars were consumed.
#[allow(dead_code)]
fn redact_pattern<F>(s: &str, prefix: &str, value_char: F, min_len: usize) -> String
where
    F: Fn(char) -> bool,
{
    let mut out = String::with_capacity(s.len());
    let mut pos = 0usize;
    while pos < s.len() {
        if s[pos..].starts_with(prefix) {
            out.push_str(prefix);
            pos += prefix.len();
            let value_start = pos;
            while pos < s.len() {
                let c = s[pos..].chars().next().unwrap();
                if !(value_char)(c) {
                    break;
                }
                pos += c.len_utf8();
            }
            let value_len = pos - value_start;
            if value_len >= min_len {
                out.push_str("[REDACTED]");
            } else {
                out.push_str(&s[value_start..pos]);
            }
        } else {
            let c = s[pos..].chars().next().unwrap();
            out.push(c);
            pos += c.len_utf8();
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Config::discover() --------------------------------------------------

    #[test]
    fn discover_returns_absolute_project_root() {
        let cfg = Config::discover();
        assert!(
            cfg.project_root.is_absolute(),
            "project_root should be absolute, got: {}",
            cfg.project_root.display()
        );
    }

    #[test]
    fn discover_akar_dir_is_child_of_project_root() {
        let cfg = Config::discover();
        assert!(
            cfg.akar_dir.starts_with(&cfg.project_root),
            ".akar_dir should be inside project_root"
        );
        assert!(cfg.akar_dir.ends_with(".akar"));
    }

    #[test]
    fn discover_global_dir_contains_akar() {
        let cfg = Config::discover();
        assert!(
            cfg.global_dir.ends_with("akar"),
            "global_dir last component should be 'akar', got: {}",
            cfg.global_dir.display()
        );
    }

    #[test]
    fn discover_project_name_is_nonempty() {
        let cfg = Config::discover();
        assert!(!cfg.project_name.is_empty());
    }

    // -- Config::validate() --------------------------------------------------

    #[test]
    fn validate_reports_missing_akar_dir_when_absent() {
        let cfg = Config {
            project_root: std::env::current_dir().unwrap(),
            akar_dir: PathBuf::from("/nonexistent/path/.akar"),
            global_dir: PathBuf::from("/nonexistent/global/akar"),
            project_name: "test".to_string(),
        };
        let issues = cfg.validate();
        assert!(
            issues.iter().any(|i| i.contains(".akar")),
            "expected issue about missing .akar dir"
        );
    }

    #[test]
    fn validate_reports_missing_global_dir_when_absent() {
        let cfg = Config {
            project_root: std::env::current_dir().unwrap(),
            akar_dir: PathBuf::from("/nonexistent/path/.akar"),
            global_dir: PathBuf::from("/nonexistent/global/akar"),
            project_name: "test".to_string(),
        };
        let issues = cfg.validate();
        assert!(
            issues.iter().any(|i| i.contains("global akar")),
            "expected issue about missing global akar dir"
        );
    }

    #[test]
    fn validate_empty_when_all_dirs_exist() {
        // Use dirs that definitely exist on any machine
        let root = std::env::current_dir().unwrap();
        let cfg = Config {
            akar_dir: root.clone(), // reuse an existing dir just to pass the check
            global_dir: root.clone(),
            project_name: "test".to_string(),
            project_root: root,
        };
        let issues = cfg.validate();
        assert!(issues.is_empty(), "expected no issues, got: {:?}", issues);
    }

    // -- redact() ------------------------------------------------------------

    #[test]
    fn redact_masks_sk_keys() {
        let input = "Authorization: sk-abc123DEF456ghi789JKL012";
        let out = redact(input);
        assert!(!out.contains("abc123"), "sk- key value should be redacted");
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redact_masks_token_eq() {
        let input = "token=supersecretvalue123";
        let out = redact(input);
        assert!(!out.contains("supersecretvalue123"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redact_masks_long_hex() {
        let input = "hash=aabbccddeeff00112233445566778899aabbccdd done";
        let out = redact(input);
        assert!(!out.contains("aabbccddeeff00112233445566778899aabbccdd"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redact_leaves_short_hex_alone() {
        // 8-char hex should NOT be redacted
        let input = "id=deadbeef rest";
        let out = redact(input);
        assert!(out.contains("deadbeef"), "short hex should not be redacted");
    }

    #[test]
    fn redact_masks_bearer_token() {
        let input = "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9";
        let out = redact(input);
        assert!(!out.contains("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redact_preserves_non_secret_text() {
        let input = "Hello, world! status=ok count=42";
        let out = redact(input);
        // "ok" and "42" are too short to be redacted
        assert!(out.contains("Hello, world!"));
        assert!(out.contains("status=ok") || out.contains("ok"));
    }
}
