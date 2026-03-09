use super::{issue_id_sort_key, lint_workspace, Workspace};
use std::io::{self, Write};
use std::{fs, path::Path};

pub fn cli_list(workspace: &Workspace, filter: Option<&str>) {
    let stats = workspace.stats();
    println!(
        "╭─ Ishoo ─── {} issues ({} open, {} active, {} done) ───╮",
        stats.total, stats.open, stats.in_progress, stats.done
    );

    let mut issues: Vec<_> = workspace.issues.iter().collect();
    if let Some(f) = filter {
        let f = f.to_lowercase();
        issues.retain(|i| {
            i.title.to_lowercase().contains(&f)
                || i.description.to_lowercase().contains(&f)
                || i.files.iter().any(|file| file.to_lowercase().contains(&f))
        });
    }
    issues.sort_by(|left, right| {
        (left.status_ord(), issue_id_sort_key(&left.id))
            .cmp(&(right.status_ord(), issue_id_sort_key(&right.id)))
    });

    let mut last_status = "";
    for issue in &issues {
        let sl = issue.status.label();
        if last_status != sl {
            println!("│\n│  ── {sl} ──");
            last_status = sl;
        }
        let fc = if issue.files.is_empty() {
            String::new()
        } else {
            format!(" ({} files)", issue.files.len())
        };
        println!("│  [{}] {}{fc}", issue.id, issue.title);
    }
    println!("╰────────────────────────────────────────────────────────╯");
}

pub fn cli_show(workspace: &Workspace, id: &str) {
    let Some(i) = workspace.issues.iter().find(|i| i.id == id) else {
        eprintln!("Issue {id} not found");
        return;
    };
    println!("┌─────────────────────────────────────────────");
    println!("│ [{}] {}", i.id, i.title);
    println!("│ Status: {}", i.status.label());
    if !i.files.is_empty() {
        println!("│ Files: {}", i.files.join(", "));
    }
    if !i.description.is_empty() {
        println!("├───���─────────────────────────────────────────");
        println!("│ {}", i.description.replace('\n', "\n│ "));
    }
    if !i.resolution.is_empty() {
        println!("├── Resolution ───────────────────────────────");
        println!("│ {}", i.resolution.replace('\n', "\n│ "));
    }
    println!("└─────────────────────────────────────────────");
}

pub fn cli_set_status(workspace: &mut Workspace, id: &str, status: &str) -> Result<(), String> {
    let issue = workspace
        .issues
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| format!("Issue {id} not found"))?;
    issue.status = super::Status::from_str(status);
    println!("Set [{}] → {}", id, issue.status.label());
    workspace.save()
}

pub fn cli_delete(workspace: &mut Workspace, id: &str, force: bool) -> Result<(), String> {
    let issue = workspace
        .issues
        .iter()
        .find(|issue| issue.id == id)
        .ok_or_else(|| format!("Issue {id} not found"))?;

    if !force {
        print!("Delete [{}] {} permanently? [y/N] ", issue.id, issue.title);
        io::stdout()
            .flush()
            .map_err(|err| format!("Failed to flush stdout: {err}"))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|err| format!("Failed to read confirmation: {err}"))?;

        let confirmed = matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes");
        if !confirmed {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    let deleted = workspace.delete_issue(id)?;
    println!("Deleted [{}] {}", deleted.id, deleted.title);
    Ok(())
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

pub fn cli_lint(root: &Path, strict: bool) -> Result<(), String> {
    let file_names = ["issues-active.md", "issues-backlog.md", "issues-done.md"];
    let mut files = Vec::new();

    for file_name in file_names {
        let path = root.join(file_name);
        if !path.exists() {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|err| format!("Failed to read {}: {err}", path.display()))?;
        files.push((file_name, text));
    }

    let findings = lint_workspace(&files);

    if findings.is_empty() {
        println!("Lint passed: no issues found.");
        return Ok(());
    }

    let level = if strict { "ERROR" } else { "WARNING" };
    for finding in &findings {
        println!(
            "{level}: {}:{} {}",
            finding.file, finding.line, finding.message
        );
    }
    println!(
        "{} {} found.",
        findings.len(),
        if strict { "errors" } else { "warnings" }
    );

    if strict {
        return Err(format!("lint failed with {} errors", findings.len()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn lint_reports_duplicate_ids_and_missing_dependencies() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root).unwrap();
        fs::write(
            root.join("issues-active.md"),
            "# ACTIVE Issues\n\n---\n\n## [BUG-01] First\n**Status:** OPEN\n**Depends on:** [BUG-02]\n\n**Resolution:** \n\n---\n",
        )
        .unwrap();
        fs::write(
            root.join("issues-backlog.md"),
            "# BACKLOG Issues\n\n---\n\n## [BUG-01] Duplicate\n**Status:** OPEN\n\n**Resolution:** \n\n---\n",
        )
        .unwrap();
        fs::write(root.join("issues-done.md"), "# DONE Issues\n\n---\n").unwrap();

        let err = cli_lint(root, true).unwrap_err();
        assert!(err.contains("lint failed"));
    }

    #[test]
    fn lint_passes_on_valid_workspace() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root).unwrap();
        fs::write(
            root.join("issues-active.md"),
            "# ACTIVE Issues\n\n---\n\n## [BUG-01] Valid\n**Status:** OPEN\n**Depends on:** [BUG-02]\n\n**Resolution:** Investigating.\n\n---\n",
        )
        .unwrap();
        fs::write(
            root.join("issues-backlog.md"),
            "# BACKLOG Issues\n\n---\n\n## [BUG-02] Support\n**Status:** OPEN\n\n**Resolution:** \n\n---\n",
        )
        .unwrap();
        fs::write(root.join("issues-done.md"), "# DONE Issues\n\n---\n").unwrap();

        assert!(cli_lint(root, false).is_ok());
    }
}
