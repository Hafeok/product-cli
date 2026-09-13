# C# inventory — example

The conformant example is the committed fixture inventory,
`product-cli/tests/fixtures/csharp-inventory/inventory.json`, produced by `tools/csharp-inventory`
over the fixture solution beside it. An excerpt, one type and one member:

```json
{
  "inventory_version": "1",
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
      "to": "T:Shop.Domain.OrderPlaced", "kind": "construct" }
  ]
}
```

Facts only. The attribute is recorded because it is declared, with its arguments as the compiler
resolved them; whether `"PlaceOrder"` names an act, or `"handler"` a role, is decided Rust-side
against the event model and the profile store — never here.
