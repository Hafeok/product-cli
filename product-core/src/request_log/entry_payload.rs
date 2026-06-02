//! Encode + decode of `EntryPayload` against canonical JSON (FT-042, FT-064).
//!
//! Split out from `entry.rs` to keep that file under the 400-line fitness
//! limit. The merge functions are called from `EntryPayload::merge_into`;
//! the parse function is called from `Entry::parse_line`.

use super::entry::{ArtifactRef, EntryPayload, EntryType};
use serde_json::{json, Map, Value};

pub(super) fn merge_payload(payload: &EntryPayload, map: &mut Map<String, Value>) {
    match payload {
        EntryPayload::Apply { request, created, changed, deleted } => {
            merge_apply(map, request, created, changed, deleted)
        }
        EntryPayload::Undo { undoes, inverse_request } => {
            merge_undo(map, undoes, inverse_request)
        }
        EntryPayload::Migrate { sources, created } => merge_migrate(map, sources, created),
        EntryPayload::SchemaUpgrade { from_version, to_version, changes } => {
            map.insert("from-version".into(), json!(from_version));
            map.insert("to-version".into(), json!(to_version));
            map.insert("changes".into(), Value::String(changes.clone()));
        }
        EntryPayload::Verify { feature, tcs_run, passing, failing, tag_created } => {
            merge_verify(map, feature, tcs_run, passing, failing, tag_created);
        }
    }
}

fn str_array(items: &[String]) -> Value {
    Value::Array(items.iter().map(|s| Value::String(s.clone())).collect())
}

fn ref_array(items: &[ArtifactRef]) -> Value {
    Value::Array(items.iter().map(|r| r.to_value()).collect())
}

fn merge_apply(
    map: &mut Map<String, Value>,
    request: &Value,
    created: &[ArtifactRef],
    changed: &[ArtifactRef],
    deleted: &[ArtifactRef],
) {
    if !request.is_null() {
        map.insert("request".into(), request.clone());
    }
    let mut result = Map::new();
    result.insert("created".into(), ref_array(created));
    result.insert("changed".into(), ref_array(changed));
    // FT-064 — only emit `deleted:` when non-empty so historical entries
    // continue to canonicalise to the same bytes (hash-chain stability).
    if !deleted.is_empty() {
        result.insert("deleted".into(), ref_array(deleted));
    }
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

pub(super) fn parse_payload(obj: &Map<String, Value>, entry_type: EntryType) -> EntryPayload {
    match entry_type {
        EntryType::Create
        | EntryType::Change
        | EntryType::CreateAndChange
        | EntryType::Delete => EntryPayload::Apply {
            request: obj.get("request").cloned().unwrap_or(Value::Null),
            created: result_ref_array(obj, "created"),
            changed: result_ref_array(obj, "changed"),
            deleted: result_ref_array(obj, "deleted"),
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

pub(super) fn str_field(obj: &Map<String, Value>, key: &str) -> String {
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

fn result_ref_array(obj: &Map<String, Value>, key: &str) -> Vec<ArtifactRef> {
    obj.get("result")
        .and_then(|v| v.as_object())
        .and_then(|r| r.get(key))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(ArtifactRef::parse_value).collect())
        .unwrap_or_default()
}
