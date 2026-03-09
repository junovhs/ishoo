use super::{parse_markdown, Issue, LintFinding};
use std::collections::{BTreeMap, BTreeSet};

pub fn lint_markdown(text: &str, file: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let mut current: Option<IssueLintBlock> = None;
    let file_name = file.to_owned();
    let file_kind = CoreFileKind::from_name(file);

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if let Some(heading) = line.trim().strip_prefix("## [") {
            if let Some(previous) = current.take() {
                previous.push_findings(&file_name, &mut findings);
            }

            let Some((id, title)) = parse_issue_heading(heading) else {
                findings.push(LintFinding {
                    file: file_name.clone(), // neti:allow(P02)
                    line: line_number,
                    message: "invalid issue heading: missing closing ']'".to_owned(), // neti:allow(P02)
                });
                continue;
            };

            current = Some(IssueLintBlock {
                id,
                title,
                heading_line: line_number,
                file_kind,
                has_status: false,
                has_resolution: false,
                has_labels: false,
                status: None,
            });
            continue;
        }

        let Some(block) = current.as_mut() else {
            continue;
        };

        if let Some(rest) = line.strip_prefix("**Status:**") {
            block.has_status = true;
            block.status = Some(rest.trim().to_owned());
        } else if line.starts_with("**Labels:**") {
            block.has_labels = true;
        } else if line.starts_with("**Resolution:**") {
            block.has_resolution = true;
        }
    }

    if let Some(previous) = current.take() {
        previous.push_findings(&file_name, &mut findings);
    }

    findings
}

fn parse_issue_heading(heading: &str) -> Option<(String, String)> {
    let (id, title) = heading.split_once(']')?;
    Some((id.trim().to_string(), title.trim().to_string()))
}

pub fn lint_workspace(files: &[(&str, String)]) -> Vec<LintFinding> {
    let mut issues = Vec::<Issue>::new();
    let mut findings = Vec::<LintFinding>::new();

    for (file_name, text) in files {
        findings.extend(lint_markdown(text, file_name));
        issues.extend(parse_markdown(text, file_name));
    }

    findings.extend(duplicate_id_findings(&issues));
    findings.extend(broken_dependency_findings(&issues));
    findings.sort_by(|left, right| {
        (left.file.as_str(), left.line, left.message.as_str()).cmp(&(
            right.file.as_str(),
            right.line,
            right.message.as_str(),
        ))
    });
    findings
}

fn duplicate_id_findings(issues: &[Issue]) -> Vec<LintFinding> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for issue in issues {
        *counts.entry(&issue.id).or_default() += 1;
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(id, _)| LintFinding {
            file: String::from("<workspace>"),
            line: 1,
            message: format!("duplicate issue id: [{id}]"),
        })
        .collect()
}

fn broken_dependency_findings(issues: &[Issue]) -> Vec<LintFinding> {
    let known_ids = issues
        .iter()
        .map(|issue| issue.id.as_str())
        .collect::<BTreeSet<_>>();
    issues
        .iter()
        .flat_map(|issue| {
            issue
                .depends_on
                .iter()
                .filter(|dependency| !known_ids.contains(dependency.as_str()))
                .map(|dependency| LintFinding {
                    file: String::from("<workspace>"),
                    line: 1,
                    message: format!(
                        "broken dependency: [{}] depends on missing [{}]",
                        issue.id, dependency
                    ),
                })
        })
        .collect()
}

struct IssueLintBlock {
    id: String,
    title: String,
    heading_line: usize,
    file_kind: CoreFileKind,
    has_status: bool,
    has_resolution: bool,
    has_labels: bool,
    status: Option<String>,
}

impl IssueLintBlock {
    fn push_findings(self, file: &str, findings: &mut Vec<LintFinding>) {
        if self.id.is_empty() {
            findings.push(LintFinding {
                file: file.to_owned(),
                line: self.heading_line,
                message: "missing required field: issue id".to_owned(),
            });
        }
        if self.title.is_empty() {
            findings.push(LintFinding {
                file: file.to_owned(),
                line: self.heading_line,
                message: "empty title".to_owned(),
            });
        }
        if !self.has_status {
            findings.push(LintFinding {
                file: file.to_owned(),
                line: self.heading_line,
                message: format!("missing required field: status for [{}]", self.id),
            });
        }
        if !self.has_labels {
            findings.push(LintFinding {
                file: file.to_owned(),
                line: self.heading_line,
                message: format!("missing required field: labels for [{}]", self.id),
            });
        }
        if !self.has_resolution {
            findings.push(LintFinding {
                file: file.to_owned(),
                line: self.heading_line,
                message: format!("missing required field: resolution for [{}]", self.id),
            });
        }
        if let Some(status) = self.status.as_deref() {
            if let Some(message) = core_file_coherence_message(self.file_kind, status, &self.id) {
                findings.push(LintFinding {
                    file: file.to_owned(),
                    line: self.heading_line,
                    message,
                });
            }
        }
    }
}

#[derive(Clone, Copy)]
enum CoreFileKind {
    Active,
    Backlog,
    Done,
    Custom,
}

impl CoreFileKind {
    fn from_name(file: &str) -> Self {
        match file {
            "issues-active.md" => Self::Active,
            "issues-backlog.md" => Self::Backlog,
            "issues-done.md" => Self::Done,
            _ => Self::Custom,
        }
    }
}

fn core_file_coherence_message(file_kind: CoreFileKind, status: &str, id: &str) -> Option<String> {
    let normalized = status.trim().to_ascii_uppercase();
    match file_kind {
        CoreFileKind::Done if normalized != "DONE" => Some(format!(
            "core section mismatch: [{}] in issues-done.md must use DONE status",
            id
        )),
        CoreFileKind::Active | CoreFileKind::Backlog if normalized == "DONE" => Some(format!(
            "core section mismatch: [{}] with DONE status must move to issues-done.md",
            id
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_markdown_flags_missing_status_labels_and_resolution() {
        let md = "# Active\n\n## [BUG-01] Missing fields\nBody only.\n";
        let findings = lint_markdown(md, "issues-active.md");
        assert!(findings
            .iter()
            .any(|f| f.message.contains("missing required field: status")));
        assert!(findings
            .iter()
            .any(|f| f.message.contains("missing required field: labels")));
        assert!(findings
            .iter()
            .any(|f| f.message.contains("missing required field: resolution")));
    }

    #[test]
    fn lint_markdown_flags_empty_title() {
        let md = "# Active\n\n## [BUG-01]\n**Status:** OPEN\n**Resolution:** \n";
        let findings = lint_markdown(md, "issues-active.md");
        assert!(findings.iter().any(|f| f.message == "empty title"));
    }

    #[test]
    fn lint_markdown_flags_invalid_heading_without_closing_bracket() {
        let md = "# Active\n\n## [BUG-01 Missing bracket\n**Status:** OPEN\n**Labels:** cli\n**Resolution:** \n";
        let findings = lint_markdown(md, "issues-active.md");
        assert!(findings
            .iter()
            .any(|f| f.message.contains("invalid issue heading")));
    }

    #[test]
    fn lint_workspace_reports_duplicate_ids_and_missing_dependencies() {
        let findings = lint_workspace(&[
            (
                "issues-active.md",
                "# ACTIVE Issues\n\n---\n\n## [BUG-01] First\n**Status:** OPEN\n**Labels:** cli\n**Depends on:** [BUG-02]\n\n**Resolution:** \n\n---\n".to_string(),
            ),
            (
                "issues-backlog.md",
                "# BACKLOG Issues\n\n---\n\n## [BUG-01] Duplicate\n**Status:** OPEN\n**Labels:** cli\n\n**Resolution:** \n\n---\n".to_string(),
            ),
        ]);

        assert!(findings
            .iter()
            .any(|f| f.message.contains("duplicate issue id")));
        assert!(findings
            .iter()
            .any(|f| f.message.contains("broken dependency")));
    }

    #[test]
    fn lint_workspace_enforces_core_section_coherence_only_for_builtin_files() {
        let findings = lint_workspace(&[
            (
                "issues-active.md",
                "# ACTIVE Issues\n\n---\n\n## [BUG-01] Wrong file\n**Status:** DONE\n**Labels:** cli\n\n**Resolution:** Closed.\n\n---\n".to_string(),
            ),
            (
                "issues-done.md",
                "# DONE Issues\n\n---\n\n## [BUG-02] Wrong status\n**Status:** OPEN\n**Labels:** cli\n\n**Resolution:** \n\n---\n".to_string(),
            ),
            (
                "issues-graphics.md",
                "# GRAPHICS Issues\n\n---\n\n## [BUG-03] Custom section\n**Status:** OPEN\n**Labels:** graphics\n\n**Resolution:** \n\n---\n".to_string(),
            ),
        ]);

        assert!(findings
            .iter()
            .any(|f| f.message.contains("must move to issues-done.md")));
        assert!(findings
            .iter()
            .any(|f| f.message.contains("must use DONE status")));
        assert!(!findings.iter().any(|f| {
            f.file == "issues-graphics.md" && f.message.contains("core section mismatch")
        }));
    }
}
