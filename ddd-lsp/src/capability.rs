//! Capability probing — an operation's advertisement is a claim, never a verdict.
//!
//! An LSP server's `InitializeResult` capability set is a statement of
//! intent by its author. The only evidence that an operation works is a
//! live call returning a usable result, so every operation of the
//! declaration-level subset is probed at start-up against a known
//! declaration (G-track PRD §5.2, binding extractor-wide).
//!
//! Both failure directions have been observed on real servers:
//!
//! - **Over-advertising** — the flag is `true`, the call returns `null`.
//!   This is the dangerous one. It is the shape of presumed discharge
//!   (`DDD-delivery-02`) inside our own instrument: a gate that reads as
//!   passed, zero edges extracted, a run that reports success. Left
//!   unguarded, missing edges reach cross-codebase comparison, which reads
//!   absence as divergence — ground fabricated by an instrument fault.
//! - **Under-advertising** — no flag, the call answers anyway.
//!
//! Therefore the probe result, not the flag, gates the mapping rows. An
//! operation that did not answer makes its row's absence *reported*, so a
//! reviewer can tell "this codebase has no hierarchy" apart from "the
//! instrument did not answer".

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::declaration::{self, Operation};
use crate::error::Result;
use crate::host::Host;
use crate::protocol::RawSymbol;

/// LSP `SymbolKind`s that name a type declaration — the anchor the probe
/// wants, because all six operations have a non-empty expectation there.
const TYPE_KINDS: [u64; 5] = [5, 10, 11, 23, 26];

/// Where the probe called: a real type declaration in the corpus.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Anchor {
    pub file: PathBuf,
    pub name: String,
    pub line: u64,
    pub character: u64,
}

/// The two ways an advertisement disagrees with behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Divergence {
    /// Advertised, did not answer. The dangerous direction.
    OverAdvertised,
    /// Not advertised, answered anyway.
    UnderAdvertised,
}

/// One operation's verdict.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OperationProbe {
    pub operation: Operation,
    /// The `ServerCapabilities` flag as read; `None` when the server names
    /// no such key at all.
    pub advertised: Option<bool>,
    /// Did a live call return a usable result? This is the verdict.
    pub answered: bool,
    pub divergence: Option<Divergence>,
    /// What the call actually produced, in one line.
    pub detail: String,
}

/// The whole probe for one host.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Probe {
    pub language: String,
    /// `None` when `documentSymbol` returned no type declaration to anchor
    /// on — in which case every later operation is unprobed, not absent.
    pub anchor: Option<Anchor>,
    pub operations: Vec<OperationProbe>,
}

impl Probe {
    /// Is this operation usable? The probe answers, never the flag.
    pub fn available(&self, operation: Operation) -> bool {
        self.operations.iter().any(|o| o.operation == operation && o.answered)
    }

    /// The operations that did not answer, whatever they advertise.
    pub fn unavailable(&self) -> Vec<Operation> {
        self.operations.iter().filter(|o| !o.answered).map(|o| o.operation).collect()
    }

    /// Every advertisement that disagreed with behaviour.
    pub fn divergences(&self) -> Vec<&OperationProbe> {
        self.operations.iter().filter(|o| o.divergence.is_some()).collect()
    }

    /// A one-operation-per-line report.
    pub fn report(&self) -> String {
        let mut out = format!("capability probe — {}\n", self.language);
        match &self.anchor {
            Some(a) => out.push_str(&format!(
                "  anchor: {} at {}:{}:{}\n",
                a.name,
                a.file.display(),
                a.line + 1,
                a.character + 1
            )),
            None => out.push_str("  anchor: none — documentSymbol found no type declaration\n"),
        }
        for probe in &self.operations {
            let flag = match probe.advertised {
                Some(true) => "advertised",
                Some(false) => "declined",
                None => "unadvertised",
            };
            let verdict = if probe.answered { "AVAILABLE" } else { "UNAVAILABLE" };
            let note = match probe.divergence {
                Some(Divergence::OverAdvertised) => "  <- OVER-ADVERTISED",
                Some(Divergence::UnderAdvertised) => "  <- under-advertised",
                None => "",
            };
            out.push_str(&format!(
                "  {:<16} {:<13} {:<12} {}{}\n",
                probe.operation.as_str(),
                flag,
                verdict,
                probe.detail,
                note
            ));
        }
        out
    }
}

/// Probe every operation of the subset against a known declaration in
/// `file`.
///
/// `documentSymbol` runs first: the anchor for the other five is derived
/// from its result, so a host that cannot list symbols yields a probe with
/// no anchor rather than five misleading absences.
pub fn probe(host: &mut Host, file: &Path) -> Result<Probe> {
    let language = host.language().to_string();
    let capabilities = host.server_capabilities()?;
    let symbols = declaration::document_symbols(host, file).unwrap_or_default();
    let anchor = anchor_from(file, &symbols);
    let mut operations = vec![row(
        Operation::DocumentSymbol,
        &capabilities,
        !symbols.is_empty(),
        format!("{} symbol(s)", symbols.len()),
    )];
    for operation in Operation::ALL.iter().skip(1) {
        let (answered, detail) = match &anchor {
            Some(a) => call(host, *operation, a),
            None => (false, "not probed — no anchor".to_string()),
        };
        operations.push(row(*operation, &capabilities, answered, detail));
    }
    Ok(Probe { language, anchor, operations })
}

/// Run one operation at the anchor, reducing it to answered-plus-detail.
///
/// An error is not an escape: it is one way of not answering, and it is
/// recorded as the detail so a reviewer sees why.
fn call(host: &mut Host, operation: Operation, at: &Anchor) -> (bool, String) {
    let (file, line, ch) = (at.file.as_path(), at.line, at.character);
    match operation {
        Operation::DocumentSymbol => (false, "probed first".to_string()),
        Operation::WorkspaceSymbol => {
            count(declaration::workspace_symbols(host, &at.name).map(|s| s.len()), "symbol")
        }
        Operation::Definition => {
            count(declaration::definition(host, file, line, ch).map(|l| l.len()), "location")
        }
        Operation::References => count(
            declaration::references(host, file, line, ch, true).map(|l| l.len()),
            "reference",
        ),
        Operation::Hover => match declaration::hover(host, file, line, ch) {
            Ok(Some(text)) => (true, format!("{} char(s)", text.chars().count())),
            Ok(None) => (false, "empty".to_string()),
            Err(e) => (false, format!("error: {e}")),
        },
        Operation::TypeHierarchy => count(
            declaration::prepare_type_hierarchy(host, file, line, ch).map(|i| i.len()),
            "item",
        ),
    }
}

fn count(result: Result<usize>, noun: &str) -> (bool, String) {
    match result {
        Ok(0) => (false, format!("0 {noun}s")),
        Ok(n) => (true, format!("{n} {noun}(s)")),
        Err(e) => (false, format!("error: {e}")),
    }
}

/// Read the flag, compare it to behaviour, name the disagreement.
fn row(
    operation: Operation,
    capabilities: &Value,
    answered: bool,
    detail: String,
) -> OperationProbe {
    let advertised = advertises(capabilities, operation);
    let divergence = match (advertised, answered) {
        (Some(true), false) => Some(Divergence::OverAdvertised),
        (Some(false) | None, true) => Some(Divergence::UnderAdvertised),
        _ => None,
    };
    OperationProbe { operation, advertised, answered, divergence, detail }
}

/// The capability flag as the server states it.
///
/// A provider key may be a bare boolean or an options object; an object is
/// an advertisement just as much as `true` is.
fn advertises(capabilities: &Value, operation: Operation) -> Option<bool> {
    match capabilities.get(operation.capability_key()) {
        None | Some(Value::Null) => None,
        Some(Value::Bool(b)) => Some(*b),
        Some(_) => Some(true),
    }
}

/// The first type declaration in the file — the probe's known declaration.
fn anchor_from(file: &Path, symbols: &[RawSymbol]) -> Option<Anchor> {
    let symbol = symbols
        .iter()
        .find(|s| TYPE_KINDS.contains(&s.kind))
        .or_else(|| symbols.first())?;
    Some(Anchor {
        file: file.to_path_buf(),
        name: symbol.name.clone(),
        line: symbol.sel_line,
        character: symbol.sel_char,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sym(name: &str, kind: u64, line: u64, ch: u64) -> RawSymbol {
        RawSymbol {
            name: name.into(),
            kind,
            container: String::new(),
            container_kind: 0,
            start_line: line,
            end_line: line,
            sel_line: line,
            sel_char: ch,
            detail: None,
        }
    }

    #[test]
    fn the_anchor_prefers_a_type_declaration_over_a_namespace() {
        let symbols = vec![sym("Acme.Sql", 3, 0, 10), sym("ProfileSettings", 5, 4, 21)];
        let anchor = anchor_from(Path::new("/x/A.cs"), &symbols).expect("anchor");
        assert_eq!(anchor.name, "ProfileSettings");
        assert_eq!((anchor.line, anchor.character), (4, 21));
    }

    #[test]
    fn no_symbols_means_no_anchor() {
        assert!(anchor_from(Path::new("/x/A.cs"), &[]).is_none());
    }

    /// The kotlin-lsp shape: `typeHierarchyProvider: true`, result `null`.
    /// This must read as UNAVAILABLE, flagged, never as an empty hierarchy.
    #[test]
    fn an_advertised_operation_that_does_not_answer_is_over_advertised() {
        let caps = json!({"typeHierarchyProvider": true});
        let probe = row(Operation::TypeHierarchy, &caps, false, "0 items".into());
        assert!(!probe.answered);
        assert_eq!(probe.divergence, Some(Divergence::OverAdvertised));
    }

    /// The Roslyn shape: no `typeHierarchy` client capability declared,
    /// the server answers regardless.
    #[test]
    fn an_unadvertised_operation_that_answers_is_under_advertised() {
        let probe = row(Operation::TypeHierarchy, &json!({}), true, "1 item".into());
        assert!(probe.answered);
        assert_eq!(probe.divergence, Some(Divergence::UnderAdvertised));
    }

    #[test]
    fn agreement_in_either_direction_is_not_a_divergence() {
        let yes = json!({"hoverProvider": true});
        assert_eq!(row(Operation::Hover, &yes, true, String::new()).divergence, None);
        let no = json!({"hoverProvider": false});
        assert_eq!(row(Operation::Hover, &no, false, String::new()).divergence, None);
    }

    #[test]
    fn an_options_object_is_an_advertisement_just_as_true_is() {
        let caps = json!({"referencesProvider": {"workDoneProgress": true}});
        assert_eq!(advertises(&caps, Operation::References), Some(true));
        assert_eq!(advertises(&json!({}), Operation::References), None);
        assert_eq!(advertises(&json!({"referencesProvider": null}), Operation::References), None);
    }

    #[test]
    fn an_error_is_a_way_of_not_answering_not_an_escape() {
        let (answered, detail) = count(Err(crate::LspError::Io("boom".into())), "location");
        assert!(!answered);
        assert!(detail.starts_with("error:"), "{detail}");
    }

    #[test]
    fn availability_reads_the_probe_never_the_flag() {
        let probe = Probe {
            language: "csharp".into(),
            anchor: None,
            operations: vec![
                row(Operation::TypeHierarchy, &json!({"typeHierarchyProvider": true}), false, "0 items".into()),
                row(Operation::Hover, &json!({}), true, "12 chars".into()),
            ],
        };
        assert!(!probe.available(Operation::TypeHierarchy));
        assert!(probe.available(Operation::Hover));
        assert_eq!(probe.unavailable(), vec![Operation::TypeHierarchy]);
        assert_eq!(probe.divergences().len(), 2);
    }
}
