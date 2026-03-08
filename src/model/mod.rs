mod cli;
mod parse;
mod workspace;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub use cli::{cli_heatmap, cli_list, cli_set_status, cli_show};
pub use parse::parse_markdown;
pub use workspace::Workspace;

const ISSUE_FILES: [&str; 3] = ["issues-active.md", "issues-backlog.md", "issues-done.md"];
const DEFAULT_SUBDIR: &str = "docs/issues";
const DEFAULT_ISSUE_CATEGORY: &str = "ISS";

/// Returns the default init path (base/docs/issues)
pub fn default_init_path(base: &Path) -> PathBuf {
    base.join(DEFAULT_SUBDIR)
}

/// Initialize a new ishoo workspace at the exact path specified
pub fn init_workspace_at(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {e}"))?;

    let active = path.join("issues-active.md");
    let backlog = path.join("issues-backlog.md");
    let done = path.join("issues-done.md");

    if active.exists() || backlog.exists() || done.exists() {
        return Err("Issue files already exist in this directory".to_string());
    }

    fs::write(&active, "# ACTIVE Issues\n\n---\n")
        .map_err(|e| format!("Failed to write issues-active.md: {e}"))?;
    fs::write(&backlog, "# BACKLOG Issues\n\n---\n")
        .map_err(|e| format!("Failed to write issues-backlog.md: {e}"))?;
    fs::write(&done, "# DONE Issues\n\n---\n")
        .map_err(|e| format!("Failed to write issues-done.md: {e}"))?;

    Ok(())
}

/// Initialize at base/docs/issues (for CLI)
pub fn init_workspace(base: &Path) -> Result<PathBuf, String> {
    let target = default_init_path(base);
    init_workspace_at(&target)?;
    Ok(target)
}

/// Reinitialize (erase and recreate) at the exact path
pub fn reinit_workspace(path: &Path) -> Result<(), String> {
    let active = path.join("issues-active.md");
    let backlog = path.join("issues-backlog.md");
    let done = path.join("issues-done.md");

    // Remove existing files
    let _ = fs::remove_file(&active);
    let _ = fs::remove_file(&backlog);
    let _ = fs::remove_file(&done);

    // Create fresh
    fs::write(&active, "# ACTIVE Issues\n\n---\n")
        .map_err(|e| format!("Failed to write issues-active.md: {e}"))?;
    fs::write(&backlog, "# BACKLOG Issues\n\n---\n")
        .map_err(|e| format!("Failed to write issues-backlog.md: {e}"))?;
    fs::write(&done, "# DONE Issues\n\n---\n")
        .map_err(|e| format!("Failed to write issues-done.md: {e}"))?;

    Ok(())
}

/// Check if a workspace exists at the given path
pub fn workspace_exists(path: &Path) -> bool {
    ISSUE_FILES.iter().any(|f| path.join(f).exists())
}

/// Searches common subdirectories for issue markdown files.
pub fn discover_root(base: &Path) -> PathBuf {
    let candidates = [
        base.join("docs/issues"),
        base.to_path_buf(),
        base.join("docs"),
        base.join("issues"),
        base.join("ishoo"),
        base.join(".issues"),
    ];

    let matches = candidates
        .iter()
        .filter(|candidate| ISSUE_FILES.iter().any(|f| candidate.join(f).exists()))
        .cloned()
        .collect::<Vec<_>>();

    if let Some(chosen) = matches.first() {
        if matches.len() > 1 {
            eprintln!(
                "Warning: multiple issue roots found; using {}",
                chosen.display()
            );
            for candidate in &matches {
                eprintln!("  - {}", candidate.display());
            }
        }
        return chosen.clone();
    }

    base.join(DEFAULT_SUBDIR)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Open,
    InProgress,
    Done,
    Descoped,
}

impl Status {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::InProgress => "IN PROGRESS",
            Self::Done => "DONE",
            Self::Descoped => "DESCOPED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        let upper = s.trim().to_uppercase();
        if upper.contains("PROGRESS") {
            Self::InProgress
        } else if upper.contains("DONE") {
            Self::Done
        } else if upper.contains("DESCOP") {
            Self::Descoped
        } else {
            Self::Open
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in-progress",
            Self::Done => "done",
            Self::Descoped => "descoped",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub files: Vec<String>,
    pub labels: Vec<String>,
    pub links: Vec<String>,
    pub description: String,
    pub resolution: String,
    pub section: String,
    pub depends_on: Vec<String>,
}

impl Issue {
    pub fn status_ord(&self) -> u8 {
        match self.status {
            Status::InProgress => 0,
            Status::Open => 1,
            Status::Done => 2,
            Status::Descoped => 3,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub open: usize,
    pub in_progress: usize,
    pub done: usize,
    pub descoped: usize,
    pub total: usize,
}

pub fn normalize_issue_category(raw: &str) -> String {
    let cleaned = raw
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_uppercase();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        DEFAULT_ISSUE_CATEGORY.to_string()
    } else {
        trimmed.chars().take(4).collect()
    }
}

pub fn format_issue_id(category: &str, number: u32) -> String {
    format!("{}-{number:02}", normalize_issue_category(category))
}

pub fn split_issue_id(id: &str) -> (String, String) {
    let trimmed = id.trim();
    if let Some((category, number)) = parse_categorical_issue_id(trimmed) {
        return (category, format!("{number:02}"));
    }
    if trimmed.chars().all(|ch| ch.is_ascii_digit()) && !trimmed.is_empty() {
        return (DEFAULT_ISSUE_CATEGORY.to_string(), trimmed.to_string());
    }
    if let Some((category, suffix)) = trimmed.split_once('-') {
        return (category.to_ascii_uppercase(), suffix.to_string());
    }
    (DEFAULT_ISSUE_CATEGORY.to_string(), trimmed.to_string())
}

pub fn issue_id_sort_key(id: &str) -> (String, u32, String) {
    if let Some((category, number)) = parse_categorical_issue_id(id) {
        return (category, number, id.to_string());
    }
    if let Ok(number) = id.parse::<u32>() {
        return (DEFAULT_ISSUE_CATEGORY.to_string(), number, id.to_string());
    }
    let (category, suffix) = split_issue_id(id);
    (category, 0, suffix)
}

pub fn parse_categorical_issue_id(id: &str) -> Option<(String, u32)> {
    let (category, number) = id.trim().split_once('-')?;
    if category.is_empty()
        || category.len() > 4
        || !category.chars().all(|ch| ch.is_ascii_uppercase())
        || number.is_empty()
        || !number.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    Some((category.to_string(), number.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_creates_in_docs_issues() {
        let dir = tempdir().unwrap();
        let result = init_workspace(dir.path()).unwrap();
        assert_eq!(result, dir.path().join("docs/issues"));
        assert!(dir.path().join("docs/issues/issues-active.md").exists());
    }

    #[test]
    fn test_init_at_creates_at_exact_path() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("my/custom/path");
        init_workspace_at(&target).unwrap();
        assert!(target.join("issues-active.md").exists());
    }

    #[test]
    fn test_init_fails_if_exists() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("docs/issues");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("issues-active.md"), "# Test").unwrap();
        assert!(init_workspace(dir.path()).is_err());
    }

    #[test]
    fn test_reinit_replaces_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("issues-active.md"), "# Old").unwrap();
        reinit_workspace(dir.path()).unwrap();
        let content = fs::read_to_string(dir.path().join("issues-active.md")).unwrap();
        assert!(content.contains("ACTIVE Issues"));
    }

    #[test]
    fn test_discover_prefers_docs_issues() {
        let dir = tempdir().unwrap();
        let docs_issues = dir.path().join("docs/issues");
        fs::create_dir_all(&docs_issues).unwrap();
        fs::write(docs_issues.join("issues-active.md"), "# Test").unwrap();
        assert_eq!(discover_root(dir.path()), docs_issues);
    }

    #[test]
    fn test_discover_still_picks_first_when_multiple_roots_exist() {
        let dir = tempdir().unwrap();
        let docs_issues = dir.path().join("docs/issues");
        let bare_issues = dir.path().to_path_buf();
        fs::create_dir_all(&docs_issues).unwrap();
        fs::write(docs_issues.join("issues-active.md"), "# Docs").unwrap();
        fs::write(bare_issues.join("issues-active.md"), "# Bare").unwrap();

        assert_eq!(discover_root(dir.path()), docs_issues);
    }
}
