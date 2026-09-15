use super::*;

fn identity(s: &str) -> Identity {
    s.parse().expect("test identity parses")
}

fn closure(kind: ClosureKind, dets: &[&str]) -> Closure {
    Closure {
        kind,
        principal: identity("emil@example.com"),
        at: Utc::now(),
        determinations: dets.iter().map(|s| (*s).to_string()).collect(),
        binds: "sha256:00".into(),
        signature: None,
    }
}

#[test]
fn determinations_needs_a_non_empty_list() {
    assert!(closure(ClosureKind::Determinations, &["det/a"]).kind_matches_payload());
    assert!(!closure(ClosureKind::Determinations, &[]).kind_matches_payload());
}

#[test]
fn nothing_arose_needs_an_empty_list() {
    assert!(closure(ClosureKind::NothingArose, &[]).kind_matches_payload());
    assert!(!closure(ClosureKind::NothingArose, &["det/a"]).kind_matches_payload());
}

#[test]
fn kinds_serialise_in_the_on_disk_spelling() {
    let yaml = serde_yaml::to_string(&ClosureKind::NothingArose).expect("serialise");
    assert_eq!(yaml.trim(), "nothing-arose");
    assert_eq!(ClosureKind::NothingArose.as_str(), "nothing-arose");
}
