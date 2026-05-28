---
id: TC-872
title: unknown route returns 404 with rustc-style error JSON
type: scenario
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_872_unknown_route_returns_404_with_rustc_style_error_json
observes:
- mcp-response
---

**observes:** [mcp-response]

`GET /does/not/exist` → `404`. Browser routes (`Accept: text/html`)
return an HTML error page that includes the rustc-style block: an
error code header, the requested path, and a `hint` to visit `/`.

JSON routes (`Accept: application/json` or path starting with `/api/`)
return the ADR-013 envelope:

```json
{
  "errors": [
    {
      "code": "E***",
      "tier": "...",
      "message": "route not found",
      "detail": "no handler for GET /does/not/exist",
      "hint": "see / for the dashboard root"
    }
  ],
  "warnings": [],
  "summary": { "errors": 1, "warnings": 0 }
}
```

Surface:
- **mcp-response:** `404` body matches ADR-013 shape per content type.
