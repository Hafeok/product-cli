//! The generation gate, run before a single byte reaches disk.
//!
//! Two checks. **A** — every parameter round-trips byte-identically: the
//! renderer recorded the span it wrote each value into, the gate reads those
//! spans back out of the rendered text, and the count of spans is cross-checked
//! against a naive occurrence count over the template. **B** — no placeholder
//! survives anywhere in the tree.
//!
//! A failure is a refusal, not a warning: `plan_generation` returns the error
//! and nothing is written, so a corrupted parameter can never reach a tree.
//! Check A is what would have caught the first instance's mangled base IRI.

use std::collections::BTreeMap;

use crate::error::{ProductError, Result};

use super::substitute::{tokens_are_disjoint, Site, Token};
use super::template::HOST_SENTINEL;

/// One rendered file as the gate sees it.
pub struct Gated<'a> {
    /// Path relative to the instance root.
    pub path: &'a str,
    /// The template's own text for this file.
    pub source: &'a str,
    /// The rendered text.
    pub rendered: &'a str,
    /// Spans the renderer wrote values into.
    pub sites: &'a [Site],
    /// Whether parameters were substituted into this file.
    pub substituted: bool,
}

/// What the gate saw: how many sites each parameter reached.
#[derive(Debug, Default, serde::Serialize)]
pub struct GateReport {
    /// Sites per parameter label.
    pub sites_per_label: BTreeMap<String, usize>,
}

/// Run both checks over a rendered tree.
pub fn gate(files: &[Gated], tokens: &[Token]) -> Result<GateReport> {
    tokens_are_disjoint(tokens).map_err(|e| fault(&e))?;
    let report = check_round_trip(files, tokens)?;
    check_no_survivors(files)?;
    Ok(report)
}

/// Check A — every emitted value is byte-identical to its parameter, every
/// token occurrence in the template produced exactly one emission, and every
/// parameter reached the output at least once.
fn check_round_trip(files: &[Gated], tokens: &[Token]) -> Result<GateReport> {
    let by_label: BTreeMap<&str, &Token> = tokens.iter().map(|t| (t.label, t)).collect();
    let mut seen: BTreeMap<String, usize> = tokens.iter().map(|t| (t.label.to_string(), 0)).collect();

    for f in files {
        for site in f.sites {
            let token = by_label
                .get(site.label.as_str())
                .ok_or_else(|| fault(&format!("{}: value for unknown parameter '{}'", f.path, site.label)))?;
            let end = site.offset + site.len;
            let written = f.rendered.get(site.offset..end).ok_or_else(|| {
                fault(&format!("{}: '{}' was written past the end of the file", f.path, site.label))
            })?;
            if written != token.value {
                return Err(fault(&format!(
                    "{}: '{}' did not round-trip — supplied {:?}, output holds {:?}",
                    f.path, site.label, token.value, written
                )));
            }
            *seen.entry(site.label.clone()).or_default() += 1;
        }
    }
    check_counts(files, tokens, &seen)?;
    Ok(GateReport { sites_per_label: seen })
}

/// Cross-check the renderer's own tally against a naive occurrence count over
/// the template sources — two mechanisms counting the same thing.
fn check_counts(files: &[Gated], tokens: &[Token], seen: &BTreeMap<String, usize>) -> Result<()> {
    for token in tokens {
        let expected: usize = files
            .iter()
            .filter(|f| f.substituted)
            .map(|f| f.source.matches(token.token).count())
            .sum();
        let got = seen.get(token.label).copied().unwrap_or(0);
        if got != expected {
            return Err(fault(&format!(
                "'{}' was written {got} time(s), the template holds {expected} occurrence(s) of {}",
                token.label, token.token
            )));
        }
        if got == 0 {
            return Err(fault(&format!(
                "'{}' reaches no file in the generated tree — a parameter that lands nowhere cannot be verified",
                token.label
            )));
        }
    }
    Ok(())
}

/// Check B — no `{{PLACEHOLDER}}` and no base-IRI sentinel survives in a
/// substituted file; a verbatim file is byte-identical to the template's.
fn check_no_survivors(files: &[Gated]) -> Result<()> {
    for f in files {
        if !f.substituted {
            if f.rendered != f.source {
                return Err(fault(&format!("{}: travels verbatim, but the output differs from the template", f.path)));
            }
            continue;
        }
        let template_text = mask_values(f.rendered, f.sites);
        if let Some(found) = first_placeholder(&template_text) {
            return Err(fault(&format!(
                "{}: placeholder {found} survived — every placeholder needs a typed parameter",
                f.path
            )));
        }
        if template_text.contains(HOST_SENTINEL) {
            return Err(fault(&format!(
                "{}: the base-IRI placeholder host '{HOST_SENTINEL}' survived",
                f.path
            )));
        }
    }
    Ok(())
}

/// Everything in a rendered file *except* the spans parameter values were
/// written into — i.e. the template's own text as it survived rendering.
///
/// Check B reads this rather than the raw output, which keeps the two checks
/// coherent: a parameter's bytes are data, checked for round-trip by Check A,
/// and anything placeholder-shaped **outside** a value is still an unwired
/// placeholder and still aborts. A display name may honestly contain
/// `{{OWNER_ORG}}`; a template file may not.
fn mask_values(rendered: &str, sites: &[Site]) -> String {
    let mut out = String::with_capacity(rendered.len());
    let mut cursor = 0usize;
    for site in sites {
        if site.offset < cursor {
            continue;
        }
        out.push_str(rendered.get(cursor..site.offset).unwrap_or_default());
        out.push('\n');
        cursor = site.offset + site.len;
    }
    out.push_str(rendered.get(cursor..).unwrap_or_default());
    out
}

/// The first `{{IDENT}}` in `text`, if any.
pub fn first_placeholder(text: &str) -> Option<String> {
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find("}}") {
            let ident = &after[..end];
            if !ident.is_empty()
                && ident.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                return Some(format!("{{{{{ident}}}}}"));
            }
        }
        rest = &rest[start + 2..];
    }
    None
}

fn fault(message: &str) -> ProductError {
    ProductError::ConfigError(format!("generation gate: {message}"))
}

#[cfg(test)]
#[path = "verify_tests.rs"]
mod tests;
