//! M7 phase 2 — the screenshot-diff harness for the open "looks right"
//! predicate.
//!
//! The interceptor deliberately treats a same-type token *value* change as
//! no event (that silence is what a token buys); the visual consequence of
//! such a change is exactly what nothing else checks. This harness renders
//! the pure-geometry fixture panel with the pre-installed headless
//! Chromium (deterministic: fixed window, no fonts, no GPU) and asserts
//! (a) same input → identical bytes, (b) a token value drift → different
//! bytes. Gated on a local browser, like the .NET SARIF regeneration
//! tests (`dec/ddd/fixtures-not-sdk`): CI runs against nothing here, a
//! workstation with Chromium reproduces the evidence on `DDD-web-visual-01`.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The headless browser, when this machine has one: `DDD_CHROMIUM`
/// overrides; otherwise the Playwright install location, then PATH.
fn headless_shell() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("DDD_CHROMIUM") {
        return Some(PathBuf::from(p));
    }
    if let Ok(rd) = std::fs::read_dir("/opt/pw-browsers") {
        for entry in rd.flatten() {
            let cand = entry.path().join("chrome-linux/headless_shell");
            if cand.exists() {
                return Some(cand);
            }
        }
    }
    ["chromium", "chromium-browser", "headless_shell"].iter().find_map(|name| {
        let out = Command::new("which").arg(name).output().ok()?;
        let path = String::from_utf8(out.stdout).ok()?;
        let path = path.trim();
        (!path.is_empty()).then(|| PathBuf::from(path))
    })
}

fn screenshot(shell: &Path, page: &Path, out: &Path) -> Vec<u8> {
    let status = Command::new(shell)
        .args([
            "--headless",
            "--no-sandbox",
            "--disable-gpu",
            "--hide-scrollbars",
            "--force-device-scale-factor=1",
            "--window-size=440,140",
        ])
        .arg(format!("--screenshot={}", out.display()))
        .arg(format!("file://{}", page.display()))
        .status()
        .expect("spawn headless browser");
    assert!(status.success(), "screenshot render failed");
    std::fs::read(out).expect("screenshot bytes")
}

#[test]
fn a_token_value_drift_is_caught_as_pixels_where_the_interceptor_is_silent() {
    let Some(shell) = headless_shell() else {
        eprintln!("skipped: no headless Chromium on this machine (set DDD_CHROMIUM)");
        return;
    };
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/visual/panel.html");
    let baseline_html = std::fs::read_to_string(&fixture).expect("fixture");
    let tmp = tempfile::tempdir().expect("tmp");
    let write = |name: &str, content: &str| -> PathBuf {
        let p = tmp.path().join(name);
        std::fs::write(&p, content).expect("write");
        p
    };

    // (a) Determinism: the same panel renders to identical bytes.
    let page = write("baseline.html", &baseline_html);
    let first = screenshot(&shell, &page, &tmp.path().join("baseline-1.png"));
    let second = screenshot(&shell, &page, &tmp.path().join("baseline-2.png"));
    assert_eq!(first, second, "same input must render byte-identically");

    // (b) The regression: --status-ok drifts to another green. Same type,
    // so the interceptor sees no event; the pixels move anyway.
    let drifted = baseline_html.replace("--status-ok: #1a7f37;", "--status-ok: #2ea043;");
    assert_ne!(drifted, baseline_html, "fixture lost the token anchor");
    let page = write("drifted.html", &drifted);
    let regressed = screenshot(&shell, &page, &tmp.path().join("drifted.png"));
    assert_ne!(first, regressed, "the visual drift must be caught as a pixel difference");
}
