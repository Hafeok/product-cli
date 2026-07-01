//! Unit tests for the guide stage-decision logic.

use super::*;

fn state() -> FrameworkState {
    FrameworkState { product: "bookstore".into(), ..Default::default() }
}

#[test]
fn empty_graph_says_capture_what() {
    let g = guide(&state());
    assert_eq!(g.stage, Stage::CaptureWhat);
    assert!(g.next_steps[0].command.contains("author domain bookstore"));
    assert!(g.progress.iter().all(|(_, done)| !done));
}

#[test]
fn nonconformant_what_says_fix_first() {
    let s = FrameworkState { what_total: 3, violations: 2, ..state() };
    let g = guide(&s);
    assert_eq!(g.stage, Stage::FixWhat);
    assert!(g.headline.contains('2'));
    assert!(g.next_steps[0].command.contains("domain validate"));
}

#[test]
fn conformant_what_without_how_says_author_how() {
    let s = FrameworkState { what_total: 6, violations: 0, has_how: false, ..state() };
    let g = guide(&s);
    assert_eq!(g.stage, Stage::AuthorHow);
    assert!(g.next_steps[0].command.contains("how init"));
}

#[test]
fn how_without_feature_says_carve_feature() {
    let s = FrameworkState { what_total: 6, has_how: true, features: 0, ..state() };
    let g = guide(&s);
    assert_eq!(g.stage, Stage::CarveFeature);
    assert!(g.next_steps[0].command.contains("feature new"));
    assert!(g.next_steps[0].command.contains("--anchor"));
}

#[test]
fn carve_feature_names_a_real_command_when_known() {
    let s = FrameworkState {
        what_total: 6, has_how: true, features: 0,
        first_command: Some("PlaceOrder".into()), ..state()
    };
    let g = guide(&s);
    assert!(g.next_steps[0].command.contains("--anchor PlaceOrder"),
        "should name the real command, got: {}", g.next_steps[0].command);
}

#[test]
fn feature_without_deliverable_says_wrap() {
    let s = FrameworkState { what_total: 6, has_how: true, features: 1, deliverables: 0, ..state() };
    let g = guide(&s);
    assert_eq!(g.stage, Stage::WrapDeliverable);
    assert!(g.next_steps[0].command.contains("deliverable new"));
}

#[test]
fn wrap_deliverable_names_a_real_feature_when_known() {
    let s = FrameworkState {
        what_total: 6, has_how: true, features: 1, deliverables: 0,
        first_feature: Some("checkout".into()), ..state()
    };
    let g = guide(&s);
    assert!(g.next_steps[0].command.contains("--feature checkout"),
        "should name the real feature, got: {}", g.next_steps[0].command);
}

#[test]
fn deliverable_present_says_build() {
    let s = FrameworkState { what_total: 6, has_how: true, features: 1, deliverables: 1, ..state() };
    let g = guide(&s);
    assert_eq!(g.stage, Stage::BuildIt);
    assert!(g.next_steps.iter().any(|n| n.command.contains("build")));
}

#[test]
fn render_includes_checklist_and_next() {
    let text = render_text(&guide(&state()));
    assert!(text.contains("Your framework journey"));
    assert!(text.contains("[ ] Captured a What model"));
    assert!(text.contains("Next:"));
}
