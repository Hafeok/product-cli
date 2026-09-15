//! Tests for the entry-point candidates: the criterion by kind, the observed positions, the path, the overlaps, the recall.

use serde_json::json;

use super::super::csharp_candidates_report::{candidates, EntryPointTruth};
use super::super::csharp_inventory::Inventory;
use super::{handler_of, page_path, Kind};

fn ty(id: &str, project: &str, base: Option<&str>, file: &str, attrs: serde_json::Value) -> serde_json::Value {
    let (ns, name) = id.trim_start_matches("T:").rsplit_once('.').unwrap_or(("", id));
    json!({"id": id, "project": project, "namespace": ns, "name": name, "kind": "class", "accessibility": "public", "base_type": base, "file": file, "attributes": attrs})
}

fn method(id: &str, declaring: &str, attrs: serde_json::Value) -> serde_json::Value {
    let name = id.split('(').next().unwrap_or(id).rsplit('.').next().unwrap_or(id);
    json!({"id": id, "declaring_type": declaring, "name": name, "kind": "method", "accessibility": "public", "attributes": attrs})
}

fn attr(t: &str, positional: serde_json::Value, named: serde_json::Value) -> serde_json::Value {
    json!({"type": t, "positional": positional, "named": named})
}

fn externals() -> serde_json::Value {
    json!([
            {"id": "T:Microsoft.AspNetCore.Mvc.ControllerBase", "name": "ControllerBase", "kind": "class", "is_abstract": true},
            {"id": "T:Microsoft.AspNetCore.Mvc.Controller", "name": "Controller", "kind": "class", "is_abstract": true, "base_type": "T:Microsoft.AspNetCore.Mvc.ControllerBase"},
            {"id": "T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel", "name": "PageModel", "kind": "class", "is_abstract": true},
            {"id": "T:Microsoft.AspNetCore.Mvc.ViewComponent", "name": "ViewComponent", "kind": "class", "is_abstract": true},
            {"id": "T:FastEndpoints.BaseEndpoint", "name": "BaseEndpoint", "kind": "class", "is_abstract": true},
            {"id": "T:FastEndpoints.Endpoint`2", "name": "Endpoint", "kind": "class", "is_abstract": true, "arity": 2, "base_type": "T:FastEndpoints.BaseEndpoint"},
            {"id": "T:Ardalis.Specification.IReadRepositoryBase`1", "name": "IReadRepositoryBase", "kind": "interface", "arity": 1, "methods": 4},
            {"id": "T:Ardalis.Specification.IRepositoryBase`1", "name": "IRepositoryBase", "kind": "interface", "arity": 1, "methods": 4, "interfaces": ["T:Ardalis.Specification.IReadRepositoryBase`1"]},
            {"id": "T:Microsoft.Extensions.Hosting.IHostedService", "name": "IHostedService", "kind": "interface", "methods": 2},
            {"id": "T:Microsoft.Extensions.Hosting.BackgroundService", "name": "BackgroundService", "kind": "class", "is_abstract": true, "interfaces": ["T:Microsoft.Extensions.Hosting.IHostedService"]}
    ])
}

fn types() -> serde_json::Value {
    let ctl = "T:Web.OrdersController";
    json!([
            ty("T:Web.Program", "P:Web", None, "Web/Program.cs", json!([])),
            {"id": "T:Web.IOrderService", "project": "P:Web", "namespace": "Web", "name": "IOrderService", "kind": "interface", "accessibility": "public"},
            {"id": "T:Web.OrderService", "project": "P:Web", "namespace": "Web", "name": "OrderService", "kind": "class", "accessibility": "public", "interfaces": ["T:Web.IOrderService"]},
            {"id": "T:Web.IRepository`1", "project": "P:Web", "namespace": "Web", "name": "IRepository", "kind": "interface", "accessibility": "public", "interfaces": ["T:Ardalis.Specification.IRepositoryBase`1"]},
            ty("T:Web.Order", "P:Web", None, "Web/Order.cs", json!([])),
            ty(ctl, "P:Web", Some("T:Microsoft.AspNetCore.Mvc.Controller"), "Web/Controllers/OrdersController.cs",
               json!([attr("T:Microsoft.AspNetCore.Mvc.RouteAttribute", json!(["[controller]"]), json!({})), attr("T:Microsoft.AspNetCore.Authorization.AuthorizeAttribute", json!([]), json!({"Roles": "Administrators"}))])),
            ty("T:Web.Pages.Basket.CheckoutModel", "P:Web", Some("T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel"), "src/Web/Pages/Basket/Checkout.cshtml.cs", json!([])),
            ty("T:Web.Pages.Account.LoginModel", "P:Web", Some("T:Microsoft.AspNetCore.Mvc.RazorPages.PageModel"), "src/Web/Areas/Identity/Pages/Account/Login.cshtml.cs", json!([])),
            ty("T:Web.BasketVC", "P:Web", Some("T:Microsoft.AspNetCore.Mvc.ViewComponent"), "Web/BasketVC.cs", json!([])),
            ty("T:Web.CreateOrderEndpoint", "P:Web", Some("T:FastEndpoints.Endpoint`2"), "Web/CreateOrderEndpoint.cs", json!([])),
            ty("T:Web.Worker", "P:Web", Some("T:Microsoft.Extensions.Hosting.BackgroundService"), "Web/Worker.cs", json!([])),
            ty("T:Tests.PingController", "P:Tests", Some("T:Microsoft.AspNetCore.Mvc.Controller"), "Tests/Ping.cs", json!([]))
    ])
}

fn members() -> serde_json::Value {
    let ctl = "T:Web.OrdersController";
    json!([
            {"id": "M:Web.Program.Main(System.String[])", "declaring_type": "T:Web.Program", "name": "Main", "kind": "method", "accessibility": "public", "is_static": true, "is_entry_point": true},
            method("M:Web.IOrderService.Place(Web.Order)", "T:Web.IOrderService", json!([])),
            {"id": "M:Web.OrderService.#ctor(Web.IRepository{Web.Order})", "declaring_type": "T:Web.OrderService", "name": ".ctor", "kind": "constructor", "accessibility": "public",
             "parameters": [{"name": "repo", "type": "T:Web.IRepository`1", "type_arguments": ["T:Web.Order"]}]},
            method("M:Web.OrderService.Place(Web.Order)", "T:Web.OrderService", json!([])),
            {"id": "M:Web.OrdersController.#ctor(Web.IOrderService)", "declaring_type": ctl, "name": ".ctor", "kind": "constructor", "accessibility": "public", "parameters": [{"name": "s", "type": "T:Web.IOrderService"}]},
            method("M:Web.OrdersController.Get(System.Int32)", ctl, json!([attr("T:Microsoft.AspNetCore.Mvc.HttpGetAttribute", json!(["{id}"]), json!({}))])),
            method("M:Web.OrdersController.Post(Web.Order)", ctl, json!([attr("T:Microsoft.AspNetCore.Mvc.HttpPostAttribute", json!([]), json!({})), attr("T:Microsoft.AspNetCore.Mvc.ValidateAntiForgeryTokenAttribute", json!([]), json!({}))])),
            method("M:Web.OrdersController.Helper", ctl, json!([attr("T:Microsoft.AspNetCore.Mvc.NonActionAttribute", json!([]), json!({}))])),
            method("M:Web.Pages.Basket.CheckoutModel.OnGet", "T:Web.Pages.Basket.CheckoutModel", json!([])),
            method("M:Web.Pages.Basket.CheckoutModel.OnPostAsync", "T:Web.Pages.Basket.CheckoutModel", json!([])),
            method("M:Web.Pages.Basket.CheckoutModel.Helper", "T:Web.Pages.Basket.CheckoutModel", json!([])),
            method("M:Web.BasketVC.InvokeAsync", "T:Web.BasketVC", json!([])),
            method("M:Web.CreateOrderEndpoint.Configure", "T:Web.CreateOrderEndpoint", json!([])),
            method("M:Tests.PingController.Get", "T:Tests.PingController", json!([attr("T:Microsoft.AspNetCore.Mvc.HttpGetAttribute", json!([]), json!({}))]))
    ])
}

fn references() -> serde_json::Value {
    json!([
            {"from": "T:Web.OrderService", "to": "T:Web.IOrderService", "kind": "implement"},
            {"from": "M:Web.OrderService.Place(Web.Order)", "to": "M:Ardalis.Specification.IRepositoryBase`1.AddAsync(`0,System.Threading.CancellationToken)", "kind": "call"},
            {"from": "M:Web.OrdersController.Get(System.Int32)", "to": "M:Web.IOrderService.Place(Web.Order)", "kind": "call"},
            {"from": "M:Web.OrdersController.Get(System.Int32)", "to": "P:Microsoft.AspNetCore.Mvc.ControllerBase.User", "kind": "access"},
            {"from": "M:Web.OrdersController.Post(Web.Order)", "to": "M:Web.IOrderService.Place(Web.Order)", "kind": "call"},
            {"from": "M:Web.CreateOrderEndpoint.Configure", "to": "M:FastEndpoints.Endpoint`2.Post(System.String[])", "kind": "call"},
            {"from": "M:Web.CreateOrderEndpoint.Configure", "to": "M:FastEndpoints.Endpoint`2.Roles(System.String[])", "kind": "call"},
            {"from": "M:Web.CreateOrderEndpoint.Configure", "to": "F:Web.Roles.ADMIN", "kind": "access"}
    ])
}

/// A solution with one of each kind, a test project, DI through `Main`, a repository write.
fn inventory() -> Inventory {
    let v = json!({
        "inventory_version": "6", "produced_by": {"tool": "t", "tool_version": "0"}, "produced_at": "now", "solution": {"path": "s.sln"},
        "projects": [
            {"id": "P:Web", "name": "Web", "path": "Web/Web.csproj", "razor_files": 3, "razor_inject_directives": 1},
            {"id": "P:Tests", "name": "Tests", "path": "Tests/Tests.csproj", "referenced_assemblies": ["xunit.core"]}
        ],
        "external_types": externals(), "types": types(), "members": members(), "references": references(),
        "registrations": [
            {"site": "M:Web.Program.Main(System.String[])", "receiver": "T:Microsoft.Extensions.DependencyInjection.IServiceCollection",
             "method": "M:Microsoft.Extensions.DependencyInjection.ServiceCollectionServiceExtensions.AddScoped``2(Microsoft.Extensions.DependencyInjection.IServiceCollection)",
             "method_name": "AddScoped", "type_arguments": ["T:Web.IOrderService", "T:Web.OrderService"]}
        ],
        "diagnostics": []
    });
    serde_json::from_value(v).expect("inventory")
}

#[test]
fn handler_rule_is_the_frameworks() {
    assert_eq!(handler_of("OnGet"), Some(("GET", "")));
    assert_eq!(handler_of("OnPostUpdateAsync"), Some(("POST", "Update")));
    assert_eq!(handler_of("OnPostAsync"), Some(("POST", "")));
    assert_eq!(handler_of("Ongoing"), None);
    assert_eq!(handler_of("Helper"), None);
}

#[test]
fn page_path_follows_the_file() {
    assert_eq!(page_path("src/Web/Pages/Basket/Checkout.cshtml.cs").as_deref(), Some("/Basket/Checkout"));
    assert_eq!(page_path("src/Web/Areas/Identity/Pages/Account/Login.cshtml.cs").as_deref(), Some("/Identity/Account/Login"));
    assert_eq!(page_path("src/Web/Controllers/X.cs"), None);
}

#[test]
fn one_candidate_per_integration_point_by_kind() {
    let r = candidates(&inventory(), None);
    assert_eq!(r.count, 8, "{:?}", r.candidates.iter().map(|c| &c.id).collect::<Vec<_>>());
    assert_eq!(r.by_kind["controller-action"], 2, "Helper carries [NonAction]; the test project's controller is excluded");
    assert_eq!(r.by_kind["razor-page-handler"], 3, "OnGet, OnPostAsync, and LoginModel's render");
    assert_eq!(r.by_kind["view-component"], 1);
    assert_eq!(r.by_kind["fastendpoints"], 1);
    assert_eq!(r.by_kind["hosted-service"], 1);
    assert!(r.candidates.iter().all(|c| c.project == "P:Web"));
}

#[test]
fn controller_actions_carry_route_method_and_observed_positions() {
    let r = candidates(&inventory(), None);
    let get = r.candidates.iter().find(|c| c.id == "controller-action:OrdersController@Web.Get#GET").expect("Get");
    assert_eq!(get.path, "/Orders/{id}");
    assert_eq!(get.path_source, "attribute template: [controller]/{id}");
    assert_eq!(get.observed.authorisation, vec!["Authorize on type (Roles=Administrators)"]);
    assert!(!get.observed.anti_forgery);
    assert_eq!(get.observed.identity_checks, vec!["OrdersController.Get → ControllerBase.User"]);
    let post = r.candidates.iter().find(|c| c.id == "controller-action:OrdersController@Web.Post#POST").expect("Post");
    assert_eq!(post.path, "/Orders");
    assert!(post.observed.anti_forgery);
}

#[test]
fn the_path_resolves_through_the_hosts_sites_and_reads_facts_by_proxy() {
    let r = candidates(&inventory(), None);
    let get = r.candidates.iter().find(|c| c.member == "Get" && c.kind == Kind::ControllerAction).expect("Get");
    assert!(get.path_types.contains(&"T:Web.OrderService".to_string()), "IOrderService resolved through Main's registration: {:?}", get.path_types);
    assert_eq!(get.path_edges.get("resolved"), Some(&1));
    assert_eq!(get.unscored, 0);
    assert_eq!(get.facts.read, vec!["T:Web.Order"]);
    assert_eq!(get.facts.written, vec!["T:Web.Order"], "OrderService calls AddAsync on its one write-capable repository");
    assert_eq!(r.hosts.len(), 1);
    assert_eq!(r.hosts[0].project, "P:Web");
}

#[test]
fn pages_view_components_endpoints_and_hosted_services() {
    let r = candidates(&inventory(), None);
    let post = r.candidates.iter().find(|c| c.id == "razor-page-handler:CheckoutModel@Basket.OnPostAsync#POST").expect("OnPostAsync");
    assert_eq!(post.path, "/Basket/Checkout");
    let render = r.candidates.iter().find(|c| c.member == "(render)").expect("render");
    assert_eq!((render.path.as_str(), render.method.as_str()), ("/Identity/Account/Login", "GET"));
    assert!(r.candidates.iter().any(|c| c.id == "view-component:BasketVC@Web.InvokeAsync#view-component"));
    let fe = r.candidates.iter().find(|c| c.kind == Kind::FastEndpoints).expect("endpoint");
    assert_eq!(fe.method, "POST");
    assert_eq!(fe.path, "not read (L-EP-2)");
    assert!(fe.identity.starts_with("incomplete — missing: route"), "{}", fe.identity);
    assert!(r.candidates.iter().filter(|c| c.kind != Kind::FastEndpoints).all(|c| c.identity == "complete"));
    assert!(r.limits.iter().any(|l| l.id == "L-EP-4" && l.incidence.starts_with("1 of 1 overlapping pairs")), "{:?}", r.limits.iter().map(|l| &l.incidence).collect::<Vec<_>>());
    assert_eq!(fe.observed.authorisation, vec!["Roles(..) called in CreateOrderEndpoint.Configure"]);
    assert_eq!(fe.observed.argument_symbols, vec!["F:Web.Roles.ADMIN"]);
    assert!(r.candidates.iter().any(|c| c.kind == Kind::HostedService && c.type_id == "T:Web.Worker"));
    assert_eq!(r.proxies[0].incidence, "2 public PageModel methods match the rule, 1 do not");
}

#[test]
fn ground_slots_stay_unfilled_and_the_determination_empty() {
    let r = candidates(&inventory(), None);
    for c in &r.candidates {
        assert!(c.ground.expected_actor_kinds.is_none() && c.ground.population_order_of_magnitude.is_none() && c.ground.rate_order_of_magnitude.is_none());
        assert!(c.supported_throughput.sustained.is_none() && c.supported_throughput.behaviour_above_limit.is_none());
    }
    let v = serde_json::to_value(&r).expect("json");
    assert!(v["candidates"][0]["ground"]["expected_actor_kinds"].is_null(), "unfilled serialises as null, never as an empty list");
}

#[test]
fn overlaps_are_reported_not_decided() {
    let r = candidates(&inventory(), None);
    let both = ["controller-action:OrdersController@Web.Get#GET", "controller-action:OrdersController@Web.Post#POST"];
    assert!(r.overlaps.identical.iter().any(|g| both.iter().all(|b| g.contains(&b.to_string()))), "{:?}", r.overlaps.identical);
    assert!(r.overlaps.shared_types.contains(&("T:Web.OrderService".to_string(), 2)));
    assert!(r.overlaps.pairs.iter().any(|p| p.jaccard == 1.0));
    assert!(r.overlaps.cross_type_pairs.is_empty(), "the two actions share a type; nothing crosses types here");
}

#[test]
fn recall_counts_the_not_visible_in_the_denominator() {
    let truth: EntryPointTruth = serde_yaml::from_str(
        "entry_points:\n  - {kind: controller-action, type: OrdersController, member: Get, method: GET}\n  - {kind: fastendpoints, type: CreateOrderEndpoint@Web, member: HandleAsync, method: POST}\n  - {kind: controller-action, type: OrdersController, member: Delete, method: DELETE}\nnot_visible:\n  - {kind: health-check, type: (none), member: MapHealthChecks, method: GET, why: L-EP-1}\n",
    )
    .expect("truth");
    let r = candidates(&inventory(), Some(&truth));
    let rc = r.recall.expect("recall");
    assert_eq!((rc.hand, rc.matched, rc.not_visible), (4, 2, 1));
    assert_eq!(rc.misses.len(), 2);
    assert_eq!(rc.extras.len(), 6);
    assert!((rc.recall_percent - 50.0).abs() < 1e-9);
}
