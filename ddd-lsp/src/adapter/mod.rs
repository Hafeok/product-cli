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

use std::path::Path;

use crate::protocol::RawSymbol;
use crate::surface::{PolicyRow, SymbolFacts};

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
    /// Notification method that marks the host ready; `None` = ready after
    /// the `initialized` handshake.
    pub ready_flag: Option<&'static str>,
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
    &[&csharp::ADAPTER, &bicep::ADAPTER]
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
        assert!(for_path(Path::new("/x/lib.rs")).is_none());
    }

    #[test]
    fn artifact_classes_stay_in_the_predicate_vocabulary() {
        for a in all() {
            assert!(["code", "configuration"].contains(&a.artifact_class), "{}", a.language);
        }
    }
}
