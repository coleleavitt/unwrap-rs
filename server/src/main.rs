//! servant-rs backend for unwrap.rs.
//!
//! One typed [`servant`] API description drives the whole edge:
//! - `GET /api/health`        — a `PlainText` liveness probe,
//! - `GET /api/social-links`  — the social links as `JSON`,
//! - everything else          — a `Raw` catch-all that serves the Trunk-built
//!   frontend from `dist/` and returns a real 404 for missing files.
//!
//! The same description is consumed by `servant_server::serve`, so routing and
//! the handler shapes cannot drift apart.

use std::net::SocketAddr;
use std::path::PathBuf;
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

/// `Raw` catch-all: serve a file from `dist/` or return a real 404.
async fn serve_static(req: RawRequest, dist: Arc<PathBuf>) -> Response<ResponseBody> {
    let tail = req.tail();

    // Reject path traversal before touching the filesystem.
    if tail
        .iter()
        .any(|seg| seg.is_empty() || seg == "." || seg == ".." || seg.contains(['/', '\\']))
    {
        return not_found();
    }

    let mut path = dist.as_ref().clone();
    if tail.is_empty() {
        path.push("index.html");
    } else {
        for seg in tail {
            path.push(seg);
        }
    }

    let name = tail.last().map_or("index.html", String::as_str);
    tokio::fs::read(&path).await.map_or_else(
        |_| not_found(),
        |bytes| respond(StatusCode::OK, content_type_for(name), bytes),
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
    println!("unwrap.rs server listening on http://{addr}");
    serve_listener(listener, service).await
}
