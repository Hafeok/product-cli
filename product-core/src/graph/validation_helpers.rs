//! Shared helpers for `graph::validation`.

use super::model::find_reference_line;
use crate::error::{CheckResult, Diagnostic};
use crate::types::Feature;
use std::path::Path;

/// Push an E002 broken-link diagnostic with optional line info.
pub(crate) fn push_broken_link(
    result: &mut CheckResult,
    path: &Path,
    from_id: &str,
    target_id: &str,
    verb: &str,
    hint: &str,
) {
    let mut diag = Diagnostic::error("E002", "broken link")
        .with_file(path.to_path_buf())
        .with_detail(&format!(
            "{} {} {} which does not exist",
            from_id, verb, target_id
        ));
    if !hint.is_empty() {
        diag = diag.with_hint(hint);
    }
    if let Some((line, content)) = find_reference_line(path, target_id) {
        diag = diag.with_line(line).with_context(&content);
    }
    result.errors.push(diag);
}

/// Push a W016 warning for a complete feature with blocking TCs.
pub(crate) fn push_blocking_tc_warning(
    result: &mut CheckResult,
    f: &Feature,
    blocking_tcs: &[&str],
) {
    let preview: Vec<&str> = blocking_tcs.iter().take(5).copied().collect();
    let suffix = if blocking_tcs.len() > 5 {
        format!(", ... ({} total)", blocking_tcs.len())
    } else {
        String::new()
    };
    result.warnings.push(
        Diagnostic::warning("W016", "complete feature has unimplemented tests")
            .with_file(f.path.clone())
            .with_detail(&format!(
                "{} is complete but has {} unimplemented/failing TC(s): {}{}",
                f.front.id,
                blocking_tcs.len(),
                preview.join(", "),
                suffix,
            ))
            .with_hint("run `product verify` to re-evaluate, or set blocking TCs to `unrunnable`"),
    );
}
