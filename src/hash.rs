//! Content hash immutability enforcement (ADR-032, FT-034)
//!
//! Computes SHA-256 hashes over protected artifact content, verifies integrity,
//! and provides amendment/sealing operations.

use crate::error::{CheckResult, Diagnostic, ProductError, Result};
use crate::types::{Adr, AdrStatus, Amendment, TestCriterion};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Hash computation
// ---------------------------------------------------------------------------

/// Normalize body text for hashing: LF line endings, trim leading/trailing whitespace.
fn normalize_body(body: &str) -> String {
    body.replace("\r\n", "\n").trim().to_string()
}

/// Compute content hash for an ADR.
/// Hash input: title + normalized body text.
pub fn compute_adr_hash(title: &str, body: &str) -> String {
    let normalized = normalize_body(body);
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(b"\n");
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

/// Compute content hash for a TC.
/// Hash input: title + type + validates.adrs + normalized body text.
pub fn compute_tc_hash(tc: &TestCriterion) -> String {
    let normalized = normalize_body(&tc.body);
    let mut hasher = Sha256::new();
    hasher.update(tc.front.title.as_bytes());
    hasher.update(b"\n");
    hasher.update(tc.front.test_type.to_string().as_bytes());
    hasher.update(b"\n");
    // Sort ADRs for deterministic hash
    let mut adrs = tc.front.validates.adrs.clone();
    adrs.sort();
    hasher.update(adrs.join(",").as_bytes());
    hasher.update(b"\n");
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

/// Verify content hashes for all ADRs and TCs. Returns a CheckResult with E014, E015, W016.
pub fn verify_all(adrs: &[&Adr], tests: &[&TestCriterion]) -> CheckResult {
    let mut result = CheckResult::new();

    for adr in adrs {
        verify_adr(adr, &mut result);
    }

    for tc in tests {
        verify_tc(tc, &mut result);
    }

    result
}

/// Verify a single ADR's content hash.
fn verify_adr(adr: &Adr, result: &mut CheckResult) {
    if adr.front.status != AdrStatus::Accepted {
        return;
    }

    match &adr.front.content_hash {
        Some(expected) => {
            let actual = compute_adr_hash(&adr.front.title, &adr.body);
            if *expected != actual {
                result.errors.push(
                    Diagnostic::error(
                        "E014",
                        "content-hash mismatch \u{2014} accepted ADR body or title was modified",
                    )
                    .with_file(adr.path.clone())
                    .with_context(&format!("content-hash: {} (expected)", expected))
                    .with_detail(&format!("recomputed:   {} (actual)", actual))
                    .with_hint(
                        "revert the change, or run `product adr amend` with --reason to record a legitimate amendment",
                    ),
                );
            }
        }
        None => {
            result.warnings.push(
                Diagnostic::warning(
                    "W016",
                    "accepted ADR has no content-hash",
                )
                .with_file(adr.path.clone())
                .with_detail(&format!(
                    "{} is accepted but has no content-hash \u{2014} seal with `product adr rehash {}`",
                    adr.front.id, adr.front.id
                ))
                .with_hint(&format!(
                    "run `product adr rehash {}` to seal it",
                    adr.front.id
                )),
            );
        }
    }
}

/// Verify a single TC's content hash.
fn verify_tc(tc: &TestCriterion, result: &mut CheckResult) {
    if let Some(ref expected) = tc.front.content_hash {
        let actual = compute_tc_hash(tc);
        if *expected != actual {
            result.errors.push(
                Diagnostic::error(
                    "E015",
                    "content-hash mismatch \u{2014} sealed TC body or protected fields were modified",
                )
                .with_file(tc.path.clone())
                .with_context(&format!("content-hash: {} (expected)", expected))
                .with_detail(&format!("recomputed:   {} (actual)", actual))
                .with_hint(
                    "revert the change, or create a new TC if the specification has fundamentally changed",
                ),
            );
        }
    }
    // TCs without content-hash are unsealed drafts — no warning
}

// ---------------------------------------------------------------------------
// Sealing operations
// ---------------------------------------------------------------------------

/// Seal an ADR by computing and writing its content hash.
/// Returns the hash string written.
pub fn seal_adr(adr: &Adr) -> Result<String> {
    if adr.front.status != AdrStatus::Accepted {
        return Err(ProductError::ConfigError(format!(
            "{} has status '{}', not 'accepted' \u{2014} only accepted ADRs can be sealed",
            adr.front.id, adr.front.status
        )));
    }
    Ok(compute_adr_hash(&adr.front.title, &adr.body))
}

/// Seal a TC by computing its content hash.
/// Returns the hash string.
pub fn seal_tc(tc: &TestCriterion) -> String {
    compute_tc_hash(tc)
}

/// Record an amendment to an accepted ADR.
/// Returns the updated front-matter fields (new hash, new amendment entry).
pub fn amend_adr(
    adr: &Adr,
    reason: &str,
) -> Result<(String, Amendment)> {
    if adr.front.status != AdrStatus::Accepted {
        return Err(ProductError::ConfigError(format!(
            "{} has status '{}', not 'accepted' \u{2014} only accepted ADRs can be amended",
            adr.front.id, adr.front.status
        )));
    }

    let previous_hash = adr
        .front
        .content_hash
        .as_ref()
        .ok_or_else(|| {
            ProductError::ConfigError(format!(
                "{} has no content-hash \u{2014} run `product adr rehash {}` first",
                adr.front.id, adr.front.id
            ))
        })?
        .clone();

    let new_hash = compute_adr_hash(&adr.front.title, &adr.body);

    if new_hash == previous_hash {
        return Err(ProductError::ConfigError(
            "nothing to amend \u{2014} content-hash matches current body".to_string(),
        ));
    }

    let amendment = Amendment {
        date: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        reason: reason.to_string(),
        previous_hash,
    };

    Ok((new_hash, amendment))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::path::PathBuf;

    fn make_adr(title: &str, body: &str, status: AdrStatus, hash: Option<&str>) -> Adr {
        Adr {
            front: AdrFrontMatter {
                id: "ADR-001".to_string(),
                title: title.to_string(),
                status,
                features: vec![],
                supersedes: vec![],
                superseded_by: vec![],
                domains: vec![],
                scope: AdrScope::FeatureSpecific,
                content_hash: hash.map(String::from),
                amendments: vec![],
                source_files: vec![],
                removes: vec![],
                deprecates: vec![],
            },
            body: body.to_string(),
            path: PathBuf::from("ADR-001.md"),
        }
    }

    fn make_tc(title: &str, body: &str, hash: Option<&str>) -> TestCriterion {
        TestCriterion {
            front: TestFrontMatter {
                id: "TC-001".to_string(),
                title: title.to_string(),
                test_type: TestType::Scenario,
                status: TestStatus::Unimplemented,
                validates: ValidatesBlock {
                    features: vec![],
                    adrs: vec!["ADR-001".to_string()],
                },
                phase: 1,
                content_hash: hash.map(String::from),
                runner: None,
                runner_args: None,
                runner_timeout: None,
                requires: vec![],
                last_run: None,
                failure_message: None,
                last_run_duration: None,
            },
            body: body.to_string(),
            path: PathBuf::from("TC-001.md"),
            formal_blocks: vec![],
        }
    }

    #[test]
    fn test_hash_deterministic() {
        let h1 = compute_adr_hash("My Title", "Body content here.");
        let h2 = compute_adr_hash("My Title", "Body content here.");
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256:"));
        assert_eq!(h1.len(), 7 + 64); // "sha256:" + 64 hex chars
    }

    #[test]
    fn test_hash_changes_with_title() {
        let h1 = compute_adr_hash("Title A", "Body");
        let h2 = compute_adr_hash("Title B", "Body");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_changes_with_body() {
        let h1 = compute_adr_hash("Title", "Body A");
        let h2 = compute_adr_hash("Title", "Body B");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_normalize_body_trims_whitespace() {
        let h1 = compute_adr_hash("Title", "  Body  ");
        let h2 = compute_adr_hash("Title", "Body");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_normalize_body_crlf() {
        let h1 = compute_adr_hash("Title", "Line1\r\nLine2");
        let h2 = compute_adr_hash("Title", "Line1\nLine2");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_tc_hash_deterministic() {
        let tc = make_tc("Test Title", "Test body.", None);
        let h1 = compute_tc_hash(&tc);
        let h2 = compute_tc_hash(&tc);
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256:"));
    }

    #[test]
    fn test_verify_adr_valid() {
        let body = "Decision body.";
        let hash = compute_adr_hash("Test ADR", body);
        let adr = make_adr("Test ADR", body, AdrStatus::Accepted, Some(&hash));
        let mut result = CheckResult::new();
        verify_adr(&adr, &mut result);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_verify_adr_tampered() {
        let hash = compute_adr_hash("Test ADR", "Original body.");
        let adr = make_adr("Test ADR", "Modified body.", AdrStatus::Accepted, Some(&hash));
        let mut result = CheckResult::new();
        verify_adr(&adr, &mut result);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].code == "E014");
    }

    #[test]
    fn test_verify_adr_no_hash_w016() {
        let adr = make_adr("Test ADR", "Body.", AdrStatus::Accepted, None);
        let mut result = CheckResult::new();
        verify_adr(&adr, &mut result);
        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].code == "W016");
    }

    #[test]
    fn test_verify_proposed_adr_skipped() {
        let adr = make_adr("Test", "Body.", AdrStatus::Proposed, None);
        let mut result = CheckResult::new();
        verify_adr(&adr, &mut result);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_verify_tc_valid() {
        let tc = make_tc("Test TC", "Body text.", None);
        let hash = compute_tc_hash(&tc);
        let mut tc_sealed = tc;
        tc_sealed.front.content_hash = Some(hash);
        let mut result = CheckResult::new();
        verify_tc(&tc_sealed, &mut result);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_verify_tc_tampered() {
        let tc = make_tc("Test TC", "Original body.", None);
        let hash = compute_tc_hash(&tc);
        let mut tc_tampered = tc;
        tc_tampered.front.content_hash = Some(hash);
        tc_tampered.body = "Modified body.".to_string();
        let mut result = CheckResult::new();
        verify_tc(&tc_tampered, &mut result);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].code == "E015");
    }

    #[test]
    fn test_amend_adr() {
        let body = "Original body.";
        let hash = compute_adr_hash("Test ADR", body);
        let adr = make_adr("Test ADR", "Fixed body.", AdrStatus::Accepted, Some(&hash));
        let (new_hash, amendment) = amend_adr(&adr, "Fix typo").expect("amend");
        assert_ne!(new_hash, hash);
        assert_eq!(amendment.reason, "Fix typo");
        assert_eq!(amendment.previous_hash, hash);
    }

    #[test]
    fn test_amend_no_change() {
        let body = "Body text.";
        let hash = compute_adr_hash("Title", body);
        let adr = make_adr("Title", body, AdrStatus::Accepted, Some(&hash));
        let result = amend_adr(&adr, "No change");
        assert!(result.is_err());
    }
}
