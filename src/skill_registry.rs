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

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SkillRole {
    Kernel,
    Methodology,
    Execution,
    Support,
    Memory,
    Design,
    Security,
    Dangerous,
    LibraryOnly,
}

// ---------------------------------------------------------------------------
// SkillEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub source: SkillSource,
    pub purpose: String,
    #[allow(dead_code)]
    pub risk: String,
    #[allow(dead_code)]
    pub token_cost: String,
    pub status: SkillStatus,
    pub role: SkillRole,
}

// ---------------------------------------------------------------------------
// classify_role
// ---------------------------------------------------------------------------

/// Classify a skill's role by name patterns and source.
pub fn classify_role(name: &str, source: &SkillSource) -> SkillRole {
    let lower = name.to_lowercase();

    if lower.contains("akar") || *source == SkillSource::Project {
        return SkillRole::Kernel;
    }
    if lower.contains("superpower") || lower.contains("tdd") || lower.contains("plan") || lower.contains("brainstorm") {
        return SkillRole::Methodology;
    }
    if lower.contains("gsd") || lower.contains("shit") || lower.contains("dispatch") || lower.contains("execute") {
        return SkillRole::Execution;
    }
    if lower.contains("recall") || lower.contains("memory") || lower.contains("self-evolve") || lower.contains("checkpoint") {
        return SkillRole::Memory;
    }
    if lower.contains("design") || lower.contains("taste") || lower.contains("visual") || lower.contains("ui") || lower.contains("frontend") {
        return SkillRole::Design;
    }
    if lower.contains("security") || lower.contains("review") {
        return SkillRole::Security;
    }
    if lower.contains("dangerous") || lower.contains("unsafe") {
        return SkillRole::Dangerous;
    }

    SkillRole::Support
}

// ---------------------------------------------------------------------------
// detect_skill_conflicts
// ---------------------------------------------------------------------------

/// Return warnings when skill combinations create conflicts.
pub fn detect_skill_conflicts(skills: &[SkillEntry]) -> Vec<String> {
    let mut warnings = Vec::new();

    // More than one active Methodology skill is a conflict.
    let active_methodology: Vec<&SkillEntry> = skills
        .iter()
        .filter(|s| s.role == SkillRole::Methodology && s.status == SkillStatus::Active)
        .collect();
    if active_methodology.len() > 1 {
        let names: Vec<&str> = active_methodology.iter().map(|s| s.name.as_str()).collect();
        warnings.push(format!(
            "conflict: multiple active Methodology skills — {}",
            names.join(", ")
        ));
    }

    // Any active Dangerous skill is a conflict.
    for s in skills {
        if s.role == SkillRole::Dangerous && s.status == SkillStatus::Active {
            warnings.push(format!(
                "conflict: skill '{}' has role Dangerous and is Active",
                s.name
            ));
        }
    }

    // Kernel skills that are not highest priority (source != Project) are a conflict.
    for s in skills {
        if s.role == SkillRole::Kernel && s.source != SkillSource::Project {
            warnings.push(format!(
                "conflict: kernel skill '{}' is not at highest priority (source={:?})",
                s.name, s.source
            ));
        }
    }

    warnings
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
        let role = classify_role(&name, &source);

        out.push(SkillEntry {
            name,
            source,
            purpose,
            risk: "low".to_string(),
            token_cost: "low".to_string(),
            status: SkillStatus::Active,
            role,
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
        let role = match skill.role {
            SkillRole::Kernel => "kernel",
            SkillRole::Methodology => "methodology",
            SkillRole::Execution => "execution",
            SkillRole::Support => "support",
            SkillRole::Memory => "memory",
            SkillRole::Design => "design",
            SkillRole::Security => "security",
            SkillRole::Dangerous => "dangerous",
            SkillRole::LibraryOnly => "library-only",
        };
        out.push_str(&format!("  - {} ({}) [{}] [{}]\n", skill.name, source, role, status));
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

    fn make_skill(name: &str, source: SkillSource, purpose: &str, status: SkillStatus) -> SkillEntry {
        let role = classify_role(name, &source);
        SkillEntry {
            name: name.to_string(),
            source,
            purpose: purpose.to_string(),
            risk: "low".to_string(),
            token_cost: "low".to_string(),
            status,
            role,
        }
    }

    #[test]
    fn detect_duplicates_finds_matching_purposes() {
        let skills = vec![
            make_skill("alpha", SkillSource::Custom, "do the thing", SkillStatus::Active),
            make_skill("beta", SkillSource::Project, "do the thing", SkillStatus::Active),
            make_skill("gamma", SkillSource::Superpower, "something else", SkillStatus::Active),
        ];

        let dups = detect_duplicates(&skills);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0], ("alpha".to_string(), "beta".to_string()));
    }

    #[test]
    fn detect_duplicates_empty_purpose_ignored() {
        let skills = vec![
            make_skill("a", SkillSource::Custom, "", SkillStatus::Active),
            make_skill("b", SkillSource::Custom, "", SkillStatus::Active),
        ];
        let dups = detect_duplicates(&skills);
        assert!(dups.is_empty(), "empty purposes should not count as duplicates");
    }

    #[test]
    fn format_registry_produces_nonempty_output() {
        let skills = vec![make_skill("my-skill", SkillSource::Project, "does stuff", SkillStatus::Active)];
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
    fn format_registry_shows_role() {
        let skills = vec![make_skill("superpower-foo", SkillSource::Superpower, "methodology skill", SkillStatus::Active)];
        let out = format_registry(&skills);
        assert!(out.contains("methodology"), "format_registry should show role");
    }

    #[test]
    fn classify_role_kernel_by_name() {
        assert_eq!(classify_role("akar-doctor", &SkillSource::Custom), SkillRole::Kernel);
    }

    #[test]
    fn classify_role_kernel_by_source() {
        assert_eq!(classify_role("anything", &SkillSource::Project), SkillRole::Kernel);
    }

    #[test]
    fn classify_role_methodology() {
        assert_eq!(classify_role("superpower-tdd", &SkillSource::Superpower), SkillRole::Methodology);
        assert_eq!(classify_role("writing-plans", &SkillSource::Custom), SkillRole::Methodology);
    }

    #[test]
    fn classify_role_memory() {
        assert_eq!(classify_role("recall", &SkillSource::Custom), SkillRole::Memory);
        assert_eq!(classify_role("memory-maintain", &SkillSource::Custom), SkillRole::Memory);
    }

    #[test]
    fn classify_role_support_default() {
        assert_eq!(classify_role("some-random-tool", &SkillSource::Custom), SkillRole::Support);
    }

    #[test]
    fn detect_skill_conflicts_methodology_conflict() {
        let skills = vec![
            make_skill("superpower-foo", SkillSource::Superpower, "", SkillStatus::Active),
            make_skill("plan-bar", SkillSource::Custom, "", SkillStatus::Active),
        ];
        let conflicts = detect_skill_conflicts(&skills);
        assert!(!conflicts.is_empty(), "two active Methodology skills should conflict");
    }

    #[test]
    fn detect_skill_conflicts_dangerous_active() {
        let skills = vec![
            make_skill("dangerous-tool", SkillSource::Custom, "", SkillStatus::Active),
        ];
        let conflicts = detect_skill_conflicts(&skills);
        assert!(!conflicts.is_empty(), "active Dangerous skill should conflict");
    }

    #[test]
    fn detect_skill_conflicts_no_conflict_clean() {
        let skills = vec![
            make_skill("superpower-foo", SkillSource::Superpower, "", SkillStatus::Active),
            make_skill("recall", SkillSource::Custom, "", SkillStatus::Active),
        ];
        let conflicts = detect_skill_conflicts(&skills);
        // One Methodology + one Memory — no conflict expected
        assert!(conflicts.is_empty(), "no conflict expected, got: {:?}", conflicts);
    }

    #[test]
    fn check_kernel_priority_warns_on_conflicting_names() {
        let skills = vec![
            make_skill("custom-doctor", SkillSource::Custom, "health check", SkillStatus::Active),
            make_skill("my-verify", SkillSource::Superpower, "verify stuff", SkillStatus::Active),
            make_skill("akar-mission", SkillSource::Project, "run mission", SkillStatus::Active),
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
        let skills = vec![make_skill("deploy-helper", SkillSource::Custom, "deploy stuff", SkillStatus::Active)];
        let warnings = check_kernel_priority(&skills);
        assert!(warnings.is_empty());
    }
}
