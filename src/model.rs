use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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
            Status::Open => "OPEN",
            Status::InProgress => "IN PROGRESS",
            Status::Done => "DONE",
            Status::Descoped => "DESCOPED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        let upper = s.trim().to_uppercase();
        if upper.contains("PROGRESS") {
            Status::InProgress
        } else if upper.contains("DONE") {
            Status::Done
        } else if upper.contains("DESCOP") {
            Status::Descoped
        } else {
            Status::Open
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::InProgress => "in-progress",
            Status::Done => "done",
            Status::Descoped => "descoped",
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

// ── Workspace ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub open: usize,
    pub in_progress: usize,
    pub done: usize,
    pub descoped: usize,
    pub total: usize,
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
        Ok(Workspace { root: root.to_path_buf(), issues })
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

// ── Serializer ─────────────────────────────────────────────────────────

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

// ── Parser ─────────────────────────────────────────────────────────────

fn parse_markdown(text: &str, default_section: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut section = default_section.to_owned();
    let mut current: Option<Issue> = None;
    let mut in_resolution = false;

    for line in text.lines() {
        // Top-level heading = section
        if line.starts_with("# ") && !line.starts_with("## ") {
            let heading = line[2..].trim();
            if !heading.is_empty() {
                section.clear();
                section.push_str(heading);
            }
            continue;
        }

        // Issue heading
        if let Some(parsed) = try_parse_heading(line) {
            if let Some(prev) = current.take() {
                issues.push(prev);
            }
            current = Some(Issue {
                id: parsed.0,
                title: parsed.1,
                status: Status::Open,
                files: vec![],
                description: String::new(),
                resolution: String::new(),
                section: section.clone(), // neti:allow(P01) — bounded by # headings count
                depends_on: vec![],
            });
            in_resolution = false;
            continue;
        }

        let Some(cur) = current.as_mut() else { continue };

        if line.trim() == "---" {
            continue;
        }

        if let Some(rest) = line.strip_prefix("**Status:**") {
            cur.status = Status::from_str(rest);
            continue;
        }
        if let Some(rest) = line.strip_prefix("**Files:**") {
            let val = rest.trim();
            if !val.eq_ignore_ascii_case("n/a") && !val.is_empty() {
                cur.files = val.split(',').map(|f| f.replace('`', "").trim().to_owned()).collect(); // neti:allow(P02)
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("**Depends on:**") {
            cur.depends_on = rest
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            continue;
        }
        if let Some(rest) = line.strip_prefix("**Resolution:**") {
            cur.resolution = rest.trim().to_owned(); // neti:allow(P02)
            in_resolution = true;
            continue;
        }

        // Accumulate body text
        let target = if in_resolution { &mut cur.resolution } else { &mut cur.description };
        if !target.is_empty() {
            target.push('\n');
        }
        target.push_str(line);
    }

    if let Some(cur) = current {
        issues.push(cur);
    }

    // Trim trailing whitespace from accumulated text
    for issue in &mut issues {
        issue.description = issue.description.trim().to_owned();
        issue.resolution = issue.resolution.trim().to_owned();
    }

    issues
}

fn try_parse_heading(line: &str) -> Option<(u32, String)> {
    let line = line.trim();
    let rest = line.strip_prefix("## [")?;
    let close = rest.find(']')?;
    let id: u32 = rest[..close].parse().ok()?;
    let title = rest[close + 1..].trim().to_owned();
    Some((id, title))
}

// ── CLI ────────────────────────────────────────────────────────────────

pub fn cli_list(workspace: &Workspace, filter: Option<&str>) {
    let stats = workspace.stats();
    println!("╭─ Linearis ─── {} issues ({} open, {} active, {} done) ───╮",
        stats.total, stats.open, stats.in_progress, stats.done);

    let mut issues: Vec<&Issue> = workspace.issues.iter().collect();
    if let Some(f) = filter {
        let f = f.to_lowercase();
        issues.retain(|i| {
            i.title.to_lowercase().contains(&f)
                || i.description.to_lowercase().contains(&f)
                || i.files.iter().any(|file| file.to_lowercase().contains(&f))
        });
    }
    issues.sort_by_key(|i| (i.status_ord(), i.id));

    let mut last_status = String::new();
    for issue in &issues {
        let sl = issue.status.label();
        if last_status != sl {
            println!("│\n│  ── {sl} ──");
            last_status = sl.to_owned();
        }
        let fc = if issue.files.is_empty() { String::new() } else { format!(" ({} files)", issue.files.len()) };
        println!("│  [{}] {}{fc}", issue.id, issue.title);
    }
    println!("╰────────────────────────────────────────────────────────╯");
}

pub fn cli_show(workspace: &Workspace, id: u32) {
    let Some(i) = workspace.issues.iter().find(|i| i.id == id) else {
        eprintln!("Issue #{id} not found");
        return;
    };
    println!("┌─────────────────────────────────────────────");
    println!("│ [{}] {}", i.id, i.title);
    println!("│ Status: {}", i.status.label());
    if !i.files.is_empty() { println!("│ Files: {}", i.files.join(", ")); }
    if !i.description.is_empty() {
        println!("├─────────────────────────────────────────────");
        println!("│ {}", i.description.replace('\n', "\n│ "));
    }
    if !i.resolution.is_empty() {
        println!("├── Resolution ───────────────────────────────");
        println!("│ {}", i.resolution.replace('\n', "\n│ "));
    }
    println!("└─────────────────────────────────────────────");
}

pub fn cli_set_status(workspace: &mut Workspace, id: u32, status: &str) -> Result<(), String> {
    let issue = workspace.issues.iter_mut().find(|i| i.id == id)
        .ok_or_else(|| format!("Issue #{id} not found"))?;
    issue.status = Status::from_str(status);
    println!("Set [{}] → {}", id, issue.status.label());
    workspace.save()
}

pub fn cli_heatmap(workspace: &Workspace) {
    let heatmap = workspace.file_heatmap();
    println!("╭─ File Heatmap ────────────────────────────────────────╮");
    let mut entries: Vec<_> = heatmap.iter().collect();
    entries.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    for (file, ids) in entries {
        let bar: String = "█".repeat(ids.len());
        let id_strs: Vec<String> = ids.iter().map(|id| format!("#{id}")).collect();
        println!("│ {file:40} {bar} {} ({})", ids.len(), id_strs.join(", "));
    }
    println!("╰────────────────────────────────────────────────────────╯");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let (id, title) = try_parse_heading("## [47] Dep Extraction Silent Failure").unwrap();
        assert_eq!(id, 47);
        assert_eq!(title, "Dep Extraction Silent Failure");
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(Status::from_str("IN PROGRESS (Python fixed)"), Status::InProgress);
        assert_eq!(Status::from_str("DONE"), Status::Done);
        assert_eq!(Status::from_str("OPEN"), Status::Open);
    }

    #[test]
    fn test_roundtrip() {
        let md = "# Test\n\n---\n\n## [1] First\n**Status:** OPEN\n**Files:** `a.rs`, `b.rs`\n\nDesc here.\n\n**Resolution:** \n\n---\n";
        let issues = parse_markdown(md, "Test");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, 1);
        assert_eq!(issues[0].files, vec!["a.rs", "b.rs"]);
    }
}
