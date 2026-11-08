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
use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_SECURITY_POLICY, CONTENT_TYPE, REFERRER_POLICY,
    X_CONTENT_TYPE_OPTIONS,
};
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
        Some("md") => "text/markdown; charset=utf-8",
        Some("asc") => "application/pgp-keys",
        _ => "application/octet-stream",
    }
}

/// Browser features the site never uses, all switched off.
const PERMISSIONS_POLICY: &str = "accelerometer=(), camera=(), geolocation=(), gyroscope=(), \
     magnetometer=(), microphone=(), payment=(), usb=()";

/// The Content-Security-Policy for the built site.
///
/// Scripts are limited to same-origin files plus the inline loader Trunk
/// writes into `index.html`, allowed by its SHA-256 hash; that script changes
/// with every build, so the hash is taken from the built page at startup.
/// WebAssembly needs `'wasm-unsafe-eval'` to compile. Styles allow inline
/// attributes because the animation positions each particle with one.
fn content_security_policy(index_html: &str) -> String {
    use base64::Engine as _;
    use sha2::Digest as _;

    let mut script_src = String::from("'self' 'wasm-unsafe-eval'");
    for script in inline_scripts(index_html) {
        let hash = base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(script));
        script_src.push_str(" 'sha256-");
        script_src.push_str(&hash);
        script_src.push('\'');
    }
    format!(
        "default-src 'none'; script-src {script_src}; style-src 'self' 'unsafe-inline'; \
         img-src 'self'; font-src 'self'; connect-src 'self'; manifest-src 'self'; \
         base-uri 'none'; form-action 'none'; frame-ancestors 'none'"
    )
}

/// Bodies of the inline `<script>` elements a browser executes: no `src`,
/// and no data type such as `application/ld+json`.
fn inline_scripts(html: &str) -> Vec<&str> {
    let mut scripts = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<script") {
        let after = &rest[start..];
        let Some(open_end) = after.find('>') else {
            break;
        };
        let attrs = &after[..open_end];
        let body_start = open_end + 1;
        let Some(close) = after[body_start..].find("</script>") else {
            break;
        };
        let executable = !attrs.contains(" src=")
            && (!attrs.contains("type=")
                || attrs.contains("type=\"module\"")
                || attrs.contains("javascript"));
        if executable {
            scripts.push(&after[body_start..body_start + close]);
        }
        rest = &after[body_start + close..];
    }
    scripts
}

/// Headers added to every static response.
struct SiteHeaders {
    csp: HeaderValue,
}

/// Build a buffered response with a fixed content type and the site's
/// security headers.
fn respond(
    headers: &SiteHeaders,
    status: StatusCode,
    content_type: &'static str,
    body: Vec<u8>,
) -> Response<ResponseBody> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, HeaderValue::from_static(content_type))
        .header(CONTENT_SECURITY_POLICY, headers.csp.clone())
        .header(
            "permissions-policy",
            HeaderValue::from_static(PERMISSIONS_POLICY),
        )
        .header(
            "cross-origin-opener-policy",
            HeaderValue::from_static("same-origin"),
        )
        .header(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"))
        .header(
            REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        )
        .body(full_body(Bytes::from(body)))
        .expect("a static-header response is always well-formed")
}

fn not_found(headers: &SiteHeaders) -> Response<ResponseBody> {
    respond(
        headers,
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
async fn serve_static(
    req: RawRequest,
    dist: Arc<PathBuf>,
    headers: Arc<SiteHeaders>,
) -> Response<ResponseBody> {
    let tail = req.tail();
    let Some(path) = resolve_path(dist.as_ref(), tail) else {
        return not_found(&headers);
    };
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return not_found(&headers);
    };
    let Ok(bytes) = tokio::fs::read(&path).await else {
        return not_found(&headers);
    };
    let mut response = respond(&headers, StatusCode::OK, content_type_for(&name), bytes);
    // Web Key Directory clients in browsers fetch keys cross-origin.
    if tail.first().is_some_and(|s| s == ".well-known")
        && tail.get(1).is_some_and(|s| s == "openpgpkey")
    {
        response
            .headers_mut()
            .insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    }
    response
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let dist = Arc::new(dist_dir());
    let index_html = std::fs::read_to_string(dist.join("index.html")).unwrap_or_default();
    let csp = content_security_policy(&index_html);
    let headers = Arc::new(SiteHeaders {
        csp: HeaderValue::from_str(&csp).expect("the CSP is built from ASCII"),
    });

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
        let headers = Arc::clone(&headers);
        async move { serve_static(req, dist, headers).await }
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
        assert_eq!(
            content_type_for("THIRD-PARTY-NOTICES.md"),
            "text/markdown; charset=utf-8"
        );
        assert_eq!(content_type_for("no-extension"), "application/octet-stream");
    }

    #[test]
    fn csp_hashes_only_executable_inline_scripts() {
        let html = r#"<script type="application/ld+json">{"a":1}</script>
<script type="module">import init from '/a.js';</script>
<script src="/b.js"></script>"#;
        let csp = content_security_policy(html);
        // sha256("import init from '/a.js';"), base64
        assert_eq!(inline_scripts(html), ["import init from '/a.js';"]);
        assert_eq!(csp.matches("'sha256-").count(), 1);
        assert!(
            csp.starts_with("default-src 'none'; script-src 'self' 'wasm-unsafe-eval' 'sha256-")
        );
        assert!(!csp.contains("script-src 'self' 'unsafe-inline'"));
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
