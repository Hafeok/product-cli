//! CTX005 — Function bodies are within the statement-line limit.

use std::fs;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, ImplItemFn, ItemFn, TraitItemFn};
use walkdir::WalkDir;

use crate::check_id::CtxId;
use crate::checks::Check;
use crate::diagnostic::Diagnostic;

const ID: CtxId = CtxId::new("CTX005");
const HARD_LIMIT: usize = 40;
const WARN_LIMIT: usize = 30;

pub struct FunctionLengthCheck;

impl Check for FunctionLengthCheck {
    fn id(&self) -> CtxId {
        ID
    }

    fn title(&self) -> &'static str {
        "Function bodies are within the 40-statement hard limit"
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
                let Ok(body) = fs::read_to_string(path) else {
                    continue;
                };
                let Ok(file) = syn::parse_file(&body) else {
                    continue;
                };
                let lines: Vec<&str> = body.lines().collect();
                let mut visitor = FnVisitor {
                    diagnostics: &mut diagnostics,
                    path,
                    lines: &lines,
                    help_url: self.help_url(),
                };
                visitor.visit_file(&file);
            }
        }
        diagnostics
    }
}

struct FnVisitor<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
    path: &'a Path,
    lines: &'a [&'a str],
    help_url: String,
}

impl<'a, 'ast> Visit<'ast> for FnVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.measure(&node.block, node.sig.ident.to_string());
        visit::visit_item_fn(self, node);
    }
    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.measure(&node.block, node.sig.ident.to_string());
        visit::visit_impl_item_fn(self, node);
    }
    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        if let Some(block) = &node.default {
            self.measure(block, node.sig.ident.to_string());
        }
        visit::visit_trait_item_fn(self, node);
    }
}

impl FnVisitor<'_> {
    fn measure(&mut self, block: &Block, name: String) {
        let span = block.span();
        let start = span.start().line;
        let end = span.end().line;
        if start == 0 || end < start {
            return;
        }
        // Lines in the body, exclusive of the outer braces' lines.
        let body_lines = self
            .lines
            .iter()
            .skip(start.saturating_sub(1))
            .take(end.saturating_sub(start).saturating_add(1))
            .copied();
        let stmts = body_lines
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && trimmed != "{" && trimmed != "}"
            })
            .count();
        // CTX005 is shipped at `severity: warn` while the codebase catches up
        // with the more accurate syn-based counting (the bash precursor
        // silently missed functions whose signatures spanned multiple lines).
        // Promote to error once outliers are addressed — see CTX005.md and
        // ADR-0005.
        if stmts > HARD_LIMIT {
            self.diagnostics.push(
                Diagnostic::warning(
                    ID,
                    format!(
                        "function `{name}` has {stmts} statement lines, exceeds {HARD_LIMIT}-line target"
                    ),
                    self.path.to_path_buf(),
                )
                .at(start as u32, 1)
                .with_help("extract a helper, or split into smaller functions along a clean seam".to_string())
                .with_help_url(self.help_url.clone())
                .with_adrs(["ADR-0005"]),
            );
        } else if stmts > WARN_LIMIT {
            self.diagnostics.push(
                Diagnostic::warning(
                    ID,
                    format!(
                        "function `{name}` has {stmts} statement lines, approaching {HARD_LIMIT}-line target"
                    ),
                    self.path.to_path_buf(),
                )
                .at(start as u32, 1)
                .with_help_url(self.help_url.clone())
                .with_adrs(["ADR-0005"]),
            );
        }
    }
}

fn source_roots() -> &'static [&'static str] {
    &["src", "xtask/src"]
}
