//! Unit tests for the C# reify slice — inference, emission, determinism.

#[path = "reify_fixtures.rs"]
mod fixtures;

use fixtures::*;
use super::*;
use crate::pf::reify_ident::{method_name, pascal};


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
    let shape = crate::pf::reify_infer::infer(&fixture_decider());
    let cmd = shape.commands.get("PlaceOrder").expect("command entry");
    assert_eq!(cmd.get("amount"), Some(&Some(crate::pf::reify_ident::CsTy::Long)));
    let ev = shape.events.get("OrderPlaced").expect("event entry");
    assert_eq!(ev.get("amount"), Some(&Some(crate::pf::reify_ident::CsTy::Long)));
    // status is typed from initial; amount is a plain `=event.amount` copy,
    // so it inherits the event field's type once scenarios pin it.
    assert_eq!(shape.state.get("status"), Some(&Some(crate::pf::reify_ident::CsTy::Str)));
    assert_eq!(shape.state.get("amount"), Some(&Some(crate::pf::reify_ident::CsTy::Long)));
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
fn realise_csharp_cell_is_valid_task_type_yaml() {
    for p in [plan(), oracle_plan()] {
        let cell = &file(&p, "realise-csharp.cell.g.yaml").content;
        let parsed = crate::pf::cell::TaskType::from_yaml(cell).expect("cell parses");
        assert_eq!(parsed.id, "realise-csharp");
        assert_eq!(parsed.audits.len(), 5);
        assert!(parsed.audits.iter().any(|a| a.checks.contains("product reify check")));
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
