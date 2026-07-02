//! Embedded web-UI serving — the 1.7.0 explorer plus the legacy live view.
//!
//! The 1.7.0 multi-view explorer (What / UI / How / Build / Delivery) is a React
//! app embedded at compile time from `src/assets/ui/`, with React + Babel
//! vendored under `vendor/` so it needs no CDN and no build step. Served at `/`
//! (and every asset it references) via the router fallback; the legacy
//! self-contained live 3-view page stays available at `/legacy`.

use axum::response::{Html, IntoResponse, Response};

/// The embedded 1.7.0 explorer UI tree (`src/assets/ui/`).
#[derive(rust_embed::Embed)]
#[folder = "src/assets/ui/"]
struct UiAssets;

/// Guess a content type from a file extension for the embedded UI assets.
/// `.jsx` is served as `text/babel` so the in-browser Babel transform picks it
/// up; everything else maps to the obvious web type.
fn ui_content_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("jsx") => "text/babel; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("md") => "text/markdown; charset=utf-8",
        Some("woff2") => "font/woff2",
        Some("map") => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Serve one embedded UI asset by path (`""`/`/` → `index.html`). Wired as the
/// router fallback, so the `/api/*` and `/mcp` routes take precedence.
pub async fn ui_handler(uri: axum::http::Uri) -> Response {
    let raw = uri.path().trim_start_matches('/');
    let path = if raw.is_empty() { "index.html" } else { raw };
    match UiAssets::get(path) {
        Some(file) => (
            [
                (axum::http::header::CONTENT_TYPE, ui_content_type(path)),
                // The view is a live tool — never serve a stale asset (the graph
                // and the JSX both change under it).
                (axum::http::header::CACHE_CONTROL, "no-cache"),
            ],
            file.data.into_owned(),
        )
            .into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

/// `GET /legacy` — the original self-contained live 3-view page (Systems /
/// Domain ER / Flows), projected from `/api/graph`. Kept because the new UI is
/// currently static; this remains the graph-connected view.
pub async fn legacy_view_handler() -> Html<&'static str> {
    Html(include_str!("assets/view.html"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_and_vendored_deps_are_embedded() {
        // The entry page, an app module, the data layer, and the vendored deps
        // must all be embedded so the page serves with no CDN and no build step.
        for path in [
            "index.html",
            "app.jsx",
            "data.js",
            "vendor/react.js",
            "vendor/react-dom.js",
            "vendor/babel.js",
        ] {
            assert!(UiAssets::get(path).is_some(), "missing embedded UI asset: {path}");
        }
        // The index references the vendored deps, not a CDN.
        let index = UiAssets::get("index.html").expect("index");
        let html = std::str::from_utf8(&index.data).expect("utf8");
        assert!(html.contains("vendor/react.js"), "index must load vendored react");
        assert!(!html.contains("unpkg.com"), "index must not reference a CDN");
    }

    #[test]
    fn content_types_map_web_extensions() {
        assert_eq!(ui_content_type("index.html"), "text/html; charset=utf-8");
        assert_eq!(ui_content_type("app.jsx"), "text/babel; charset=utf-8");
        assert_eq!(ui_content_type("data.js"), "text/javascript; charset=utf-8");
        assert_eq!(ui_content_type("styles.css"), "text/css; charset=utf-8");
        assert_eq!(ui_content_type("logo.svg"), "image/svg+xml");
    }
}
