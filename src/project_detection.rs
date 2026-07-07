//! Shared project detection (v0.31.0).
//!
//! Single canonical detector for AKAR project kind. All other modules
//! consume this — no duplicate marker-file logic anywhere else.
//!
//! Detection rules (unchanged from v0.30.0):
//!   Priority: Rust > Node > Python > Unknown
//!   Markers:  Cargo.toml → Rust
//!             package.json → Node
//!             pyproject.toml | setup.py | requirements.txt → Python
//!             none → Unknown

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// ProjectKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    Rust,
    Node,
    Python,
    Unknown,
}

impl ProjectKind {
    pub fn label(&self) -> &'static str {
        match self {
            ProjectKind::Rust => "Rust",
            ProjectKind::Node => "Node",
            ProjectKind::Python => "Python",
            ProjectKind::Unknown => "Unknown",
        }
    }
}

// ---------------------------------------------------------------------------
// ProjectDetection
// ---------------------------------------------------------------------------

/// Rich detection result: what was found, where, and why.
#[derive(Debug, Clone)]
pub struct ProjectDetection {
    pub kind: ProjectKind,
    pub root: PathBuf,
    /// The marker file that triggered detection, if any.
    pub marker_file: Option<PathBuf>,
    /// Human-readable label, e.g. "Rust (Cargo.toml)".
    pub label: String,
    /// Why this kind was chosen (priority explanation).
    pub reason: String,
}

// ---------------------------------------------------------------------------
// detect_project_kind
// ---------------------------------------------------------------------------

/// Detect the project kind from marker files in `project_root`.
///
/// Priority: Rust > Node > Python > Unknown.
/// Only checks for file existence — no file parsing, no network access.
pub fn detect_project_kind(project_root: &Path) -> ProjectKind {
    if project_root.join("Cargo.toml").exists() {
        return ProjectKind::Rust;
    }
    if project_root.join("package.json").exists() {
        return ProjectKind::Node;
    }
    if project_root.join("pyproject.toml").exists()
        || project_root.join("setup.py").exists()
        || project_root.join("requirements.txt").exists()
    {
        return ProjectKind::Python;
    }
    ProjectKind::Unknown
}

// ---------------------------------------------------------------------------
// detect_project
// ---------------------------------------------------------------------------

/// Full detection with marker file and reason for audits/reporting.
pub fn detect_project(project_root: &Path) -> ProjectDetection {
    let cargo = project_root.join("Cargo.toml");
    if cargo.exists() {
        return ProjectDetection {
            kind: ProjectKind::Rust,
            root: project_root.to_path_buf(),
            marker_file: Some(cargo),
            label: "Rust (Cargo.toml)".to_string(),
            reason: "Cargo.toml found — Rust has highest priority (priority 1/4).".to_string(),
        };
    }

    let pkg_json = project_root.join("package.json");
    if pkg_json.exists() {
        return ProjectDetection {
            kind: ProjectKind::Node,
            root: project_root.to_path_buf(),
            marker_file: Some(pkg_json),
            label: "Node (package.json)".to_string(),
            reason: "package.json found, no Cargo.toml — Node (priority 2/4).".to_string(),
        };
    }

    let pyproject = project_root.join("pyproject.toml");
    let setup_py = project_root.join("setup.py");
    let reqs = project_root.join("requirements.txt");

    let py_marker = if pyproject.exists() {
        Some(pyproject)
    } else if setup_py.exists() {
        Some(setup_py)
    } else if reqs.exists() {
        Some(reqs)
    } else {
        None
    };

    if let Some(ref marker) = py_marker {
        let fname = marker.file_name().unwrap().to_string_lossy();
        let reason = format!(
            "{} found, no Cargo.toml or package.json — Python (priority 3/4).",
            fname
        );
        let label = format!("Python ({})", fname);
        let owned = marker.clone();
        return ProjectDetection {
            kind: ProjectKind::Python,
            root: project_root.to_path_buf(),
            marker_file: Some(owned),
            label,
            reason,
        };
    }

    ProjectDetection {
        kind: ProjectKind::Unknown,
        root: project_root.to_path_buf(),
        marker_file: None,
        label: "Unknown".to_string(),
        reason: "No Cargo.toml, package.json, pyproject.toml, setup.py, or requirements.txt found — Unknown (priority 4/4).".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_project(label: &str, files: &[&str]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "akar_pd_{}_{}",
            label,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for f in files {
            fs::write(dir.join(f), "").unwrap();
        }
        dir
    }

    // ---- detect_project_kind ------------------------------------------------

    #[test]
    fn detect_rust_from_cargo_toml() {
        let dir = temp_project("rust", &["Cargo.toml"]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Rust);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detect_node_from_package_json() {
        let dir = temp_project("node", &["package.json"]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Node);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detect_python_from_pyproject_toml() {
        let dir = temp_project("py", &["pyproject.toml"]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Python);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detect_python_from_setup_py() {
        let dir = temp_project("py_setup", &["setup.py"]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Python);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detect_python_from_requirements_txt() {
        let dir = temp_project("py_req", &["requirements.txt"]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Python);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detect_unknown_from_empty_dir() {
        let dir = temp_project("empty", &[]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Unknown);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rust_priority_over_node() {
        let dir = temp_project("rust_node", &["Cargo.toml", "package.json"]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Rust);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rust_priority_over_python() {
        let dir = temp_project("rust_py", &["Cargo.toml", "pyproject.toml"]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Rust);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn node_priority_over_python() {
        let dir = temp_project("node_py", &["package.json", "setup.py"]);
        assert_eq!(detect_project_kind(&dir), ProjectKind::Node);
        fs::remove_dir_all(&dir).ok();
    }

    // ---- detect_project (rich) ----------------------------------------------

    #[test]
    fn rich_detect_records_marker_file() {
        let dir = temp_project("rich_rust", &["Cargo.toml"]);
        let d = detect_project(&dir);
        assert_eq!(d.kind, ProjectKind::Rust);
        assert!(d.marker_file.is_some());
        assert!(d.marker_file.unwrap().ends_with("Cargo.toml"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rich_detect_unknown_has_no_marker() {
        let dir = temp_project("rich_empty", &[]);
        let d = detect_project(&dir);
        assert_eq!(d.kind, ProjectKind::Unknown);
        assert!(d.marker_file.is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rich_detect_label_is_stable() {
        let dir = temp_project("rich_label", &["package.json"]);
        let d = detect_project(&dir);
        assert_eq!(d.label, "Node (package.json)");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn project_kind_label_returns_stable_strings() {
        assert_eq!(ProjectKind::Rust.label(), "Rust");
        assert_eq!(ProjectKind::Node.label(), "Node");
        assert_eq!(ProjectKind::Python.label(), "Python");
        assert_eq!(ProjectKind::Unknown.label(), "Unknown");
    }
}
