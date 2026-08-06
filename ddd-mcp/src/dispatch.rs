//! Tool-name routing for the `ddd_*` namespace.

use serde_json::Value;

use crate::state::ServeState;
use crate::{govern_tools as gov, intercept, lang_tools as lang};

/// Route one call; `None` means the name is not ours.
pub fn dispatch(state: &ServeState, name: &str, args: &Value) -> Option<Result<Value, String>> {
    Some(match name {
        "ddd_find_symbol" => lang::find_symbol(state, args),
        "ddd_references" => lang::references(state, args),
        "ddd_hover" => lang::hover(state, args),
        "ddd_diagnostics" => lang::diagnostics(state, args),
        "ddd_signature" => lang::signature(state, args),
        "ddd_rename" => lang::rename(state, args),
        "ddd_apply_edit" => intercept::apply_edit(state, args),
        "ddd_warmup" => lang::warmup(state, args),
        "ddd_why" => gov::why(state, args),
        "ddd_graph_query" => gov::graph_query(state, args),
        "ddd_declare_seam" => gov::declare_seam(state, args),
        "ddd_declare_pattern" => gov::declare_pattern(state, args),
        "ddd_accept_risk" => gov::accept_risk(state, args),
        _ => return None,
    })
}
