//! Mutation execution (set / append / remove / delete with dot-notation).

use super::super::types::*;
use super::assign::resolve_refs;
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;

pub fn apply_mutation(
    target: &mut Mapping,
    m: &Mutation,
    ref_to_id: &HashMap<String, String>,
) -> Result<(), String> {
    let path: Vec<&str> = m.field.split('.').collect();
    let resolved_value = m.value.as_ref().map(|v| resolve_refs(v, ref_to_id));

    match m.op {
        MutationOp::Set => {
            let v = resolved_value.ok_or_else(|| "set requires a value".to_string())?;
            set_path(target, &path, v);
            Ok(())
        }
        MutationOp::Append => {
            let v = resolved_value.ok_or_else(|| "append requires a value".to_string())?;
            append_path(target, &path, v)?;
            Ok(())
        }
        MutationOp::Remove => {
            let v = resolved_value.ok_or_else(|| "remove requires a value".to_string())?;
            remove_path(target, &path, &v);
            Ok(())
        }
        MutationOp::Delete => {
            delete_path(target, &path);
            Ok(())
        }
    }
}

fn set_path(m: &mut Mapping, path: &[&str], v: Value) {
    if path.is_empty() {
        return;
    }
    if path.len() == 1 {
        m.insert(Value::String(path[0].to_string()), v);
        return;
    }
    let head = Value::String(path[0].to_string());
    let mut inner = match m.get(&head) {
        Some(Value::Mapping(inner)) => inner.clone(),
        _ => Mapping::new(),
    };
    set_path(&mut inner, &path[1..], v);
    m.insert(head, Value::Mapping(inner));
}

fn append_path(m: &mut Mapping, path: &[&str], v: Value) -> Result<(), String> {
    if path.is_empty() {
        return Err("empty path".to_string());
    }
    if path.len() == 1 {
        let k = Value::String(path[0].to_string());
        let current = m.get(&k).cloned().unwrap_or(Value::Sequence(Vec::new()));
        let mut seq = match current {
            Value::Sequence(s) => s,
            Value::Null => Vec::new(),
            _ => return Err(format!("field '{}' is not a list", path[0])),
        };
        if !seq.contains(&v) {
            seq.push(v);
        }
        m.insert(k, Value::Sequence(seq));
        return Ok(());
    }
    let head = Value::String(path[0].to_string());
    let mut inner = match m.get(&head) {
        Some(Value::Mapping(inner)) => inner.clone(),
        _ => Mapping::new(),
    };
    append_path(&mut inner, &path[1..], v)?;
    m.insert(head, Value::Mapping(inner));
    Ok(())
}

fn remove_path(m: &mut Mapping, path: &[&str], v: &Value) {
    if path.is_empty() {
        return;
    }
    if path.len() == 1 {
        let k = Value::String(path[0].to_string());
        if let Some(Value::Sequence(seq)) = m.get(&k).cloned() {
            let filtered: Vec<Value> = seq.into_iter().filter(|i| i != v).collect();
            m.insert(k, Value::Sequence(filtered));
        }
        return;
    }
    let head = Value::String(path[0].to_string());
    if let Some(Value::Mapping(inner)) = m.get(&head).cloned() {
        let mut inner = inner;
        remove_path(&mut inner, &path[1..], v);
        m.insert(head, Value::Mapping(inner));
    }
}

fn delete_path(m: &mut Mapping, path: &[&str]) {
    if path.is_empty() {
        return;
    }
    if path.len() == 1 {
        m.remove(Value::String(path[0].to_string()));
        return;
    }
    let head = Value::String(path[0].to_string());
    if let Some(Value::Mapping(inner)) = m.get(&head).cloned() {
        let mut inner = inner;
        delete_path(&mut inner, &path[1..]);
        m.insert(head, Value::Mapping(inner));
    }
}
