//! Single-pass token renderer that never re-reads its own output.
//!
//! The scanner walks the input once. At each character boundary it matches the
//! longest token from a closed table; on a match it copies the parameter's
//! bytes to the output, records the span it wrote, then advances the cursor
//! past the token. The cursor never moves backwards, so emitted values are
//! never scanned again.
//!
//! The invariant, stated so a test can assert it: *the output is the
//! concatenation of literal spans of the input with byte-for-byte copies of
//! parameter values.* A value carrying `&`, `\1`, `$0` or a literal
//! `{{OWNER_ORG}}` is therefore data, not a further instruction. Sequential
//! `str::replace` does not have this property — a value inserted by an earlier
//! replacement sits in the buffer that later replacements scan.

/// One entry of the substitution table: the literal token as it appears in the
/// template, the value that replaces it, and the label the gate reports it by.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Token<'a> {
    /// The parameter's reporting name (`--base-iri`, `{{OWNER_ORG}}`, …).
    pub label: &'a str,
    /// The literal text matched in the template.
    pub token: &'a str,
    /// The bytes emitted in its place.
    pub value: &'a str,
}

/// Where one parameter value was written into a rendered file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Site {
    /// Which parameter was written.
    pub label: String,
    /// Byte offset of the value in the rendered text.
    pub offset: usize,
    /// Byte length of the value written.
    pub len: usize,
}

/// A rendered file: its text, plus every span the renderer wrote a value into.
#[derive(Debug, Clone)]
pub(crate) struct Rendered {
    /// The rendered text.
    pub text: String,
    /// One entry per emitted value, in output order.
    pub sites: Vec<Site>,
}

/// Render `input`, substituting every token occurrence exactly once.
pub(crate) fn render(input: &str, tokens: &[Token]) -> Rendered {
    let mut order: Vec<&Token> = tokens.iter().collect();
    order.sort_by_key(|t| std::cmp::Reverse(t.token.len()));

    let mut out = String::with_capacity(input.len());
    let mut sites = Vec::new();
    let mut literal_from = 0usize;
    let mut cursor = 0usize;

    while cursor < input.len() {
        if !input.is_char_boundary(cursor) {
            cursor += 1;
            continue;
        }
        let rest = &input[cursor..];
        match order.iter().find(|t| rest.starts_with(t.token)) {
            Some(t) => {
                out.push_str(&input[literal_from..cursor]);
                sites.push(Site {
                    label: t.label.to_string(),
                    offset: out.len(),
                    len: t.value.len(),
                });
                out.push_str(t.value);
                cursor += t.token.len();
                literal_from = cursor;
            }
            None => cursor += 1,
        }
    }
    out.push_str(&input[literal_from..]);
    Rendered { text: out, sites }
}

/// Fail-closed guard: no token may be a substring of another. Overlapping
/// tokens would make the scanner's matches and a naive occurrence count
/// disagree, which is exactly what the gate cross-checks.
pub(crate) fn tokens_are_disjoint(tokens: &[Token]) -> Result<(), String> {
    for a in tokens {
        for b in tokens {
            if !std::ptr::eq(a, b) && a.token.contains(b.token) {
                return Err(format!(
                    "token '{}' contains token '{}' — the substitution table must be disjoint",
                    a.token, b.token
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "substitute_tests.rs"]
mod tests;
