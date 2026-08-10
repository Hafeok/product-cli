//! HTML+CSS facts extraction — pair contract surface from source text.
//!
//! No language server produces these facts (see the M7 experiment §2), so
//! the adapter slices them itself, the way the C# and Rust adapters slice
//! visibility: adapter knowledge by construction. What is lifted is the
//! *contract surface of the pair* only — design tokens (custom properties
//! as the stylesheet's params), the class vocabulary, `@layer` order, and
//! the stylesheet links that wire the pair. Styling bodies, inline
//! `<style>`/`style=` local to one file, and `<script>` contents (opaque
//! bytes, per the M7 scope ruling) are deliberately not lifted.

use crate::adapter::AdapterFlags;
use crate::protocol::RawSymbol;
use crate::surface::SymbolFacts;

/// Derive the pair's contract-surface facts. The `RawSymbol` slice is the
/// LSP-shaped input this adapter has no producer for; it is ignored.
pub fn facts(text: &str, _symbols: &[RawSymbol], _flags: &AdapterFlags) -> Vec<SymbolFacts> {
    if looks_like_html(text) {
        html_facts(text)
    } else {
        css_facts(text)
    }
}

/// The routing the adapter's two extensions share one facts fn across.
fn looks_like_html(text: &str) -> bool {
    let head = text.trim_start();
    head.starts_with("<!") || head.starts_with('<')
}

fn fact(name: &str, kind: &str, visibility: &str, signature: &str, line: usize) -> SymbolFacts {
    SymbolFacts {
        name: name.to_string(),
        container: String::new(),
        kind: kind.to_string(),
        visibility: visibility.to_string(),
        signature: signature.to_string(),
        decorators: Vec::new(),
        exported: false,
        extra: Default::default(),
        sel_line: line as u64,
        sel_char: 0,
    }
}

// ---------------------------------------------------------------- CSS side

/// Tokens, the class vocabulary, and the `@layer` order of one stylesheet.
fn css_facts(text: &str) -> Vec<SymbolFacts> {
    let mut out = Vec::new();
    let stripped = strip_css_comments(text);
    let layers = layer_order(&stripped);
    if !layers.is_empty() {
        out.push(fact("@layer", "layer-order", "unlayered", &layers.join(", "), 0));
    }
    tokens(&stripped, &mut out);
    classes(&stripped, &mut out);
    out
}

pub(crate) fn strip_css_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start..].find("*/") {
            Some(end) => rest = &rest[start + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// The declared `@layer` statement order — the visibility grading of the
/// sheet. Both the statement form (`@layer a, b;`) and block headers
/// (`@layer a {`) contribute, in source order, first mention wins.
fn layer_order(text: &str) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    for (rest, _) in occurrences(text, "@layer") {
        let head: String = rest.chars().take_while(|c| *c != '{' && *c != ';').collect();
        for name in head.split(',') {
            let name = name.trim();
            if !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
                && !order.iter().any(|n| n == name)
            {
                order.push(name.to_string());
            }
        }
    }
    order
}

/// Every occurrence of `needle` followed by whitespace: (text after it,
/// byte offset).
fn occurrences<'a>(text: &'a str, needle: &str) -> Vec<(&'a str, usize)> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(pos) = text[from..].find(needle) {
        let at = from + pos + needle.len();
        if text[at..].starts_with(char::is_whitespace) {
            out.push((&text[at..], from + pos));
        }
        from = at;
    }
    out
}

/// `--name: value;` declarations, one symbol per token. The signature is
/// the value's *type* — a retype (dimension ↔ color ↔ keyword) is a
/// signature change; a value change within the type is no event at all,
/// which is the point of a token.
fn tokens(text: &str, out: &mut Vec<SymbolFacts>) {
    let mut seen: Vec<String> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let mut rest = line;
        while let Some(pos) = rest.find("--") {
            let cand = &rest[pos..];
            let name: String = cand
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            let after = &cand[name.len()..];
            let is_decl = name.len() > 2 && after.trim_start().starts_with(':');
            if is_decl && !seen.contains(&name) {
                let value: String = after
                    .trim_start()
                    .trim_start_matches(':')
                    .chars()
                    .take_while(|c| *c != ';' && *c != '}')
                    .collect();
                let vis = visibility_at(text, line_offset(text, i));
                out.push(fact(&name, "token", &vis, value_type(value.trim()), i));
                seen.push(name.clone());
            }
            rest = &cand[name.len().max(2)..];
        }
    }
}

/// The declared type of a token value, coarse on purpose: the retype rule
/// fires across these, never within one. The typing itself lives with the
/// token detection source so `diff` and the interceptor cannot drift.
pub fn value_type(value: &str) -> &'static str {
    ddd_core::tokenfile::value_type(value)
}

/// The class vocabulary: every class name any selector defines, one symbol
/// per unique name. Membership is the contract; the rule bodies are not.
fn classes(text: &str, out: &mut Vec<SymbolFacts>) {
    let mut seen: Vec<String> = Vec::new();
    for (offset, name) in selector_class_names(text) {
        if seen.contains(&name) {
            continue;
        }
        let line = text[..offset].matches('\n').count();
        let vis = visibility_at(text, offset);
        out.push(fact(&name, "class", &vis, "", line));
        seen.push(name);
    }
}

/// Class names appearing in selector position (outside `{ … }` bodies),
/// with their byte offsets.
pub fn selector_class_names(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut in_decl_block = false;
    for (i, c) in text.char_indices() {
        match c {
            '{' => {
                depth += 1;
                // A body only when its header was a selector, not an @-rule
                // group like `@layer x {` or `@media … {`.
                in_decl_block = depth > 0 && !header_is_at_rule(text, i);
            }
            '}' => {
                depth -= 1;
                in_decl_block = false;
            }
            '.' if !in_decl_block => {
                let name: String = text[i + 1..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                    .collect();
                let numeric = name.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true);
                if !numeric {
                    out.push((i, name));
                }
            }
            _ => {}
        }
    }
    out
}

/// Whether the `{` at `brace` closes an @-rule header (its header text,
/// back to the previous `{`/`}`/`;`, starts with `@`).
fn header_is_at_rule(text: &str, brace: usize) -> bool {
    let head_start = text[..brace].rfind(['{', '}', ';']).map(|p| p + 1).unwrap_or(0);
    text[head_start..brace].trim_start().starts_with('@')
}

/// The `@layer` a byte offset sits inside, as a visibility grade:
/// `layer:<name>` inside a named block, `unlayered` otherwise. Unlayered
/// declarations outrank every layer in the cascade.
fn visibility_at(text: &str, offset: usize) -> String {
    let mut stack: Vec<Option<String>> = Vec::new();
    for (pos, c) in text.char_indices() {
        if pos >= offset {
            break;
        }
        match c {
            '{' => {
                let head_start = text[..pos].rfind(['{', '}', ';']).map(|p| p + 1).unwrap_or(0);
                let head = text[head_start..pos].trim();
                let layer = head.strip_prefix("@layer").map(|rest| rest.trim().to_string());
                stack.push(layer.filter(|l| !l.is_empty()));
            }
            '}' => {
                stack.pop();
            }
            _ => {}
        }
    }
    match stack.iter().rev().find_map(|l| l.as_ref()) {
        Some(name) => format!("layer:{name}"),
        None => "unlayered".to_string(),
    }
}

fn line_offset(text: &str, line: usize) -> usize {
    text.split_inclusive('\n').take(line).map(str::len).sum()
}

// --------------------------------------------------------------- HTML side

/// The HTML side of the pair lifts only the pair wiring itself: which
/// stylesheets the document consumes. Markup structure and class
/// consumption are checked by the pair contract check, not the edit
/// interceptor; `<script>` contents are opaque bytes.
fn html_facts(text: &str) -> Vec<SymbolFacts> {
    let mut out = Vec::new();
    for (i, tag) in tags(text) {
        if !tag.name.eq_ignore_ascii_case("link") {
            continue;
        }
        let rel = tag.attr("rel").unwrap_or_default();
        if !rel.eq_ignore_ascii_case("stylesheet") {
            continue;
        }
        if let Some(href) = tag.attr("href") {
            let line = text[..i].matches('\n').count();
            out.push(fact(&href, "stylesheet-link", "unlayered", "", line));
        }
    }
    out
}

/// One parsed start tag: name plus raw attributes.
pub struct Tag {
    pub name: String,
    attrs: Vec<(String, String)>,
}

impl Tag {
    pub fn attr(&self, name: &str) -> Option<String> {
        self.attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    }
}

/// Every start tag with its byte offset; `<script>`/`<style>` bodies and
/// comments are skipped, not parsed.
pub fn tags(text: &str) -> Vec<(usize, Tag)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        let Some(open) = text[i..].find('<') else { break };
        let at = i + open;
        let rest = &text[at..];
        if rest.starts_with("<!--") {
            i = rest.find("-->").map(|p| at + p + 3).unwrap_or(text.len());
            continue;
        }
        if rest.starts_with("</") || rest.starts_with("<!") || rest.starts_with("<?") {
            i = at + 1;
            continue;
        }
        let name: String = rest[1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if name.is_empty() {
            i = at + 1;
            continue;
        }
        let Some(close) = rest.find('>') else { break };
        let tag = Tag { name: name.clone(), attrs: parse_attrs(&rest[1 + name.len()..close]) };
        out.push((at, tag));
        i = at + close + 1;
        // Opaque bodies: skip to the matching close tag without parsing.
        for opaque in ["script", "style"] {
            if name.eq_ignore_ascii_case(opaque) {
                let end = text[i..]
                    .to_ascii_lowercase()
                    .find(&format!("</{opaque}"))
                    .map(|p| i + p)
                    .unwrap_or(text.len());
                i = end;
            }
        }
    }
    out
}

fn parse_attrs(raw: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = raw.trim();
    while !rest.is_empty() {
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if name.is_empty() {
            rest = &rest[rest.char_indices().nth(1).map(|(p, _)| p).unwrap_or(rest.len())..];
            rest = rest.trim_start();
            continue;
        }
        rest = rest[name.len()..].trim_start();
        let mut value = String::new();
        if let Some(after_eq) = rest.strip_prefix('=') {
            let after_eq = after_eq.trim_start();
            if let Some(q) = after_eq.chars().next().filter(|c| *c == '"' || *c == '\'') {
                let body = &after_eq[1..];
                let end = body.find(q).unwrap_or(body.len());
                value = body[..end].to_string();
                rest = &body[(end + 1).min(body.len())..];
            } else {
                let end = after_eq.find(char::is_whitespace).unwrap_or(after_eq.len());
                value = after_eq[..end].to_string();
                rest = &after_eq[end..];
            }
        }
        out.push((name.to_ascii_lowercase(), value));
        rest = rest.trim_start();
    }
    out
}

#[cfg(test)]
#[path = "htmlcss_facts_tests.rs"]
mod tests;
