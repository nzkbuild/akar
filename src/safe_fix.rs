/// Safe, reversible fixes that the doctor `--fix` mode can apply.
///
/// Every fix that overwrites an existing file backs it up first via `backup::backup_file`.

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Fix variants
// ---------------------------------------------------------------------------

/// The set of fixes that `akar doctor --fix` can apply automatically.
#[derive(Debug, Clone)]
pub enum SafeFix {
    /// Create a missing directory (no backup needed — nothing to overwrite).
    CreateMissingDir(PathBuf),
    /// Copy a template file from `template_dir/<template_name>` to `dest`.
    /// Backs up `dest` first if it already exists.
    CreateMissingTemplate {
        dest: PathBuf,
        template_name: String,
    },
}

// ---------------------------------------------------------------------------
// apply_safe_fix
// ---------------------------------------------------------------------------

/// Apply a single `SafeFix` and return a human-readable description of what
/// was done, or an error string if the fix could not be applied.
///
/// `template_dir` is the directory that holds template files used by
/// `CreateMissingTemplate`.
pub fn apply_safe_fix(fix: &SafeFix, template_dir: &Path) -> Result<String, String> {
    match fix {
        SafeFix::CreateMissingDir(path) => {
            std::fs::create_dir_all(path).map_err(|e| {
                format!(
                    "CreateMissingDir: failed to create {}: {}",
                    path.display(),
                    e
                )
            })?;
            Ok(format!("created {}", path.display()))
        }

        SafeFix::CreateMissingTemplate {
            dest,
            template_name,
        } => {
            let src = template_dir.join(template_name);

            if !src.exists() {
                return Err(format!(
                    "CreateMissingTemplate: template not found: {}",
                    src.display()
                ));
            }

            // Back up dest if it already exists before overwriting.
            if dest.exists() {
                crate::backup::backup_file(dest).map_err(|e| {
                    format!("CreateMissingTemplate: backup failed for {}: {}", dest.display(), e)
                })?;
            }

            // Ensure parent directory exists.
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "CreateMissingTemplate: failed to create parent dir {}: {}",
                        parent.display(),
                        e
                    )
                })?;
            }

            std::fs::copy(&src, dest).map_err(|e| {
                format!(
                    "CreateMissingTemplate: failed to copy {} -> {}: {}",
                    src.display(),
                    dest.display(),
                    e
                )
            })?;

            Ok(format!(
                "created {} from template",
                dest.display()
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn ts() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("akar_sf_test_{}_{}", label, ts()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    // -- CreateMissingDir -----------------------------------------------------

    #[test]
    fn create_missing_dir_makes_directory() {
        let base = temp_dir("mkdir");
        let target = base.join("new_subdir").join("nested");

        let fix = SafeFix::CreateMissingDir(target.clone());
        let msg = apply_safe_fix(&fix, &base).expect("should succeed");

        assert!(target.exists(), "directory should have been created");
        assert!(
            msg.contains(&target.display().to_string()),
            "message should mention the path"
        );
    }

    #[test]
    fn create_missing_dir_is_idempotent() {
        let base = temp_dir("mkdir_idem");
        let target = base.join("already_exists");
        fs::create_dir_all(&target).unwrap();

        let fix = SafeFix::CreateMissingDir(target.clone());
        assert!(
            apply_safe_fix(&fix, &base).is_ok(),
            "creating an already-existing dir should not error"
        );
    }

    // -- CreateMissingTemplate ------------------------------------------------

    #[test]
    fn create_missing_template_copies_file() {
        let template_dir = temp_dir("tmpl_src");
        let dest_dir = temp_dir("tmpl_dst");

        // Write a template file
        let template_name = "sample.md";
        let template_content = b"# Sample template";
        let mut f = fs::File::create(template_dir.join(template_name)).unwrap();
        f.write_all(template_content).unwrap();

        let dest = dest_dir.join("output.md");

        let fix = SafeFix::CreateMissingTemplate {
            dest: dest.clone(),
            template_name: template_name.to_string(),
        };
        let msg = apply_safe_fix(&fix, &template_dir).expect("should succeed");

        assert!(dest.exists(), "dest file should have been created");
        assert_eq!(
            fs::read(&dest).unwrap(),
            template_content,
            "dest content should match template"
        );
        assert!(
            msg.contains("from template"),
            "message should mention 'from template'"
        );
    }

    #[test]
    fn create_missing_template_backs_up_existing_dest() {
        let template_dir = temp_dir("tmpl_bak_src");
        let dest_dir = temp_dir("tmpl_bak_dst");

        let template_name = "config.toml";
        fs::write(template_dir.join(template_name), b"[new]").unwrap();

        // Pre-existing dest with different content
        let dest = dest_dir.join(template_name);
        fs::write(&dest, b"[old]").unwrap();

        let fix = SafeFix::CreateMissingTemplate {
            dest: dest.clone(),
            template_name: template_name.to_string(),
        };
        apply_safe_fix(&fix, &template_dir).expect("should succeed");

        // Dest should now have new content
        assert_eq!(fs::read(&dest).unwrap(), b"[new]");

        // A backup should exist in dest_dir
        let backup = crate::backup::find_latest_backup(&dest);
        assert!(backup.is_some(), "a backup of the old dest should exist");
        assert_eq!(
            fs::read(backup.unwrap()).unwrap(),
            b"[old]",
            "backup should contain the old content"
        );
    }

    #[test]
    fn create_missing_template_errors_when_template_missing() {
        let template_dir = temp_dir("tmpl_missing");
        let dest_dir = temp_dir("tmpl_missing_dst");
        let dest = dest_dir.join("out.md");

        let fix = SafeFix::CreateMissingTemplate {
            dest,
            template_name: "nonexistent_template.md".to_string(),
        };
        assert!(
            apply_safe_fix(&fix, &template_dir).is_err(),
            "should error when template does not exist"
        );
    }

    // -- NormalizePath --------------------------------------------------------
    // (Removed in v0.22: the NormalizePath variant was dead code — never
    // constructed anywhere, only tested. Its match arm was a no-op.)
}
