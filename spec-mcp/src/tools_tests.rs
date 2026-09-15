use super::*;

#[test]
fn the_withheld_verbs_are_not_in_the_registry() {
    let tools = build();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    for withheld in WITHHELD {
        assert!(
            !names.contains(withheld),
            "`{withheld}` names a principal; an MCP surface cannot be one"
        );
    }
}

#[test]
fn the_delegable_verbs_are_all_present() {
    let tools = build();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    for expected in
        ["spec_candidates", "spec_map", "spec_check", "spec_records", "spec_policy_show", "spec_implement"]
    {
        assert!(names.contains(&expected), "`{expected}` is delegable and should be offered");
    }
}

#[test]
fn only_implement_writes() {
    let tools = build();
    let writers: Vec<&str> = tools
        .iter()
        .filter(|t| t.requires_write)
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(writers, ["spec_implement"]);
}

#[test]
fn implement_says_in_its_description_that_it_cannot_close() {
    let tools = build();
    let implement = tools.iter().find(|t| t.name == "spec_implement").expect("present");
    assert!(implement.description.contains("PENDING"), "{}", implement.description);
    assert!(implement.description.contains("principal"), "{}", implement.description);
}
