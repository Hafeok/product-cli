//! Pattern rendering — text output for `list` / `show` adapters.

use crate::types;

/// `product pattern list` text rendering.
pub fn render_list_text(patterns: &[&types::Pattern]) -> String {
    if patterns.is_empty() {
        return "no patterns".to_string();
    }
    let mut out = String::new();
    let mut sorted: Vec<&types::Pattern> = patterns.to_vec();
    sorted.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    for pat in sorted {
        out.push_str(&format!(
            "{}  {:<11} {}\n",
            pat.front.id, pat.front.status, pat.front.title,
        ));
    }
    out
}

/// `product pattern show` text rendering — front-matter summary + body.
pub fn render_show_text(pattern: &types::Pattern) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{}  {}\n  status:        {}\n",
        pattern.front.id, pattern.front.title, pattern.front.status,
    ));
    if !pattern.front.domains.is_empty() {
        out.push_str(&format!(
            "  domains:       {}\n",
            pattern.front.domains.join(", ")
        ));
    }
    if !pattern.front.adrs.is_empty() {
        out.push_str(&format!(
            "  adrs:          {}\n",
            pattern.front.adrs.join(", ")
        ));
    }
    if !pattern.front.requires.is_empty() {
        out.push_str(&format!(
            "  requires:      {}\n",
            pattern.front.requires.join(", ")
        ));
    }
    if !pattern.front.examples.is_empty() {
        out.push_str(&format!(
            "  examples:      {}\n",
            pattern.front.examples.join(", ")
        ));
    }
    if let Some(ref dep) = pattern.front.deprecated_by {
        out.push_str(&format!("  deprecated-by: {}\n", dep));
    }
    out.push('\n');
    out.push_str(&pattern.body);
    out
}
