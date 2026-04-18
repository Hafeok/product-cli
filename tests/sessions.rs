//! Session-based integration test suite (FT-043, ADR-018 amended).
//! Run with: cargo test --test sessions

#[path = "sessions/harness.rs"]
mod harness;

#[path = "sessions/harness_self_tests.rs"]
mod harness_self_tests;

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
#[path = "sessions/st_110_verify_pipeline.rs"]
mod st_110_verify_pipeline;
