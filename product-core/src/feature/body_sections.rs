//! Feature body section parser — ATX heading detection for W030 (FT-055, ADR-047).
//!
//! Pure module: given a feature body string, returns the H2 / H3 structure
//! and tracks whether each section has any non-whitespace content. Skips
//! fenced code blocks so heading-shaped lines inside markdown samples do
//! not count as real sections.

use std::collections::{HashMap, HashSet};

/// Parsed structure of a feature body's ATX headings.
#[derive(Debug, Clone, Default)]
pub struct BodySections {
    /// H2 headings in document order, deduplicated to first occurrence.
    pub h2: Vec<String>,
    /// H3 headings under each H2 parent, in order, deduplicated per parent.
    pub h3_under: HashMap<String, Vec<String>>,
    /// H2 headings with at least one non-whitespace content line before
    /// the next H2.
    pub h2_nonempty: HashSet<String>,
    /// H3 headings (per parent) with at least one non-whitespace content
    /// line before the next H2/H3.
    pub h3_nonempty: HashMap<String, HashSet<String>>,
}

impl BodySections {
    /// Whether the H2 named `name` is present and has non-whitespace content.
    pub fn h2_has_content(&self, name: &str) -> bool {
        self.h2_nonempty.contains(name)
    }

    /// Whether the H3 `child` under H2 `parent` is present and has
    /// non-whitespace content.
    pub fn h3_has_content(&self, parent: &str, child: &str) -> bool {
        self.h3_nonempty
            .get(parent)
            .map(|set| set.contains(child))
            .unwrap_or(false)
    }
}

/// Mutable scan state — kept in a struct so per-line handlers can mutate
/// it without the parent function juggling 7+ local variables.
struct Scan {
    out: BodySections,
    h2_seen: HashSet<String>,
    h3_seen: HashMap<String, HashSet<String>>,
    current_h2: Option<String>,
    current_h3: Option<String>,
    h2_content_seen: bool,
    h3_content_seen: bool,
    in_fence: bool,
}

impl Scan {
    fn new() -> Self {
        Self {
            out: BodySections::default(),
            h2_seen: HashSet::new(),
            h3_seen: HashMap::new(),
            current_h2: None,
            current_h3: None,
            h2_content_seen: false,
            h3_content_seen: false,
            in_fence: false,
        }
    }

    fn mark_content(&mut self) {
        if self.current_h3.is_some() {
            self.h3_content_seen = true;
            if self.current_h2.is_some() {
                self.h2_content_seen = true;
            }
        } else if self.current_h2.is_some() {
            self.h2_content_seen = true;
        }
    }

    fn close_h3(&mut self) {
        if let (Some(h3), Some(h2)) = (self.current_h3.as_ref(), self.current_h2.as_ref()) {
            if self.h3_content_seen {
                self.out.h3_nonempty.entry(h2.clone()).or_default().insert(h3.clone());
            }
        }
    }

    fn close_h2(&mut self) {
        if let Some(h2) = self.current_h2.as_ref() {
            if self.h2_content_seen {
                self.out.h2_nonempty.insert(h2.clone());
            }
        }
    }

    fn open_h2(&mut self, title: String) {
        self.close_h3();
        self.close_h2();
        if self.h2_seen.insert(title.clone()) {
            self.out.h2.push(title.clone());
        }
        self.current_h2 = Some(title);
        self.current_h3 = None;
        self.h2_content_seen = false;
        self.h3_content_seen = false;
    }

    fn open_h3(&mut self, title: String) {
        self.close_h3();
        let parent = self.current_h2.clone().unwrap_or_default();
        let entry = self.h3_seen.entry(parent.clone()).or_default();
        if entry.insert(title.clone()) {
            self.out.h3_under.entry(parent).or_default().push(title.clone());
        }
        self.current_h3 = Some(title);
        self.h3_content_seen = false;
    }
}

/// Parse a feature body into its H2 / H3 structure.
///
/// - ATX-style headings only (`## ...`, `### ...`).
/// - Headings inside fenced code blocks (```` ``` ```` delimited) are
///   ignored.
/// - Duplicate headings under the same parent are de-duplicated.
/// - A section is "non-empty" when at least one non-whitespace line
///   appears between its heading and the next same-or-higher-level
///   heading.
pub fn parse_body_sections(body: &str) -> BodySections {
    let mut s = Scan::new();
    for raw in body.lines() {
        scan_line(&mut s, raw);
    }
    s.close_h3();
    s.close_h2();
    s.out
}

fn scan_line(s: &mut Scan, raw: &str) {
    let trimmed_start = raw.trim_start();
    if trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~") {
        s.in_fence = !s.in_fence;
        s.mark_content();
        return;
    }
    if s.in_fence {
        if !raw.trim().is_empty() {
            s.mark_content();
        }
        return;
    }
    if let Some(title) = atx_heading(raw, 2) {
        s.open_h2(title);
    } else if let Some(title) = atx_heading(raw, 3) {
        s.open_h3(title);
    } else if !raw.trim().is_empty() {
        s.mark_content();
    }
}

/// Extract the title of an ATX heading at exactly `level`. Returns `None`
/// for any non-heading line, or for headings at a different level.
///
/// The match requires `#` repeated `level` times followed by at least one
/// space, then any title text. The title is trimmed at both ends.
/// Trailing punctuation is left intact, so `## Foo:` returns `Some("Foo:")`
/// — callers compare exact strings.
fn atx_heading(line: &str, level: usize) -> Option<String> {
    let bytes = line.as_bytes();
    // Allow up to 3 leading spaces per CommonMark for ATX headings.
    let mut i = 0;
    while i < bytes.len() && i < 3 && bytes[i] == b' ' {
        i += 1;
    }
    // Match the # run.
    let start = i;
    while i < bytes.len() && bytes[i] == b'#' {
        i += 1;
    }
    let hashes = i - start;
    if hashes != level {
        return None;
    }
    // Must be followed by at least one space (or be end-of-line for an
    // empty heading; we don't recognise empty headings — they have no
    // title text to match against required-section names).
    if i >= bytes.len() || bytes[i] != b' ' {
        return None;
    }
    // Skip spaces between hashes and title.
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    let title = line[i..].trim_end();
    if title.is_empty() {
        return None;
    }
    // CommonMark also allows a trailing run of `#` for closing — strip it.
    let trimmed = title.trim_end_matches(' ').trim_end_matches('#').trim_end();
    if trimmed.is_empty() {
        // Ignore closing-only headings like `###`.
        return None;
    }
    Some(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_h2_functional_specification() {
        let body = "## Description\n\nProse.\n\n## Functional Specification\n\n### Inputs\n\n- foo\n";
        let s = parse_body_sections(body);
        assert!(s.h2.iter().any(|h| h == "Functional Specification"));
        assert!(s.h2.iter().any(|h| h == "Description"));
    }

    #[test]
    fn case_sensitive_match() {
        let body = "## functional specification\n\nx\n";
        let s = parse_body_sections(body);
        assert!(!s.h2.iter().any(|h| h == "Functional Specification"));
    }

    #[test]
    fn trailing_colon_does_not_match() {
        let body = "## Functional Specification:\n\nx\n";
        let s = parse_body_sections(body);
        assert!(!s.h2.iter().any(|h| h == "Functional Specification"));
        assert!(s.h2.iter().any(|h| h == "Functional Specification:"));
    }

    #[test]
    fn fenced_block_suppresses_headings() {
        let body = "## Description\n\n```markdown\n## Functional Specification\n```\n\nProse.\n";
        let s = parse_body_sections(body);
        assert_eq!(s.h2, vec!["Description".to_string()]);
    }

    #[test]
    fn detects_all_subsections_under_fs() {
        let body = "\
## Functional Specification

### Inputs

x

### Outputs

x

### State

x

### Behaviour

x

### Invariants

x

### Error handling

x

### Boundaries

x
";
        let s = parse_body_sections(body);
        let h3 = s.h3_under.get("Functional Specification").expect("h3 set");
        assert_eq!(
            h3,
            &vec![
                "Inputs".to_string(),
                "Outputs".to_string(),
                "State".to_string(),
                "Behaviour".to_string(),
                "Invariants".to_string(),
                "Error handling".to_string(),
                "Boundaries".to_string(),
            ]
        );
    }

    #[test]
    fn duplicate_subsection_dedup() {
        let body = "## Functional Specification\n\n### Boundaries\n\nx\n\n### Boundaries\n\ny\n";
        let s = parse_body_sections(body);
        let h3 = s.h3_under.get("Functional Specification").expect("h3 set");
        assert_eq!(h3, &vec!["Boundaries".to_string()]);
    }

    #[test]
    fn h3_outside_fs_not_attributed() {
        let body = "## Description\n\n### Inputs\n\nx\n\n## Functional Specification\n\n### Outputs\n\nx\n";
        let s = parse_body_sections(body);
        let h3 = s.h3_under.get("Functional Specification").expect("h3 set");
        assert_eq!(h3, &vec!["Outputs".to_string()]);
    }

    #[test]
    fn empty_meaning_section_has_content() {
        let body = "## Functional Specification\n\n### State\n\nStateless. No data is retained.\n";
        let s = parse_body_sections(body);
        assert!(s.h3_has_content("Functional Specification", "State"));
    }

    #[test]
    fn whitespace_only_section_has_no_content() {
        let body = "## Functional Specification\n\n### State\n\n   \n\n### Behaviour\n\nx\n";
        let s = parse_body_sections(body);
        assert!(!s.h3_has_content("Functional Specification", "State"));
        assert!(s.h3_has_content("Functional Specification", "Behaviour"));
    }

    #[test]
    fn trailing_whitespace_in_heading_matches() {
        let body = "## Functional Specification   \n\nx\n";
        let s = parse_body_sections(body);
        assert!(s.h2.iter().any(|h| h == "Functional Specification"));
    }
}
