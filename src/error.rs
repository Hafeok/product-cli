//! Error model (ADR-013): four-tier taxonomy with rustc-style diagnostics

use std::fmt;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, ProductError>;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub tier: DiagnosticTier,
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
    pub context: Option<String>,
    pub detail: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticTier {
    Error,
    Warning,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.tier {
            DiagnosticTier::Error => "error",
            DiagnosticTier::Warning => "warning",
        };
        write!(f, "{}[{}]: {}", prefix, self.code, self.message)?;
        if let Some(ref file) = self.file {
            write!(f, "\n  --> {}", file.display())?;
            if let Some(line) = self.line {
                write!(f, ":{}", line)?;
            }
        }
        if let Some(ref ctx) = self.context {
            write!(f, "\n   |\n   | {}", ctx)?;
        }
        if let Some(ref detail) = self.detail {
            write!(f, "\n   |   {}", detail)?;
        }
        if let Some(ref hint) = self.hint {
            write!(f, "\n   = hint: {}", hint)?;
        }
        Ok(())
    }
}

impl Diagnostic {
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            tier: DiagnosticTier::Error,
            message: message.to_string(),
            file: None,
            line: None,
            context: None,
            detail: None,
            hint: None,
        }
    }

    pub fn warning(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            tier: DiagnosticTier::Warning,
            message: message.to_string(),
            file: None,
            line: None,
            context: None,
            detail: None,
            hint: None,
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_context(mut self, ctx: &str) -> Self {
        self.context = Some(ctx.to_string());
        self
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_string());
        self
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.code,
            "tier": format!("{:?}", self.tier).to_lowercase(),
            "message": self.message,
            "file": self.file.as_ref().map(|p| p.display().to_string()),
            "line": self.line,
            "context": self.context,
            "detail": self.detail,
            "hint": self.hint,
        })
    }
}

#[derive(Debug)]
pub enum ProductError {
    /// E001: Malformed YAML front-matter
    ParseError {
        file: PathBuf,
        line: Option<usize>,
        message: String,
    },
    /// E002: Broken link
    #[allow(dead_code)]
    BrokenLink {
        file: PathBuf,
        line: Option<usize>,
        source_id: String,
        target_id: String,
    },
    /// E003: Dependency cycle
    DependencyCycle {
        cycle: Vec<String>,
    },
    /// E004: Supersession cycle
    #[allow(dead_code)]
    SupersessionCycle {
        cycle: Vec<String>,
    },
    /// E005: Invalid artifact ID format
    InvalidId {
        file: PathBuf,
        id: String,
    },
    /// E006: Missing required front-matter field
    MissingField {
        file: PathBuf,
        field: String,
    },
    /// E008: Schema version mismatch
    SchemaVersionMismatch {
        declared: u32,
        supported: u32,
    },
    /// E009: File write failure
    WriteError {
        path: PathBuf,
        message: String,
    },
    /// E010: Repository locked
    #[allow(dead_code)]
    LockError {
        message: String,
    },
    /// E016: Lifecycle gate — verify blocked by proposed ADR(s)
    LifecycleGate {
        feature_id: String,
        proposed_adrs: Vec<String>,
    },
    /// Configuration error
    ConfigError(String),
    /// Generic IO error
    IoError(String),
    /// Artifact not found
    NotFound(String),
    /// Internal error (Tier 4)
    Internal(String),
}

impl fmt::Display for ProductError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError { file, line, message } => {
                let loc = line.map_or(String::new(), |l| format!(":{}", l));
                write!(f, "error[E001]: malformed front-matter\n  --> {}{}\n   = {}", file.display(), loc, message)
            }
            Self::BrokenLink { file, source_id, target_id, .. } => write!(
                f, "error[E002]: broken link\n  --> {}\n   | {} references {} which does not exist\n   = hint: create the file or remove the reference",
                file.display(), source_id, target_id
            ),
            Self::DependencyCycle { cycle } => write!(f, "error[E003]: dependency cycle detected\n   = cycle: {}", cycle.join(" -> ")),
            Self::SupersessionCycle { cycle } => write!(f, "error[E004]: supersession cycle detected\n   = cycle: {}", cycle.join(" -> ")),
            Self::InvalidId { file, id } => write!(f, "error[E005]: invalid artifact ID '{}'\n  --> {}", id, file.display()),
            Self::MissingField { file, field } => write!(f, "error[E006]: missing required field '{}'\n  --> {}", field, file.display()),
            Self::SchemaVersionMismatch { declared, supported } => write!(
                f, "error[E008]: schema version mismatch\n   | this repository requires schema version {}\n   | this binary supports up to schema version {}\n   = hint: upgrade product with `cargo install product --force`",
                declared, supported
            ),
            Self::WriteError { path, message } => write!(f, "error[E009]: write failed\n  --> {}\n   = {}", path.display(), message),
            Self::LockError { message } => write!(f, "error[E010]: repository locked\n   = {}", message),
            Self::LifecycleGate { feature_id, proposed_adrs } => write!(
                f, "error[E016]: cannot verify {} — governing ADR(s) not yet accepted: {}",
                feature_id, proposed_adrs.join(", ")
            ),
            Self::ConfigError(msg) => write!(f, "error: {}", msg),
            Self::IoError(msg) => write!(f, "error: {}", msg),
            Self::NotFound(msg) => write!(f, "error: not found — {}", msg),
            Self::Internal(msg) => write!(f, "internal error: {}\n  This is a bug in Product. Please report it.", msg),
        }
    }
}

impl std::error::Error for ProductError {}

impl From<std::io::Error> for ProductError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e.to_string())
    }
}

/// Internal error exit code (Tier 4, ADR-013)
#[allow(dead_code)]
pub const INTERNAL_ERROR_EXIT_CODE: i32 = 3;

/// Macro for Tier 4 internal errors — captures file and line, prints diagnostic, exits with code 3.
/// Use this instead of panic!() for unexpected states that represent bugs in Product.
#[macro_export]
macro_rules! internal_error {
    ($($arg:tt)*) => {{
        eprintln!(
            "internal error: {} at {}:{}",
            format!($($arg)*),
            file!(),
            line!()
        );
        eprintln!("  This is a bug in Product. Please report it.");
        eprintln!("  Product version: {}", env!("CARGO_PKG_VERSION"));
        std::process::exit($crate::error::INTERNAL_ERROR_EXIT_CODE);
    }};
}

/// Check result of graph validation: errors, warnings, exit code
#[derive(Default)]
pub struct CheckResult {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

impl CheckResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        if !self.errors.is_empty() {
            1
        } else if !self.warnings.is_empty() {
            2
        } else {
            0
        }
    }

    pub fn print_stderr(&self) {
        for e in &self.errors {
            eprintln!("{}", e);
            eprintln!();
        }
        for w in &self.warnings {
            eprintln!("{}", w);
            eprintln!();
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "errors": self.errors.iter().map(|e| e.to_json()).collect::<Vec<_>>(),
            "warnings": self.warnings.iter().map(|w| w.to_json()).collect::<Vec<_>>(),
            "summary": {
                "errors": self.errors.len(),
                "warnings": self.warnings.len(),
            }
        })
    }
}
