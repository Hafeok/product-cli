//! Live projection of the §5 Build-seam work units into the shape the Build view
//! consumes. The graph's `WorkUnit` (an SPMC bundle) has no recorded verdict, so
//! the verdict half is a `pending` placeholder until a build has run.

use std::path::Path;

use product_core::pf::work_unit::WorkUnit;
use serde_json::{json, Value};

use super::load_all;

pub fn project_work_units(base: &Path) -> Value {
    let units = load_all(base, "work-units", WorkUnit::from_yaml);
    let v: Vec<Value> = units
        .iter()
        .map(|w| {
            json!({
                "id": w.id,
                "lineage": w.applies.first().cloned().unwrap_or_default(),
                "hash": w.context.hash.clone().unwrap_or_else(|| "—".to_string()),
                "status": "n/a",
                "bundle": {
                    "schema": {
                        "artifact": w.produces.artifact,
                        "criteria": [format!("produces {}", w.produces.path)],
                    },
                    "prompt": w.prompt,
                    "model": w.model.clone().unwrap_or_else(|| "code-implementation".to_string()),
                    "context": w.context.derived_from,
                },
                "verdict": { "event": "—", "at": "not yet built", "verdict": "n/a", "consequence": "", "findings": [] },
            })
        })
        .collect();
    Value::Array(v)
}
