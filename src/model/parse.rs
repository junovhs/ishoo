use super::{Issue, Status};

pub fn parse_markdown(text: &str, default_section: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut section = default_section.to_owned();
    let mut current: Option<Issue> = None;
    let mut in_resolution = false;

    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("# ") {
            if !line.starts_with("## ") {
                let heading = heading.trim();
                if !heading.is_empty() {
                    section.clear();
                    section.push_str(heading);
                }
                continue;
            }
        }

        if let Some(parsed) = try_parse_heading(line) {
            if let Some(prev) = current.take() {
                issues.push(prev);
            }
            current = Some(new_issue(parsed.0, parsed.1, &section));
            in_resolution = false;
            continue;
        }

        let Some(cur) = current.as_mut() else {
            continue;
        };

        if line.trim() == "---" {
            continue;
        }

        if try_parse_field(cur, line, &mut in_resolution) {
            continue;
        }

        accumulate_text(cur, line, in_resolution);
    }

    if let Some(cur) = current {
        issues.push(cur);
    }

    for issue in &mut issues {
        issue.description = issue.description.trim().to_owned();
        issue.resolution = issue.resolution.trim().to_owned();
    }

    issues
}

fn new_issue(id: u32, title: String, section: &str) -> Issue {
    Issue {
        id,
        title,
        status: Status::Open,
        files: vec![],
        description: String::new(),
        resolution: String::new(),
        section: section.to_owned(),
        depends_on: vec![],
    }
}

fn try_parse_field(cur: &mut Issue, line: &str, in_resolution: &mut bool) -> bool {
    if let Some(rest) = line.strip_prefix("**Status:**") {
        cur.status = Status::from_str(rest);
        return true;
    }
    if let Some(rest) = line.strip_prefix("**Files:**") {
        parse_files(cur, rest.trim());
        return true;
    }
    if let Some(rest) = line.strip_prefix("**Depends on:**") {
        cur.depends_on = parse_dep_ids(rest);
        return true;
    }
    if let Some(rest) = line.strip_prefix("**Resolution:**") {
        cur.resolution = rest.trim().to_owned();
        *in_resolution = true;
        return true;
    }
    false
}

fn parse_files(cur: &mut Issue, val: &str) {
    if val.eq_ignore_ascii_case("n/a") || val.is_empty() {
        return;
    }
    cur.files = val
        .split(',')
        .map(|f| f.replace('`', "").trim().to_owned())
        .collect();
}

fn parse_dep_ids(rest: &str) -> Vec<u32> {
    rest.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect()
}

fn accumulate_text(cur: &mut Issue, line: &str, in_resolution: bool) {
    let target = if in_resolution {
        &mut cur.resolution
    } else {
        &mut cur.description
    };
    if !target.is_empty() {
        target.push('\n');
    }
    target.push_str(line);
}

fn try_parse_heading(line: &str) -> Option<(u32, String)> {
    let line = line.trim();
    let rest = line.strip_prefix("## [")?;
    let close = rest.find(']')?;
    let id: u32 = rest[..close].parse().ok()?;
    let title = rest[close + 1..].trim().to_owned();
    Some((id, title))
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
        assert_eq!(
            Status::from_str("IN PROGRESS (Python fixed)"),
            Status::InProgress
        );
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

    #[test]
    fn test_section_tracking() {
        let md = "# Active\n\n## [1] First\n**Status:** OPEN\n\n**Resolution:** \n\n# Backlog\n\n## [2] Second\n**Status:** OPEN\n\n**Resolution:** \n";
        let issues = parse_markdown(md, "Default");
        assert_eq!(issues[0].section, "Active");
        assert_eq!(issues[1].section, "Backlog");
    }

    #[test]
    fn test_depends_on_parsing() {
        let md = "# Test\n\n## [3] Third\n**Status:** OPEN\n**Depends on:** [1], [2]\n\n**Resolution:** \n";
        let issues = parse_markdown(md, "Test");
        assert_eq!(issues[0].depends_on, vec![1, 2]);
    }
}
