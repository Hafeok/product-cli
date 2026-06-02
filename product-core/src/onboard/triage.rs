//! Phase 2: Triage — structured team review (ADR-027)

use crate::error::{ProductError, Result};
use std::collections::HashMap;
use std::io::BufRead;

use super::types::*;

/// Triage candidates interactively, reading actions from a BufRead source (stdin or test harness).
pub fn triage_interactive<R: BufRead>(
    scan_output: &ScanOutput,
    input: &mut R,
) -> Result<TriageOutput> {
    let mut triaged = Vec::new();

    // Build a map of pending candidates for merge lookups
    let candidate_map: HashMap<String, &Candidate> = scan_output
        .candidates
        .iter()
        .map(|c| (c.id.clone(), c))
        .collect();

    let mut merged_ids: Vec<String> = Vec::new();

    for candidate in &scan_output.candidates {
        // Skip candidates that have been merged into another
        if merged_ids.contains(&candidate.id) {
            continue;
        }

        // Print candidate info to stderr for interactive display
        eprintln!(
            "--- {} [{}] confidence: {} ---",
            candidate.id, candidate.signal_type, candidate.confidence
        );
        eprintln!("{}", candidate.title);
        eprintln!();
        eprintln!("Observation: {}", candidate.observation);
        eprintln!();
        eprintln!("Evidence:");
        for ev in &candidate.evidence {
            let valid_marker = if ev.evidence_valid { "" } else { " [INVALID]" };
            eprintln!("  {}:{}    {}{}", ev.file, ev.line, ev.snippet, valid_marker);
        }
        eprintln!();
        eprintln!(
            "Hypothesised consequence: {}",
            candidate.hypothesised_consequence
        );
        eprintln!();
        eprintln!("  [c]onfirm  [m]erge with DC-XXX  [r]eject  [s]kip");

        // Read action
        let mut line_buf = String::new();
        let bytes_read = input
            .read_line(&mut line_buf)
            .map_err(|e| ProductError::IoError(format!("failed to read triage input: {}", e)))?;

        if bytes_read == 0 {
            // EOF — skip remaining
            triaged.push(TriagedCandidate {
                candidate: candidate.clone(),
                triage_status: TriageStatus::Skipped,
                merged_into: None,
            });
            continue;
        }

        let action = line_buf.trim().to_lowercase();
        match action.as_str() {
            "c" | "confirm" => {
                triaged.push(TriagedCandidate {
                    candidate: candidate.clone(),
                    triage_status: TriageStatus::Confirmed,
                    merged_into: None,
                });
            }
            "r" | "reject" => {
                triaged.push(TriagedCandidate {
                    candidate: candidate.clone(),
                    triage_status: TriageStatus::Rejected,
                    merged_into: None,
                });
            }
            "s" | "skip" => {
                triaged.push(TriagedCandidate {
                    candidate: candidate.clone(),
                    triage_status: TriageStatus::Skipped,
                    merged_into: None,
                });
            }
            s if s.starts_with('m') || s.starts_with("merge") => {
                // Parse merge target: "m\nDC-001" or "merge DC-001"
                let target_id = s
                    .strip_prefix("merge")
                    .or_else(|| s.strip_prefix("m"))
                    .map(|rest| rest.trim().to_string())
                    .unwrap_or_default();

                // If target wasn't on same line, read next line
                let target_id = if target_id.is_empty() {
                    let mut target_buf = String::new();
                    let _ = input.read_line(&mut target_buf);
                    target_buf.trim().to_string()
                } else {
                    target_id
                };

                if let Some(target) = candidate_map.get(&target_id) {
                    // Merge: current candidate is absorbed into target
                    // Mark this candidate as merged
                    triaged.push(TriagedCandidate {
                        candidate: candidate.clone(),
                        triage_status: TriageStatus::Merged,
                        merged_into: Some(target_id.clone()),
                    });

                    // Find or create the target in triaged list and add evidence
                    let mut found = false;
                    for tc in triaged.iter_mut() {
                        if tc.candidate.id == target_id {
                            // Add evidence from merged candidate
                            tc.candidate
                                .evidence
                                .extend(candidate.evidence.clone());
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        // Target hasn't been processed yet — create a confirmed entry with merged evidence
                        let mut merged_candidate = (*target).clone();
                        merged_candidate
                            .evidence
                            .extend(candidate.evidence.clone());
                        triaged.push(TriagedCandidate {
                            candidate: merged_candidate,
                            triage_status: TriageStatus::Confirmed,
                            merged_into: None,
                        });
                        merged_ids.push(target_id);
                    }
                } else {
                    // Invalid merge target — treat as skip
                    eprintln!(
                        "warning: merge target '{}' not found, skipping {}",
                        target_id, candidate.id
                    );
                    triaged.push(TriagedCandidate {
                        candidate: candidate.clone(),
                        triage_status: TriageStatus::Skipped,
                        merged_into: None,
                    });
                }
            }
            _ => {
                // Unknown action — skip
                triaged.push(TriagedCandidate {
                    candidate: candidate.clone(),
                    triage_status: TriageStatus::Skipped,
                    merged_into: None,
                });
            }
        }
    }

    Ok(TriageOutput {
        candidates: triaged,
    })
}

/// Batch-confirm all candidates (non-interactive triage).
pub fn triage_batch_confirm(scan_output: &ScanOutput) -> TriageOutput {
    TriageOutput {
        candidates: scan_output
            .candidates
            .iter()
            .map(|c| TriagedCandidate {
                candidate: c.clone(),
                triage_status: TriageStatus::Confirmed,
                merged_into: None,
            })
            .collect(),
    }
}
