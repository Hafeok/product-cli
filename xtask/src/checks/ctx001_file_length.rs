//! CTX001 — Source files must stay within the 400-line hard limit.

use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::check_id::CtxId;
use crate::checks::Check;
use crate::diagnostic::Diagnostic;

const ID: CtxId = CtxId::new("CTX001");
const HARD_LIMIT: usize = 400;
const WARN_LIMIT: usize = 300;

pub struct FileLengthCheck;

impl Check for FileLengthCheck {
    fn id(&self) -> CtxId {
        ID
    }

    fn title(&self) -> &'static str {
        "Source file length is within the 400-line hard limit"
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
                // Validate the file as Rust — a parse error itself surfaces a
                // diagnostic so we don't silently miss broken files.
                if let Err(err) = syn::parse_file(&body) {
                    let span = err.span().start();
                    diagnostics.push(
                        Diagnostic::error(
                            self.id(),
                            format!("file does not parse as Rust: {err}"),
                            path.to_path_buf(),
                        )
                        .at(span.line as u32, (span.column + 1) as u32)
                        .with_help_url(self.help_url()),
                    );
                    continue;
                }
                let lines = body.lines().count();
                if lines > HARD_LIMIT {
                    diagnostics.push(
                        Diagnostic::error(
                            self.id(),
                            format!(
                                "file is {lines} lines, exceeds {HARD_LIMIT}-line hard limit"
                            ),
                            path.to_path_buf(),
                        )
                        .at(1, 1)
                        .with_help(format!(
                            "split this module into focused submodules, each under {HARD_LIMIT} lines"
                        ))
                        .with_help_url(self.help_url())
                        .with_adrs(["ADR-0001"]),
                    );
                } else if lines > WARN_LIMIT {
                    diagnostics.push(
                        Diagnostic::warning(
                            self.id(),
                            format!(
                                "file is {lines} lines, approaching {HARD_LIMIT}-line hard limit"
                            ),
                            path.to_path_buf(),
                        )
                        .at(1, 1)
                        .with_help_url(self.help_url())
                        .with_adrs(["ADR-0001"]),
                    );
                }
            }
        }
        diagnostics
    }
}

fn source_roots() -> &'static [&'static str] {
    &["src", "xtask/src"]
}
