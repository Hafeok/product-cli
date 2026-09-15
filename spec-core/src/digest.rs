//! The domain-separated digests a record binds itself with.
//!
//! Both ride `ledger_core`'s canonicalisation law under their own prefixes —
//! one law in this workspace, never a second scheme. Each builder
//! destructures its subject exhaustively with no `..` rest pattern, so a new
//! wire field is a compile error until someone decides whether it is hashed.

use ledger_core::canon::{put, put_set};
use ledger_core::hash::domain_hash;
use serde_json::{Map, Value};

use crate::closure::Closure;
use crate::record::ActRecord;

/// Domain-separation prefix for the opening digest.
pub const RECORD_FORM: &str = "spec.act-record.v1";
/// Domain-separation prefix for the closure digest.
pub const CLOSURE_FORM: &str = "spec.act-closure.v1";

/// The digest of a record's opening, ignoring any closure on it.
pub fn record_digest(record: &ActRecord) -> String {
    let ActRecord {
        form: _,
        id,
        act,
        slice,
        act_ref,
        opened_at,
        opened_by,
        base_revision,
        binds: _,
        closure: _,
    } = record;

    let mut m = Map::new();
    put(&mut m, "act", Some(act.clone()));
    put(&mut m, "act_ref", Some(act_ref.clone()));
    put(&mut m, "base_revision", Some(base_revision.clone()));
    put(&mut m, "id", Some(id.clone()));
    put(&mut m, "opened_at", Some(opened_at.to_rfc3339()));
    put(&mut m, "opened_by", Some(opened_by.as_str().to_string()));
    put(&mut m, "slice", Some(slice.clone()));
    domain_hash(RECORD_FORM, canonical_bytes(m).as_slice())
}

/// The digest of a closure, over the opening digest it discharges.
pub fn closure_digest(opening: &str, closure: &Closure) -> String {
    let Closure {
        kind,
        principal,
        at,
        determinations,
        binds: _,
        // A signature covers the digest; it cannot be inside it.
        signature: _,
    } = closure;

    let mut m = Map::new();
    put(&mut m, "at", Some(at.to_rfc3339()));
    put_set(&mut m, "determinations", determinations.iter().cloned());
    put(&mut m, "kind", Some(kind.as_str().to_string()));
    put(&mut m, "opening", Some(opening.to_string()));
    put(&mut m, "principal", Some(principal.as_str().to_string()));
    domain_hash(CLOSURE_FORM, canonical_bytes(m).as_slice())
}

/// Serialise a canonical object. `serde_json::Map` is a `BTreeMap` here — the
/// crate's `preserve_order` feature is off workspace-wide — so key order is
/// the code-point order the law requires, not insertion order.
fn canonical_bytes(map: Map<String, Value>) -> Vec<u8> {
    serde_json::to_vec(&Value::Object(map)).unwrap_or_default()
}

#[path = "digest_tests.rs"]
#[cfg(test)]
mod tests;
