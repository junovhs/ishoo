use super::{Issue, Status};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};
use std::collections::BTreeSet;

pub fn parse_markdown(text: &str, default_section: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut section = default_section.to_owned();
    let mut current: Option<Issue> = None;
    let mut in_resolution = false;
    let mut parser = Parser::new_ext(text, Options::all()).into_offset_iter();

    while let Some((event, range)) = parser.next() {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => {
                let heading_text = skip_and_extract_text(&mut parser);
                let trimmed_heading = heading_text.trim();
                if !trimmed_heading.is_empty() {
                    section.clear();
                    section.push_str(trimmed_heading);
                }
            }
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                let heading_text = skip_and_extract_text(&mut parser);
                process_h2(
                    text,
                    range,
                    &heading_text,
                    &section,
                    &mut current,
                    &mut issues,
                    &mut in_resolution,
                );
            }
            Event::Start(_) => {
                let _ = skip_and_extract_text(&mut parser);
                process_block(text, range, &mut current, &mut in_resolution);
            }
            Event::Rule => {}
            Event::Text(_) | Event::Html(_) | Event::Code(_) | Event::InlineHtml(_) => {
                process_raw_block(text, range, &mut current, in_resolution);
            }
            _ => {}
        }
    }

    if let Some(cur) = current {
        issues.push(cur);
    }

    for issue in &mut issues {
        issue.description = issue.description.trim().to_owned();
        issue.resolution = issue.resolution.trim().to_owned();
        issue.links = extract_links(issue);
    }

    issues
}

fn process_h2(
    text: &str,
    range: std::ops::Range<usize>,
    heading_text: &str,
    section: &str,
    current: &mut Option<Issue>,
    issues: &mut Vec<Issue>,
    in_resolution: &mut bool,
) {
    let fake_line = format!("## {}", heading_text);
    if let Some(parsed) = try_parse_heading(&fake_line) {
        if let Some(prev) = current.take() {
            issues.push(prev);
        }
        *current = Some(new_issue(parsed.0, parsed.1, section));
        *in_resolution = false;
        return;
    }

    if let Some(cur) = current.as_mut() {
        for line in text[range].lines() {
            accumulate_text(cur, line, *in_resolution);
        }
    }
}

fn process_block(
    text: &str,
    range: std::ops::Range<usize>,
    current: &mut Option<Issue>,
    in_resolution: &mut bool,
) {
    if let Some(cur) = current.as_mut() {
        ensure_blank_line(cur, *in_resolution);
        for line in text[range].lines() {
            if try_parse_field(cur, line, in_resolution) {
                continue;
            }
            accumulate_text(cur, line, *in_resolution);
        }
    }
}

fn process_raw_block(
    text: &str,
    range: std::ops::Range<usize>,
    current: &mut Option<Issue>,
    in_resolution: bool,
) {
    if let Some(cur) = current.as_mut() {
        ensure_blank_line(cur, in_resolution);
        for line in text[range].lines() {
            accumulate_text(cur, line, in_resolution);
        }
    }
}

fn ensure_blank_line(cur: &mut Issue, in_resolution: bool) {
    let target = if in_resolution {
        &mut cur.resolution
    } else {
        &mut cur.description
    };
    if !target.is_empty() && !target.ends_with("\n\n") {
        if target.ends_with('\n') {
            target.push('\n');
        } else {
            target.push_str("\n\n");
        }
    }
}

fn skip_and_extract_text<'a>(
    parser: &mut impl Iterator<Item = (Event<'a>, std::ops::Range<usize>)>,
) -> String {
    let mut extracted = String::new();
    let mut nesting = 1;
    for (inner_ev, _) in parser {
        match inner_ev {
            Event::Start(_) => nesting += 1,
            Event::End(_) => nesting -= 1,
            Event::Text(t) | Event::Code(t) => extracted.push_str(&t),
            _ => {}
        }
        if nesting == 0 {
            break;
        }
    }
    extracted
}

fn new_issue(id: String, title: String, section: &str) -> Issue {
    Issue {
        id,
        title,
        status: Status::Open,
        files: vec![],
        labels: vec![],
        links: vec![],
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
    if let Some(rest) = line.strip_prefix("**Labels:**") {
        cur.labels = parse_labels(rest.trim());
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
        .map(|file| file.replace('`', "").trim().to_owned())
        .collect();
}

fn parse_labels(val: &str) -> Vec<String> {
    if val.eq_ignore_ascii_case("n/a") || val.is_empty() {
        return vec![];
    }
    val.split(',')
        .map(|label| label.trim())
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_dep_ids(rest: &str) -> Vec<String> {
    rest.split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| token.trim_matches(|ch| ch == '[' || ch == ']'))
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn accumulate_text(cur: &mut Issue, line: &str, in_resolution: bool) {
    let target = if in_resolution {
        &mut cur.resolution
    } else {
        &mut cur.description
    };
    if !target.is_empty() && !target.ends_with('\n') {
        target.push('\n');
    }
    target.push_str(line);
}

fn try_parse_heading(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let rest = line.strip_prefix("## [")?;
    let close = rest.find(']')?;
    let id = rest[..close].trim().to_owned();
    if id.is_empty() {
        return None;
    }
    let title = rest[close + 1..].trim().to_owned();
    Some((id, title))
}

fn extract_links(issue: &Issue) -> Vec<String> {
    let ordered_mentions = [&issue.title, &issue.description, &issue.resolution]
        .into_iter()
        .flat_map(|text| extract_mentions(text).into_iter())
        .collect::<Vec<_>>();
    let mut unique_mentions = Vec::<&str>::new();
    let mut seen = BTreeSet::<&str>::new();

    for mention in ordered_mentions {
        if mention != issue.id && seen.insert(mention) {
            unique_mentions.push(mention);
        }
    }

    unique_mentions.into_iter().map(str::to_owned).collect()
}

fn extract_mentions(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut mentions = vec![];
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes[idx] != b'#' {
            idx += 1;
            continue;
        }

        let start = idx + 1;
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-') {
            end += 1;
        }

        if end > start && has_issue_id_boundary(bytes, idx) {
            let candidate = &text[start..end];
            if is_issue_reference(candidate) {
                mentions.push(candidate);
            }
        }

        idx = end.max(idx + 1);
    }

    mentions
}

fn has_issue_id_boundary(bytes: &[u8], hash_idx: usize) -> bool {
    bytes
        .get(hash_idx.wrapping_sub(1))
        .is_none_or(|prev| !prev.is_ascii_alphanumeric() && *prev != b'-')
}

fn is_issue_reference(candidate: &str) -> bool {
    candidate.chars().all(|ch| ch.is_ascii_digit())
        || super::parse_categorical_issue_id(candidate).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let (id, title) = try_parse_heading("## [BUG-47] Dep Extraction Silent Failure").unwrap();
        assert_eq!(id, "BUG-47");
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
        let md = "# Test\n\n---\n\n## [BUG-01] First\n**Status:** OPEN\n**Files:** `a.rs`, `b.rs`\n**Labels:** parser, ui polish\n**Depends on:** [FT-02], [3]\n\nDesc here.\n\n**Resolution:** \n\n---\n";
        let issues = parse_markdown(md, "Test");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "BUG-01");
        assert_eq!(issues[0].files, vec!["a.rs", "b.rs"]);
        assert_eq!(issues[0].labels, vec!["parser", "ui polish"]);
        assert_eq!(issues[0].depends_on, vec!["FT-02", "3"]);
    }

    #[test]
    fn parse_markdown_extracts_unique_issue_mentions_from_body_and_resolution() {
        let md = "# Test\n\n---\n\n## [BUG-08] Mentioned links\n**Status:** OPEN\n\nFollow up after #FT-03 and #BUG-12.\nRepeat #FT-03 here.\n\n**Resolution:** Closed by #UI-09\n\n---\n";
        let issues = parse_markdown(md, "Test");
        assert_eq!(issues[0].links, vec!["FT-03", "BUG-12", "UI-09"]);
    }

    #[test]
    fn parse_markdown_ignores_self_mentions_and_embedded_hashes() {
        let md = "# Test\n\n---\n\n## [BUG-08] Mentioned links\n**Status:** OPEN\n\nIgnore self #BUG-08 and wordabc#BUG-09 and slug-#BUG-07.\n\n**Resolution:** \n\n---\n";
        let issues = parse_markdown(md, "Test");
        assert!(issues[0].links.is_empty());
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
        let md = "# Test\n\n## [BUG-03] Third\n**Status:** OPEN\n**Depends on:** [FT-01], [UI-02]\n\n**Resolution:** \n";
        let issues = parse_markdown(md, "Test");
        assert_eq!(issues[0].depends_on, vec!["FT-01", "UI-02"]);
    }

    #[test]
    fn extract_mentions_supports_numeric_ids() {
        assert_eq!(extract_mentions("refs #12 and #34"), vec!["12", "34"]);
    }
}
