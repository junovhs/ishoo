use super::{parse_markdown, Issue, Stats, Status};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Workspace { // neti:allow(CBO, SFOUT)
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
        self.issues
            .iter()
            .flat_map(|i| i.files.iter().map(move |f| (f.clone(), i.id)))
            .for_each(|(f, id)| map.entry(f).or_default().push(id));
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
        if !issue.labels.is_empty() {
            md.push_str(&format!("**Labels:** {}\n", issue.labels.join(", ")));
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
    use tempfile::tempdir;

    #[test]
    fn save_and_load_preserves_labels() {
        let dir = tempdir().unwrap();
        let ws = Workspace {
            root: dir.path().to_path_buf(),
            issues: vec![Issue {
                id: 21,
                title: "Labels".to_string(),
                status: Status::Open,
                files: vec!["src/model/parse.rs".to_string()],
                labels: vec!["parser".to_string(), "ui".to_string()],
                description: "Desc".to_string(),
                resolution: String::new(),
                section: "ACTIVE Issues".to_string(),
                depends_on: vec![],
            }],
        };

        ws.save().unwrap();
        let loaded = Workspace::load(dir.path()).unwrap();
        assert_eq!(loaded.issues[0].labels, vec!["parser", "ui"]);
    }
}
