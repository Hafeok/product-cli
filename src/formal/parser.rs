//! Formal block parser — parse_formal_blocks, diagnostics, block-start finder (ADR-011, ADR-016)

use regex::Regex;

use super::blocks::*;

/// Result of parsing formal blocks — blocks + any diagnostics
pub struct FormalParseResult {
    pub blocks: Vec<FormalBlock>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Parse all formal blocks from a test criterion body
pub fn parse_formal_blocks(body: &str) -> Vec<FormalBlock> {
    parse_formal_blocks_with_diagnostics(body).blocks
}

/// Parse formal blocks with full diagnostic reporting (ADR-016)
pub fn parse_formal_blocks_with_diagnostics(body: &str) -> FormalParseResult {
    let mut result = FormalParseResult {
        blocks: Vec::new(),
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Parse evidence blocks: \u{27E6}\u{0395}\u{27E7}\u{27E8}...\u{27E9}
    let evidence_re = Regex::new(r"\u{27E6}\u{0395}\u{27E7}\u{27E8}([^\u{27E9}]+)\u{27E9}").expect("constant regex");
    for cap in evidence_re.captures_iter(body) {
        match parse_evidence_fields_validated(&cap[1]) {
            Ok(eb) => result.blocks.push(FormalBlock::Evidence(eb)),
            Err(msg) => result.errors.push(msg),
        }
    }

    // Detect unclosed block delimiters: \u{27E6} without matching \u{27E7}
    let open_count = body.matches('\u{27E6}').count();
    let close_count = body.matches('\u{27E7}').count();
    if open_count > close_count {
        result.errors.push(format!(
            "E001: unclosed block delimiter \u{2014} found {} \u{27E6} but only {} \u{27E7}",
            open_count, close_count
        ));
    }

    // Detect unrecognised block types: \u{27E6}X:Unknown\u{27E7}
    let block_type_re = Regex::new(r"\u{27E6}([^\u{27E7}]+)\u{27E7}").expect("constant regex");
    let known = ["\u{03A3}:Types", "\u{0393}:Invariants", "\u{039B}:Scenario", "\u{039B}:ExitCriteria", "\u{039B}:ExitCritera", "\u{0395}"];
    for cap in block_type_re.captures_iter(body) {
        let bt = &cap[1];
        if !known.contains(&bt) {
            result.errors.push(format!(
                "E001: unrecognised block type \u{27E6}{}\u{27E7}",
                bt
            ));
        }
    }

    // Parse typed blocks: \u{27E6}BlockType\u{27E7}{ ... }
    let block_starts: Vec<(usize, &str)> = find_block_starts(body);

    for (start, block_type) in block_starts {
        let rest = &body[start..];
        if let Some(brace_start) = rest.find('{') {
            let content_start = start + brace_start + 1;
            if let Some(brace_end) = find_matching_brace(body, content_start) {
                let content = body[content_start..brace_end].trim();

                // W004: empty block body
                if content.is_empty() {
                    result.warnings.push(format!(
                        "W004: empty block body \u{27E6}{}\u{27E7}{{}}",
                        block_type
                    ));
                    continue;
                }

                match block_type {
                    "\u{03A3}:Types" => {
                        result.blocks.push(FormalBlock::Types(parse_type_defs(content)));
                    }
                    "\u{0393}:Invariants" => {
                        result.blocks.push(FormalBlock::Invariants(parse_invariants(content)));
                    }
                    "\u{039B}:Scenario" => {
                        result.blocks.push(FormalBlock::Scenario(parse_scenario(content)));
                    }
                    "\u{039B}:ExitCriteria" | "\u{039B}:ExitCritera" => {
                        result.blocks.push(FormalBlock::ExitCriteria(parse_exit_criteria(content)));
                    }
                    _ => {}
                }
            } else {
                // Unclosed brace
                let line = body[..start].lines().count() + 1;
                result.errors.push(format!(
                    "E001: unclosed '{{' for \u{27E6}{}\u{27E7} block at line {}",
                    block_type, line
                ));
            }
        }
    }

    result
}

/// Validate evidence fields, return error on out-of-range values
fn parse_evidence_fields_validated(fields: &str) -> std::result::Result<EvidenceBlock, String> {
    let mut delta = None;
    let mut phi = None;
    let mut tau = Stability::Unknown;

    for field in fields.split(';') {
        let field = field.trim();
        if let Some(val) = field.strip_prefix("\u{03B4}\u{225C}") {
            delta = val.trim().parse::<f64>().ok();
        } else if let Some(val) = field.strip_prefix("\u{03C6}\u{225C}") {
            phi = val.trim().parse::<u8>().ok();
        } else if let Some(val) = field.strip_prefix("\u{03C4}\u{225C}") {
            let val = val.trim();
            tau = if val.contains("\u{25CA}\u{207A}") {
                Stability::Stable
            } else if val.contains("\u{25CA}\u{207B}") {
                Stability::Unstable
            } else {
                Stability::Unknown
            };
        }
    }

    let d = delta.unwrap_or(0.0);
    let p = phi.unwrap_or(0);

    if !(0.0..=1.0).contains(&d) {
        return Err(format!("E001: evidence \u{03B4}={:.2} is out of range [0.0, 1.0]", d));
    }
    // phi is u8 so max is 255, but we restrict to 100
    if p > 100 {
        return Err(format!("E001: evidence \u{03C6}={} is out of range [0, 100]", p));
    }

    Ok(EvidenceBlock { delta: d, phi: p, tau })
}

fn find_block_starts(body: &str) -> Vec<(usize, &str)> {
    let mut results = Vec::new();
    let known_types = ["\u{03A3}:Types", "\u{0393}:Invariants", "\u{039B}:Scenario", "\u{039B}:ExitCriteria", "\u{039B}:ExitCritera"];
    for bt in &known_types {
        let pattern = format!("\u{27E6}{}\u{27E7}", bt);
        let mut search_start = 0;
        while let Some(pos) = body[search_start..].find(&pattern) {
            results.push((search_start + pos, *bt));
            search_start += pos + pattern.len();
        }
    }
    results.sort_by_key(|(pos, _)| *pos);
    results
}

fn find_matching_brace(body: &str, start: usize) -> Option<usize> {
    let mut depth = 1;
    for (i, c) in body[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if formal blocks have any evidence blocks, return aggregate metrics
#[allow(dead_code)]
pub fn aggregate_evidence(blocks: &[FormalBlock]) -> Option<EvidenceBlock> {
    let evidence: Vec<&EvidenceBlock> = blocks
        .iter()
        .filter_map(|b| match b {
            FormalBlock::Evidence(e) => Some(e),
            _ => None,
        })
        .collect();

    if evidence.is_empty() {
        return None;
    }

    let avg_delta = evidence.iter().map(|e| e.delta).sum::<f64>() / evidence.len() as f64;
    let avg_phi = evidence.iter().map(|e| e.phi as f64).sum::<f64>() / evidence.len() as f64;

    // Worst-case stability
    let tau = if evidence.iter().any(|e| e.tau == Stability::Unstable) {
        Stability::Unstable
    } else if evidence.iter().any(|e| e.tau == Stability::Unknown) {
        Stability::Unknown
    } else {
        Stability::Stable
    };

    Some(EvidenceBlock {
        delta: avg_delta,
        phi: avg_phi as u8,
        tau,
    })
}
