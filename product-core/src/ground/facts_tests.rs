//! Facts: nullability, range resolution, containment lookup.

use super::*;

fn decl(name: &str, file: &str, start: u64, end: u64) -> Declaration {
    Declaration {
        name: name.into(),
        kind: DeclKind::Class,
        module: "Acme.Sql".into(),
        file: file.into(),
        start_line: start,
        end_line: end,
        doc: None,
        properties: Vec::new(),
    }
}

#[test]
fn a_nullability_marker_is_read_off_the_type_not_a_separate_field() {
    let nullable = PropertyDecl { name: "DisabledReason".into(), type_name: "int?".into() };
    assert!(nullable.nullable());
    assert_eq!(nullable.bare_type(), "int");
    let required = PropertyDecl { name: "Id".into(), type_name: "ChargingProfileId".into() };
    assert!(!required.nullable());
    assert_eq!(required.bare_type(), "ChargingProfileId");
}

#[test]
fn an_unmeasured_operation_is_not_evidence_that_it_answers() {
    let facts = CorpusFacts {
        corpus: "corpus-backend".into(),
        git_ref: "abc".into(),
        instrument: "test".into(),
        operations: vec![OperationStatus {
            operation: "documentSymbol".into(),
            answered: true,
            detail: "15 symbols".into(),
        }],
        declarations: Vec::new(),
        hierarchy: Vec::new(),
        references: Vec::new(),
        unread_files: Vec::new(),
    };
    assert!(facts.answered("documentSymbol"));
    // Never probed at all: absence of a record is not a pass.
    assert!(!facts.answered("typeHierarchy"));
}

/// The containment lookup picks the *innermost* declaration, so a reference
/// inside a nested type is attributed to that type rather than its outer one.
#[test]
fn containment_picks_the_innermost_declaration() {
    let facts = CorpusFacts {
        corpus: "c".into(),
        git_ref: "r".into(),
        instrument: "i".into(),
        operations: Vec::new(),
        declarations: vec![decl("Outer", "a.cs", 0, 40), decl("Inner", "a.cs", 10, 20)],
        hierarchy: Vec::new(),
        references: Vec::new(),
        unread_files: Vec::new(),
    };
    assert_eq!(facts.enclosing("a.cs", 15).map(|d| d.name.as_str()), Some("Inner"));
    assert_eq!(facts.enclosing("a.cs", 35).map(|d| d.name.as_str()), Some("Outer"));
    assert!(facts.enclosing("b.cs", 15).is_none());
    assert!(facts.enclosing("a.cs", 99).is_none());
}
