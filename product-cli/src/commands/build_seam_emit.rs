//! §5.1 build-seam emission — turn a deliverable's frozen work units into the
//! canonical contract WorkUnit envelopes an executor admits. Split from `build`
//! to keep files within the length budget.

use std::path::{Path, PathBuf};

use product_core::pf::build_seam::ModelBinding;
use product_core::pf::work_unit::WorkUnit;

use super::BoxResult;

/// §5.1 — emit the deliverable's work units as canonical build-seam envelopes (a
/// JSON array), the contract's kebab-case WorkUnit shape: each travels by value
/// with its content-hash identity, a fully-pinned model binding, and a sealed
/// cell-graph, ready for any executor honouring the seam. The deliverable id is
/// the `parent-deliverable`.
pub(crate) fn emit_seam_envelopes(deliverable: &str, role: &str, units: &[WorkUnit], out: Option<PathBuf>) -> BoxResult {
    use product_core::pf::build_seam::{to_seam_envelope, AcceptanceClass, ArtifactDelivery, SeamParams};
    if units.is_empty() {
        return Err(format!("no work units to emit for '{deliverable}' — dispatch cells first (`product cell dispatch`)").into());
    }
    let (tier, capability_tag, binding) = resolve_seam_binding(role);
    let envelopes = units.iter()
        .map(|wu| {
            let params = SeamParams {
                acceptance_class: AcceptanceClass::NeedsVerdict,
                parent_deliverable: deliverable,
                tier: tier.clone(),
                binding: binding.clone(),
                capability_tag: capability_tag.clone(),
                ladder_position: 0,
                artifact_delivery: ArtifactDelivery::Inline,
            };
            to_seam_envelope(wu, &params)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let json = serde_json::to_string_pretty(&envelopes)?;
    if out.as_deref().map(Path::as_os_str) == Some(std::ffi::OsStr::new("-")) {
        println!("{json}");
        return Ok(());
    }
    let path = out.unwrap_or_else(|| {
        super::shared::domain_root().join(".product").join("build").join(format!("{deliverable}.seam.json"))
    });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &json)?;
    println!("Wrote {} build-seam work unit(s) → {}", envelopes.len(), path.display());
    Ok(())
}

/// Resolve the fully-pinned model binding a work unit is emitted against
/// (contract SPMC-M, RFC 0002 axis precision): a served binding from
/// `.product/role-bindings.yaml` when present, else a reported placeholder so the
/// emitted unit stays canonically valid.
fn resolve_seam_binding(role: &str) -> (String, Option<String>, ModelBinding) {
    binding_from_config(role).unwrap_or_else(|| {
        eprintln!(
            "note: no binding for role '{role}' in .product/role-bindings.yaml — emitting a placeholder \
             binding (provider=unpinned). Pin a served binding there before dispatching to an executor."
        );
        placeholder_binding(role)
    })
}

/// A schema-valid placeholder binding for a role with no configured entry.
fn placeholder_binding(role: &str) -> (String, Option<String>, ModelBinding) {
    (
        role.to_string(),
        Some(role.to_string()),
        ModelBinding {
            provider: "unpinned".into(),
            model_id: role.to_string(),
            revision: None,
            architecture: None,
            quantization: "unpinned".into(),
            invocation: serde_json::json!({ "temperature": 0 }),
        },
    )
}

/// Read a role's served binding from `.product/role-bindings.yaml` — a map of
/// `role → { tier, capability-tag, provider, model-id, quantization, invocation }`.
/// `None` when the file, the entry, or a required field (provider / model-id /
/// quantization) is absent.
fn binding_from_config(role: &str) -> Option<(String, Option<String>, ModelBinding)> {
    let path = super::shared::domain_root().join(".product").join("role-bindings.yaml");
    let text = std::fs::read_to_string(&path).ok()?;
    let doc = serde_yaml::from_str::<serde_json::Value>(&text).ok()?;
    let entry = doc.get(role)?;
    let field = |k: &str| entry.get(k).and_then(|v| v.as_str()).map(str::to_string);
    let (provider, model_id, quantization) = (field("provider")?, field("model-id")?, field("quantization")?);
    let tier = field("tier").unwrap_or_else(|| role.to_string());
    let capability_tag = field("capability-tag").or_else(|| Some(role.to_string()));
    let invocation = entry.get("invocation").cloned().unwrap_or_else(|| serde_json::json!({ "temperature": 0 }));
    Some((
        tier,
        capability_tag,
        ModelBinding { provider, model_id, revision: field("revision"), architecture: field("architecture"), quantization, invocation },
    ))
}
