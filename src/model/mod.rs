// src/model/mod.rs
mod cli;
mod parse;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use cli::{cli_heatmap, cli_list, cli_set_status, cli_show};
pub use parse::parse_markdown;

const ISSUE_FILES: [&str; 3] = ["issues-active.md", "issues-backlog.md", "issues-done.md"];

/// Initialize a new ishoo workspace with empty issue files
pub fn init_workspace(path: &Path) -> Result<(), String> {
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

/// Check if a workspace exists at the given path
pub fn workspace_exists(path: &Path) -> bool {
    ISSUE_FILES.iter().any(|f| path.join(f).exists())
}

/// Searches common subdirectories for issue markdown files.
/// Returns the first directory containing at least one match,
/// or falls back to the given base path.
pub fn discover_root(base: &Path) -> PathBuf {
    let candidates = [
        base.to_path_buf(),
        base.join("docs/issues"),
        base.join("docs"),
        base.join("issues"),
        base.join("ishoo"),
        base.join(".issues"),
    ];

    for candidate in &candidates {
        if ISSUE_FILES.iter().any(|f| candidate.join(f).exists()) {
            return candidate.clone();
        }
    }

    base.to_path_buf()
}

// ── Data Model ─────────────────────────────────────────────────────────

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
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub files: Vec<String>,
    pub description: String,
    pub resolution: String,
    pub section: String,
    pub depends_on: Vec<u32>,
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

// ── Workspace ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub issues: Vec<Issue>,
}

impl Workspace {
    pub fn load(root: &Path) -> Result<Self, String> {
        let mut issues = Vec::new();
        let files = [
            ("issues-active.md", "Active"),
            ("issues-backlog.md", "Backlog"),
            ("issues-done.md", "Done"),
        ];
        for (filename, default_sec) in &files {
            let path = root.join(filename);
            if path.exists() {
                let text = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {filename}: {e}"))?;
                issues.extend(parse_markdown(&text, default_sec));
            }
        }
        Ok(Workspace {
            root: root.to_path_buf(),
            issues,
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let (mut active, mut backlog, mut done): (Vec<&Issue>, Vec<&Issue>, Vec<&Issue>) =
            (Vec::new(), Vec::new(), Vec::new());

        for issue in &self.issues {
            let sec = issue.section.to_lowercase();
            if sec.contains("done") || issue.status == Status::Done {
                done.push(issue);
            } else if sec.contains("backlog") {
                backlog.push(issue);
            } else {
                active.push(issue);
            }
        }

        write_section(&self.root, "issues-active.md", "ACTIVE Issues", &active)?;
        write_section(&self.root, "issues-backlog.md", "BACKLOG Issues", &backlog)?;
        write_section(&self.root, "issues-done.md", "DONE Issues", &done)?;
        Ok(())
    }

    pub fn stats(&self) -> Stats {
        let mut s = Stats::default();
        for issue in &self.issues {
            match issue.status {
                Status::Open => s.open += 1,
                Status::InProgress => s.in_progress += 1,
                Status::Done => s.done += 1,
                Status::Descoped => s.descoped += 1,
            }
        }
        s.total = self.issues.len();
        s
    }

    pub fn file_heatmap(&self) -> BTreeMap<String, Vec<u32>> {
        let mut map: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for issue in &self.issues {
            for file in &issue.files {
                map.entry(file.clone()).or_default().push(issue.id);
            }
        }
        map
    }

    pub fn dependency_edges(&self) -> Vec<(u32, u32)> {
        self.issues
            .iter()
            .flat_map(|i| i.depends_on.iter().map(move |&dep| (dep, i.id)))
            .collect()
    }
}

fn write_section(root: &Path, name: &str, title: &str, issues: &[&Issue]) -> Result<(), String> {
    let mut md = format!("# {title}\n\n---\n");
    for issue in issues {
        md.push_str(&format!("\n## [{}] {}\n", issue.id, issue.title));
        md.push_str(&format!("**Status:** {}\n", issue.status.label()));
        if !issue.files.is_empty() {
            let f: Vec<String> = issue.files.iter().map(|f| format!("`{f}`")).collect();
            md.push_str(&format!("**Files:** {}\n", f.join(", ")));
        }
        if !issue.depends_on.is_empty() {
            let d: Vec<String> = issue.depends_on.iter().map(|d| format!("[{d}]")).collect();
            md.push_str(&format!("**Depends on:** {}\n", d.join(", ")));
        }
        if !issue.description.is_empty() {
            md.push('\n');
            md.push_str(&issue.description);
            md.push('\n');
        }
        md.push_str(&format!("\n**Resolution:** {}\n\n---\n", issue.resolution));
    }
    fs::write(root.join(name), md).map_err(|e| format!("Failed to write {name}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_init_workspace_creates_files() {
        let dir = tempdir().unwrap();
        init_workspace(dir.path()).unwrap();

        assert!(dir.path().join("issues-active.md").exists());
        assert!(dir.path().join("issues-backlog.md").exists());
        assert!(dir.path().join("issues-done.md").exists());
    }

    #[test]
    fn test_init_workspace_fails_if_exists() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("issues-active.md"), "# Test").unwrap();

        let result = init_workspace(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_workspace_exists() {
        let dir = tempdir().unwrap();
        assert!(!workspace_exists(dir.path()));

        fs::write(dir.path().join("issues-active.md"), "# Test").unwrap();
        assert!(workspace_exists(dir.path()));
    }
}
