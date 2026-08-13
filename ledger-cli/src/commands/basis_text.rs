//! Resolving a basis pointer to the argument it carries.
//!
//! A pointer's *type* is not the thing a principal rules on. `mandate:…`
//! says an edge is a mandate; what the mandate asserts is one sentence
//! away, in the store the pointer names. For this repo that store is
//! `.ddd/`, so this resolver reads it — display only, never a correctness
//! path, and silent when the store is absent.
//!
//! `ledger-core` stays free of it deliberately: the substrate an outside
//! implementation imports must not carry one adopter's ontology. The
//! resolver is an input to the screen, not part of it.

use std::collections::BTreeMap;
use std::path::Path;

use ledger_core::show::BasisText;

/// Arguments resolved once per pass, keyed by pointer.
pub struct DddBasisText {
    by_pointer: BTreeMap<String, String>,
}

impl BasisText for DddBasisText {
    fn text(&self, pointer: &str) -> Option<String> {
        self.resolve(pointer)
    }
}

impl DddBasisText {
    /// Read every argument the `.ddd/` store can supply. An absent store
    /// yields an empty resolver, which renders bare pointers — the same
    /// output a store-less adopter gets.
    pub fn load(root: &Path) -> Self {
        let mut by_pointer = BTreeMap::new();
        if !root.join(ddd_core::store::STORE_DIR).is_dir() {
            return Self { by_pointer };
        }
        let store = ddd_core::store::load(root);
        for d in &store.decisions {
            index_decision(&mut by_pointer, d);
        }
        for c in &store.claims {
            // A claim pointer carries a content pin, so the key is a
            // prefix match rather than an equality — indexed by id here
            // and matched in `text` through the prefix table below.
            by_pointer.insert(format!("claim:{}", c.id), c.statement.trim().to_string());
        }
        for s in &store.seams {
            by_pointer
                .insert(format!("ddd-content:{}", s.id), s.verdict_knowledge.trim().to_string());
        }
        Self { by_pointer }
    }

    /// Resolve a pointer, tolerating the `@sha256:…` pin every content
    /// pointer carries: the pin identifies a state, the text is the same
    /// text either way, and refusing to render it because of the suffix
    /// would leave the reader with a type and no argument.
    pub fn resolve(&self, pointer: &str) -> Option<String> {
        if let Some(hit) = self.by_pointer.get(pointer) {
            return Some(hit.clone());
        }
        let unpinned = pointer.split('@').next()?;
        self.by_pointer.get(unpinned).cloned()
    }
}

/// A decision contributes two kinds of argument: its own rationale, behind
/// the `ddd-content:` pointer that pins it, and each typed basis statement,
/// behind the `<type>:<decision-id>` pointer the migration wrote.
fn index_decision(out: &mut BTreeMap<String, String>, d: &ddd_core::decision::Decision) {
    out.insert(format!("ddd-content:{}", d.id), d.rationale.trim().to_string());
    for basis in &d.based_on {
        let ddd_core::decision::BasedOn::Typed(t) = basis else { continue };
        let Some(statement) = t.statement.as_deref().map(str::trim).filter(|s| !s.is_empty())
        else {
            continue;
        };
        // Several typed bases of the same kind on one decision would
        // collide on this key. Joining rather than overwriting keeps every
        // argument visible; a silently dropped one is the failure mode
        // that matters here.
        out.entry(format!("{}:{}", t.basis_type.as_str(), d.id))
            .and_modify(|existing| {
                existing.push('\n');
                existing.push_str(statement);
            })
            .or_insert_with(|| statement.to_string());
    }
}
