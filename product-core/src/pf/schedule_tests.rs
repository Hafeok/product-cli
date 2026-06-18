//! Tests for dependency-ordered work-unit layering.

use super::*;
use crate::pf::work_unit::{Context, Produces, WorkUnit};

fn unit(id: &str, derived_from: &[&str]) -> WorkUnit {
    WorkUnit {
        id: id.to_string(),
        schema: String::new(),
        prompt: String::new(),
        model: None,
        context: Context {
            derived_from: derived_from.iter().map(|s| s.to_string()).collect(),
            frozen: true,
            hash: None,
        },
        produces: Produces { artifact: String::new(), path_hint: None },
        applies: Vec::new(),
        trace: None,
    }
}

#[test]
fn implement_layers_after_the_test_it_derives_from() {
    // implement derives_from the write-test cell id → must run in a later layer.
    let units = vec![
        unit("implement-spec-depth", &["write-test"]),
        unit("write-test-spec-depth", &["domain:e-verification"]),
    ];
    let ls = layers(&units);
    assert_eq!(ls.len(), 2, "two dependency layers");
    assert_eq!(ls[0], vec![1], "write-test runs first");
    assert_eq!(ls[1], vec![0], "implement runs after");
}

#[test]
fn independent_units_share_one_layer() {
    let units = vec![unit("a", &["domain:x"]), unit("b", &["domain:y"])];
    let ls = layers(&units);
    assert_eq!(ls.len(), 1);
    assert_eq!(ls[0], vec![0, 1]);
}

#[test]
fn a_cycle_is_released_not_deadlocked() {
    let units = vec![unit("a", &["b"]), unit("b", &["a"])];
    let ls = layers(&units);
    // both can never become "ready"; they release together in one final layer.
    assert_eq!(ls.len(), 1);
    assert_eq!(ls[0], vec![0, 1]);
}
