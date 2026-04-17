//! Entry schema for `requests.jsonl` (FT-042, ADR-039).

use serde_json::{json, Map, Value};

/// Sentinel string indicating this migrate entry documents the log-path move
/// from `.product/request-log.jsonl` to `requests.jsonl`.
pub const MIGRATE_LOG_SENTINEL: &str = "log-path";

/// Seven entry types per ADR-039 decision 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Create,
    Change,
    CreateAndChange,
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
            "undo" => Some(Self::Undo),
            "migrate" => Some(Self::Migrate),
            "schema-upgrade" => Some(Self::SchemaUpgrade),
            "verify" => Some(Self::Verify),
            _ => None,
        }
    }
}

/// Type-specific payload carried by an Entry (ADR-039 decision 4).
#[derive(Debug, Clone)]
pub enum EntryPayload {
    /// `create` / `change` / `create-and-change`
    Apply {
        /// Full request source (as JSON) — optional, may be a summary
        request: Value,
        /// `result.created` — list of created artifact IDs
        created: Vec<String>,
        /// `result.changed` — list of changed artifact IDs
        changed: Vec<String>,
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

    fn merge_into(&self, map: &mut Map<String, Value>) {
        match self {
            Self::Apply { request, created, changed } => merge_apply(map, request, created, changed),
            Self::Undo { undoes, inverse_request } => merge_undo(map, undoes, inverse_request),
            Self::Migrate { sources, created } => merge_migrate(map, sources, created),
            Self::SchemaUpgrade { from_version, to_version, changes } => {
                map.insert("from-version".into(), json!(from_version));
                map.insert("to-version".into(), json!(to_version));
                map.insert("changes".into(), Value::String(changes.clone()));
            }
            Self::Verify { feature, tcs_run, passing, failing, tag_created } => {
                merge_verify(map, feature, tcs_run, passing, failing, tag_created);
            }
        }
    }
}

fn str_array(items: &[String]) -> Value {
    Value::Array(items.iter().map(|s| Value::String(s.clone())).collect())
}

fn merge_apply(map: &mut Map<String, Value>, request: &Value, created: &[String], changed: &[String]) {
    if !request.is_null() {
        map.insert("request".into(), request.clone());
    }
    let mut result = Map::new();
    result.insert("created".into(), str_array(created));
    result.insert("changed".into(), str_array(changed));
    map.insert("result".into(), Value::Object(result));
}

fn merge_undo(map: &mut Map<String, Value>, undoes: &str, inverse: &Value) {
    map.insert("undoes".into(), Value::String(undoes.into()));
    map.insert("inverse-request".into(), inverse.clone());
}

fn merge_migrate(map: &mut Map<String, Value>, sources: &[String], created: &[String]) {
    map.insert("sources".into(), str_array(sources));
    let mut result = Map::new();
    result.insert("created".into(), str_array(created));
    map.insert("result".into(), Value::Object(result));
}

fn merge_verify(
    map: &mut Map<String, Value>,
    feature: &str,
    tcs_run: &[String],
    passing: &[String],
    failing: &[String],
    tag_created: &Option<String>,
) {
    map.insert("feature".into(), Value::String(feature.into()));
    let mut result = Map::new();
    result.insert("tcs-run".into(), str_array(tcs_run));
    result.insert("passing".into(), str_array(passing));
    result.insert("failing".into(), str_array(failing));
    let tag = match tag_created {
        Some(t) => Value::String(t.clone()),
        None => Value::Null,
    };
    result.insert("tag-created".into(), tag);
    map.insert("result".into(), Value::Object(result));
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
        self.payload.merge_into(&mut map);
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
    /// Returns a tuple of the parsed Entry (best-effort) and the stored
    /// canonical JSON `Value` so callers can introspect unknown fields.
    pub fn parse_line(line: &str) -> Result<(Entry, Value), String> {
        let value: Value = serde_json::from_str(line)
            .map_err(|e| format!("malformed JSON: {}", e))?;
        let obj = value
            .as_object()
            .ok_or_else(|| "entry must be a JSON object".to_string())?;
        let entry_type_str = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let entry_type = EntryType::parse(entry_type_str)
            .ok_or_else(|| format!("unknown entry type '{}'", entry_type_str))?;
        let payload = parse_payload(obj, entry_type);
        let entry = Entry {
            id: str_field(obj, "id"),
            applied_at: str_field(obj, "applied-at"),
            applied_by: str_field(obj, "applied-by"),
            commit: str_field(obj, "commit"),
            entry_type,
            reason: str_field(obj, "reason"),
            prev_hash: str_field(obj, "prev-hash"),
            entry_hash: str_field(obj, "entry-hash"),
            payload,
        };
        Ok((entry, value))
    }
}

fn str_field(obj: &Map<String, Value>, key: &str) -> String {
    obj.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn str_array_field(obj: &Map<String, Value>, key: &str) -> Vec<String> {
    obj.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

fn result_array(obj: &Map<String, Value>, key: &str) -> Vec<String> {
    obj.get("result")
        .and_then(|v| v.as_object())
        .map(|r| str_array_field(r, key))
        .unwrap_or_default()
}

fn parse_payload(obj: &Map<String, Value>, entry_type: EntryType) -> EntryPayload {
    match entry_type {
        EntryType::Create | EntryType::Change | EntryType::CreateAndChange => EntryPayload::Apply {
            request: obj.get("request").cloned().unwrap_or(Value::Null),
            created: result_array(obj, "created"),
            changed: result_array(obj, "changed"),
        },
        EntryType::Undo => EntryPayload::Undo {
            undoes: str_field(obj, "undoes"),
            inverse_request: obj.get("inverse-request").cloned().unwrap_or(Value::Null),
        },
        EntryType::Migrate => EntryPayload::Migrate {
            sources: str_array_field(obj, "sources"),
            created: result_array(obj, "created"),
        },
        EntryType::SchemaUpgrade => EntryPayload::SchemaUpgrade {
            from_version: obj.get("from-version").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            to_version: obj.get("to-version").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            changes: str_field(obj, "changes"),
        },
        EntryType::Verify => EntryPayload::Verify {
            feature: str_field(obj, "feature"),
            tcs_run: result_array(obj, "tcs-run"),
            passing: result_array(obj, "passing"),
            failing: result_array(obj, "failing"),
            tag_created: obj
                .get("result")
                .and_then(|v| v.as_object())
                .and_then(|r| r.get("tag-created"))
                .and_then(|v| v.as_str())
                .map(String::from),
        },
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
                created: vec!["FT-001".into()],
                changed: vec![],
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
        let hb = a.compute_hash();
        assert_ne!(ha, hb);
        // original still matches sibling
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
