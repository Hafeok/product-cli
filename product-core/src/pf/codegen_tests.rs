//! Unit tests for the C# codegen slice — inference, emission, determinism.

#[path = "codegen_fixtures.rs"]
mod fixtures;

use fixtures::*;
use super::*;
use crate::pf::codegen_ident::{method_name, pascal};


#[test]
fn identifiers_pascal_case_totally() {
    assert_eq!(pascal("cmd-add-bounded-context"), "CmdAddBoundedContext");
    assert_eq!(pascal("PlaceOrder"), "PlaceOrder");
    assert_eq!(pascal("2fa-code"), "N2FaCode");
    assert_eq!(pascal(""), "X");
    assert_eq!(method_name("non-positive rejected"), "Non_positive_rejected");
}

#[test]
fn inference_recovers_payload_and_state_fields() {
    let shape = crate::pf::codegen_infer::infer(&fixture_decider());
    let cmd = shape.commands.get("PlaceOrder").expect("command entry");
    assert_eq!(cmd.get("amount"), Some(&Some(crate::pf::codegen_ident::CsTy::Long)));
    let ev = shape.events.get("OrderPlaced").expect("event entry");
    assert_eq!(ev.get("amount"), Some(&Some(crate::pf::codegen_ident::CsTy::Long)));
    // status is typed from initial; amount is a plain `=event.amount` copy,
    // so it inherits the event field's type once scenarios pin it.
    assert_eq!(shape.state.get("status"), Some(&Some(crate::pf::codegen_ident::CsTy::Str)));
    assert_eq!(shape.state.get("amount"), Some(&Some(crate::pf::codegen_ident::CsTy::Long)));
}

#[test]
fn declared_payload_fields_override_inference() {
    use crate::pf::model::Attribute;
    let mut g = fixture_graph();
    if let Some(c) = g.commands.iter_mut().find(|c| c.id == "PlaceOrder") {
        c.fields = vec![
            Attribute { name: "amount".into(), ty: Some("string".into()) }, // overrides inferred Long
            Attribute { name: "currency".into(), ty: Some("string".into()) }, // never inferred
            Attribute { name: "hint".into(), ty: None },                   // declared, untyped
        ];
    }
    let shape = crate::pf::codegen_infer::infer_shape(&fixture_decider(), &g);
    let cmd = shape.commands.get("PlaceOrder").expect("command entry");
    assert_eq!(cmd.get("amount"), Some(&Some(crate::pf::codegen_ident::CsTy::Str)));
    assert_eq!(cmd.get("currency"), Some(&Some(crate::pf::codegen_ident::CsTy::Str)));
    assert_eq!(cmd.get("hint"), Some(&None), "untyped declaration exists, type open");
}

#[test]
fn typed_contracts_are_emitted() {
    let p = plan();
    let types = &file(&p, "Order/OrderTypes.g.cs").content;
    assert!(types.contains("public sealed record PlaceOrder(long? Amount = null) : IOrderCommand"));
    assert!(types.contains("public string WireId => \"PlaceOrder\";"));
    assert!(types.contains("public sealed record OrderPlaced(long? Amount = null) : IOrderEvent"));
    assert!(types.contains("if (Amount is not null) d[\"amount\"] = Amount;"));
    assert!(types.contains("public string Status { get; init; } = \"placed\";") == false);
    assert!(types.contains("public string Status { get; init; } = \"new\";"));
    let domain = &file(&p, "Domain.g.cs").content;
    assert!(domain.contains("public sealed record Order(string? CustomerId = null);"));
    assert!(domain.contains("public sealed record Money;"));
    assert!(domain.contains("public enum ShippingMethod"));
    assert!(domain.contains("Express2Day,"));
}

#[test]
fn decider_frame_and_typed_adapter_are_emitted() {
    let p = plan();
    let frame = &file(&p, "OrderDecider.g.cs").content;
    assert!(frame.contains("public static partial DecisionResult Decide(OrderState state, IOrderCommand command);"));
    assert!(frame.contains("public static partial OrderState Evolve(OrderState state, IOrderEvent evt);"));
    assert!(frame.contains("public static readonly string[] Handles = { \"PlaceOrder\" };"));
    let adapter = &file(&p, "OrderAdapter.g.cs").content;
    assert!(adapter.contains("public sealed class OrderAdapter : IConformanceAdapter"));
    assert!(adapter.contains("\"PlaceOrder\" => new PlaceOrder(Amount: PfWire.GetLong(wire.With, \"amount\")),"));
    let program = &file(&p, "Program.g.cs").content;
    assert!(program.contains("\"order-decider\" => new OrderAdapter(),"));
    let oracle = &file(&p, "Oracle.g.cs").content;
    assert!(oracle.contains("public interface IConformanceAdapter"));
    assert!(oracle.contains("ConformanceOutcome Run(string deciderId, IReadOnlyList<WireEvent> given, WireCommand when);"));
}

#[test]
fn oracle_only_mode_emits_the_verification_shell_without_types() {
    let p = oracle_plan();
    assert!(!p.files.iter().any(|f| f.path.contains("Domain.g.cs")), "no domain types");
    assert!(!p.files.iter().any(|f| f.path.contains("OrderTypes.g.cs")), "no typed records");
    let stub = file(&p, "ConformanceAdapter.cs");
    assert!(!stub.overwrite, "adapter stub is scaffolded once");
    assert!(stub.content.contains("public sealed class ConformanceAdapter : IConformanceAdapter"));
    let csproj = file(&p, "Bookstore.Conformance/Bookstore.Conformance.csproj");
    assert!(!csproj.overwrite, "oracle csproj is realiser-owned (domain reference)");
    let program = &file(&p, "Program.g.cs").content;
    assert!(program.contains("static IConformanceAdapter ResolveDecider(string id) => new ConformanceAdapter();"));
    let tests = &file(&p, "Bookstore.Conformance.Tests/OrderScenarioTests.g.cs").content;
    assert!(tests.contains("var adapter = new ConformanceAdapter();"));
    assert!(tests.contains("adapter.Run(\"order-decider\""));
    assert!(tests.contains("new WireCommand(\"PlaceOrder\", new Dictionary<string, object?> { [\"amount\"] = 5L })"));
    assert!(tests.contains("Assert.Equal(\"inv-positive-amount\", outcome.Reject);"));
}

#[test]
fn projector_frame_view_record_and_facts_are_emitted() {
    let p = plan();
    let frame = &file(&p, "Views/OrderSummaryProjector.g.cs").content;
    assert!(frame.contains("public sealed record OrderSummaryView"));
    // initial field: non-nullable with default, always in wire state.
    assert!(frame.contains("public long Count { get; init; } = 0;"));
    assert!(frame.contains("d[\"count\"] = Count;"));
    // apply-only field: nullable, in wire state once set — oracle semantics.
    assert!(frame.contains("public long? LastAmount { get; init; }"));
    assert!(frame.contains("if (LastAmount is not null) d[\"last_amount\"] = LastAmount;"));
    assert!(frame.contains("public static partial OrderSummaryView Apply(OrderSummaryView view, WireEvent evt);"));
    let stub = file(&p, "Views/OrderSummaryProjector.cs");
    assert!(!stub.overwrite);
    let adapter = &file(&p, "OrderSummaryProjectionAdapter.g.cs").content;
    assert!(adapter.contains("public sealed class OrderSummaryProjectionAdapter : IProjectionAdapter"));
    let tests = &file(&p, "Bookstore.Domain.Tests/OrderSummaryProjectionTests.g.cs").content;
    assert!(tests.contains("OrderSummaryProjector.Fold(given).WireState()"));
    assert!(tests.contains("Assert.Equal(1L, Assert.IsType<long>(wire[\"count\"]));"));
    let program = &file(&p, "Program.g.cs").content;
    assert!(program.contains("\"ordersummary-projector\" => new OrderSummaryProjectionAdapter(),"));
    // Full mode hosts the wire seam in the Domain project (frames consume it).
    assert!(p.files.iter().any(|f| f.path == "Bookstore.Domain/Oracle.g.cs"));
}

#[test]
fn oracle_only_projectors_go_through_the_projection_adapter() {
    let p = oracle_plan();
    let stub = file(&p, "ProjectionAdapter.cs");
    assert!(!stub.overwrite);
    assert!(stub.content.contains("public sealed class ProjectionAdapter : IProjectionAdapter"));
    let tests = &file(&p, "Bookstore.Conformance.Tests/OrderSummaryProjectionTests.g.cs").content;
    assert!(tests.contains("new ProjectionAdapter().Run(\"ordersummary-projector\", given)"));
    let program = &file(&p, "Program.g.cs").content;
    assert!(program.contains("\"ordersummary-projector\" => new ProjectionAdapter(),"));
    assert!(!p.files.iter().any(|f| f.path.contains("Views/")), "no typed views in oracle mode");
}

#[test]
fn projectors_move_the_input_hash() {
    let with = plan();
    let without =
        plan_csharp(&fixture_graph(), &[fixture_decider()], &[], &opts()).expect("plan");
    assert_ne!(with.graph_hash, without.graph_hash);
}

#[test]
fn flow_facts_bake_the_oracle_chain_across_both_seams() {
    let p = plan();
    let flows = &file(&p, "FlowTests.g.cs").content;
    assert!(flows.contains("public void Buy_a_book()"));
    // The command step drives the decider adapter with the scenario payload…
    assert!(flows.contains("new OrderAdapter().Run(\"order-decider\", stream, new WireCommand(\"PlaceOrder\", new Dictionary<string, object?> { [\"amount\"] = 5L }));"));
    assert!(flows.contains("stream.AddRange(o0.Emit!);"));
    // …and the terminal view is the oracle's projection over the stream.
    assert!(flows.contains("new OrderSummaryProjectionAdapter().Run(\"ordersummary-projector\", stream);"));
    assert!(flows.contains("Assert.Equal(1L, Assert.IsType<long>(view0[\"count\"]));"));
    // Oracle-only routes the same chain through the scaffolded adapters.
    let o = oracle_plan();
    let oflows = &file(&o, "FlowTests.g.cs").content;
    assert!(oflows.contains("new ConformanceAdapter().Run(\"order-decider\""));
    assert!(oflows.contains("new ProjectionAdapter().Run(\"ordersummary-projector\""));
}

#[test]
fn screen_facts_pin_surfaces_offers_and_state_coverage() {
    let p = plan();
    let seam = &file(&p, "Bookstore.Domain/ScreenSeam.g.cs").content;
    assert!(seam.contains("public interface IScreenAdapter"));
    let stub = file(&p, "ScreenAdapter.cs");
    assert!(!stub.overwrite);
    let tests = &file(&p, "UiCheckoutScreenTests.g.cs").content;
    // Present state: fixture from the projector oracle, every surface + offer.
    assert!(tests.contains("Render(\"ui-checkout\", \"present\", new Dictionary<string, object?> { [\"count\"] = 1L, [\"last_amount\"] = 5L })"));
    assert!(tests.contains("Assert.Contains(\"OrderSummary\", screen.Projections);"));
    assert!(tests.contains("Assert.Contains(\"PlaceOrder\", screen.OfferedCommands);"));
    // Non-waived degraded state gets a fact; the waived one does not.
    assert!(tests.contains("public void OrderSummary_empty_state_is_handled()"));
    assert!(!tests.contains("failed_state_is_handled"), "waived state must not be tested");
}

#[test]
fn openapi_contract_projects_commands_and_views() {
    let p = plan();
    let api = &file(&p, "openapi.g.json").content;
    let doc: serde_json::Value = serde_json::from_str(api).expect("valid JSON");
    assert_eq!(doc["openapi"], "3.0.3");
    assert_eq!(doc["info"]["x-pf-graph-hash"], format!("sha256:{}", p.graph_hash));
    let place = &doc["paths"]["/commands/PlaceOrder"]["post"];
    assert_eq!(
        place["requestBody"]["content"]["application/json"]["schema"]["properties"]["amount"]["type"],
        "integer"
    );
    assert!(place["responses"]["409"].is_object(), "rejection response present");
    let view = &doc["paths"]["/views/OrderSummary"]["get"];
    assert_eq!(
        view["responses"]["200"]["content"]["application/json"]["schema"]["properties"]["count"]["type"],
        "integer"
    );
}

#[test]
fn kotlin_backend_emits_the_full_verification_shell() {
    use crate::pf::codegen_kotlin::plan_kotlin;
    let o = ReifyOptions { oracle_only: true, ..opts() };
    let p = plan_kotlin(&fixture_graph(), &[fixture_decider()], &[fixture_projector()], &o)
        .expect("plan");
    let oracle = &file(&p, "src/main/kotlin/bookstore/Oracle.g.kt").content;
    assert!(oracle.contains("interface IConformanceAdapter"));
    assert!(oracle.contains("fun run(projectorId: String, given: List<WireEvent>): Map<String, Any?>"));
    let main = &file(&p, "Main.g.kt").content;
    assert!(main.contains("\"ordersummary-projector\" -> ProjectionAdapter()"));
    let facts = &file(&p, "OrderScenarioTests.g.kt").content;
    assert!(facts.contains("ConformanceAdapter().run(\"order-decider\", emptyList(), WireCommand(\"PlaceOrder\", mapOf(\"amount\" to 5L)))"));
    assert!(facts.contains("assertEquals(\"inv-positive-amount\", outcome.reject)"));
    let proj = &file(&p, "OrderSummaryProjectionTests.g.kt").content;
    assert!(proj.contains("assertEquals(1L, wire[\"count\"])"));
    let flows = &file(&p, "FlowTests.g.kt").content;
    assert!(flows.contains("stream.addAll(o0.emit!!)"));
    let screens = &file(&p, "UiCheckoutScreenTests.g.kt").content;
    assert!(screens.contains("assertTrue(\"PlaceOrder\" in screen.offeredCommands)"));
    // Same graph, same hash as the C# tree — the cross-language pin.
    assert_eq!(p.graph_hash, oracle_plan().graph_hash);
    // Scaffolds: gradle files + three adapters, never overwritten.
    for scaffold in ["build.gradle.kts", "ConformanceAdapter.kt", "ProjectionAdapter.kt", "ScreenAdapter.kt"] {
        assert!(!file(&p, scaffold).overwrite, "{scaffold} is realiser-owned");
    }
    let cell = &file(&p, "realise-kotlin.cell.g.yaml").content;
    let parsed = crate::pf::cell::TaskType::from_yaml(cell).expect("cell parses");
    assert_eq!(parsed.id, "realise-kotlin");
    assert!(parsed.audits.iter().any(|a| a.checks.contains("gradle test")));
}

#[test]
fn manifest_carries_the_whole_oracle_by_value() {
    let m = crate::pf::codegen_manifest::manifest(
        &fixture_graph(), &[fixture_decider()], &[fixture_projector()], &opts(),
    )
    .expect("manifest");
    assert_eq!(m.manifest_version, "1");
    assert_eq!(m.graph_hash, format!("sha256:{}", plan().graph_hash), "same pin as the plans");
    let agg = &m.aggregates[0];
    assert_eq!(agg.aggregate, "Order");
    assert_eq!(agg.commands["PlaceOrder"]["amount"], Some("long"));
    assert_eq!(agg.scenarios.len(), 2);
    let p = &m.projectors[0];
    assert_eq!(p.view["count"], Some("long"));
    // Flow chain baked: the accepted outcome with its emitted event.
    let flow = &m.flows[0];
    assert_eq!(flow.commands[0].decider_id, "order-decider");
    assert!(matches!(&flow.commands[0].outcome, crate::pf::decider_sim::Outcome::Accepted(e) if e[0].event == "OrderPlaced"));
    assert_eq!(flow.views[0].projector_id, "ordersummary-projector");
    // Screen fact: fixture from the projector oracle, waived state absent.
    let screen = &m.screens[0];
    assert_eq!(screen.offers, vec!["PlaceOrder"]);
    assert_eq!(screen.degraded_states, vec![("OrderSummary".to_string(), "empty".to_string())]);
    assert!(screen.present_fixture.is_some());
    // And the whole document serializes.
    assert!(serde_json::to_string(&m).expect("json").contains("\"reject\""));
}

#[test]
fn unit_slice_cuts_the_manifest_to_a_neighbourhood_with_the_full_hash() {
    use crate::pf::codegen_manifest::{manifest, manifest_unit};
    // An unrelated projector: folds an event no retained decider produces.
    let mut stray = fixture_projector();
    stray.id = "stray-projector".into();
    stray.projects_for = "StrayView".into();
    stray.folds = vec!["UnrelatedEvent".into()];
    let projectors = [fixture_projector(), stray];
    let full = manifest(&fixture_graph(), &[fixture_decider()], &projectors, &opts()).expect("full");
    let sliced = manifest_unit(&fixture_graph(), &[fixture_decider()], &projectors, &opts(), "order-decider")
        .expect("slice");
    // The neighbourhood: the decider, the projector folding its events —
    // the stray is gone; the screen surfacing OrderSummary stays.
    assert_eq!(sliced.aggregates.len(), 1);
    assert_eq!(sliced.projectors.len(), 1);
    assert_eq!(sliced.projectors[0].projector_id, "ordersummary-projector");
    assert_eq!(sliced.flows.len(), 1);
    assert_eq!(sliced.screens.len(), 1);
    // A slice is a view of the same spec: identical pin, unlike a sub-spec.
    assert_eq!(sliced.graph_hash, full.graph_hash);
    // Slicing by the projector pulls its upstream decider in.
    let by_p = manifest_unit(&fixture_graph(), &[fixture_decider()], &projectors, &opts(), "ordersummary-projector")
        .expect("slice");
    assert_eq!(by_p.aggregates.len(), 1);
    assert_eq!(by_p.projectors.len(), 1);
    // Unknown units name what exists.
    let err = manifest_unit(&fixture_graph(), &[fixture_decider()], &projectors, &opts(), "ghost").err().map(|e| e.to_string()).unwrap_or_default();
    assert!(err.contains("order-decider"), "unknown unit names the known set: {err}");
}

#[test]
fn backend_registry_resolves_and_rejects() {
    use crate::pf::codegen_backend::{backend, backends};
    assert_eq!(backends().len(), 3);
    assert!(backend("web").expect("web").oracle_only_forced());
    assert!(!backend("csharp").expect("csharp").oracle_only_forced());
    assert!(backend("kotlin").expect("kotlin").oracle_only_forced());
    assert!(backend("cobol").is_err());
}

#[test]
fn external_plan_parses_appends_provenance_and_rejects_escapes() {
    use crate::pf::codegen_backend::external_plan;
    let (g, d, p, o) = (fixture_graph(), [fixture_decider()], [fixture_projector()], opts());
    let plan = external_plan(
        r#"{"files": [{"path": "src/x.ts", "content": "// hi"}, {"path": "adapter.ts", "content": "", "overwrite": false}]}"#,
        &g, &d, &p, &o,
    )
    .expect("plan");
    assert_eq!(plan.files.len(), 3, "two plugin files + provenance");
    assert!(!plan.files[1].overwrite, "scaffold flag honoured");
    let prov = &plan.files[2];
    assert_eq!(prov.path, "provenance.g.json");
    assert_eq!(recorded_hash(&prov.content).expect("hash"), plan.graph_hash, "check works on plugin trees");
    assert!(!prov.content.contains("adapter.ts\""), "scaffolds excluded from the manifest");
    for bad in [r#"{"files": [{"path": "/etc/x"}]}"#, r#"{"files": [{"path": "a/../../x"}]}"#] {
        assert!(external_plan(bad, &g, &d, &p, &o).is_err(), "escape rejected: {bad}");
    }
}

#[test]
fn realise_csharp_cell_is_valid_task_type_yaml() {
    for p in [plan(), oracle_plan()] {
        let cell = &file(&p, "realise-csharp.cell.g.yaml").content;
        let parsed = crate::pf::cell::TaskType::from_yaml(cell).expect("cell parses");
        assert_eq!(parsed.id, "realise-csharp");
        assert_eq!(parsed.audits.len(), 5);
        assert!(parsed.audits.iter().any(|a| a.checks.contains("product codegen check")));
        assert!(parsed.audits.iter().any(|a| a.checks.contains("decider conform")));
    }
}

#[test]
fn scenarios_become_xunit_facts() {
    let p = plan();
    let tests = &file(&p, "OrderScenarioTests.g.cs").content;
    assert!(tests.contains("public void Order_accepted()"));
    assert!(tests.contains("var state = OrderDecider.InitialState();"));
    assert!(tests.contains("OrderDecider.Decide(state, new PlaceOrder(Amount: 5L));"));
    assert!(tests.contains("Assert.Single(result.Events);"));
    assert!(tests.contains("Assert.Equal(5L, Assert.IsType<long>(with0[\"amount\"]));"));
    assert!(tests.contains("Assert.Equal(\"inv-positive-amount\", result.RejectedInvariant);"));
}

#[test]
fn stub_is_scaffold_only_and_excluded_from_manifest() {
    let p = plan();
    let stub = file(&p, "Order/OrderDecider.cs");
    assert!(!stub.overwrite);
    assert!(stub.content.contains("NotImplementedException"));
    let prov = &file(&p, "provenance.g.json").content;
    assert!(!prov.contains("OrderDecider.cs\""));
    assert!(prov.contains("OrderDecider.g.cs"));
}

#[test]
fn provenance_pins_the_input_hash() {
    let p = plan();
    let prov = &file(&p, "provenance.g.json").content;
    assert_eq!(recorded_hash(prov).expect("hash"), p.graph_hash);
    let cs = &file(&p, "Provenance.g.cs").content;
    assert!(cs.contains(&format!("sha256:{}", p.graph_hash)));
    assert!(cs.contains("AssemblyMetadata(\"PF.WhatVersion\", \"1.0\")"));
}

#[test]
fn generation_is_deterministic_and_hash_tracks_the_graph() {
    let a = plan();
    let b = plan();
    assert_eq!(a.graph_hash, b.graph_hash);
    for (fa, fb) in a.files.iter().zip(&b.files) {
        assert_eq!(fa.path, fb.path);
        assert_eq!(fa.content, fb.content, "non-deterministic file {}", fa.path);
    }
    let mut moved = fixture_graph();
    moved.events.push(crate::pf::model::Event {
        fields: vec![],
        id: "OrderCancelled".into(),
        label: "Order cancelled".into(),
        context: "Catalog".into(),
        changes: "Order".into(),
    });
    let c = plan_csharp(&moved, &[fixture_decider()], &[], &opts()).expect("plan");
    assert_ne!(a.graph_hash, c.graph_hash);
}

#[test]
fn duplicate_aggregate_deciders_are_rejected() {
    let mut d2 = fixture_decider();
    d2.id = "order-decider-2".into();
    let err = plan_csharp(&fixture_graph(), &[fixture_decider(), d2], &[], &opts());
    assert!(err.is_err());
}
