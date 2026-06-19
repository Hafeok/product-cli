//! `product guide` — onboarding adapter over `product_core::guide`.
//!
//! Probes the framework graph, then prints where the user is in the
//! What → How → Delivery journey plus the next move.

use product_core::guide::{self, FrameworkState};

use super::output::{CmdResult, Output};

pub(crate) fn handle_guide() -> CmdResult {
    let root = super::shared::domain_root();
    let product = super::shared::default_product_name()
        .unwrap_or_else(|| "your-product".to_string());
    let state = FrameworkState::probe(&root, &product);
    let guidance = guide::guide(&state);
    let text = guide::render_text(&guidance);
    let json = serde_json::to_value(&guidance).unwrap_or(serde_json::Value::Null);
    Ok(Output::both(text, json))
}
