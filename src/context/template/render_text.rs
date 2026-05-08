//! Markdown / plain-text renderer. `framed = true` adds a "Context Bundle:"
//! prefix on the title; `false` matches the legacy `human` template output.

use super::collect::Collected;
use super::loader::Template;
use super::render::ProductInfo;
use super::sections::{build_section, ordered_sections, section_title};

pub fn render(
    c: &Collected,
    tpl: &Template,
    product_info: Option<&ProductInfo<'_>>,
    framed: bool,
) -> String {
    let mut out = String::new();
    let prefix = if framed { "Context Bundle: " } else { "" };
    out.push_str(&format!(
        "# {}{} — {}\n\n",
        prefix, c.feature.front.id, c.feature.front.title,
    ));
    if let Some(pi) = product_info {
        out.push_str(&format!("**Product:** {} — {}\n\n", pi.name, pi.responsibility));
    }
    for name in ordered_sections(tpl) {
        if let Some(body) = build_section(&name, c) {
            out.push_str(&format!("## {}\n\n", section_title(&name)));
            out.push_str(body.trim());
            out.push_str("\n\n");
        }
    }
    out
}
