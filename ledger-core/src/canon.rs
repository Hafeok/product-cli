//! The canonical form of a version — the bytes a hash is taken over.
//!
//! YAML is the *file* format; canonical JSON is the *hash* form. Routing
//! through a second, restricted serialisation is what makes
//! "formatting-only edits leave the hash unchanged" structural rather than a
//! rule someone has to remember: quoting style, key order, indentation,
//! comments, line endings and `null`-versus-absent all disappear in the
//! parse, before anything is hashed. It also gives an outside implementation
//! — the Org Ledger imports this format — a target with no YAML ambiguity to
//! reproduce: no anchors, no tags, no four ways to write a multi-line scalar.
//!
//! The full specification, including the pinned conformance vector, is
//! `docs/ledger-format-v1.md`. This module is its implementation, not its
//! definition; where they disagree the document wins.

use std::collections::BTreeSet;

use serde_json::{Map, Value};
use unicode_normalization::UnicodeNormalization;

use crate::version::VersionRaw;

/// The canonical-form version, carried as the hash's domain-separation
/// prefix. Independent of a file's `format`: a format bump that changes what
/// a hashed field *means* must bump this too.
pub const CANONICAL_FORM: &str = "ledger.decision-version.v1";

/// Normalise a string the way canonicalisation requires, in order:
/// line endings to LF, NFC, then leading/trailing ASCII whitespace removed.
/// A value that ends up empty is treated as absent.
fn norm(s: &str) -> Option<String> {
    let unified = s.replace("\r\n", "\n").replace('\r', "\n");
    let composed: String = unified.nfc().collect();
    let trimmed = composed.trim_matches(|c: char| c.is_ascii_whitespace());
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// Insert a string field, omitting it when it normalises to nothing.
fn put(map: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(v) = value.as_deref().and_then(norm) {
        map.insert(key.to_string(), Value::String(v));
    }
}

/// Insert a list field as a deduplicated, code-point-sorted array of the
/// members' canonical string forms. A discharge list and a basis list are
/// *sets*: reordering them in a file is formatting, not meaning.
fn put_set(map: &mut Map<String, Value>, key: &str, items: impl Iterator<Item = String>) {
    let sorted: BTreeSet<String> = items.filter_map(|s| norm(&s)).collect();
    if !sorted.is_empty() {
        let array = sorted.into_iter().map(Value::String).collect();
        map.insert(key.to_string(), Value::Array(array));
    }
}

/// The canonical JSON text of a version's hashed content.
///
/// Exhaustive destructuring with no `..` rest pattern is the guard: adding a
/// field to [`VersionRaw`] breaks this function at compile time, so no field
/// can join the wire form without someone deciding whether it is inside the
/// hash. The same class of bug — a field silently missing from a parallel
/// enumeration — is the one that bites the `pf` graph's node kinds, and it is
/// worth closing structurally rather than by review.
pub fn canonical_json(raw: &VersionRaw) -> String {
    // `hash` is bound and discarded on purpose: the stored digest is what
    // the hash is compared *to*, so including it would be circular. The two
    // tolerance inputs are both hashed rather than the resolved tier — the
    // pin is history, the override is authored, and keeping them apart is
    // what lets override-rate-per-set (§10) come out of hashed content.
    let VersionRaw {
        decision, parent, merged_from, hash: _, set, statement, allocation, discharge,
        discharge_stage, actor, expectation, exposure, accepted_by, review_by,
        tolerance_floor_at_creation, tolerance_override, based_on, supersedes,
    } = raw;
    let mut m = Map::new();
    for (key, value) in [
        ("decision", Some(decision.to_string())),
        ("parent", parent.as_ref().map(ToString::to_string)),
        // Hashed when present; omitted when absent, which is what keeps
        // every format-1 digest stable under the format-2 addition.
        ("merged_from", merged_from.as_ref().map(ToString::to_string)),
        ("set", Some(set.clone())),
        ("statement", Some(statement.clone())),
        ("allocation", allocation.map(|a| a.as_str().to_string())),
        ("discharge_stage", discharge_stage.map(|s| s.to_string())),
        ("actor", actor.as_ref().map(ToString::to_string)),
        ("expectation", expectation.clone()),
        ("exposure", exposure.clone()),
        ("accepted_by", accepted_by.as_ref().map(ToString::to_string)),
        ("review_by", review_by.map(|d| d.to_string())),
        ("tolerance_floor_at_creation", Some(tolerance_floor_at_creation.to_string())),
        ("tolerance_override", tolerance_override.map(|t| t.to_string())),
        ("supersedes", supersedes.as_ref().map(ToString::to_string)),
    ] {
        put(&mut m, key, value);
    }
    put_set(&mut m, "discharge", discharge.iter().map(ToString::to_string));
    put_set(&mut m, "based_on", based_on.iter().map(ToString::to_string));
    // serde_json's map is a BTreeMap whose writer emits no insignificant
    // whitespace, so this is key-sorted and compact by construction. Keys in
    // this format are ASCII, where code-point order and the JCS UTF-16
    // code-unit order coincide.
    Value::Object(m).to_string()
}

/// The canonical bytes, as UTF-8. No trailing newline.
pub fn canonical_bytes(raw: &VersionRaw) -> Vec<u8> {
    canonical_json(raw).into_bytes()
}

#[path = "canon_tests.rs"]
#[cfg(test)]
mod tests;
