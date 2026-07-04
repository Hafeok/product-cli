//! C# identifier + primitive-type mapping for the reify emitters.
//!
//! Graph ids are kebab/camel/free-form strings; C# wants PascalCase
//! identifiers. `pascal` is total: any non-alphanumeric byte is a word
//! break, a leading digit gets an `N` prefix, an empty id becomes `X`.
//! PascalCase output cannot collide with C# keywords (all lowercase), so
//! no verbatim-identifier escaping is needed.

/// A C# scalar type inferred for a payload/state field. Mirrors the pf
/// `Scalar` alphabet (§3.3): bool / int / string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsTy {
    Bool,
    Long,
    Str,
}

impl CsTy {
    /// The C# type name.
    pub fn name(self) -> &'static str {
        match self {
            CsTy::Bool => "bool",
            CsTy::Long => "long",
            CsTy::Str => "string",
        }
    }

    /// Merge two observations of the same field: agreement keeps the type,
    /// disagreement falls back to `string` (the widest wire form).
    pub fn merge(a: Option<CsTy>, b: CsTy) -> CsTy {
        match a {
            None => b,
            Some(t) if t == b => t,
            Some(_) => CsTy::Str,
        }
    }
}

/// Convert a graph id into a PascalCase C# identifier.
pub fn pascal(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    let mut upper_next = true;
    for c in id.chars() {
        if c.is_ascii_alphanumeric() {
            if upper_next {
                out.extend(c.to_uppercase());
            } else {
                out.push(c);
            }
            upper_next = c.is_ascii_digit();
        } else {
            upper_next = true;
        }
    }
    if out.is_empty() {
        return "X".to_string();
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        return format!("N{out}");
    }
    out
}

/// Map a §3.1 attribute/datatype string to a C# type. The vocabulary is the
/// `TypeConstraint` set (`string · integer · number · boolean · date`) plus
/// the common aliases; anything unknown stays `string`.
pub fn attr_ty(ty: Option<&str>) -> &'static str {
    match ty.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("int") | Some("integer") => "long",
        Some("bool") | Some("boolean") => "bool",
        Some("number") | Some("decimal") | Some("float") => "decimal",
        Some("date") | Some("datetime") => "DateOnly",
        _ => "string",
    }
}

/// Escape a string into a C# double-quoted literal (without the quotes).
pub fn cs_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Sanitize a scenario name into a C# method identifier: PascalCase words
/// joined by underscores, so `first order accepted` → `First_order_accepted`.
pub fn method_name(name: &str) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect();
    if words.is_empty() {
        return "Scenario".to_string();
    }
    let joined = words.join("_");
    let mut out: String = joined
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect();
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'N');
    }
    out
}
