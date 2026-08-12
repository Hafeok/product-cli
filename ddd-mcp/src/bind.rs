//! Signed-binding resolution — which stored declaration discharges an
//! edit's transition (M8, spec invariant 3).
//!
//! Same-session matching is retired (`DDD-arch-09`: it discharged
//! whatever change next touched the symbol). A declaration now signs the
//! change it discharges — subject symbol, before/after content hashes,
//! base revision — and the interceptor matches edits against those
//! signatures alone. The parent state must be committed: a dirty file
//! refuses to bind, so a binding always names a reproducible transition.

use std::path::Path;

use ddd_core::binding::SeamBinding;
use ddd_core::contracts::{content_hash, ABSENT};
use ddd_core::gitrev;
use ddd_core::seam::SeamDeclaration;
use ddd_core::surface::SurfaceEvent;

/// The transition an edit proposes, content-addressed.
#[derive(Debug, Clone)]
pub struct Transition {
    /// Repo-relative file, forward slashes.
    pub rel: String,
    /// Hash of the parent state, or [`ABSENT`].
    pub before: String,
    /// Hash of the proposed state.
    pub after: String,
    /// The HEAD commit, when the root is a git repository.
    pub head: Option<String>,
    /// Whether the parent state is exactly what HEAD carries — the
    /// precondition for binding (M8 ruling 2: pin against HEAD, never
    /// against uncommitted parent state).
    pub committed_parent: bool,
}

impl Transition {
    /// Content-address the edit and check its parent state against HEAD.
    pub fn of(root: &Path, file: &Path, rel: &str, old_text: &str, new_text: &str) -> Self {
        let on_disk = file.exists();
        let before =
            if on_disk { content_hash(old_text.as_bytes()) } else { ABSENT.to_string() };
        let after = content_hash(new_text.as_bytes());
        let head = gitrev::head(root).ok();
        let committed_parent = match &head {
            None => false,
            Some(h) => match (gitrev::bytes_at(root, h, rel).ok().flatten(), on_disk) {
                (None, false) => true,
                (Some(bytes), true) => bytes.as_slice() == old_text.as_bytes(),
                _ => false,
            },
        };
        Self { rel: rel.to_string(), before, after, head, committed_parent }
    }

    /// The refusal text when the parent state is not committed.
    pub fn dirty_reason(&self) -> String {
        match &self.head {
            None => "the repository has no HEAD commit — a declaration binds against committed \
                     parent state, and there is none to bind against (M8 ruling 2)"
                .to_string(),
            Some(h) => format!(
                "{} is dirty against HEAD ({}) — a declaration never binds uncommitted parent \
                 state; commit the current state first (M8 ruling 2, the ledger's \
                 construction-limit precedent)",
                self.rel,
                &h[..12.min(h.len())]
            ),
        }
    }
}

/// The stored seam whose signed binding discharges each event, matched on
/// the exact transition: subject symbol, file, before/after hashes, and
/// the pinned base revision equal to the current HEAD.
pub fn match_bindings(
    seams: &[SeamDeclaration],
    events: &[SurfaceEvent],
    t: &Transition,
) -> Vec<Option<String>> {
    events
        .iter()
        .map(|e| {
            seams
                .iter()
                .find(|s| {
                    s.bindings.iter().any(|b| {
                        binding_signs(b, &e.facts.name, t)
                            && Some(&b.base_revision) == t.head.as_ref()
                    })
                })
                .map(|s| s.id.clone())
        })
        .collect()
}

/// Warn-mode advisory links: a signed binding when one matches, else the
/// generous file arm over stored declarations (the edit applies either
/// way, so a broad link can inform without discharging).
pub fn advisory_links(
    seams: &[SeamDeclaration],
    events: &[SurfaceEvent],
    t: &Transition,
) -> Vec<Option<String>> {
    let exact = match_bindings(seams, events, t);
    events
        .iter()
        .zip(exact)
        .map(|(e, hit)| {
            hit.or_else(|| {
                seams
                    .iter()
                    .find(|s| {
                        s.contract_location.contains(&t.rel)
                            || s.bindings.iter().any(|b| b.symbol == e.facts.name)
                    })
                    .map(|s| s.id.clone())
            })
        })
        .collect()
}

fn binding_signs(b: &SeamBinding, symbol: &str, t: &Transition) -> bool {
    b.signs(symbol, &t.rel, &t.before, &t.after)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn seam(id: &str, bindings: Vec<SeamBinding>) -> SeamDeclaration {
        SeamDeclaration {
            format: 2,
            id: id.to_string(),
            boundary: "b".into(),
            verdict_knowledge: "v".into(),
            contract_location: "src/a.rs#f".into(),
            obligations: Vec::new(),
            metadata: BTreeMap::new(),
            bindings,
            notes: None,
        }
    }

    fn event(name: &str) -> SurfaceEvent {
        SurfaceEvent {
            change: ddd_core::surface::ChangeKind::Added,
            facts: ddd_core::surface::SymbolFacts {
                name: name.into(),
                container: String::new(),
                kind: "fn".into(),
                visibility: "public".into(),
                signature: "fn f()".into(),
                decorators: Vec::new(),
                exported: false,
                extra: BTreeMap::new(),
                sel_line: 0,
                sel_char: 0,
            },
            before: None,
            surface: true,
            rule: None,
            rule_claim: None,
        }
    }

    fn transition() -> Transition {
        Transition {
            rel: "src/a.rs".into(),
            before: "sha256:aa".into(),
            after: "sha256:bb".into(),
            head: Some("headsha".into()),
            committed_parent: true,
        }
    }

    #[test]
    fn a_binding_discharges_only_its_exact_transition() {
        let t = transition();
        let b = SeamBinding::seal("f", "src/a.rs", "sha256:aa", "sha256:bb", "headsha");
        let seams = vec![seam("seam/rust/f", vec![b])];
        let hit = match_bindings(&seams, &[event("f")], &t);
        assert_eq!(hit, vec![Some("seam/rust/f".to_string())]);

        // A different edit landing on identical content: same after,
        // different before — refused.
        let other = Transition { before: "sha256:cc".into(), ..t.clone() };
        assert_eq!(match_bindings(&seams, &[event("f")], &other), vec![None]);

        // Same content pair pinned at a different base revision — refused.
        let moved = Transition { head: Some("othersha".into()), ..t };
        assert_eq!(match_bindings(&seams, &[event("f")], &moved), vec![None]);
    }

    #[test]
    fn warn_mode_links_generously_but_only_signed_bindings_discharge() {
        let t = transition();
        let seams = vec![seam("seam/rust/f", Vec::new())];
        assert_eq!(match_bindings(&seams, &[event("f")], &t), vec![None]);
        let links = advisory_links(&seams, &[event("f")], &t);
        assert_eq!(links, vec![Some("seam/rust/f".to_string())]);
    }
}
