//! Diagnostic format for xtask checks.
//!
//! Mirrors `rustc`'s human and JSON output so editor tooling and LLM agents
//! that already parse cargo diagnostics can consume xtask output unchanged.

use std::path::PathBuf;

use serde::Serialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Format {
    Text,
    Json,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    /// Optional one-line suggestion shown after `= help:`.
    pub help: Option<String>,
    /// Permalink to the convention doc, shown after `= note:`.
    pub help_url: Option<String>,
    /// ADR identifier(s) that established the rule, shown after `= note:`.
    pub adrs: Vec<String>,
}

impl Diagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>, file: PathBuf) -> Self {
        Self::new(Severity::Error, code, message, file)
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>, file: PathBuf) -> Self {
        Self::new(Severity::Warning, code, message, file)
    }

    fn new(
        severity: Severity,
        code: impl Into<String>,
        message: impl Into<String>,
        file: PathBuf,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
            file,
            line: 1,
            column: 1,
            help: None,
            help_url: None,
            adrs: Vec::new(),
        }
    }

    pub fn at(mut self, line: u32, column: u32) -> Self {
        self.line = line;
        self.column = column;
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_help_url(mut self, url: impl Into<String>) -> Self {
        self.help_url = Some(url.into());
        self
    }

    pub fn with_adrs<I, S>(mut self, adrs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.adrs = adrs.into_iter().map(Into::into).collect();
        self
    }
}

pub fn emit(diagnostics: &[Diagnostic], format: Format) {
    match format {
        Format::Text => emit_text(diagnostics),
        Format::Json => emit_json(diagnostics),
    }
}

fn emit_text(diagnostics: &[Diagnostic]) {
    for diag in diagnostics {
        eprintln!(
            "{}[{}]: {}",
            diag.severity.label(),
            diag.code,
            diag.message
        );
        eprintln!(
            "  --> {}:{}:{}",
            diag.file.display(),
            diag.line,
            diag.column
        );
        if let Some(help) = &diag.help {
            eprintln!("   = help: {help}");
        }
        if let Some(url) = &diag.help_url {
            eprintln!("   = note: see {url}");
        }
        if !diag.adrs.is_empty() {
            eprintln!("   = note: established by {}", diag.adrs.join(", "));
        }
        eprintln!();
    }
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len() - errors;
    if errors > 0 || warnings > 0 {
        eprintln!("{errors} error(s), {warnings} warning(s)");
    } else {
        eprintln!("OK: no convention violations");
    }
}

fn emit_json(diagnostics: &[Diagnostic]) {
    match serde_json::to_string_pretty(diagnostics) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("error: failed to serialize diagnostics: {e}"),
    }
}
