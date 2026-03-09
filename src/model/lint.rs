use super::{parse_markdown, Issue, LintFinding};
use std::collections::{BTreeMap, BTreeSet};

const INVALID_HEADING_MESSAGE: &str = "invalid issue heading: missing closing ']'";
const MISSING_ID_MESSAGE: &str = "missing required field: issue id";
const EMPTY_TITLE_MESSAGE: &str = "empty title";
const WORKSPACE_FILE: &str = "<workspace>";

pub fn lint_markdown(text: &str, file: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let mut current: Option<IssueLintBlock> = None;
    let file_name = file.to_owned();
    let file_kind = CoreFileKind::from_name(file);
    let invalid_heading_message = INVALID_HEADING_MESSAGE.to_owned();

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if let Some(heading) = line.trim().strip_prefix("## [") {
            if let Some(previous) = current.take() {
                previous.push_findings(&file_name, &mut findings);
            }

            let Some((id, title)) = parse_issue_heading(heading) else {
                findings.push(LintFinding {
                    file: file_name.clone(),
                    line: line_number,
                    message: invalid_heading_message.clone(),
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
            file: WORKSPACE_FILE.to_owned(),
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
                    file: WORKSPACE_FILE.to_owned(),
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
                message: MISSING_ID_MESSAGE.to_owned(),
            });
        }
        if self.title.is_empty() {
            findings.push(LintFinding {
                file: file.to_owned(),
                line: self.heading_line,
                message: EMPTY_TITLE_MESSAGE.to_owned(),
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
    let status = status.trim().to_ascii_uppercase();
    match file_kind {
        CoreFileKind::Done if status != "DONE" && status != "DESCOPED" => Some(format!(
            "done file coherence: [{}] in issues-done.md must be DONE or DESCOPED, found {}",
            id, status
        )),
        CoreFileKind::Active | CoreFileKind::Backlog if status == "DONE" => Some(format!(
            "core file coherence: [{}] is DONE but still lives outside issues-done.md",
            id
        )),
        _ => None,
    }
}
