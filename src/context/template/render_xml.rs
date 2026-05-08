//! XML structural renderer for context bundles.

use super::collect::Collected;
use super::loader::Template;
use super::render::ProductInfo;
use super::sections::{build_section, ordered_sections};

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn render(c: &Collected, tpl: &Template, product_info: Option<&ProductInfo<'_>>) -> String {
    let mut out = String::new();
    out.push_str("<context_bundle");
    if tpl.format.xml.include_attributes {
        out.push_str(&format!(
            " feature=\"{}\" phase=\"{}\" status=\"{}\"",
            xml_escape(&c.feature.front.id),
            c.feature.front.phase,
            c.feature.front.status,
        ));
        if let Some(pi) = product_info {
            out.push_str(&format!(" product=\"{}\"", xml_escape(pi.name)));
        }
    }
    out.push_str(">\n");
    if let Some(pi) = product_info {
        out.push_str(&format!(
            "  <product>\n    <name>{}</name>\n    <responsibility>{}</responsibility>\n  </product>\n",
            xml_escape(pi.name),
            xml_escape(pi.responsibility),
        ));
    }
    let omit_empty = tpl.format.xml.empty_section_handling == "omit";
    for name in ordered_sections(tpl) {
        let body = build_section(&name, c);
        match body {
            Some(b) => {
                out.push_str(&format!("  <{}>\n", name));
                for line in b.lines() {
                    out.push_str("    ");
                    out.push_str(&xml_escape(line));
                    out.push('\n');
                }
                out.push_str(&format!("  </{}>\n", name));
            }
            None if !omit_empty => {
                out.push_str(&format!("  <{}/>\n", name));
            }
            None => {}
        }
    }
    out.push_str("</context_bundle>\n");
    out
}
