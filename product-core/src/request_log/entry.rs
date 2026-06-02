//! Entry schema for `requests.jsonl` (FT-042, ADR-039, FT-051, FT-064).
//!
//! `EntryType` enumerates the eight entry kinds (FT-064 added `delete` to
//! the original seven). `EntryPayload` carries type-specific fields. The
//! canonical-JSON merge / parse helpers live in `entry_payload.rs` to keep
//! this file under the 400-line fitness limit.

use super::entry_payload;
use serde_json::{Map, Value};

/// Sentinel — log-path migration from `.product/request-log.jsonl`.
pub const MIGRATE_LOG_SENTINEL: &str = "log-path";

/// Sentinel — `product migrate consolidate` (FT-057, ADR-048).
pub const MIGRATE_LOG_SENTINEL_CONSOLIDATE: &str = "consolidate-paths";

/// Entry types per ADR-039 decision 4, extended by FT-064 with `delete`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Create,
    Change,
    CreateAndChange,
    Delete,
    Undo,
    Migrate,
    SchemaUpgrade,
    Verify,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Change => "change",
            Self::CreateAndChange => "create-and-change",
            Self::Delete => "delete",
            Self::Undo => "undo",
            Self::Migrate => "migrate",
            Self::SchemaUpgrade => "schema-upgrade",
            Self::Verify => "verify",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "create" => Some(Self::Create),
            "change" => Some(Self::Change),
            "create-and-change" => Some(Self::CreateAndChange),
            "delete" => Some(Self::Delete),
            "undo" => Some(Self::Undo),
            "migrate" => Some(Self::Migrate),
            "schema-upgrade" => Some(Self::SchemaUpgrade),
            "verify" => Some(Self::Verify),
            _ => None,
        }
    }
}

/// One artifact referenced by an Apply entry. Carries the artifact ID and,
/// when known, the repo-relative file path the write landed at (FT-051).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRef {
    pub id: String,
    pub file: Option<String>,
}

impl ArtifactRef {
    pub fn id_only(id: impl Into<String>) -> Self {
        ArtifactRef { id: id.into(), file: None }
    }
    pub fn new(id: impl Into<String>, file: impl Into<String>) -> Self {
        ArtifactRef { id: id.into(), file: Some(file.into()) }
    }
    pub(super) fn to_value(&self) -> Value {
        match &self.file {
            Some(f) => {
                let mut m = Map::new();
                m.insert("id".into(), Value::String(self.id.clone()));
                m.insert("file".into(), Value::String(f.clone()));
                Value::Object(m)
            }
            None => Value::String(self.id.clone()),
        }
    }
    pub(super) fn parse_value(v: &Value) -> Option<Self> {
        if let Some(s) = v.as_str() {
            return Some(ArtifactRef::id_only(s));
        }
        let obj = v.as_object()?;
        let id = obj.get("id").and_then(|x| x.as_str())?;
        let file = obj.get("file").and_then(|x| x.as_str()).map(String::from);
        Some(ArtifactRef { id: id.to_string(), file })
    }
}

/// Type-specific payload carried by an Entry (ADR-039 decision 4).
#[derive(Debug, Clone)]
pub enum EntryPayload {
    /// `create` / `change` / `create-and-change` / `delete`
    Apply {
        /// Full request source (as JSON) — optional, may be a summary
        request: Value,
        /// `result.created` — list of created artifact refs (id + optional file)
        created: Vec<ArtifactRef>,
        /// `result.changed` — list of changed artifact refs (id + optional file)
        changed: Vec<ArtifactRef>,
        /// `result.deleted` — list of deleted artifact refs (FT-064).
        deleted: Vec<ArtifactRef>,
    },
    Undo {
        undoes: String,
        inverse_request: Value,
    },
    Migrate {
        sources: Vec<String>,
        created: Vec<String>,
    },
    SchemaUpgrade {
        from_version: u32,
        to_version: u32,
        changes: String,
    },
    Verify {
        feature: String,
        tcs_run: Vec<String>,
        passing: Vec<String>,
        failing: Vec<String>,
        tag_created: Option<String>,
    },
}

impl EntryPayload {
    pub fn entry_type(&self) -> EntryType {
        match self {
            Self::Apply { .. } => EntryType::Create,
            Self::Undo { .. } => EntryType::Undo,
            Self::Migrate { .. } => EntryType::Migrate,
            Self::SchemaUpgrade { .. } => EntryType::SchemaUpgrade,
            Self::Verify { .. } => EntryType::Verify,
        }
    }
}

/// One log entry.
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub applied_at: String,
    pub applied_by: String,
    pub commit: String,
    pub entry_type: EntryType,
    pub reason: String,
    pub prev_hash: String,
    pub entry_hash: String,
    pub payload: EntryPayload,
}

impl Entry {
    /// Serialise to a `serde_json::Value` — shared envelope plus type-specific
    /// payload fields, with `entry-hash` included (possibly empty).
    pub fn to_value(&self) -> Value {
        let mut map = Map::new();
        map.insert("id".into(), Value::String(self.id.clone()));
        map.insert("applied-at".into(), Value::String(self.applied_at.clone()));
        map.insert("applied-by".into(), Value::String(self.applied_by.clone()));
        map.insert("commit".into(), Value::String(self.commit.clone()));
        map.insert("type".into(), Value::String(self.entry_type.as_str().into()));
        map.insert("reason".into(), Value::String(self.reason.clone()));
        map.insert("prev-hash".into(), Value::String(self.prev_hash.clone()));
        map.insert("entry-hash".into(), Value::String(self.entry_hash.clone()));
        entry_payload::merge_payload(&self.payload, &mut map);
        Value::Object(map)
    }

    /// Canonical-JSON serialisation with `entry-hash` blanked — the input to
    /// SHA-256 for `entry-hash` computation (ADR-039 decision 2).
    pub fn canonical_for_hash(&self) -> String {
        let mut v = self.to_value();
        if let Value::Object(ref mut m) = v {
            m.insert("entry-hash".into(), Value::String("".into()));
        }
        super::canonical::canonical_json(&v)
    }

    /// Compute the entry hash against the current contents.
    pub fn compute_hash(&self) -> String {
        super::canonical::sha256_hex(self.canonical_for_hash().as_bytes())
    }

    /// Canonical-JSON line for storage — includes the real entry-hash.
    pub fn canonical_line(&self) -> String {
        super::canonical::canonical_json(&self.to_value())
    }

    /// Parse a single line of `requests.jsonl` into an Entry.
    pub fn parse_line(line: &str) -> Result<(Entry, Value), String> {
        let value: Value = serde_json::from_str(line)
            .map_err(|e| format!("malformed JSON: {}", e))?;
        let obj = value
            .as_object()
            .ok_or_else(|| "entry must be a JSON object".to_string())?;
        let entry_type_str = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let entry_type = EntryType::parse(entry_type_str)
            .ok_or_else(|| format!("unknown entry type '{}'", entry_type_str))?;
        let payload = entry_payload::parse_payload(obj, entry_type);
        let entry = Entry {
            id: entry_payload::str_field(obj, "id"),
            applied_at: entry_payload::str_field(obj, "applied-at"),
            applied_by: entry_payload::str_field(obj, "applied-by"),
            commit: entry_payload::str_field(obj, "commit"),
            entry_type,
            reason: entry_payload::str_field(obj, "reason"),
            prev_hash: entry_payload::str_field(obj, "prev-hash"),
            entry_hash: entry_payload::str_field(obj, "entry-hash"),
            payload,
        };
        Ok((entry, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> Entry {
        Entry {
            id: "req-20260417-001".into(),
            applied_at: "2026-04-17T12:00:00Z".into(),
            applied_by: "git:Test <t@example.com>".into(),
            commit: "abc123".into(),
            entry_type: EntryType::Create,
            reason: "sample".into(),
            prev_hash: "0000000000000000".into(),
            entry_hash: "".into(),
            payload: EntryPayload::Apply {
                request: serde_json::Value::Null,
                created: vec![ArtifactRef::id_only("FT-001")],
                changed: vec![],
                deleted: vec![],
            },
        }
    }

    #[test]
    fn hash_deterministic() {
        let e = sample_entry();
        assert_eq!(e.compute_hash(), e.compute_hash());
    }

    #[test]
    fn hash_changes_on_field_change() {
        let mut a = sample_entry();
        let b = sample_entry();
        let ha = a.compute_hash();
        a.reason = "different".into();
        let _hb = a.compute_hash();
        assert_ne!(a.compute_hash(), ha);
        assert_eq!(b.compute_hash(), ha);
    }

    #[test]
    fn canonical_line_roundtrips() {
        let mut e = sample_entry();
        e.entry_hash = e.compute_hash();
        let line = e.canonical_line();
        let (parsed, _) = Entry::parse_line(&line).expect("parse");
        assert_eq!(parsed.id, e.id);
        assert_eq!(parsed.compute_hash(), e.entry_hash);
    }
}
