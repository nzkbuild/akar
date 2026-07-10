use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::config;

// ---------------------------------------------------------------------------
// Note: context_pack builds a file/path tier list only.
// It does NOT read file contents. It checks existence and mtime, then
// returns paths for display purposes. Content reading is not implemented.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ContextTier {
    Hot,
    Warm,
    Cold,
    External,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContextFile {
    pub path: PathBuf,
    pub tier: ContextTier,
    pub reason: String,
    pub stale: bool,
}

#[derive(Debug)]
pub struct ContextPack {
    pub project_name: String,
    pub files: Vec<ContextFile>,
    pub warnings: Vec<String>,
    pub total_files: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if the file's mtime is older than 30 days.
/// Gracefully returns false on any metadata error.
fn is_stale(path: &PathBuf) -> bool {
    let threshold = Duration::from_secs(30 * 24 * 60 * 60);
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = SystemTime::now().duration_since(modified) {
                return age > threshold;
            }
        }
    }
    false
}

/// Add a file to the list only if it exists on disk.
fn add_if_exists(files: &mut Vec<ContextFile>, path: PathBuf, tier: ContextTier, reason: &str) {
    if path.exists() {
        let stale = is_stale(&path);
        files.push(ContextFile {
            path,
            tier,
            reason: reason.to_string(),
            stale,
        });
    }
}

// ---------------------------------------------------------------------------
// build_pack
// ---------------------------------------------------------------------------

pub fn build_pack(cfg: &config::Config) -> ContextPack {
    let mut files: Vec<ContextFile> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let root = &cfg.project_root;
    let akar = &cfg.akar_dir;

    // --- HOT tier ---
    add_if_exists(
        &mut files,
        root.join("Cargo.toml"),
        ContextTier::Hot,
        "Rust project manifest",
    );
    add_if_exists(
        &mut files,
        root.join("package.json"),
        ContextTier::Hot,
        "Node.js project manifest",
    );

    // All .rs files in src/
    let src_dir = root.join("src");
    if src_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            let mut rs_paths: Vec<PathBuf> = entries
                .flatten()
                .filter_map(|e| {
                    let p = e.path();
                    if p.extension().and_then(|x| x.to_str()) == Some("rs") {
                        Some(p)
                    } else {
                        None
                    }
                })
                .collect();
            rs_paths.sort(); // deterministic order
            for p in rs_paths {
                add_if_exists(&mut files, p, ContextTier::Hot, "Rust source file");
            }
        }
    }

    // --- WARM tier ---
    let readme = root.join("README.md");
    if !readme.exists() {
        warnings.push("no README.md found".to_string());
    }
    add_if_exists(&mut files, readme, ContextTier::Warm, "Project readme");

    if !akar.exists() {
        warnings.push("no .akar/ dir found".to_string());
    }
    add_if_exists(
        &mut files,
        akar.join("PROJECT_DNA.md"),
        ContextTier::Warm,
        "AKAR project DNA",
    );
    add_if_exists(
        &mut files,
        akar.join("DECISIONS.md"),
        ContextTier::Warm,
        "AKAR decisions log",
    );
    add_if_exists(
        &mut files,
        akar.join("DESIGN_DNA.md"),
        ContextTier::Warm,
        "AKAR design DNA",
    );
    add_if_exists(
        &mut files,
        akar.join("MODEL_PROFILE.md"),
        ContextTier::Warm,
        "AKAR model profile",
    );

    // --- COLD tier ---
    add_if_exists(
        &mut files,
        akar.join("LESSONS.md"),
        ContextTier::Cold,
        "AKAR lessons learned",
    );
    add_if_exists(
        &mut files,
        akar.join("KNOWN_BUGS.md"),
        ContextTier::Cold,
        "AKAR known bugs",
    );

    let total_files = files.len();

    ContextPack {
        project_name: cfg.project_name.clone(),
        files,
        warnings,
        total_files,
    }
}

// ---------------------------------------------------------------------------
// format_pack
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn format_pack(pack: &ContextPack) -> String {
    let mut out = String::new();

    out.push_str(&format!("context pack: {}\n", pack.project_name));

    let hot_count = pack
        .files
        .iter()
        .filter(|f| f.tier == ContextTier::Hot)
        .count();
    let warm_count = pack
        .files
        .iter()
        .filter(|f| f.tier == ContextTier::Warm)
        .count();
    let cold_count = pack
        .files
        .iter()
        .filter(|f| f.tier == ContextTier::Cold)
        .count();

    out.push_str(&format!(
        "  tiers: Hot: {}, Warm: {}, Cold: {}\n",
        hot_count, warm_count, cold_count
    ));

    out.push_str("  hot files:\n");
    for f in pack.files.iter().filter(|f| f.tier == ContextTier::Hot) {
        let stale_marker = if f.stale { " [stale]" } else { "" };
        out.push_str(&format!("    - {}{}\n", f.path.display(), stale_marker));
    }

    if !pack.warnings.is_empty() {
        out.push_str("  warnings:\n");
        for w in &pack.warnings {
            out.push_str(&format!("    - {}\n", w));
        }
    }

    out.push_str("  note: context pack is temporary — do not persist\n");

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cfg() -> config::Config {
        config::Config::discover()
    }

    #[test]
    fn build_pack_smoke() {
        // Just verifies it doesn't panic and returns a valid pack.
        let cfg = test_cfg();
        let pack = build_pack(&cfg);
        // total_files must equal files.len()
        assert_eq!(pack.total_files, pack.files.len());
        // project_name is non-empty
        assert!(!pack.project_name.is_empty());
    }

    #[test]
    fn format_pack_returns_nonempty() {
        let cfg = test_cfg();
        let pack = build_pack(&cfg);
        let out = format_pack(&pack);
        assert!(!out.is_empty());
        assert!(out.contains("context pack:"));
        assert!(out.contains("note: context pack is temporary"));
    }

    #[test]
    fn nonexistent_file_not_included() {
        let cfg = config::Config {
            project_root: std::env::current_dir().unwrap(),
            // Point akar_dir somewhere that definitely doesn't exist
            akar_dir: std::path::PathBuf::from("C:\\nonexistent_path_zzzzzz\\.akar"),
            global_dir: config::home_dir().join(".claude").join("akar"),
            project_name: "test".to_string(),
        };
        let pack = build_pack(&cfg);
        // None of the .akar/* files should appear since the dir doesn't exist
        for f in &pack.files {
            assert!(
                f.path.exists(),
                "pack contains a file that does not exist on disk: {}",
                f.path.display()
            );
        }
    }
}
