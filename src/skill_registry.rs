use std::path::Path;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SkillSource {
    ClaudeBundled,
    Superpower,
    Custom,
    Project,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SkillStatus {
    Active,
    Wrapped,
    Disabled,
    Replaced,
    Testing,
}

// ---------------------------------------------------------------------------
// SkillEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub source: SkillSource,
    pub purpose: String,
    pub risk: String,
    pub token_cost: String,
    pub status: SkillStatus,
}

// ---------------------------------------------------------------------------
// scan_skills
// ---------------------------------------------------------------------------

/// Scan `claude_dir/.claude/commands/` for `.md` files and return a registry.
/// Returns an empty vec on any error or if the directory doesn't exist.
pub fn scan_skills(claude_dir: &Path) -> Vec<SkillEntry> {
    let commands_dir = claude_dir.join("commands");
    if !commands_dir.exists() {
        return Vec::new();
    }

    let mut skills = Vec::new();
    collect_skills(&commands_dir, &mut skills);
    skills
}

/// Recursively collect `.md` files under `dir`.
fn collect_skills(dir: &Path, out: &mut Vec<SkillEntry>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_skills(&path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Normalise path separators for matching
        let path_str = path.to_string_lossy().replace('\\', "/");

        let source = if path_str.contains("superpowers") {
            SkillSource::Superpower
        } else if path_str.contains("akar") {
            SkillSource::Project
        } else {
            SkillSource::Custom
        };

        let purpose = read_first_line(&path);

        out.push(SkillEntry {
            name,
            source,
            purpose,
            risk: "low".to_string(),
            token_cost: "low".to_string(),
            status: SkillStatus::Active,
        });
    }
}

/// Read the first non-empty, non-frontmatter line from a file.
/// Returns an empty string on any error or if nothing useful is found.
fn read_first_line(path: &Path) -> String {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let mut in_frontmatter = false;
    let mut frontmatter_seen = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle YAML front-matter fences
        if trimmed == "---" {
            if !frontmatter_seen {
                in_frontmatter = true;
                frontmatter_seen = true;
                continue;
            } else if in_frontmatter {
                in_frontmatter = false;
                continue;
            }
        }

        if in_frontmatter {
            // Look for a `description:` field inside front-matter
            if let Some(rest) = trimmed.strip_prefix("description:") {
                let desc = rest.trim().to_string();
                if !desc.is_empty() {
                    return desc;
                }
            }
            continue;
        }

        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    String::new()
}

// ---------------------------------------------------------------------------
// detect_duplicates
// ---------------------------------------------------------------------------

/// Return pairs of `(name_a, name_b)` where both skills share the same purpose.
/// Only non-empty purposes are compared.
#[allow(dead_code)]
pub fn detect_duplicates(skills: &[SkillEntry]) -> Vec<(String, String)> {
    let mut pairs = Vec::new();

    for i in 0..skills.len() {
        for j in (i + 1)..skills.len() {
            let a = &skills[i];
            let b = &skills[j];
            if !a.purpose.is_empty() && a.purpose == b.purpose {
                pairs.push((a.name.clone(), b.name.clone()));
            }
        }
    }

    pairs
}

// ---------------------------------------------------------------------------
// format_registry
// ---------------------------------------------------------------------------

/// Format the skill registry as a human-readable string.
pub fn format_registry(skills: &[SkillEntry]) -> String {
    let mut out = String::new();
    out.push_str(&format!("skills: {} registered\n", skills.len()));
    for skill in skills {
        let source = match skill.source {
            SkillSource::ClaudeBundled => "claude-bundled",
            SkillSource::Superpower => "superpower",
            SkillSource::Custom => "custom",
            SkillSource::Project => "project",
        };
        let status = match skill.status {
            SkillStatus::Active => "active",
            SkillStatus::Wrapped => "wrapped",
            SkillStatus::Disabled => "disabled",
            SkillStatus::Replaced => "replaced",
            SkillStatus::Testing => "testing",
        };
        out.push_str(&format!("  - {} ({}) [{}]\n", skill.name, source, status));
    }
    out
}

// ---------------------------------------------------------------------------
// check_kernel_priority
// ---------------------------------------------------------------------------

/// Return warnings for any skill whose name contains a kernel-reserved keyword
/// and whose source is not `Project`.
pub fn check_kernel_priority(skills: &[SkillEntry]) -> Vec<String> {
    const KERNEL_KEYWORDS: &[&str] = &["mission", "doctor", "verify", "bootstrap"];

    skills
        .iter()
        .filter(|s| s.source != SkillSource::Project)
        .filter(|s| {
            let lower = s.name.to_lowercase();
            KERNEL_KEYWORDS.iter().any(|kw| lower.contains(kw))
        })
        .map(|s| {
            format!(
                "warning: skill '{}' ({:?}) overrides AKAR kernel behavior",
                s.name, s.source
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn home_dir() -> PathBuf {
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
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    /// Phase 11 created akar-* command files under .claude/commands/.
    /// This test scans the real ~/.claude/ directory and checks we get results.
    #[test]
    fn scan_skills_finds_commands_in_claude_dir() {
        let claude_dir = home_dir().join(".claude");
        // If the directory doesn't exist on this machine, skip gracefully.
        if !claude_dir.join("commands").exists() {
            return;
        }
        let skills = scan_skills(&claude_dir);
        // We found at least one .md file (dev-preferences.md exists)
        assert!(
            !skills.is_empty(),
            "expected at least one skill, found none in {}",
            claude_dir.join("commands").display()
        );
        // Every entry should have a name
        for s in &skills {
            assert!(!s.name.is_empty(), "skill name should not be empty");
        }
    }

    #[test]
    fn scan_skills_on_nonexistent_dir_returns_empty() {
        let fake = PathBuf::from("/nonexistent/path/that/does/not/exist");
        let skills = scan_skills(&fake);
        assert!(skills.is_empty());
    }

    #[test]
    fn detect_duplicates_finds_matching_purposes() {
        let skills = vec![
            SkillEntry {
                name: "alpha".to_string(),
                source: SkillSource::Custom,
                purpose: "do the thing".to_string(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: SkillStatus::Active,
            },
            SkillEntry {
                name: "beta".to_string(),
                source: SkillSource::Project,
                purpose: "do the thing".to_string(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: SkillStatus::Active,
            },
            SkillEntry {
                name: "gamma".to_string(),
                source: SkillSource::Superpower,
                purpose: "something else".to_string(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: SkillStatus::Active,
            },
        ];

        let dups = detect_duplicates(&skills);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0], ("alpha".to_string(), "beta".to_string()));
    }

    #[test]
    fn detect_duplicates_empty_purpose_ignored() {
        let skills = vec![
            SkillEntry {
                name: "a".to_string(),
                source: SkillSource::Custom,
                purpose: String::new(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: SkillStatus::Active,
            },
            SkillEntry {
                name: "b".to_string(),
                source: SkillSource::Custom,
                purpose: String::new(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: SkillStatus::Active,
            },
        ];
        let dups = detect_duplicates(&skills);
        assert!(dups.is_empty(), "empty purposes should not count as duplicates");
    }

    #[test]
    fn format_registry_produces_nonempty_output() {
        let skills = vec![SkillEntry {
            name: "my-skill".to_string(),
            source: SkillSource::Project,
            purpose: "does stuff".to_string(),
            risk: "low".to_string(),
            token_cost: "low".to_string(),
            status: SkillStatus::Active,
        }];
        let out = format_registry(&skills);
        assert!(!out.is_empty());
        assert!(out.contains("my-skill"));
        assert!(out.contains("project"));
        assert!(out.contains("active"));
        assert!(out.contains("1 registered"));
    }

    #[test]
    fn format_registry_zero_skills() {
        let out = format_registry(&[]);
        assert!(out.contains("0 registered"));
    }

    #[test]
    fn check_kernel_priority_warns_on_conflicting_names() {
        let skills = vec![
            SkillEntry {
                name: "custom-doctor".to_string(),
                source: SkillSource::Custom,
                purpose: "health check".to_string(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: SkillStatus::Active,
            },
            SkillEntry {
                name: "my-verify".to_string(),
                source: SkillSource::Superpower,
                purpose: "verify stuff".to_string(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: SkillStatus::Active,
            },
            SkillEntry {
                name: "akar-mission".to_string(),
                source: SkillSource::Project,
                purpose: "run mission".to_string(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: SkillStatus::Active,
            },
        ];

        let warnings = check_kernel_priority(&skills);
        // custom-doctor and my-verify should warn; akar-mission is Project so no warn
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.contains("custom-doctor")));
        assert!(warnings.iter().any(|w| w.contains("my-verify")));
        assert!(!warnings.iter().any(|w| w.contains("akar-mission")));
    }

    #[test]
    fn check_kernel_priority_no_warnings_for_safe_names() {
        let skills = vec![SkillEntry {
            name: "deploy-helper".to_string(),
            source: SkillSource::Custom,
            purpose: "deploy stuff".to_string(),
            risk: "low".to_string(),
            token_cost: "low".to_string(),
            status: SkillStatus::Active,
        }];
        let warnings = check_kernel_priority(&skills);
        assert!(warnings.is_empty());
    }
}
