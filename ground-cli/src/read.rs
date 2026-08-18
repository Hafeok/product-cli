//! Reading a codebase through the declaration-level LSP subset.
//!
//! The only half of extraction that knows a language server exists. It fills
//! `CorpusFacts` and hands it over, which is what keeps the mapping half —
//! and `product-core` — free of any dependency on the LSP stack.
//!
//! Every read is gated by the capability probe first. An operation that did
//! not answer is not called, and its absence travels with the facts so a
//! mapping row can report *why* it found nothing rather than being silently
//! empty.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ddd_lsp::capability::{self, Probe};
use ddd_lsp::declaration::{self, Operation};
use ddd_lsp::host::Host;
use product_core::ground::facts::{CorpusFacts, HierarchyEdge, OperationStatus, ReferenceSite};

use crate::symbols::{self, Anchor};

/// What to read, and from where.
pub struct ReadRequest {
    /// The corpus's abstracted name — never its real repository name.
    pub corpus: String,
    /// Commit the read is pinned to.
    pub git_ref: String,
    /// Repository root, so every citation is repo-relative.
    pub repo_root: PathBuf,
    /// Source files to read, absolute.
    pub files: Vec<PathBuf>,
    /// The file the capability probe anchors on.
    pub probe_file: PathBuf,
    /// Instrument string recorded in every Reading's assurance.
    pub instrument: String,
}

/// The facts, plus everything about the read a reviewer should know.
pub struct ReadOutcome {
    pub facts: CorpusFacts,
    pub probe: Probe,
    /// Operations that answered but no mapping row consumes.
    pub unconsumed: Vec<String>,
    /// Declarations whose name carried a generic parameter list.
    pub generic_declarations: usize,
    /// Types declared inside another type.
    pub nested_declarations: usize,
    /// Members read as fields rather than properties — implementation the
    /// declaration surface cannot tell from domain structure.
    pub field_members: usize,
    /// One type declared across several files — C#'s `partial`. Benign for
    /// the class row (the same triple collapses), but it means a declaration
    /// count is not a type count.
    pub partial_declarations: Vec<String>,
    /// Two *different* types sharing a simple name in different modules.
    /// This is the collision that makes name-based range resolution unsafe,
    /// measured rather than assumed away.
    pub name_collisions: Vec<String>,
}

/// Operations the current rule set actually calls.
const CONSUMED: [Operation; 4] =
    [Operation::DocumentSymbol, Operation::TypeHierarchy, Operation::References, Operation::Hover];

/// Read the corpus.
pub fn read(host: &mut Host, request: &ReadRequest) -> Result<ReadOutcome, String> {
    host.wait_ready(Duration::from_secs(900)).map_err(|e| e.to_string())?;
    let probe = capability::probe(host, &request.probe_file).map_err(|e| e.to_string())?;
    let mut facts = empty_facts(request, &probe);
    if !probe.available(Operation::DocumentSymbol) {
        return Ok(outcome(facts, probe, Tally::default()));
    }
    let mut anchors: Vec<Anchor> = Vec::new();
    let mut tally = Tally::default();
    for file in &request.files {
        let relative_path = relative(&request.repo_root, file);
        match declaration::document_symbols(host, file) {
            Ok(symbols) if !symbols.is_empty() => {
                for read in symbols::declarations_in(&relative_path, &symbols) {
                    tally.count(&read);
                    anchors.push(read.anchor);
                    facts.declarations.push(read.declaration);
                }
            }
            _ => facts.unread_files.push(relative_path),
        }
    }
    enrich(host, request, &probe, &anchors, &mut facts);
    Ok(outcome(facts, probe, tally))
}

/// Counts the read accumulates about its own shape.
#[derive(Default)]
struct Tally {
    generic: usize,
    nested: usize,
    fields: usize,
}

impl Tally {
    fn count(&mut self, read: &symbols::Read) {
        if read.generics.is_some() {
            self.generic += 1;
        }
        if read.nested {
            self.nested += 1;
        }
        self.fields += read.field_members;
    }
}

fn empty_facts(request: &ReadRequest, probe: &Probe) -> CorpusFacts {
    CorpusFacts {
        corpus: request.corpus.clone(),
        git_ref: request.git_ref.clone(),
        instrument: request.instrument.clone(),
        operations: probe
            .operations
            .iter()
            .map(|o| OperationStatus {
                operation: o.operation.as_str().to_string(),
                answered: o.answered,
                detail: o.detail.clone(),
            })
            .collect(),
        declarations: Vec::new(),
        hierarchy: Vec::new(),
        references: Vec::new(),
        unread_files: Vec::new(),
    }
}

fn outcome(facts: CorpusFacts, probe: Probe, tally: Tally) -> ReadOutcome {
    let unconsumed = probe
        .operations
        .iter()
        .filter(|o| o.answered && !CONSUMED.contains(&o.operation))
        .map(|o| o.operation.as_str().to_string())
        .collect();
    let (partial_declarations, name_collisions) = repeated_names(&facts);
    ReadOutcome {
        facts,
        probe,
        unconsumed,
        generic_declarations: tally.generic,
        nested_declarations: tally.nested,
        field_members: tally.fields,
        partial_declarations,
        name_collisions,
    }
}

/// Names declared more than once, split by whether it is one type or two.
///
/// The distinction is load-bearing and easy to miss. A name repeated inside
/// **one** module is C#'s `partial` — one type, several files — and merging
/// it is correct. The same name in **two** modules is two types, and
/// name-based range resolution would conflate them. Reporting a single
/// "duplicates" count would have said the wrong thing about both.
fn repeated_names(facts: &CorpusFacts) -> (Vec<String>, Vec<String>) {
    let mut modules: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for decl in &facts.declarations {
        modules.entry(&decl.name).or_default().insert(&decl.module);
        *counts.entry(&decl.name).or_default() += 1;
    }
    let mut partial = Vec::new();
    let mut collisions = Vec::new();
    for (name, in_modules) in modules {
        if in_modules.len() > 1 {
            collisions.push(name.to_string());
        } else if counts.get(name).copied().unwrap_or(0) > 1 {
            partial.push(name.to_string());
        }
    }
    (partial, collisions)
}

/// Second pass: the per-declaration operations, each gated on the probe.
fn enrich(
    host: &mut Host,
    request: &ReadRequest,
    probe: &Probe,
    anchors: &[Anchor],
    facts: &mut CorpusFacts,
) {
    for anchor in anchors {
        let path = request.repo_root.join(&anchor.file);
        if probe.available(Operation::TypeHierarchy) {
            collect_hierarchy(host, &path, anchor, facts);
        }
        if probe.available(Operation::References) {
            collect_references(host, request, &path, anchor, facts);
        }
        if probe.available(Operation::Hover) {
            collect_doc(host, &path, anchor, facts);
        }
    }
}

fn collect_hierarchy(host: &mut Host, path: &Path, anchor: &Anchor, facts: &mut CorpusFacts) {
    let Ok(items) = declaration::prepare_type_hierarchy(host, path, anchor.line, anchor.character)
    else {
        return;
    };
    for item in &items {
        let Ok(supers) = declaration::supertypes(host, item) else { continue };
        for sup in supers {
            let (sup_name, _) = symbols::split_generics(&sup.name);
            facts.hierarchy.push(HierarchyEdge { sub: anchor.name.clone(), sup: sup_name });
        }
    }
}

fn collect_references(
    host: &mut Host,
    request: &ReadRequest,
    path: &Path,
    anchor: &Anchor,
    facts: &mut CorpusFacts,
) {
    let Ok(sites) = declaration::references(host, path, anchor.line, anchor.character, false) else {
        return;
    };
    for site in sites {
        facts.references.push(ReferenceSite {
            target: anchor.name.clone(),
            file: relative(&request.repo_root, &site.file),
            line: site.line,
        });
    }
}

fn collect_doc(host: &mut Host, path: &Path, anchor: &Anchor, facts: &mut CorpusFacts) {
    let Ok(Some(text)) = declaration::hover(host, path, anchor.line, anchor.character) else {
        return;
    };
    if let Some(decl) =
        facts.declarations.iter_mut().find(|d| d.name == anchor.name && d.file == anchor.file)
    {
        decl.doc = Some(text);
    }
}

/// A path relative to the repository root, so no machine's layout enters a
/// citation a reviewer will read.
pub fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
#[path = "read_tests.rs"]
mod tests;
