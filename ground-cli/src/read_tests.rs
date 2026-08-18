//! The reader's pure parts: relative paths, duplicate-name detection.

use super::*;
use product_core::ground::facts::{DeclKind, Declaration};

fn decl(name: &str, file: &str) -> Declaration {
    module_decl(name, file, "N")
}

fn module_decl(name: &str, file: &str, module: &str) -> Declaration {
    Declaration {
        name: name.into(),
        kind: DeclKind::Class,
        module: module.into(),
        file: file.into(),
        start_line: 0,
        end_line: 1,
        doc: None,
        properties: Vec::new(),
    }
}

fn facts_with(declarations: Vec<Declaration>) -> CorpusFacts {
    CorpusFacts {
        corpus: "corpus-backend".into(),
        git_ref: "r".into(),
        instrument: "i".into(),
        operations: Vec::new(),
        declarations,
        hierarchy: Vec::new(),
        references: Vec::new(),
        unread_files: Vec::new(),
    }
}

#[test]
fn citations_are_repo_relative_so_no_machine_layout_leaks() {
    let root = Path::new("/home/runner/corpus");
    assert_eq!(relative(root, Path::new("/home/runner/corpus/src/A.cs")), "src/A.cs");
    // A path outside the root stays whole rather than being silently rebased.
    assert_eq!(relative(root, Path::new("/elsewhere/B.cs")), "/elsewhere/B.cs");
}

/// A name repeated inside one module is C#'s `partial` — one type across
/// files. Reporting it as a conflation hazard would say the wrong thing.
#[test]
fn a_partial_type_is_one_type_not_a_collision() {
    let facts = facts_with(vec![
        module_decl("InitialCreate", "M/x.cs", "Acme.Sql.Migrations"),
        module_decl("InitialCreate", "M/x.Designer.cs", "Acme.Sql.Migrations"),
        decl("Settings", "s.cs"),
    ]);
    let (partial, collisions) = repeated_names(&facts);
    assert_eq!(partial, ["InitialCreate"]);
    assert!(collisions.is_empty(), "{collisions:?}");
}

/// The same simple name in two modules is two types, and range resolution
/// matches on the simple name — so this one is the real hazard.
#[test]
fn the_same_name_in_two_modules_is_a_collision() {
    let facts = facts_with(vec![
        module_decl("Location", "a.cs", "Acme.Sql.Entities"),
        module_decl("Location", "b.cs", "Acme.Dto"),
    ]);
    let (partial, collisions) = repeated_names(&facts);
    assert!(partial.is_empty(), "{partial:?}");
    assert_eq!(collisions, ["Location"]);
}

#[test]
fn a_name_declared_once_is_neither() {
    let facts = facts_with(vec![decl("A", "a.cs"), decl("B", "b.cs")]);
    assert_eq!(repeated_names(&facts), (Vec::new(), Vec::new()));
}
