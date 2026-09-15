use super::*;
use crate::check::Class;

fn repo() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn ratification(principal: &str) -> Ratification {
    Ratification {
        act_id: "act/settle-basket".into(),
        name: "Settle a basket".into(),
        settles: "What the customer owes when the basket closes.".into(),
        principal: principal.parse().expect("identity parses"),
        at: Utc::now(),
        realised_at: vec!["ep/one".into()],
        from_candidate: Some("cand/x".into()),
    }
}

fn refusal(principal: &str) -> Refusal {
    Refusal {
        candidate: "cand/health".into(),
        reason: "A health probe is not an act.".into(),
        principal: principal.parse().expect("identity parses"),
        at: Utc::now(),
    }
}

#[test]
fn a_principal_ratifies_and_it_reads_back() {
    let dir = repo();
    let Ratified::Filed(act) = accept(dir.path(), ratification("emil@example.com"), None).expect("accept")
    else {
        panic!("a well-formed ratification files");
    };
    assert_eq!(act.name, "Settle a basket");

    let loaded = load_acts(dir.path()).expect("load");
    assert_eq!(loaded, vec![*act]);
}

#[test]
fn a_machine_cannot_ratify() {
    let dir = repo();
    let Ratified::Refused(findings) = accept(dir.path(), ratification("ci@example.com"), None)
        .expect("the gate answers rather than erroring")
    else {
        panic!("a machine must not ratify");
    };
    assert_eq!(findings.first().map(|f| f.class), Some(Class::S002));
    assert!(load_acts(dir.path()).expect("load").is_empty(), "nothing must have landed");
}

#[test]
fn ratifying_twice_refuses_rather_than_overwriting() {
    let dir = repo();
    accept(dir.path(), ratification("emil@example.com"), None).expect("first");
    let err = accept(dir.path(), ratification("emil@example.com"), None).expect_err("second");
    assert!(err.to_string().contains("already ratified"), "{err}");
}

#[test]
fn a_principal_refuses_and_the_reason_is_filed() {
    let dir = repo();
    let Refused::Filed(rejection) = reject(dir.path(), refusal("emil@example.com"), None).expect("reject")
    else {
        panic!("a well-formed refusal files");
    };
    assert_eq!(rejection.reason, "A health probe is not an act.");
    assert_eq!(load_rejections(dir.path()).expect("load").len(), 1);
}

#[test]
fn a_machine_cannot_refuse() {
    let dir = repo();
    let Refused::Blocked(findings) = reject(dir.path(), refusal("dependabot@example.com"), None)
        .expect("the gate answers rather than erroring")
    else {
        panic!("a machine must not refuse");
    };
    assert_eq!(findings.first().map(|f| f.class), Some(Class::S002));
    assert!(load_rejections(dir.path()).expect("load").is_empty());
}

#[test]
fn refusing_twice_refuses_rather_than_overwriting() {
    let dir = repo();
    reject(dir.path(), refusal("emil@example.com"), None).expect("first");
    let err = reject(dir.path(), refusal("emil@example.com"), None).expect_err("second");
    assert!(err.to_string().contains("already refused"), "{err}");
}

#[test]
fn an_empty_store_loads_empty() {
    let dir = repo();
    assert!(load_acts(dir.path()).expect("acts").is_empty());
    assert!(load_rejections(dir.path()).expect("rejections").is_empty());
}

#[test]
fn an_address_becomes_one_filesystem_safe_name() {
    assert_eq!(slug("act/settle-basket"), "act-settle-basket");
    assert_eq!(slug("cand/Shop.Api#HttpPost"), "cand-shop-api-httppost");
}
