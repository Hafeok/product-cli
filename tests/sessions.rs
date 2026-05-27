//! Session-based integration test suite (FT-043, ADR-018 amended).
//! Run with: cargo test --test sessions

#[path = "sessions/harness.rs"]
mod harness;

#[path = "sessions/harness_self_tests.rs"]
mod harness_self_tests;

#[path = "sessions/repo_scaffold.rs"]
mod repo_scaffold;

#[path = "sessions/st_001_create_feature_with_adr_and_tc.rs"]
mod st_001_create_feature_with_adr_and_tc;

#[path = "sessions/st_002_create_dep_requires_governing_adr.rs"]
mod st_002_create_dep_requires_governing_adr;

#[path = "sessions/st_003_create_dep_with_adr_in_same_request.rs"]
mod st_003_create_dep_with_adr_in_same_request;

#[path = "sessions/st_004_create_with_forward_references.rs"]
mod st_004_create_with_forward_references;

#[path = "sessions/st_005_create_multiple_adrs_same_phase.rs"]
mod st_005_create_multiple_adrs_same_phase;

#[path = "sessions/st_006_create_cross_links_bidirectional.rs"]
mod st_006_create_cross_links_bidirectional;

// Change operations (ST-010..ST-015) — ADR-018 Design 2
#[path = "sessions/st_010_change_append_domain.rs"]
mod st_010_change_append_domain;

#[path = "sessions/st_011_change_set_acknowledgement.rs"]
mod st_011_change_set_acknowledgement;

#[path = "sessions/st_012_change_invalid_target.rs"]
mod st_012_change_invalid_target;

#[path = "sessions/st_013_change_body_mutation.rs"]
mod st_013_change_body_mutation;

#[path = "sessions/st_014_change_remove_from_array.rs"]
mod st_014_change_remove_from_array;

#[path = "sessions/st_015_change_append_deduplicates.rs"]
mod st_015_change_append_deduplicates;

#[path = "sessions/st_020_failed_apply_leaves_zero_files.rs"]
mod st_020_failed_apply_leaves_zero_files;

#[path = "sessions/st_021_failed_apply_mid_write_recovery.rs"]
mod st_021_failed_apply_mid_write_recovery;

#[path = "sessions/st_022_concurrent_apply_serialised.rs"]
mod st_022_concurrent_apply_serialised;

#[path = "sessions/st_030_validation_e013_dep_no_adr.rs"]
mod st_030_validation_e013_dep_no_adr;

#[path = "sessions/st_031_validation_e002_broken_ref.rs"]
mod st_031_validation_e002_broken_ref;

#[path = "sessions/st_032_validation_e003_dep_cycle.rs"]
mod st_032_validation_e003_dep_cycle;

#[path = "sessions/st_033_validation_e012_unknown_domain.rs"]
mod st_033_validation_e012_unknown_domain;

#[path = "sessions/st_034_validation_e011_empty_acknowledgement.rs"]
mod st_034_validation_e011_empty_acknowledgement;

#[path = "sessions/st_035_validation_domain_not_in_vocabulary.rs"]
mod st_035_validation_domain_not_in_vocabulary;

#[path = "sessions/exit_criteria.rs"]
mod exit_criteria;

// FT-044 — Unified Verify Pipeline
// Phase-gate sessions (ST-040..042) — ADR-018 Design 2 + ADR-040
#[path = "sessions/st_040_phase_gate_blocks_on_failing_exit_criteria.rs"]
mod st_040_phase_gate_blocks_on_failing_exit_criteria;

#[path = "sessions/st_041_phase_gate_opens_after_verify.rs"]
mod st_041_phase_gate_opens_after_verify;

#[path = "sessions/st_042_phase_gate_no_exit_criteria_always_open.rs"]
mod st_042_phase_gate_no_exit_criteria_always_open;

// Verify + lifecycle sessions (ST-050..056) — ADR-018 Design 2 + ADR-021 + ADR-034
#[path = "sessions/st_050_verify_creates_completion_tag.rs"]
mod st_050_verify_creates_completion_tag;

#[path = "sessions/st_051_verify_complete_feature_status.rs"]
mod st_051_verify_complete_feature_status;

#[path = "sessions/st_052_verify_failing_tc_stays_in_progress.rs"]
mod st_052_verify_failing_tc_stays_in_progress;

#[path = "sessions/st_054_drift_check_no_tag_emits_w020.rs"]
mod st_054_drift_check_no_tag_emits_w020;

// Context bundle sessions (ST-060..062) — ADR-018 Design 2 + ADR-006
#[path = "sessions/st_060_context_includes_dependency_section.rs"]
mod st_060_context_includes_dependency_section;

#[path = "sessions/st_061_context_depth_2_includes_shared_adrs.rs"]
mod st_061_context_depth_2_includes_shared_adrs;

#[path = "sessions/st_062_context_measure_writes_bundle_block.rs"]
mod st_062_context_measure_writes_bundle_block;

#[path = "sessions/st_110_verify_pipeline.rs"]
mod st_110_verify_pipeline;

// FT-045 — LLM Boundary — Semantic Analysis Bundles
#[path = "sessions/st_120_gap_bundle.rs"]
mod st_120_gap_bundle;

#[path = "sessions/st_126_drift_diff.rs"]
mod st_126_drift_diff;

#[path = "sessions/st_131_conflict_bundle.rs"]
mod st_131_conflict_bundle;

// FT-046 — MCP Parity for ADR Lifecycle Operations
#[path = "sessions/st_140_mcp_adr_lifecycle.rs"]
mod st_140_mcp_adr_lifecycle;

// FT-047 — Removal & Deprecation Tracking
#[path = "sessions/st_147_removal_deprecation.rs"]
mod st_147_removal_deprecation;

// FT-052 — Product Request Builder
#[path = "sessions/ft_052_builder.rs"]
mod ft_052_builder;

// FT-057 — Consolidate Product CLI State Under .product/ Folder
#[path = "sessions/st_700_migrate_consolidate.rs"]
mod st_700_migrate_consolidate;

#[path = "sessions/st_701_discover_canonical_alias_legacy.rs"]
mod st_701_discover_canonical_alias_legacy;

#[path = "sessions/st_702_ft057_exit_criteria.rs"]
mod st_702_ft057_exit_criteria;

// FT-061 — MCP server and CLI honor `.product/config.toml` discovery
#[path = "sessions/st_761_mcp_canonical_layout.rs"]
mod st_761_mcp_canonical_layout;

// FT-062 — MCP parity for feature `depends-on` and strict request shape validation
#[path = "sessions/ft_062_depends_on_and_strict_validation.rs"]
mod ft_062_depends_on_and_strict_validation;

// FT-064 — strict change-spec validation and artifact deletion surface
#[path = "sessions/ft_064_strict_change_spec_and_deletion.rs"]
mod ft_064_strict_change_spec_and_deletion;

// FT-066 — MCP parity for feature/TC status writes and reciprocal linking
#[path = "sessions/ft_066_mcp_status_and_link_parity.rs"]
mod ft_066_mcp_status_and_link_parity;

// FT-069 — MCP parity for `product_graph_check`
#[path = "sessions/ft_069_mcp_graph_check_parity.rs"]
mod ft_069_mcp_graph_check_parity;
