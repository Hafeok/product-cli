//! Unit tests for codebase onboarding (ADR-027)

use super::*;
use std::io::Cursor;

#[test]
fn test_truncate_str() {
    assert_eq!(evidence::truncate_str("hello", 10), "hello");
    assert_eq!(evidence::truncate_str("hello world", 8), "hello...");
}

#[test]
fn test_extract_import_target() {
    assert_eq!(evidence::extract_import_target("use sqlx::PgPool;"), Some("sqlx".to_string()));
    assert_eq!(evidence::extract_import_target("import os"), Some("os".to_string()));
    assert_eq!(evidence::extract_import_target("from flask import Flask"), Some("flask".to_string()));
    assert_eq!(evidence::extract_import_target("use std::path::Path;"), None); // std excluded
    assert_eq!(evidence::extract_import_target("let x = 5;"), None);
}

#[test]
fn test_validate_evidence_nonexistent_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut candidates = vec![Candidate {
        id: "DC-001".to_string(),
        signal_type: "boundary".to_string(),
        title: "Test".to_string(),
        observation: "Test observation".to_string(),
        evidence: vec![Evidence {
            file: "nonexistent.rs".to_string(),
            line: 1,
            snippet: "test".to_string(),
            evidence_valid: true,
        }],
        hypothesised_consequence: "Bad things".to_string(),
        confidence: "high".to_string(),
        warnings: Vec::new(),
    }];

    validate_all_evidence(dir.path(), &mut candidates);
    assert!(!candidates[0].evidence[0].evidence_valid);
    assert!(!candidates[0].warnings.is_empty());
}

#[test]
fn test_validate_evidence_valid_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("test.rs"), "line 1\nline 2\nline 3\n").expect("write");

    let mut candidates = vec![Candidate {
        id: "DC-001".to_string(),
        signal_type: "boundary".to_string(),
        title: "Test".to_string(),
        observation: "Test observation".to_string(),
        evidence: vec![Evidence {
            file: "test.rs".to_string(),
            line: 2,
            snippet: "line 2".to_string(),
            evidence_valid: true,
        }],
        hypothesised_consequence: "Bad things".to_string(),
        confidence: "high".to_string(),
        warnings: Vec::new(),
    }];

    validate_all_evidence(dir.path(), &mut candidates);
    assert!(candidates[0].evidence[0].evidence_valid);
    assert!(candidates[0].warnings.is_empty());
}

#[test]
fn test_validate_evidence_line_exceeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("test.rs"), "line 1\nline 2\n").expect("write");

    let mut candidates = vec![Candidate {
        id: "DC-001".to_string(),
        signal_type: "boundary".to_string(),
        title: "Test".to_string(),
        observation: "Test observation".to_string(),
        evidence: vec![Evidence {
            file: "test.rs".to_string(),
            line: 99,
            snippet: "does not exist".to_string(),
            evidence_valid: true,
        }],
        hypothesised_consequence: "Bad things".to_string(),
        confidence: "high".to_string(),
        warnings: Vec::new(),
    }];

    validate_all_evidence(dir.path(), &mut candidates);
    assert!(!candidates[0].evidence[0].evidence_valid);
}

#[test]
fn test_triage_confirm() {
    let scan = ScanOutput {
        candidates: vec![Candidate {
            id: "DC-001".to_string(),
            signal_type: "boundary".to_string(),
            title: "Test decision".to_string(),
            observation: "Observed pattern".to_string(),
            evidence: vec![],
            hypothesised_consequence: "Bad things".to_string(),
            confidence: "high".to_string(),
            warnings: Vec::new(),
        }],
        scan_metadata: ScanMetadata {
            files_scanned: 1,
            prompt_version: "test".to_string(),
        },
    };

    let mut input = Cursor::new("c\n");
    let result = triage_interactive(&scan, &mut input).expect("triage");
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].triage_status, TriageStatus::Confirmed);
}

#[test]
fn test_triage_reject() {
    let scan = ScanOutput {
        candidates: vec![Candidate {
            id: "DC-001".to_string(),
            signal_type: "boundary".to_string(),
            title: "Test decision".to_string(),
            observation: "Observed pattern".to_string(),
            evidence: vec![],
            hypothesised_consequence: "Bad things".to_string(),
            confidence: "high".to_string(),
            warnings: Vec::new(),
        }],
        scan_metadata: ScanMetadata {
            files_scanned: 1,
            prompt_version: "test".to_string(),
        },
    };

    let mut input = Cursor::new("r\n");
    let result = triage_interactive(&scan, &mut input).expect("triage");
    assert_eq!(result.candidates[0].triage_status, TriageStatus::Rejected);
}

#[test]
fn test_batch_confirm() {
    let scan = ScanOutput {
        candidates: vec![
            Candidate {
                id: "DC-001".to_string(),
                signal_type: "boundary".to_string(),
                title: "Test 1".to_string(),
                observation: "Obs 1".to_string(),
                evidence: vec![],
                hypothesised_consequence: "Bad 1".to_string(),
                confidence: "high".to_string(),
                warnings: Vec::new(),
            },
            Candidate {
                id: "DC-002".to_string(),
                signal_type: "consistency".to_string(),
                title: "Test 2".to_string(),
                observation: "Obs 2".to_string(),
                evidence: vec![],
                hypothesised_consequence: "Bad 2".to_string(),
                confidence: "medium".to_string(),
                warnings: Vec::new(),
            },
        ],
        scan_metadata: ScanMetadata {
            files_scanned: 2,
            prompt_version: "test".to_string(),
        },
    };

    let result = triage_batch_confirm(&scan);
    assert_eq!(result.candidates.len(), 2);
    assert!(result
        .candidates
        .iter()
        .all(|c| c.triage_status == TriageStatus::Confirmed));
}

#[test]
fn test_plan_seed_ids() {
    let triage = TriageOutput {
        candidates: vec![TriagedCandidate {
            candidate: Candidate {
                id: "DC-001".to_string(),
                signal_type: "boundary".to_string(),
                title: "Test decision".to_string(),
                observation: "Observation".to_string(),
                evidence: vec![Evidence {
                    file: "src/test.rs".to_string(),
                    line: 1,
                    snippet: "test".to_string(),
                    evidence_valid: true,
                }],
                hypothesised_consequence: "Bad".to_string(),
                confidence: "high".to_string(),
                warnings: Vec::new(),
            },
            triage_status: TriageStatus::Confirmed,
            merged_into: None,
        }],
    };

    let result = plan_seed(
        &triage,
        &["ADR-001".to_string(), "ADR-002".to_string()],
        &["FT-001".to_string()],
        "ADR",
        "FT",
    );

    assert_eq!(result.adrs.len(), 1);
    assert_eq!(result.adrs[0].id, "ADR-003");
    assert_eq!(result.features.len(), 1);
    assert_eq!(result.features[0].id, "FT-002");
}

#[test]
fn test_group_into_features_by_directory() {
    let adrs = vec![
        ProposedAdr {
            id: "ADR-001".to_string(),
            title: "API decision".to_string(),
            observation: "obs".to_string(),
            evidence: vec![Evidence {
                file: "src/api/handler.rs".to_string(),
                line: 1,
                snippet: "test".to_string(),
                evidence_valid: true,
            }],
            hypothesised_consequence: "bad".to_string(),
            filename: "ADR-001-api.md".to_string(),
        },
        ProposedAdr {
            id: "ADR-002".to_string(),
            title: "Storage decision".to_string(),
            observation: "obs".to_string(),
            evidence: vec![Evidence {
                file: "src/storage/db.rs".to_string(),
                line: 1,
                snippet: "test".to_string(),
                evidence_valid: true,
            }],
            hypothesised_consequence: "bad".to_string(),
            filename: "ADR-002-storage.md".to_string(),
        },
    ];

    let features = seed::group_into_features(&adrs, &[], "FT");
    assert_eq!(features.len(), 2);
    assert!(features[0].adr_ids.contains(&"ADR-001".to_string()));
    assert!(features[1].adr_ids.contains(&"ADR-002".to_string()));
}
