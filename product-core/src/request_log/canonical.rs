//! Deterministic canonical JSON (FT-042, ADR-039 decision 3).
//!
//! - Keys sorted alphabetically at every nesting level
//! - No whitespace, no pretty-printing
//! - UTF-8, no BOM, standard JSON escaping
//! - Numbers emitted via serde_json default (shortest round-tripping form)
//! - `null`, `true`, `false` as lowercase literals
//!
//! The same input value always produces the same byte sequence, and the same
//! byte sequence always produces the same SHA-256 hash. This is the cornerstone
//! of the hash chain — if canonical JSON is not deterministic, the chain is
//! meaningless.

use serde_json::Value;

/// Canonical JSON byte serialisation.
///
/// The returned string is valid JSON, byte-for-byte deterministic for the same
/// logical value, and parseable by any conforming JSON library.
pub fn canonical_json(value: &Value) -> String {
    let mut buf = String::new();
    write_value(&mut buf, value);
    buf
}

fn write_value(buf: &mut String, value: &Value) {
    match value {
        Value::Null => buf.push_str("null"),
        Value::Bool(true) => buf.push_str("true"),
        Value::Bool(false) => buf.push_str("false"),
        Value::Number(n) => {
            // serde_json preserves integer/float distinction; render without
            // whitespace and without scientific notation when possible.
            buf.push_str(&n.to_string());
        }
        Value::String(s) => write_string(buf, s),
        Value::Array(items) => {
            buf.push('[');
            let mut first = true;
            for item in items {
                if !first {
                    buf.push(',');
                }
                first = false;
                write_value(buf, item);
            }
            buf.push(']');
        }
        Value::Object(map) => {
            // Sort keys alphabetically (byte-wise — stable across locales).
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            buf.push('{');
            let mut first = true;
            for k in keys {
                if !first {
                    buf.push(',');
                }
                first = false;
                write_string(buf, k);
                buf.push(':');
                if let Some(v) = map.get(k) {
                    write_value(buf, v);
                }
            }
            buf.push('}');
        }
    }
}

fn write_string(buf: &mut String, s: &str) {
    buf.push('"');
    for c in s.chars() {
        match c {
            '"' => buf.push_str("\\\""),
            '\\' => buf.push_str("\\\\"),
            '\n' => buf.push_str("\\n"),
            '\r' => buf.push_str("\\r"),
            '\t' => buf.push_str("\\t"),
            '\x08' => buf.push_str("\\b"),
            '\x0c' => buf.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                buf.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => buf.push(c),
        }
    }
    buf.push('"');
}

/// Compute the SHA-256 of the canonical serialisation, hex-encoded (lowercase).
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deterministic_key_order() {
        let a = json!({"b": 1, "a": 2});
        let b = json!({"a": 2, "b": 1});
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(canonical_json(&a), "{\"a\":2,\"b\":1}");
    }

    #[test]
    fn nested_key_order() {
        let v = json!({"z": {"b": 1, "a": 2}, "a": {"d": 4, "c": 3}});
        assert_eq!(
            canonical_json(&v),
            "{\"a\":{\"c\":3,\"d\":4},\"z\":{\"a\":2,\"b\":1}}"
        );
    }

    #[test]
    fn arrays_preserve_order() {
        let v = json!([3, 1, 2]);
        assert_eq!(canonical_json(&v), "[3,1,2]");
    }

    #[test]
    fn strings_are_escaped() {
        let v = json!("line1\nline2\t\"quoted\"");
        assert_eq!(canonical_json(&v), "\"line1\\nline2\\t\\\"quoted\\\"\"");
    }

    #[test]
    fn null_and_booleans() {
        assert_eq!(canonical_json(&json!(null)), "null");
        assert_eq!(canonical_json(&json!(true)), "true");
        assert_eq!(canonical_json(&json!(false)), "false");
    }

    #[test]
    fn integer_numbers() {
        assert_eq!(canonical_json(&json!(42)), "42");
        assert_eq!(canonical_json(&json!(-7)), "-7");
        assert_eq!(canonical_json(&json!(0)), "0");
    }

    #[test]
    fn same_input_same_hash() {
        let a = json!({"x": 1, "y": 2});
        let b = json!({"y": 2, "x": 1});
        assert_eq!(sha256_hex(canonical_json(&a).as_bytes()),
                   sha256_hex(canonical_json(&b).as_bytes()));
    }

    /// Pinning test — exact byte sequence for a known entry. Catches silent
    /// behaviour changes in the JSON library across dependency upgrades.
    #[test]
    fn pinning_fixed_fixture() {
        let v = json!({
            "type": "create",
            "id": "req-20260417-001",
            "applied-by": "git:Alice <a@example.com>",
            "applied-at": "2026-04-17T12:00:00Z",
            "commit": "abc123",
            "reason": "hello",
            "prev-hash": "0000000000000000",
            "entry-hash": "",
            "payload": {"result": {"created": ["FT-001"]}}
        });
        let expected = concat!(
            "{",
            "\"applied-at\":\"2026-04-17T12:00:00Z\",",
            "\"applied-by\":\"git:Alice <a@example.com>\",",
            "\"commit\":\"abc123\",",
            "\"entry-hash\":\"\",",
            "\"id\":\"req-20260417-001\",",
            "\"payload\":{\"result\":{\"created\":[\"FT-001\"]}},",
            "\"prev-hash\":\"0000000000000000\",",
            "\"reason\":\"hello\",",
            "\"type\":\"create\"",
            "}"
        );
        assert_eq!(canonical_json(&v), expected);
    }
}
