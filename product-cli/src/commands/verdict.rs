//! Build-seam verdict validation (§5.1) — the inbound half of the seam.
//!
//! `product verdict <file>` checks that a verdict event a executor emitted
//! conforms to the contract: required fields, the pinned §6.2 verdict vocabulary,
//! and nothing outside the envelope but the one opaque `executor_extension` slot.

use std::path::PathBuf;

use super::BoxResult;

pub(crate) fn handle_verdict(file: PathBuf) -> BoxResult {
    let text = std::fs::read_to_string(&file)
        .map_err(|e| format!("cannot read verdict file {}: {e}", file.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("verdict file is not valid JSON: {e}"))?;
    let ev = product_core::pf::build_seam::validate_verdict(&value)?;
    println!(
        "valid verdict event '{}' — unit '{}' ran against bundle {} → {}",
        ev.event_id,
        ev.unit_ref,
        ev.bundle_hash,
        serde_json::to_value(ev.verdict)?.as_str().unwrap_or("?"),
    );
    Ok(())
}
