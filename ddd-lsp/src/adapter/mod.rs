//! Language adapters — the only place language knowledge lives.
//!
//! An adapter answers exactly three questions (PRD §9): which symbol
//! events are contract-surface, what visibility means, what constitutes a
//! signature change. It also carries the mechanics those answers need:
//! the host command line, handshake quirks, facts extraction from LSP
//! symbols plus source text. The core routes by file extension; nothing
//! outside this directory names a language.

pub mod bicep;
pub mod csharp;
pub mod rust;
mod rust_facts;

use std::path::Path;

use serde_json::{json, Value};

use crate::client::ServerState;
use crate::protocol::RawSymbol;
use crate::surface::{PolicyRow, SymbolFacts};

/// How a host announces that it can answer semantic requests.
///
/// Not every server carries readiness in a method name. Roslyn sends one
/// `workspace/projectInitializationComplete` and means it; rust-analyzer
/// sends `experimental/serverStatus` repeatedly and puts the answer in the
/// payload. Both shapes are declared here so [`crate::host::Host`] stays
/// out of the business of knowing which host is which.
#[derive(Clone, Copy, Debug)]
pub enum ReadySignal {
    /// Ready once the `initialized` handshake completes.
    AfterInitialized,
    /// Ready once this notification method has been seen at least once.
    Notification(&'static str),
    /// Ready once this notification's most recent params satisfy the
    /// predicate.
    NotificationWhere(&'static str, fn(&Value) -> bool),
}

impl ReadySignal {
    /// Has the host signalled readiness, given what the reader thread has
    /// recorded so far?
    pub fn satisfied(&self, state: &ServerState) -> bool {
        match self {
            Self::AfterInitialized => true,
            Self::Notification(method) => {
                state.flags.lock().map(|f| f.contains(*method)).unwrap_or(false)
            }
            Self::NotificationWhere(method, predicate) => state
                .last_params
                .lock()
                .map(|p| p.get(*method).map(predicate).unwrap_or(false))
                .unwrap_or(false),
        }
    }

    /// What a still-loading host is being waited on for.
    pub fn describe(&self) -> &'static str {
        match self {
            Self::AfterInitialized => "initialization",
            Self::Notification(method) | Self::NotificationWhere(method, _) => method,
        }
    }
}

/// The client capabilities every host is announced with; adapters whose
/// host needs nothing beyond them use this.
pub fn no_extra_capabilities() -> Value {
    json!({})
}

/// Per-language behavior switches, resolved from `.ddd/config.yaml`
/// (`adapter.<language>.*`, config format 3).
#[derive(Debug, Clone)]
pub struct AdapterFlags {
    /// Treat `internal` visibility as contract surface (library-repo
    /// posture; `dec/ddd/internal-not-surface` sets the default off).
    pub internal_is_surface: bool,
    /// Attribute markers that make a symbol an exported endpoint.
    pub exported_attributes: Vec<String>,
}

impl Default for AdapterFlags {
    fn default() -> Self {
        Self {
            internal_is_surface: false,
            exported_attributes: vec!["McpServerTool".to_string()],
        }
    }
}

/// One language adapter: host wiring plus the three §9 answers.
pub struct Adapter {
    pub language: &'static str,
    /// The predicate-vocabulary artifact class this language's edits govern
    /// under (drives the per-class `intercept` mode).
    pub artifact_class: &'static str,
    pub extensions: &'static [&'static str],
    /// LSP `languageId` for didOpen.
    pub language_id: &'static str,
    /// Default host command; `adapter.<language>.command` overrides it.
    pub default_command: &'static [&'static str],
    /// How this host says it is ready.
    pub ready: ReadySignal,
    /// Client capabilities merged into the `initialize` payload. Some hosts
    /// stay silent about their own readiness unless asked — rust-analyzer
    /// sends no `experimental/serverStatus` at all without the matching
    /// capability — so the announcement has to be the adapter's to make.
    pub extra_capabilities: fn() -> Value,
    /// Host wants the workspace's build inputs announced after initialize.
    pub needs_open_handshake: bool,
    /// Derive symbol facts from source text plus LSP symbols.
    pub facts: fn(&str, &[RawSymbol], &AdapterFlags) -> Vec<SymbolFacts>,
    /// The contract-surface policy table (data, not logic — PRD §9).
    pub policy: fn(&AdapterFlags) -> Vec<PolicyRow>,
    /// Order visibilities by exposure (higher = more exposed).
    pub visibility_rank: fn(&str) -> u8,
    /// A repo-posture warning worth raising for edits near this file
    /// (e.g. C#'s InternalsVisibleTo → suggest the library posture).
    pub posture_warning: fn(&Path, &AdapterFlags) -> Option<String>,
}

/// Every adapter the build ships.
pub fn all() -> &'static [&'static Adapter] {
    &[&csharp::ADAPTER, &bicep::ADAPTER, &rust::ADAPTER]
}

/// The adapter registered for a language name.
pub fn for_language(language: &str) -> Option<&'static Adapter> {
    all().iter().copied().find(|a| a.language == language)
}

/// The adapter claiming a file's extension.
pub fn for_path(path: &Path) -> Option<&'static Adapter> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    all().iter().copied().find(|a| a.extensions.contains(&ext.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_by_extension() {
        assert_eq!(for_path(Path::new("/x/A.cs")).map(|a| a.language), Some("csharp"));
        assert_eq!(for_path(Path::new("/x/m.bicep")).map(|a| a.language), Some("bicep"));
        assert_eq!(for_path(Path::new("/x/lib.rs")).map(|a| a.language), Some("rust"));
        assert!(for_path(Path::new("/x/notes.md")).is_none());
    }

    #[test]
    fn artifact_classes_stay_in_the_predicate_vocabulary() {
        for a in all() {
            assert!(["code", "configuration"].contains(&a.artifact_class), "{}", a.language);
        }
    }
}
