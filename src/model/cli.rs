use super::{issue_id_sort_key, Workspace};

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
