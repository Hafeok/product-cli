//! Front-matter schema definitions for all artifact types (ADR-031)

/// Generate human-readable schema for a specific artifact type.
pub fn generate_schema(artifact_type: &str) -> Result<String, String> {
    match artifact_type {
        "feature" => Ok(feature_schema()),
        "adr" => Ok(adr_schema()),
        "test" => Ok(test_schema()),
        "dep" | "dependency" => Ok(dep_schema()),
        _ => Err(format!(
            "Unknown artifact type: '{}'. Supported: feature, adr, test, dep",
            artifact_type
        )),
    }
}

/// Generate all schemas as a single document.
pub fn generate_all_schemas() -> String {
    let mut out = String::new();
    out.push_str("# Front-Matter Schemas\n\n");
    out.push_str("## Feature\n\n");
    out.push_str(&feature_schema());
    out.push_str("\n\n## ADR\n\n");
    out.push_str(&adr_schema());
    out.push_str("\n\n## Test Criterion\n\n");
    out.push_str(&test_schema());
    out.push_str("\n\n## Dependency\n\n");
    out.push_str(&dep_schema());
    out
}

pub(crate) fn feature_schema() -> String {
    r#"```yaml
id: String           # Required. Format: FT-NNN (e.g. FT-001)
title: String        # Required. Human-readable feature name
phase: Integer       # Default: 1. Delivery phase number
status: Enum         # Default: planned. Values: planned, in-progress, complete, abandoned
depends-on: [String] # Default: []. Feature IDs this feature depends on
adrs: [String]       # Default: []. ADR IDs implementing this feature
tests: [String]      # Default: []. Test criterion IDs validating this feature
domains: [String]    # Default: []. Concern domain names this feature touches
domains-acknowledged: # Default: {}. Map of domain -> reasoning for acknowledged gaps
  domain-name: String
bundle:              # Optional. Written by `product context --measure`
  depth-1-adrs: Integer
  tcs: Integer
  domains: [String]
  tokens-approx: Integer
  measured-at: String  # ISO 8601 timestamp
```"#
    .to_string()
}

pub(crate) fn adr_schema() -> String {
    r#"```yaml
id: String             # Required. Format: ADR-NNN (e.g. ADR-001)
title: String          # Required. Human-readable decision name
status: Enum           # Default: proposed. Values: proposed, accepted, superseded, abandoned
features: [String]     # Default: []. Feature IDs this ADR implements
supersedes: [String]   # Default: []. ADR IDs this decision supersedes
superseded-by: [String] # Default: []. ADR IDs that supersede this decision
domains: [String]      # Default: []. Concern domains this ADR governs
scope: Enum            # Default: feature-specific. Values: cross-cutting, domain, feature-specific
content-hash: String   # Optional. SHA-256 hash for immutability enforcement
amendments:            # Default: []. Audit trail for approved changes
  - date: String       # ISO 8601 date
    reason: String     # Why the amendment was made
    previous-hash: String # Hash before amendment
source-files: [String] # Default: []. Source files governed by this ADR
```"#
    .to_string()
}

pub(crate) fn test_schema() -> String {
    test_schema_with_config(None)
}

/// ADR-042: render TC schema with structural / built-in / custom partition.
pub fn test_schema_with_config(config: Option<&crate::config::ProductConfig>) -> String {
    let custom_line = match config {
        Some(c) if !c.tc_types.custom.is_empty() => {
            let names = c.tc_types.custom.join(" | ");
            format!(
                "                     # Custom (this project): {}\n",
                names
            )
        }
        _ => String::from(
            "                     # Custom (this project): (none configured)\n",
        ),
    };
    format!(
        r#"```yaml
id: String           # Required. Format: TC-NNN (e.g. TC-001)
title: String        # Required. Human-readable test criterion name
type: Enum           # Default: scenario.
                     # Structural: exit-criteria | invariant | chaos | absence
                     # Built-in descriptive: scenario | benchmark
{custom}status: Enum         # Default: unimplemented. Values: unimplemented, implemented, passing, failing, unrunnable
validates:           # Default: empty
  features: [String] # Feature IDs this test validates
  adrs: [String]     # ADR IDs this test validates
phase: Integer       # Default: 1. Phase this test belongs to
content-hash: String # Optional. SHA-256 hash for immutability enforcement
runner: String       # Optional. Test runner name (e.g. cargo-test)
runner-args: String  # Optional. Arguments for the test runner
runner-timeout: Integer # Optional. Timeout in seconds
requires: [String]   # Default: []. TC IDs that must pass before this TC
last-run: String     # Optional. ISO 8601 timestamp of last run
failure-message: String # Optional. Last failure message
last-run-duration: Float # Optional. Last run duration in seconds
```"#,
        custom = custom_line
    )
}

pub fn generate_schema_with_config(
    artifact_type: &str,
    config: Option<&crate::config::ProductConfig>,
) -> Result<String, String> {
    match artifact_type {
        "feature" => Ok(feature_schema()),
        "adr" => Ok(adr_schema()),
        "test" => Ok(test_schema_with_config(config)),
        "dep" | "dependency" => Ok(dep_schema()),
        _ => Err(format!(
            "Unknown artifact type: '{}'. Supported: feature, adr, test, dep",
            artifact_type
        )),
    }
}

pub fn generate_all_schemas_with_config(config: Option<&crate::config::ProductConfig>) -> String {
    let mut out = String::new();
    out.push_str("# Front-Matter Schemas\n\n");
    out.push_str("## Feature\n\n");
    out.push_str(&feature_schema());
    out.push_str("\n\n## ADR\n\n");
    out.push_str(&adr_schema());
    out.push_str("\n\n## Test Criterion\n\n");
    out.push_str(&test_schema_with_config(config));
    out.push_str("\n\n## Dependency\n\n");
    out.push_str(&dep_schema());
    out
}

pub(crate) fn dep_schema() -> String {
    r#"```yaml
id: String                # Required. Format: DEP-NNN (e.g. DEP-001)
title: String             # Required. Human-readable dependency name
type: Enum                # Default: library. Values: library, service, api, tool, hardware, runtime
source: String            # Optional. Package repository URL or identifier
version: String           # Optional. Version constraint
status: Enum              # Default: active. Values: active, evaluating, deprecated, migrating
features: [String]        # Default: []. Feature IDs that use this dependency
adrs: [String]            # Default: []. ADR IDs governing this dependency
availability-check: String # Optional. Script or command to verify availability
breaking-change-risk: Enum # Default: low. Values: low, medium, high
supersedes: [String]      # Default: []. Deprecated dependency IDs this replaces
interface:                # Optional. Required for service and api types
  protocol: String        # e.g. http, grpc, mqtt
  port: Integer           # Service port number
  auth: String            # Authentication method
  connection-string-env: String # Environment variable for connection string
  health-endpoint: String # Health check endpoint path
  base-url: String        # API base URL
  auth-env: String        # Environment variable for auth credentials
  rate-limit: String      # Rate limit description
  error-model: String     # Error response model description
  arch: String            # Hardware architecture (for hardware type)
  storage-min-gb: Integer # Minimum storage in GB (for hardware type)
  storage-device-pattern: String # Storage device glob pattern (for hardware type)
```

### Dependency Types

| Type | Description |
|---|---|
| `library` | Code dependency linked at build time |
| `service` | External service accessed over network (requires `interface`) |
| `api` | External API accessed over HTTP/gRPC (requires `interface`) |
| `tool` | Build or development tool |
| `hardware` | Physical hardware requirement |
| `runtime` | Runtime platform requirement |"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_feature_contains_all_fields() {
        let schema = feature_schema();
        for field in &[
            "id:", "title:", "phase:", "status:", "depends-on:", "adrs:",
            "tests:", "domains:", "domains-acknowledged:", "bundle:",
        ] {
            assert!(schema.contains(field), "Feature schema missing field: {}", field);
        }
    }

    #[test]
    fn schema_adr_contains_all_fields() {
        let schema = adr_schema();
        for field in &[
            "id:", "title:", "status:", "features:", "supersedes:",
            "superseded-by:", "domains:", "scope:", "source-files:",
        ] {
            assert!(schema.contains(field), "ADR schema missing field: {}", field);
        }
    }

    #[test]
    fn schema_test_contains_all_fields() {
        let schema = test_schema();
        for field in &[
            "id:", "title:", "type:", "status:", "validates:", "phase:",
            "runner:", "runner-args:",
        ] {
            assert!(schema.contains(field), "Test schema missing field: {}", field);
        }
    }

    #[test]
    fn schema_dep_contains_all_types() {
        let schema = dep_schema();
        for dep_type in &["library", "service", "api", "tool", "hardware", "runtime"] {
            assert!(schema.contains(dep_type), "Dep schema missing type: {}", dep_type);
        }
        assert!(schema.contains("interface:"), "Should document interface block");
        assert!(schema.contains("availability-check:"), "Should document availability-check");
    }

    #[test]
    fn generate_schema_rejects_unknown_type() {
        assert!(generate_schema("unknown").is_err());
    }

    #[test]
    fn all_schemas_contains_four_types() {
        let all = generate_all_schemas();
        assert!(all.contains("## Feature"));
        assert!(all.contains("## ADR"));
        assert!(all.contains("## Test Criterion"));
        assert!(all.contains("## Dependency"));
    }
}
