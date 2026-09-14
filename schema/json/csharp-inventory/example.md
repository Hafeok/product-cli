# C# inventory — example

The conformant example is the committed fixture inventory,
`product-cli/tests/fixtures/csharp-inventory/inventory.json`, produced by `tools/csharp-inventory`
over the fixture solution beside it. An excerpt, one type and one member:

```json
{
  "inventory_version": "3",
  "types": [
    {
      "id": "T:Shop.Api.Orders.PlaceOrderHandler",
      "project": "P:Shop.Api",
      "namespace": "Shop.Api.Orders",
      "name": "PlaceOrderHandler",
      "kind": "class",
      "accessibility": "public",
      "is_static": false, "is_abstract": false, "is_partial": false,
      "base_type": "T:System.Object",
      "interfaces": ["T:Shop.Api.Infrastructure.IHandler`1"],
      "file": "Shop.Api/Orders.cs", "line": 13,
      "attributes": [
        { "type": "T:Product.Binding.SliceAttribute",
          "positional": ["PlaceOrder", "handler"],
          "named": { "Profile": "rest-api-v1" } }
      ]
    }
  ],
  "members": [
    {
      "id": "M:Shop.Api.Orders.PlaceOrderHandler.Handle(Shop.Api.Orders.PlaceOrderCommand)",
      "declaring_type": "T:Shop.Api.Orders.PlaceOrderHandler",
      "name": "Handle", "kind": "method", "accessibility": "public",
      "is_static": false, "is_entry_point": false,
      "parameters": [ { "name": "command", "type": "T:Shop.Api.Orders.PlaceOrderCommand" } ],
      "return_type": null,
      "file": "Shop.Api/Orders.cs", "line": 24,
      "attributes": []
    }
  ],
  "references": [
    { "from": "M:Shop.Api.Orders.PlaceOrderHandler.Handle(Shop.Api.Orders.PlaceOrderCommand)",
      "to": "T:Shop.Domain.OrderPlaced", "kind": "construct" },
    { "from": "M:Shop.Api.Orders.OrdersEndpoints.#ctor(…)",
      "to": "T:System.Collections.Generic.IComparer`1", "kind": "parameter" }
  ],
  "external_types": [
    { "id": "T:System.Collections.Generic.IComparer`1", "assembly": "System.Runtime",
      "namespace": "System.Collections.Generic", "name": "IComparer", "kind": "interface",
      "is_abstract": false, "arity": 1, "methods": 1, "properties": 0, "events": 0, "abstract_returns": [] }
  ],
  "registrations": [
    { "site": "M:Shop.Api.Program.Main(System.String[])",
      "method": "M:Microsoft.Extensions.DependencyInjection.ServiceCollectionServiceExtensions.AddScoped``2(Microsoft.Extensions.DependencyInjection.IServiceCollection)",
      "method_name": "AddScoped",
      "type_arguments": ["T:Shop.Api.Persistence.IOrderRepository", "T:Shop.Api.Persistence.OrderRepository"],
      "typeof_arguments": [], "constructs": [], "has_lambda": false, "conditional": false,
      "file": "Shop.Api/Program.cs", "line": 13 }
  ]
}
```

Facts only. The attribute is recorded because it is declared, with its arguments as the compiler
resolved them; whether `"PlaceOrder"` names an act, or `"handler"` a role, is decided Rust-side
against the event model and the profile store — never here. The registration is recorded because
the call was written; that it registers `OrderRepository` for `IOrderRepository`, and that the
container would supply it where the walk reaches the interface, is the Rust resolver's reading.
The `parameter` edge is recorded because the constructor declares it; that it is a composition
edge (the type is container-constructed) and that `IComparer<T>` is a service rather than a
data contract is the consumer's, through the declared proxies in `pf::csharp_roles`.
