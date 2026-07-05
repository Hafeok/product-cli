//! Reify provenance — the generated tree's machine-readable identity.
//!
//! `provenance.g.json` records what a reify run produced and the exact inputs
//! it was produced from: the pinned graph hash, the What-version, and — when a
//! design system is bound — its id + manifest hash. `reify check` reads these
//! back as the drift gate.

use serde_json::json;

use crate::error::{ProductError, Result};

use super::reify::{GenFile, ReifyOptions};

/// The machine-readable provenance manifest (`provenance.g.json`).
pub fn provenance_json(opts: &ReifyOptions, graph_hash: &str, files: &[GenFile]) -> String {
    let generated: Vec<&str> =
        files.iter().filter(|f| f.overwrite).map(|f| f.path.as_str()).collect();
    let mut v = json!({
        "product": opts.product,
        "namespace": opts.namespace,
        "what_version": opts.what_version,
        "graph_hash": format!("sha256:{graph_hash}"),
        "generator": "product reify csharp",
        "generated_files": generated,
    });
    if let (Some(spec), Some(obj)) = (&opts.design_system, v.as_object_mut()) {
        let ds = &spec.manifest.design_system;
        obj.insert("design_system".to_string(), json!({
            "id": ds.id, "version": ds.version, "hash": format!("sha256:{}", spec.hash),
        }));
    }
    let mut s = serde_json::to_string_pretty(&v).unwrap_or_default();
    s.push('\n');
    s
}

/// Extract the recorded graph hash from a `provenance.g.json` text.
pub fn recorded_hash(provenance: &str) -> Result<String> {
    let v: serde_json::Value = serde_json::from_str(provenance)
        .map_err(|e| ProductError::ConfigError(format!("invalid provenance.g.json: {e}")))?;
    v.get("graph_hash")
        .and_then(|h| h.as_str())
        .map(|h| h.trim_start_matches("sha256:").to_string())
        .ok_or_else(|| ProductError::ConfigError("provenance.g.json carries no graph_hash".to_string()))
}

