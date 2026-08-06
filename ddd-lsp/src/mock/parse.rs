//! Naive fixture-syntax symbol extraction for the mock host.
//!
//! Parses the *fixture dialects* only: one declaration per line, C# types
//! with braced bodies, Bicep declarations headed by their keyword. Real
//! hosts do this with compilers; the mock only needs symbol names, kinds,
//! ranges — the adapters slice the source text themselves.

use serde_json::{json, Value};

/// DocumentSymbol tree for a uri, dispatched on the file extension.
pub fn document_symbols(uri: &str, text: &str) -> Value {
    if uri.ends_with(".bicep") || uri.ends_with(".bicepparam") {
        bicep_symbols(text)
    } else {
        csharp_symbols(text)
    }
}

fn symbol(name: &str, kind: u64, start: usize, end: usize, sel: (usize, usize)) -> Value {
    json!({
        "name": name, "kind": kind,
        "range": {"start": {"line": start, "character": 0},
                   "end": {"line": end, "character": 200}},
        "selectionRange": {"start": {"line": sel.0, "character": sel.1},
                            "end": {"line": sel.0, "character": sel.1 + name.len()}},
        "children": [],
    })
}

fn brace_delta(line: &str) -> i32 {
    line.chars().map(|c| match c {
        '{' => 1,
        '}' => -1,
        _ => 0,
    }).sum()
}

const TYPE_KEYWORDS: &[(&str, u64)] =
    &[("class", 5), ("interface", 11), ("struct", 23), ("enum", 10), ("record", 5)];

struct TypeFrame {
    node: Value,
    name: String,
    entry_depth: i32,
    opened: bool,
    children: Vec<Value>,
}

/// Fixture-grade C#: types at any depth, one-line member declarations
/// directly inside a type body.
fn csharp_symbols(text: &str) -> Value {
    let mut depth = 0i32;
    let mut stack: Vec<TypeFrame> = Vec::new();
    let mut top: Vec<Value> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//") && !trimmed.is_empty() && !trimmed.starts_with('[') {
            if let Some((kind, name)) = type_decl(trimmed) {
                let sel = (i, line.find(&name).unwrap_or(0));
                stack.push(TypeFrame {
                    node: symbol(&name, kind, i, i, sel),
                    name,
                    entry_depth: depth,
                    opened: false,
                    children: Vec::new(),
                });
            } else if let Some(frame) = stack.last_mut() {
                if depth == frame.entry_depth + 1 {
                    if let Some(child) = member_decl(trimmed, &frame.name, i, line) {
                        frame.children.push(child);
                    }
                }
            }
        }
        depth += brace_delta(line);
        for frame in stack.iter_mut() {
            if depth > frame.entry_depth {
                frame.opened = true;
            }
        }
        while stack.last().map(|f| f.opened && depth <= f.entry_depth).unwrap_or(false) {
            let Some(mut frame) = stack.pop() else { break };
            frame.node["range"]["end"]["line"] = json!(i);
            frame.node["children"] = Value::Array(std::mem::take(&mut frame.children));
            match stack.last_mut() {
                Some(parent) => parent.children.push(frame.node),
                None => top.push(frame.node),
            }
        }
    }
    while let Some(mut frame) = stack.pop() {
        frame.node["children"] = Value::Array(std::mem::take(&mut frame.children));
        top.push(frame.node);
    }
    Value::Array(top)
}

/// `[modifiers] class|interface|struct|enum|record Name ...`
fn type_decl(trimmed: &str) -> Option<(u64, String)> {
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    for (i, t) in tokens.iter().enumerate() {
        if let Some((_, kind)) = TYPE_KEYWORDS.iter().find(|(k, _)| k == t) {
            let name = tokens.get(i + 1)?;
            let name: String =
                name.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if name.is_empty() {
                return None;
            }
            return Some((*kind, name));
        }
    }
    None
}

const CONTROL: &[&str] = &["if", "for", "foreach", "while", "switch", "return", "using", "throw"];

/// One-line member declarations: methods/ctors (have parens), properties
/// (braced accessors), fields (semicolon-terminated).
fn member_decl(trimmed: &str, type_name: &str, i: usize, line: &str) -> Option<Value> {
    let first = trimmed.split_whitespace().next().unwrap_or("");
    if CONTROL.contains(&first) || trimmed.starts_with('}') {
        return None;
    }
    if let Some(paren) = trimmed.find('(') {
        let head = &trimmed[..paren];
        let name = head.split_whitespace().next_back()?.to_string();
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.is_empty() {
            return None;
        }
        let kind = if name == type_name { 9 } else { 6 };
        return Some(symbol(&name, kind, i, i, (i, line.find(&name).unwrap_or(0))));
    }
    if trimmed.contains('{') {
        let head = trimmed.split('{').next().unwrap_or("");
        let name = head.split_whitespace().next_back()?.to_string();
        if name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.is_empty() {
            return Some(symbol(&name, 7, i, i, (i, line.find(&name).unwrap_or(0))));
        }
        return None;
    }
    if trimmed.ends_with(';') {
        let head = trimmed.trim_end_matches(';').split('=').next().unwrap_or("");
        let name = head.split_whitespace().next_back()?.to_string();
        if name.chars().all(|c| c.is_alphanumeric() || c == '_')
            && head.split_whitespace().count() >= 2
        {
            return Some(symbol(&name, 8, i, i, (i, line.find(&name).unwrap_or(0))));
        }
    }
    None
}

const BICEP_KEYWORDS: &[(&str, u64)] =
    &[("param", 13), ("output", 13), ("var", 13), ("resource", 23), ("module", 2), ("type", 26)];

/// Fixture-grade Bicep: keyword-headed declarations; braced blocks span
/// to their closing brace.
fn bicep_symbols(text: &str) -> Value {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let mut tokens = trimmed.split_whitespace();
        let keyword = tokens.next().unwrap_or("");
        if let Some((_, kind)) = BICEP_KEYWORDS.iter().find(|(k, _)| *k == keyword) {
            if let Some(name) = tokens.next() {
                let name: String =
                    name.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                let end = block_end(&lines, i);
                let sel = (i, lines[i].find(&name).unwrap_or(0));
                out.push(symbol(&name, *kind, i, end, sel));
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    Value::Array(out)
}

/// The line where a declaration's braces close (its own line if unbraced).
fn block_end(lines: &[&str], start: usize) -> usize {
    let mut depth = 0i32;
    let mut any = false;
    for (offset, line) in lines.iter().skip(start).enumerate() {
        depth += brace_delta(line);
        if brace_delta(line) != 0 {
            any = true;
        }
        if any && depth <= 0 {
            return start + offset;
        }
        if offset == 0 && !line.contains('{') {
            return start;
        }
    }
    lines.len().saturating_sub(1)
}
