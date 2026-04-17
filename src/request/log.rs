//! Request audit log — `.product/request-log.jsonl` (FT-041, ADR-038).

use super::apply::{ChangedArtifact, CreatedArtifact};
use super::types::Request;
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Append one JSON line per successful apply to `.product/request-log.jsonl`.
pub fn append_log(
    repo_root: &Path,
    request: &Request,
    created: &[CreatedArtifact],
    changed: &[ChangedArtifact],
) -> std::io::Result<()> {
    let dir = repo_root.join(".product");
    fs::create_dir_all(&dir)?;
    let log_path = dir.join("request-log.jsonl");

    let mut hasher = Sha256::new();
    hasher.update(request.source_yaml.as_bytes());
    let request_hash = format!("{:x}", hasher.finalize());

    let entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "reason": request.reason,
        "request_hash": request_hash,
        "type": request.request_type.to_string(),
        "created": created,
        "changed": changed,
    });

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    writeln!(f, "{}", entry)?;
    Ok(())
}
