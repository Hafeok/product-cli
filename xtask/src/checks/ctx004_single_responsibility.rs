//! CTX004 — Source files declare a single responsibility via `//!` doc comment.

use std::fs;
use std::path::Path;

use syn::Attribute;
use walkdir::WalkDir;

use crate::check_id::CtxId;
use crate::checks::Check;
use crate::diagnostic::Diagnostic;

const ID: CtxId = CtxId::new("CTX004");

pub struct SingleResponsibilityCheck;

impl Check for SingleResponsibilityCheck {
    fn id(&self) -> CtxId {
        ID
    }

    fn title(&self) -> &'static str {
        "Source files declare a single responsibility via module doc comment"
    }

    fn run(&self, root: &Path) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for src_dir in source_roots() {
            let abs = root.join(src_dir);
            if !abs.exists() {
                continue;
            }
            for entry in WalkDir::new(&abs).into_iter().flatten() {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }
                if is_excluded(path) {
                    continue;
                }
                let Ok(body) = fs::read_to_string(path) else {
                    continue;
                };
                let Ok(file) = syn::parse_file(&body) else {
                    // CTX001 already reports unparseable files; don't double up.
                    continue;
                };
                if let Some(diag) = check_attrs(self, path, &file.attrs) {
                    diagnostics.push(diag);
                }
            }
        }
        diagnostics
    }
}

fn check_attrs(
    check: &SingleResponsibilityCheck,
    path: &Path,
    attrs: &[Attribute],
) -> Option<Diagnostic> {
    let first_doc = attrs.iter().find_map(extract_inner_doc);
    let Some(doc) = first_doc else {
        return Some(
            Diagnostic::error(
                check.id(),
                "missing module doc comment (first line must be `//! ...`)".to_string(),
                path.to_path_buf(),
            )
            .at(1, 1)
            .with_help("add a `//!` line at the top of the file describing the single responsibility of this module".to_string())
            .with_help_url(check.help_url())
            .with_adrs(["ADR-0005"]),
        );
    };
    let first_line = doc.lines().next().unwrap_or("").trim();
    if contains_word_and(first_line) {
        return Some(
            Diagnostic::error(
                check.id(),
                format!("module doc comment first line contains `and`: `{first_line}`"),
                path.to_path_buf(),
            )
            .at(1, 1)
            .with_help("split this module along the conjunction — each side becomes its own module".to_string())
            .with_help_url(check.help_url())
            .with_adrs(["ADR-0005"]),
        );
    }
    None
}

/// Return the doc string from an inner doc attribute (`#![doc = "..."]` /
/// `//!`), or `None` if the attribute is something else.
fn extract_inner_doc(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("doc") {
        return None;
    }
    if matches!(attr.style, syn::AttrStyle::Outer) {
        return None;
    }
    let meta = match &attr.meta {
        syn::Meta::NameValue(nv) => nv,
        _ => return None,
    };
    let lit = match &meta.value {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s,
        _ => return None,
    };
    Some(lit.value().trim().to_string())
}

fn contains_word_and(line: &str) -> bool {
    // Match the bash script's behavior: `" and "` (surrounded by spaces).
    line.contains(" and ")
}

fn is_excluded(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|s| s.to_str()),
        Some("mod.rs") | Some("main.rs") | Some("lib.rs")
    )
}

fn source_roots() -> &'static [&'static str] {
    &["src", "xtask/src"]
}
