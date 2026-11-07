//! Servant-rs backend for unwrap.rs.
//!
//! One typed [`servant`] API description drives the whole edge:
//! - `GET /api/health` — a `PlainText` liveness probe,
//! - `GET /api/social-links` — the social links as `JSON`,
//! - everything else — a `Raw` catch-all that serves the Trunk-built
//!   frontend from `dist/` and returns a real 404 for missing files.
//!
//! The same description is consumed by `servant_server::serve`, so routing and
//! the handler shapes cannot drift apart.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bytes::Bytes;
use http::header::CONTENT_TYPE;
use http::{HeaderValue, Response, StatusCode};
use serde::Serialize;
use servant::prelude::*;
use servant_server::adapter::serve_listener;
use servant_server::response::{ResponseBody, full_body};
use servant_server::{RawRequest, RouterService, serve};
use tokio::net::TcpListener;

/// A social link, rendered as JSON by the `/api/social-links` endpoint.
///
/// Mirrors the frontend's `SocialLink` so the two stay in sync; the `icon_path`
/// values resolve against the Trunk-copied `dist/icons/` directory.
#[derive(Clone, Debug, Serialize)]
struct SocialLink {
    url: &'static str,
    icon_path: &'static str,
    label: &'static str,
}

// This data intentionally mirrors the frontend at the HTTP API boundary.
//noinspection DuplicatedCode
const SOCIAL_LINKS: &[SocialLink] = &[
    SocialLink {
        url: "https://github.com/coleleavitt",
        icon_path: "icons/github-original.svg",
        label: "GitHub",
    },
    SocialLink {
        url: "https://www.linkedin.com/in/coleleavitt/",
        icon_path: "icons/linkedin.svg",
        label: "LinkedIn",
    },
    SocialLink {
        url: "mailto:cole@unwrap.rs",
        icon_path: "icons/envelope-fill.svg",
        label: "Email",
    },
];

/// Directory the built frontend lives in. Override with `DIST_DIR`.
fn dist_dir() -> PathBuf {
    std::env::var_os("DIST_DIR").map_or_else(|| PathBuf::from("frontend/dist"), PathBuf::from)
}

/// Bind address. Override with `ADDR` (e.g. `0.0.0.0:8080`).
fn bind_addr() -> SocketAddr {
    std::env::var("ADDR")
        .ok()
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 8080)))
}

/// Map a file name to a `Content-Type`, defaulting to octet-stream.
fn content_type_for(name: &str) -> &'static str {
    match name.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        Some("xml") => "application/xml; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("asc") => "application/pgp-keys",
        _ => "application/octet-stream",
    }
}

/// Build a buffered response with a fixed content type.
fn respond(
    status: StatusCode,
    content_type: &'static str,
    body: Vec<u8>,
) -> Response<ResponseBody> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, HeaderValue::from_static(content_type))
        .body(full_body(Bytes::from(body)))
        .expect("a static-header response is always well-formed")
}

fn not_found() -> Response<ResponseBody> {
    respond(
        StatusCode::NOT_FOUND,
        "text/plain; charset=utf-8",
        b"404 Not Found".to_vec(),
    )
}

/// Resolve a request tail to a file path under `dist`.
///
/// A single trailing empty segment (a trailing slash) selects a directory.
/// Every other empty segment, `.`, `..`, and slash-bearing segment is
/// rejected, so path traversal never reaches the filesystem. A resolved
/// directory serves its `index.html`.
fn resolve_path(dist: &Path, tail: &[String]) -> Option<PathBuf> {
    let segs: &[String] = if tail.last().is_some_and(String::is_empty) {
        &tail[..tail.len() - 1]
    } else {
        tail
    };
    if segs
        .iter()
        .any(|seg| seg.is_empty() || seg == "." || seg == ".." || seg.contains(['/', '\\']))
    {
        return None;
    }

    let mut path = dist.to_path_buf();
    if segs.is_empty() {
        path.push("index.html");
    } else {
        for seg in segs {
            path.push(seg);
        }
        if path.is_dir() {
            path.push("index.html");
        }
    }
    Some(path)
}

/// `Raw` catch-all: serve a file from `dist/` or return a real 404.
async fn serve_static(req: RawRequest, dist: Arc<PathBuf>) -> Response<ResponseBody> {
    let Some(path) = resolve_path(dist.as_ref(), req.tail()) else {
        return not_found();
    };
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return not_found();
    };
    tokio::fs::read(&path).await.map_or_else(
        |_| not_found(),
        |bytes| respond(StatusCode::OK, content_type_for(&name), bytes),
    )
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let dist = Arc::new(dist_dir());

    // One description, four interpretations — here we drive the server.
    //   "api" :> "health" :> Get '[PlainText] String
    //     :<|> "api" :> "social-links" :> Get '[JSON] [SocialLink]
    //     :<|> Raw
    // Alternatives carry full path prefixes: `serve` dispatches each branch by
    // its terminal verb, so a shared prefix is distributed across the branches
    // rather than wrapping an `alt` inside one `path`.
    let api = alt(
        alt(
            path("api", path("health", get::<(PlainText,), String>())),
            path(
                "api",
                path("social-links", get::<(Json,), Vec<SocialLink>>()),
            ),
        ),
        raw(),
    );

    let health = || async { Ok::<_, ServerError>("ok".to_string()) };
    let social = || async { Ok::<_, ServerError>(SOCIAL_LINKS.to_vec()) };
    let static_handler = move |req: RawRequest| {
        let dist = Arc::clone(&dist);
        async move { serve_static(req, dist).await }
    };

    let router = serve(api, ((health, social), static_handler));
    let service = RouterService::new(router);

    let addr = bind_addr();
    let listener = TcpListener::bind(addr).await?;
    println!("unwrap.rs server listening on {addr}");
    serve_listener(listener, service).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dist(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("unwrap-rs-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("writing").join("some-post")).expect("mkdir");
        fs::write(dir.join("robots.txt"), "User-agent: *\n").expect("write");
        fs::write(
            dir.join("writing").join("some-post").join("index.html"),
            "<html>",
        )
        .expect("write");
        dir
    }

    #[test]
    fn content_types_cover_served_assets() {
        assert_eq!(content_type_for("JetBrainsMono-latin.woff2"), "font/woff2");
        assert_eq!(content_type_for("app_bg.wasm"), "application/wasm");
        assert_eq!(
            content_type_for("feed.xml"),
            "application/xml; charset=utf-8"
        );
        assert_eq!(content_type_for("pgp.asc"), "application/pgp-keys");
        assert_eq!(content_type_for("no-extension"), "application/octet-stream");
    }

    #[test]
    fn empty_tail_serves_index() {
        let dist = temp_dist("empty");
        assert_eq!(resolve_path(&dist, &[]), Some(dist.join("index.html")));
        let _ = fs::remove_dir_all(&dist);
    }

    #[test]
    fn trailing_slash_resolves_directory_index() {
        let dist = temp_dist("slash");
        let tail = vec!["writing".to_string(), String::new()];
        assert_eq!(
            resolve_path(&dist, &tail),
            Some(dist.join("writing").join("index.html"))
        );
        let _ = fs::remove_dir_all(&dist);
    }

    #[test]
    fn nested_post_resolves_its_index() {
        let dist = temp_dist("nested");
        let tail = vec!["writing".to_string(), "some-post".to_string()];
        assert_eq!(
            resolve_path(&dist, &tail),
            Some(dist.join("writing").join("some-post").join("index.html"))
        );
        let _ = fs::remove_dir_all(&dist);
    }

    #[test]
    fn plain_file_is_served_directly() {
        let dist = temp_dist("file");
        let tail = vec!["robots.txt".to_string()];
        assert_eq!(resolve_path(&dist, &tail), Some(dist.join("robots.txt")));
        let _ = fs::remove_dir_all(&dist);
    }

    #[test]
    fn traversal_segments_are_rejected() {
        let dist = temp_dist("traversal");
        for tail in [
            vec!["..".to_string()],
            vec![".".to_string()],
            vec!["a/../b".to_string()],
            vec![String::new(), String::new()],
            vec!["a".to_string(), String::new(), "b".to_string()],
        ] {
            assert_eq!(resolve_path(&dist, &tail), None, "rejected: {tail:?}");
        }
        let _ = fs::remove_dir_all(&dist);
    }
}
