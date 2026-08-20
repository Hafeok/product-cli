//! Facts: nullability, range resolution, containment lookup.

use super::*;

#[test]
fn a_nullability_marker_is_read_off_the_type_not_a_separate_field() {
    let nullable = PropertyDecl { name: "DisabledReason".into(), type_name: "int?".into(), kind: Default::default() };
    assert!(nullable.nullable());
    assert_eq!(nullable.bare_type(), "int");
    let required = PropertyDecl { name: "Id".into(), type_name: "ChargingProfileId".into(), kind: Default::default() };
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
            advertised: Some(true),
            divergence: None,
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
