//! The declaration-level LSP subset against the mock host: the two newly
//! wired operations, plus the capability probe that gates every row.
//!
//! Hermetic — the workspace CI has no .NET SDK (`dec/ddd/fixtures-not-sdk`).
//! Live-host confirmation is the `DDD_LSP_E2E=1` run.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ddd_lsp::capability::{self, Divergence};
use ddd_lsp::declaration::{self, Operation};
use ddd_lsp::host::Host;

fn mock_command(extra: &[&str]) -> Vec<String> {
    let mut cmd = vec![env!("CARGO_BIN_EXE_ddd-mock-lsp").to_string()];
    cmd.extend(extra.iter().map(|s| s.to_string()));
    cmd
}

fn write(dir: &Path, rel: &str, content: &str) -> PathBuf {
    let path = dir.join(rel);
    std::fs::write(&path, content).expect("write fixture");
    path
}

/// A fixture in the corpus's own idioms: a marker interface, records
/// implementing it, and a value-object identity type — the shapes §5.2's
/// `rdfs:subClassOf` row fires on.
const MESSAGES: &str = "\
public interface ICommand
{
}
public record UpsertProfileSettingsCommand(string Value) : ICommand;
public record DisableProfileCommand(string Reason) : ICommand;
public class Unrelated
{
}
";

const IDENTITY: &str = "\
public interface IStringIdentity
{
}
public readonly record struct LocationId(string Value) : IStringIdentity;
public readonly record struct CustomerId(string Value);
";

fn host(dir: &Path, extra: &[&str]) -> Host {
    let adapter = ddd_lsp::adapter::for_language("csharp").expect("adapter");
    Host::new(adapter, mock_command(extra), dir.to_path_buf())
}

fn ready_host(dir: &Path, extra: &[&str]) -> Host {
    let mut h = host(dir, extra);
    h.wait_ready(Duration::from_secs(10)).expect("readiness");
    h
}

#[test]
fn definition_resolves_through_the_existing_location_normaliser() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    let messages = write(tmp.path(), "Messages.cs", MESSAGES);
    let mut h = ready_host(tmp.path(), &[]);
    h.ensure_open(&messages).expect("open");
    // Position: the `ICommand` mention on the first record's base list.
    let line = MESSAGES.lines().position(|l| l.contains("UpsertProfileSettings")).expect("line");
    let col = MESSAGES.lines().nth(line).and_then(|l| l.rfind("ICommand")).expect("col");
    let found = declaration::definition(&mut h, &messages, line as u64, col as u64).expect("definition");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].file, messages);
    assert_eq!(found[0].line, 0, "resolves to the interface's own declaration");
}

#[test]
fn type_hierarchy_answers_in_both_directions() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    let messages = write(tmp.path(), "Messages.cs", MESSAGES);
    let mut h = ready_host(tmp.path(), &[]);
    h.ensure_open(&messages).expect("open");

    let sub_line = MESSAGES.lines().position(|l| l.contains("UpsertProfileSettings")).expect("line");
    let prepared = declaration::prepare_type_hierarchy(&mut h, &messages, sub_line as u64, 14)
        .expect("prepare");
    assert_eq!(prepared.len(), 1, "{prepared:?}");
    let supers = declaration::supertypes(&mut h, &prepared[0]).expect("supertypes");
    assert_eq!(supers.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(), ["ICommand"]);

    let marker = declaration::prepare_type_hierarchy(&mut h, &messages, 0, 17).expect("prepare");
    let subs = declaration::subtypes(&mut h, &marker[0]).expect("subtypes");
    let mut names: Vec<&str> = subs.iter().map(|i| i.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names, ["DisableProfileCommand", "UpsertProfileSettingsCommand"]);
}

/// The item goes back to the server exactly as it arrived. The entry probe
/// named this the wiring detail most likely to be got wrong: Roslyn
/// resolves the directional calls only from its own opaque `data`.
#[test]
fn the_prepared_item_travels_back_untouched() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    let identity = write(tmp.path(), "Identity.cs", IDENTITY);
    let mut h = ready_host(tmp.path(), &[]);
    h.ensure_open(&identity).expect("open");

    let line = IDENTITY.lines().position(|l| l.contains("LocationId")).expect("line") as u64;
    let items = declaration::prepare_type_hierarchy(&mut h, &identity, line, 29).expect("prepare");
    assert!(items[0].data.is_some(), "the server sent an opaque blob");
    let supers = declaration::supertypes(&mut h, &items[0]).expect("supertypes");
    assert_eq!(supers.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(), ["IStringIdentity"]);

    // The sibling identity type declares no interface — a real empty
    // hierarchy, which must read differently from an instrument failure.
    let sibling = IDENTITY.lines().position(|l| l.contains("CustomerId")).expect("line") as u64;
    let items = declaration::prepare_type_hierarchy(&mut h, &identity, sibling, 29).expect("prepare");
    assert_eq!(items.len(), 1, "the type itself prepares");
    assert!(declaration::supertypes(&mut h, &items[0]).expect("supertypes").is_empty());
}

#[test]
fn the_probe_finds_every_operation_available_on_a_answering_host() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    let messages = write(tmp.path(), "Messages.cs", MESSAGES);
    let mut h = ready_host(tmp.path(), &[]);
    let probe = capability::probe(&mut h, &messages).expect("probe");
    assert_eq!(probe.unavailable(), Vec::<Operation>::new(), "{}", probe.report());
    let anchor = probe.anchor.expect("anchor");
    assert_eq!(anchor.name, "ICommand", "the probe anchors on a type declaration");
}

/// The kotlin-lsp shape, end to end: the flag says yes, the call answers
/// `null`. The row must read UNAVAILABLE and be flagged over-advertised —
/// never as a codebase that happens to have no hierarchy.
#[test]
fn an_over_advertising_host_is_caught_by_the_probe_not_believed() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    let messages = write(tmp.path(), "Messages.cs", MESSAGES);
    let mut h = ready_host(tmp.path(), &["--over-advertise", "type-hierarchy"]);
    let probe = capability::probe(&mut h, &messages).expect("probe");

    assert!(!probe.available(Operation::TypeHierarchy), "{}", probe.report());
    assert_eq!(probe.unavailable(), vec![Operation::TypeHierarchy]);
    let flagged = probe.divergences();
    assert_eq!(flagged.len(), 1);
    assert_eq!(flagged[0].operation, Operation::TypeHierarchy);
    assert_eq!(flagged[0].divergence, Some(Divergence::OverAdvertised));
    assert_eq!(flagged[0].advertised, Some(true), "the claim still stands");
    assert!(probe.report().contains("OVER-ADVERTISED"), "{}", probe.report());

    // Everything else still answers: an over-advertised operation costs its
    // own row, never the run.
    for op in [Operation::DocumentSymbol, Operation::Definition, Operation::References] {
        assert!(probe.available(op), "{op} should still answer\n{}", probe.report());
    }
}

#[test]
fn a_file_with_no_declarations_yields_no_anchor_rather_than_five_absences() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    let empty = write(tmp.path(), "Empty.cs", "// nothing declared here\n");
    let mut h = ready_host(tmp.path(), &[]);
    let probe = capability::probe(&mut h, &empty).expect("probe");
    assert!(probe.anchor.is_none(), "{}", probe.report());
    assert!(probe.report().contains("no type declaration"));
    for op in Operation::ALL.iter().skip(1) {
        let row = probe.operations.iter().find(|r| r.operation == *op).expect("row");
        assert!(row.detail.contains("not probed"), "{op}: {}", row.detail);
    }
}
