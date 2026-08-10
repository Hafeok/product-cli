//! Classifier coverage: every policy row triggered plus not triggered,
//! with the committed M3/M4 fixture files as the edit substrate.

use super::*;
use crate::adapter::{bicep, csharp, AdapterFlags};
use crate::mock::parse::document_symbols;
use crate::protocol::flatten_symbols;
use crate::surface::SurfaceEvent;

/// The committed fixture substrates (extending the M2 fixture set).
const CS_BASE: &str = include_str!("../../ddd-cli/tests/fixtures/csharp/Library.cs");
const BI_BASE: &str = include_str!("../../ddd-cli/tests/fixtures/bicep/storage-module.bicep");

fn events(
    adapter: &Adapter,
    uri: &str,
    flags: &AdapterFlags,
    before: &str,
    after: &str,
) -> Vec<SurfaceEvent> {
    let bs = flatten_symbols(&document_symbols(uri, before));
    let as_ = flatten_symbols(&document_symbols(uri, after));
    classify_edit(adapter, flags, before, &bs, after, &as_)
}

fn cs(before: &str, after: &str) -> Vec<SurfaceEvent> {
    events(&csharp::ADAPTER, "file:///t.cs", &AdapterFlags::default(), before, after)
}

fn bi(before: &str, after: &str) -> Vec<SurfaceEvent> {
    events(&bicep::ADAPTER, "file:///m.bicep", &AdapterFlags::default(), before, after)
}

fn surface_rules(evts: &[SurfaceEvent]) -> Vec<&'static str> {
    evts.iter().filter(|e| e.surface).filter_map(|e| e.rule).collect()
}

/// Assert the fixture actually contains the anchor an edit rewrites.
fn edit(base: &str, from: &str, to: &str) -> String {
    assert!(base.contains(from), "fixture lost the anchor `{from}` — update the tests");
    base.replace(from, to)
}

#[test]
fn cs_add_exposed_triggers_on_public_member_not_private() {
    let with_public =
        edit(CS_BASE, "    public void Ping() { }\n", "    public void Ping() { }\n    public void Pong() { }\n");
    assert_eq!(surface_rules(&cs(CS_BASE, &with_public)), vec!["cs-add-exposed"]);
    let with_private =
        edit(CS_BASE, "    private void Helper() { }\n", "    private void Helper() { }\n    private void Quiet() { }\n");
    let evts = cs(CS_BASE, &with_private);
    assert!(surface_rules(&evts).is_empty(), "{evts:?}");
    assert_eq!(evts[0].rule, Some("cs-private-nonsurface"));
}

#[test]
fn cs_signature_change_triggers_on_public_not_on_body() {
    let changed = edit(CS_BASE, "public void Ping() { }", "public void Ping(int count) { }");
    assert_eq!(surface_rules(&cs(CS_BASE, &changed)), vec!["cs-signature-exposed"]);
    let body = edit(CS_BASE, "private void Helper() { }", "private void Helper() { Ping(); }");
    assert!(cs(CS_BASE, &body).is_empty());
}

#[test]
fn cs_remove_exposed_triggers_on_public_not_private() {
    let no_public = edit(CS_BASE, "    protected void Guarded() { }\n", "");
    assert_eq!(surface_rules(&cs(CS_BASE, &no_public)), vec!["cs-remove-exposed"]);
    let no_private = edit(CS_BASE, "    private void Helper() { }\n", "");
    assert!(surface_rules(&cs(CS_BASE, &no_private)).is_empty());
}

#[test]
fn cs_new_ctor_param_on_public_type_is_surface() {
    let after =
        edit(CS_BASE, "public PublicApi(int retries) { }", "public PublicApi(int retries, string name) { }");
    let evts = cs(CS_BASE, &after);
    assert_eq!(surface_rules(&evts), vec!["cs-signature-exposed"]);
    let surfaced: Vec<_> = evts.iter().filter(|e| e.surface).collect();
    assert_eq!(surfaced[0].facts.kind, "constructor");
}

#[test]
fn cs_interface_member_forces_implementors_when_exposed() {
    let after = edit(CS_BASE, "    void Handle();\n", "    void Handle();\n    void Retry();\n");
    assert_eq!(surface_rules(&cs(CS_BASE, after.as_str())), vec!["cs-interface-member"]);
    let internal_before = edit(CS_BASE, "public interface IContract", "internal interface IContract");
    let internal_after =
        edit(&after, "public interface IContract", "internal interface IContract");
    assert!(surface_rules(&cs(&internal_before, &internal_after)).is_empty());
}

#[test]
fn cs_exported_attribute_marks_any_visibility_as_surface() {
    let changed = edit(CS_BASE, "private string Fetch() { return \"x\"; }", "private string Fetch(int id) { return \"x\"; }");
    assert_eq!(surface_rules(&cs(CS_BASE, &changed)), vec!["cs-exported-endpoint"]);
    let plain_attr = edit(
        CS_BASE,
        "    private void Helper() { }\n",
        "    private void Helper() { }\n    [Obsolete]\n    private void Old() { }\n",
    );
    assert!(surface_rules(&cs(CS_BASE, &plain_attr)).is_empty());
}

#[test]
fn cs_internal_flips_with_the_library_posture_flag() {
    let after = edit(
        CS_BASE,
        "    internal void Reach() { }\n",
        "    internal void Reach() { }\n    internal void Farther() { }\n",
    );
    assert!(surface_rules(&cs(CS_BASE, &after)).is_empty());
    let lib = AdapterFlags { internal_is_surface: true, ..AdapterFlags::default() };
    let evts = events(&csharp::ADAPTER, "file:///t.cs", &lib, CS_BASE, &after);
    assert_eq!(surface_rules(&evts), vec!["cs-internal-surface"]);
}

#[test]
fn cs_visibility_flip_is_judged_at_the_more_exposed_side() {
    let promoted = edit(CS_BASE, "private int hidden;", "public int hidden;");
    assert_eq!(surface_rules(&cs(CS_BASE, &promoted)), vec!["cs-visibility-boundary"]);
    let demoted = edit(CS_BASE, "public void Ping() { }", "private void Ping() { }");
    assert_eq!(surface_rules(&cs(CS_BASE, &demoted)), vec!["cs-visibility-boundary"]);
    let sideways = edit(CS_BASE, "private int hidden;", "internal int hidden;");
    assert!(surface_rules(&cs(CS_BASE, &sideways)).is_empty());
}

/// `dec/ddd/enum-member-gap-priced`: the DDD-adapter-03 hole, closed. A
/// member added to a public enum is demanded; the same edit on an internal
/// enum is not (the app-repo posture caps it below EXPOSED).
#[test]
fn cs_enum_member_addition_triggers_on_public_enum_not_internal() {
    let grown = edit(CS_BASE, "    Active,\n", "    Active,\n    Escalated,\n");
    let evts = cs(CS_BASE, &grown);
    assert_eq!(surface_rules(&evts), vec!["cs-enum-member"], "{evts:?}");
    let added: Vec<_> = evts.iter().filter(|e| e.facts.name == "Escalated").collect();
    assert_eq!(added[0].facts.kind, "enum-member");
    assert_eq!(added[0].facts.visibility, "public");
    let internal_grown = edit(CS_BASE, "    On,\n", "    On,\n    Auto,\n");
    let evts = cs(CS_BASE, &internal_grown);
    assert!(surface_rules(&evts).is_empty(), "{evts:?}");
    assert_eq!(evts[0].rule, Some("cs-internal-nonsurface"));
}

/// Removal breaks consumers the same way (their arms stop compiling), so the
/// row demands it too.
#[test]
fn cs_enum_member_removal_is_surface() {
    let shrunk = edit(CS_BASE, "    Active,\n    Suspended,\n", "    Active,\n");
    assert_eq!(surface_rules(&cs(CS_BASE, &shrunk)), vec!["cs-enum-member"]);
}

#[test]
fn cs_container_visibility_caps_members() {
    let after = edit(
        CS_BASE,
        "    public void LoudButCapped() { }\n",
        "    public void LoudButCapped() { }\n    public void AlsoCapped() { }\n",
    );
    let evts = cs(CS_BASE, &after);
    assert!(surface_rules(&evts).is_empty(), "{evts:?}");
    let added: Vec<_> = evts.iter().filter(|e| e.facts.name == "AlsoCapped").collect();
    assert_eq!(added[0].facts.visibility, "internal");
}

#[test]
fn bi_param_membership_triggers_on_param_not_var() {
    let with_param = edit(BI_BASE, "param accountName string\n", "param accountName string\nparam replicas int = 2\n");
    assert_eq!(surface_rules(&bi(BI_BASE, &with_param)), vec!["bi-param-membership"]);
    let removed = edit(BI_BASE, "param skuName string = 'Standard_LRS'\n", "");
    // Removing the param also changes the resource body reference? No —
    // sku lines are body, not name/scope, so only the param event fires.
    assert_eq!(surface_rules(&bi(BI_BASE, &removed)), vec!["bi-param-membership"]);
    let with_var = edit(BI_BASE, "var namePrefix = 'ddd'\n", "var namePrefix = 'ddd'\nvar suffix = 'x'\n");
    let evts = bi(BI_BASE, &with_var);
    assert!(surface_rules(&evts).is_empty());
    assert_eq!(evts[0].rule, Some("bi-internal"));
}

#[test]
fn bi_param_retype_triggers_but_default_change_does_not() {
    let retyped = edit(BI_BASE, "param skuName string =", "param skuName int =");
    assert_eq!(surface_rules(&bi(BI_BASE, &retyped)), vec!["bi-param-retype"]);
    let new_default = edit(BI_BASE, "param skuName string = 'Standard_LRS'", "param skuName string = 'Premium_LRS'");
    assert!(bi(BI_BASE, &new_default).is_empty());
}

#[test]
fn bi_decorator_change_is_a_tolerance_change() {
    let widened = edit(
        BI_BASE,
        "@allowed(['westeurope', 'northeurope'])",
        "@allowed(['westeurope', 'northeurope', 'swedencentral'])",
    );
    assert_eq!(surface_rules(&bi(BI_BASE, &widened)), vec!["bi-param-tolerance"]);
    let comment = edit(BI_BASE, "// every policy row", "// each policy row");
    assert!(bi(BI_BASE, &comment).is_empty());
}

#[test]
fn bi_output_change_is_surface() {
    let changed = edit(BI_BASE, "output storageId string = stg.id", "output storageId string = stg.properties.primaryEndpoints.blob");
    assert_eq!(surface_rules(&bi(BI_BASE, &changed)), vec!["bi-output-contract"]);
    let removed = edit(BI_BASE, "output storageName string = stg.name\n", "");
    assert_eq!(surface_rules(&bi(BI_BASE, &removed)), vec!["bi-output-contract"]);
}

#[test]
fn bi_resource_name_scope_is_contract_but_body_is_not() {
    let renamed = edit(BI_BASE, "  name: '${namePrefix}${accountName}'", "  name: accountName");
    assert_eq!(surface_rules(&bi(BI_BASE, &renamed)), vec!["bi-resource-contract"]);
    let body = edit(BI_BASE, "  kind: 'StorageV2'", "  kind: 'BlobStorage'");
    assert!(bi(BI_BASE, &body).is_empty());
}

#[test]
fn bi_param_ground_provenance_maps_per_the_table() {
    let syms = flatten_symbols(&document_symbols("file:///m.bicep", BI_BASE));
    let facts = (bicep::ADAPTER.facts)(BI_BASE, &syms, &AdapterFlags::default());
    let prov: Vec<(&str, &str)> = facts
        .iter()
        .filter(|f| f.kind == "param")
        .map(|f| (f.name.as_str(), f.extra["ground_provenance"].as_str()))
        .collect();
    assert_eq!(
        prov,
        vec![
            ("location", "controlled"),
            ("accountName", "observed"),
            ("skuName", "controlled"),
            ("deployedAt", "inferred"),
        ]
    );
}
