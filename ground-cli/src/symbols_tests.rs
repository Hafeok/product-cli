//! Symbol-tree reading: declarations, members, anchors, generics.

use super::*;

fn sym(name: &str, kind: u64, container: &str, line: u64, ch: u64, end: u64) -> RawSymbol {
    RawSymbol {
        name: name.into(),
        kind,
        container: container.into(),
        container_kind: 0,
        start_line: line,
        end_line: end,
        sel_line: line,
        sel_char: ch,
        detail: None,
    }
}

#[test]
fn a_typed_member_is_read_off_the_rendered_symbol_name() {
    let m = typed_member("Id : ProfileId").expect("member");
    assert_eq!((m.name.as_str(), m.type_name.as_str()), ("Id", "ProfileId"));
    let nullable = typed_member("DisabledReason : int?").expect("member");
    assert_eq!(nullable.type_name, "int?");
}

/// A member the server rendered without a type is skipped: a property with a
/// guessed type is a claim nobody made.
#[test]
fn an_untyped_member_is_skipped_rather_than_guessed_at() {
    assert!(typed_member("Ping()").is_none());
    assert!(typed_member("Id :").is_none());
    assert!(typed_member(": ProfileId").is_none());
}

#[test]
fn generics_are_stripped_from_a_declared_name() {
    assert_eq!(split_generics("ICommandHandler<TCommand>"), ("ICommandHandler".into(), Some("TCommand".into())));
    assert_eq!(split_generics("Settings"), ("Settings".into(), None));
}

/// The anchor is the declaration's own name position, which the position
/// operations need; the declaration keeps only the span, which is what a
/// citation and the containment join use.
#[test]
fn the_anchor_carries_the_name_position_the_declaration_does_not() {
    let symbols = vec![
        sym("Acme.Sql.Entities", 3, "", 2, 10, 2),
        sym("Settings", 5, "Acme.Sql.Entities", 9, 22, 30),
        sym("Id : ProfileId", 7, "Acme.Sql.Entities/Settings", 12, 30, 12),
    ];
    let reads = declarations_in("src/Entities/Settings.cs", &symbols);
    assert_eq!(reads.len(), 1);
    let read = &reads[0];
    assert_eq!(read.anchor.character, 22);
    assert_eq!((read.declaration.start_line, read.declaration.end_line), (9, 30));
    assert_eq!(read.declaration.module, "Acme.Sql.Entities");
    assert_eq!(read.declaration.properties.len(), 1);
    assert_eq!(read.declaration.properties[0].type_name, "ProfileId");
}

/// Members are attributed to the type that directly contains them, so a
/// second type's property never lands on the first.
#[test]
fn members_attach_only_to_their_own_type() {
    let symbols = vec![
        sym("A", 5, "N", 0, 6, 10),
        sym("B", 5, "N", 11, 6, 20),
        sym("X : int", 7, "N/A", 2, 8, 2),
        sym("Y : string", 7, "N/B", 13, 8, 13),
    ];
    let reads = declarations_in("f.cs", &symbols);
    let a = reads.iter().find(|r| r.declaration.name == "A").expect("A");
    let b = reads.iter().find(|r| r.declaration.name == "B").expect("B");
    assert_eq!(a.declaration.properties.len(), 1);
    assert_eq!(a.declaration.properties[0].name, "X");
    assert_eq!(b.declaration.properties[0].name, "Y");
}

/// A nested type's module is its enclosing **namespace**, never its outer
/// type: reading the innermost container proposed a module axis named after
/// a class.
#[test]
fn a_nested_types_module_is_its_namespace_not_its_outer_type() {
    let mut nested = sym("Inner", 5, "Acme.Serialization/Converter", 12, 18, 20);
    nested.container_kind = 5;
    let mut top = sym("Converter", 5, "Acme.Serialization", 4, 14, 30);
    top.container_kind = 3;
    let reads = declarations_in("f.cs", &[top, nested]);
    for read in &reads {
        assert_eq!(read.declaration.module, "Acme.Serialization", "{}", read.declaration.name);
    }
    let inner = reads.iter().find(|r| r.declaration.name == "Inner").expect("Inner");
    assert!(inner.nested);
    let outer = reads.iter().find(|r| r.declaration.name == "Converter").expect("Converter");
    assert!(!outer.nested);
}

/// Fields and properties both carry types and both reach the row; the count
/// is kept so the indistinguishability is reported rather than absorbed.
#[test]
fn field_members_are_counted_separately_from_properties() {
    let symbols = vec![
        sym("Converter", 5, "N", 0, 6, 20),
        sym("Value : int", 7, "N/Converter", 2, 8, 2),
        sym("cache : Dictionary", 8, "N/Converter", 3, 8, 3),
        sym("lockObject : object", 8, "N/Converter", 4, 8, 4),
    ];
    let reads = declarations_in("f.cs", &symbols);
    assert_eq!(reads[0].declaration.properties.len(), 3, "all three reach the row");
    assert_eq!(reads[0].field_members, 2, "two of them are fields");
}

#[test]
fn only_type_kinds_become_declarations() {
    let symbols = vec![sym("Ping", 6, "N/A", 0, 4, 1), sym("N", 3, "", 0, 0, 9)];
    assert!(declarations_in("f.cs", &symbols).is_empty());
}
