//! Static site generator for the /writing/ section.
//!
//! Reads Markdown posts from `content/writing/` (run from the repo root, or set
//! `CONTENT_DIR`/`DIST_DIR`) and writes plain HTML pages, an Atom feed and a
//! sitemap into the Trunk output directory. Posts are static HTML on purpose:
//! they are readable without JavaScript and crawlable, which the CSR app
//! cannot offer.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use pulldown_cmark::{Options, Parser, html};

const SITE_URL: &str = "https://unwrap.rs";

/// One parsed post.
struct Post {
    slug: String,
    title: String,
    date: String,
    description: String,
    body_html: String,
}

impl Post {
    fn url(&self) -> String {
        format!("{SITE_URL}/writing/{}/", self.slug)
    }
}

/// Parse `key: value` frontmatter delimited by `+++` lines.
/// Returns the fields and the remaining Markdown body.
fn parse_frontmatter(src: &str) -> (Vec<(String, String)>, &str) {
    let Some(rest) = src.strip_prefix("+++") else {
        return (Vec::new(), src);
    };
    let Some(end) = rest.find(
        "
+++",
    ) else {
        return (Vec::new(), src);
    };
    let (block, body) = rest.split_at(end);
    let body = body
        .strip_prefix(
            "
+++
",
        )
        .unwrap_or(body);
    let fields = block
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    (fields, body)
}

fn field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// Render post Markdown to HTML. ` ```mermaid ` fences are rendered to
/// inline SVG at build time via `mmd-svg`; everything else goes through
/// pulldown-cmark.
fn render_markdown(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let mut out = String::new();
    let mut plain = String::new();
    let mut lines = md.lines();
    while let Some(line) = lines.next() {
        if line.trim_end() != "```mermaid" {
            plain.push_str(line);
            plain.push('\n');
            continue;
        }
        html::push_html(&mut out, Parser::new_ext(&plain, options));
        plain.clear();
        let mut src = String::new();
        for line in lines.by_ref() {
            if line.trim_end() == "```" {
                break;
            }
            src.push_str(line);
            src.push('\n');
        }
        match mmd_svg::render(&src) {
            Ok(svg) => {
                // focusable so keyboard users can scroll a wide diagram
                let _ = writeln!(
                    out,
                    "<figure class=\"diagram\" tabindex=\"0\">{svg}</figure>"
                );
            }
            Err(e) => panic!("mermaid block failed to render: {e}"),
        }
    }
    html::push_html(&mut out, Parser::new_ext(&plain, options));
    out
}

/// Escape text for HTML element and attribute content.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Days since the Unix epoch to a `YYYY-MM-DD` UTC date
/// (Howard Hinnant's `civil_from_days` algorithm).
fn civil_date(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

fn today_utc() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs().cast_signed() / 86_400);
    civil_date(days)
}

/// Extract the Trunk-hashed stylesheet href from the built `index.html`.
fn stylesheet_href(index_html: &str) -> String {
    let Some(rel_pos) = index_html.find("rel=\"stylesheet\"") else {
        return "index.css".to_string();
    };
    let rest = &index_html[rel_pos..];
    let Some(href_start) = rest.find("href=\"").map(|o| o + "href=\"".len()) else {
        return "index.css".to_string();
    };
    let after = &rest[href_start..];
    after
        .find('"')
        .map_or_else(|| "index.css".to_string(), |end| after[..end].to_string())
}

/// The homepage's header, repeated on every generated page so the site
/// reads as one: the name (linking home here, since posts own the h1),
/// the sentence, the links and the PGP line. Keep in step with
/// `frontend/index.html`.
const SITE_HEADER: &str = r#"<header class="intro">
      <p class="name"><a href="/">Cole Leavitt</a></p>
      <p class="statement">I build and break systems software in <span class="accent">Rust</span>.</p>
      <nav class="links" aria-label="Elsewhere">
        <a href="/writing/"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M5 10.5a.5.5 0 0 1 .5-.5h2a.5.5 0 0 1 0 1h-2a.5.5 0 0 1-.5-.5m0-2a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m0-2a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m0-2a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5"/><path d="M3 0h10a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2v-1h1v1a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1H3a1 1 0 0 0-1 1v1H1V2a2 2 0 0 1 2-2"/><path d="M1 5v-.5a.5.5 0 0 1 1 0V5h.5a.5.5 0 0 1 0 1h-2a.5.5 0 0 1 0-1zm0 3v-.5a.5.5 0 0 1 1 0V8h.5a.5.5 0 0 1 0 1h-2a.5.5 0 0 1 0-1zm0 3v-.5a.5.5 0 0 1 1 0v.5h.5a.5.5 0 0 1 0 1h-2a.5.5 0 0 1 0-1z"/></svg>Writing</a>
        <a href="https://github.com/coleleavitt" rel="noopener noreferrer"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8"/></svg>GitHub</a>
        <a href="https://www.linkedin.com/in/coleleavitt/" rel="noopener noreferrer"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M0 1.146C0 .513.526 0 1.175 0h13.65C15.474 0 16 .513 16 1.146v13.708c0 .633-.526 1.146-1.175 1.146H1.175C.526 16 0 15.487 0 14.854zm4.943 12.248V6.169H2.542v7.225zm-1.2-8.212c.837 0 1.358-.554 1.358-1.248-.015-.709-.52-1.248-1.342-1.248S2.4 3.226 2.4 3.934c0 .694.521 1.248 1.327 1.248zm4.908 8.212V9.359c0-.216.016-.432.08-.586.173-.431.568-.878 1.232-.878.869 0 1.216.662 1.216 1.634v3.865h2.401V9.25c0-2.22-1.184-3.252-2.764-3.252-1.274 0-1.845.7-2.165 1.193v.025h-.016l.016-.025V6.169h-2.4c.03.678 0 7.225 0 7.225z"/></svg>LinkedIn</a>
        <a href="mailto:cole@unwrap.rs"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M.05 3.555A2 2 0 0 1 2 2h12a2 2 0 0 1 1.95 1.555L8 8.414zM0 4.697v7.104l5.803-3.558zM6.761 8.83l-6.57 4.027A2 2 0 0 0 2 14h12a2 2 0 0 0 1.808-1.144l-6.57-4.027L8 9.586zm3.436-.586L16 11.801V4.697z"/></svg>Email</a>
      </nav>
      <p class="pgp"><a href="/pgp.asc" title="OpenPGP key"><span><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M3.5 11.5a3.5 3.5 0 1 1 3.163-5H14L15.5 8 14 9.5l-1-1-1 1-1-1-1 1-1-1-1 1H6.663a3.5 3.5 0 0 1-3.163 2M2.5 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2"/></svg>PGP DEC2 3202 3B77 3F66 98EA</span> <span>54F1 1DFB 3167 910E 6193</span></a><a href="https://keyoxide.org/DEC232023B773F6698EA54F11DFB3167910E6193">verify</a></p>
    </header>"#;

const LATEST_START: &str = "<!-- sitegen:latest -->";
const LATEST_END: &str = "<!-- /sitegen:latest -->";
const LATEST_COUNT: usize = 5;

/// Page shell matching the homepage: same stylesheet, font and column.
fn page(
    title: &str,
    description: &str,
    url: &str,
    og_type: &str,
    css_href: &str,
    main: &str,
) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>{title}</title>
  <meta name="description" content="{description}" />
  <meta name="author" content="Cole Leavitt" />
  <link rel="canonical" href="{url}" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover" />
  <meta name="theme-color" content="#0d0a10" />
  <meta property="og:type" content="{og_type}" />
  <meta property="og:title" content="{title}" />
  <meta property="og:description" content="{description}" />
  <meta property="og:url" content="{url}" />
  <meta property="og:site_name" content="Cole Leavitt" />
  <link rel="icon" href="/favicon.ico" sizes="32x32" />
  <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
  <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
  <link rel="alternate" type="application/atom+xml" title="Cole Leavitt: writing" href="/feed.xml" />
  <link rel="preload" href="/fonts/JetBrainsMono-latin.woff2" as="font" type="font/woff2" crossorigin />
  <link rel="stylesheet" href="{css_href}" />
</head>
<body>
  <main class="page">
    {SITE_HEADER}
{main}
  </main>
</body>
</html>
"##
    )
}

/// Post list shared by /writing/ and the homepage, laid out like the other
/// homepage entries: the title on the left, the month on the right.
fn dated_entries<'a>(posts: impl IntoIterator<Item = &'a Post>) -> String {
    let mut out = String::from("    <ul class=\"entries\">\n");
    for p in posts {
        writeln!(
            out,
            r#"      <li><a href="/writing/{slug}/"><span class="entry-title">{title}</span><time class="entry-meta" datetime="{date}">{month}</time></a></li>"#,
            slug = p.slug,
            title = esc(&p.title),
            date = p.date,
            month = p.date.get(..7).unwrap_or(&p.date),
        )
        .expect("writing to a String cannot fail");
    }
    out.push_str("    </ul>\n");
    out
}

fn post_page(post: &Post, css_href: &str) -> String {
    let main = format!(
        r#"    <article>
      <h1 class="post-title">{title}</h1>
      <p class="post-meta"><time datetime="{date}">{date}</time></p>
      <div class="post-body">
{body}      </div>
    </article>"#,
        title = esc(&post.title),
        date = post.date,
        body = post.body_html,
    );
    page(
        &format!("{} — Cole Leavitt", esc(&post.title)),
        &esc(&post.description),
        &post.url(),
        "article",
        css_href,
        &main,
    )
}

fn writing_index(posts: &[Post], css_href: &str) -> String {
    let list = if posts.is_empty() {
        r#"    <p class="entries-empty">No posts yet. Current work is on <a href="https://github.com/coleleavitt">GitHub</a>.</p>
"#
        .to_string()
    } else {
        dated_entries(posts)
    };
    let main = format!(
        r#"    <h1 class="post-title">Writing</h1>
    <p class="post-meta">Research notes, advisories and write-ups. <a href="/feed.xml"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2zm1.5 2.5c5.523 0 10 4.477 10 10a1 1 0 1 1-2 0 8 8 0 0 0-8-8 1 1 0 0 1 0-2m0 4a6 6 0 0 1 6 6 1 1 0 1 1-2 0 4 4 0 0 0-4-4 1 1 0 0 1 0-2m.5 7a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3"/></svg>Atom feed</a></p>
{list}"#
    );
    page(
        "Writing — Cole Leavitt",
        "Security research notes, advisories and write-ups by Cole Leavitt.",
        &format!("{SITE_URL}/writing/"),
        "website",
        css_href,
        &main,
    )
}

/// Homepage section listing the newest posts that the homepage does not
/// already link to (a post featured under Research is not repeated here).
/// Empty when no such post exists.
fn latest_section(posts: &[Post], home_html: &str) -> String {
    // Ignore links inside an earlier run's injected region.
    let page = inject_latest(home_html, "").unwrap_or_else(|| home_html.to_string());
    let unfeatured: Vec<&Post> = posts
        .iter()
        .filter(|p| !page.contains(&format!("href=\"/writing/{}/\"", p.slug)))
        .take(LATEST_COUNT)
        .collect();
    if unfeatured.is_empty() {
        return String::new();
    }
    let list = dated_entries(unfeatured);
    format!(
        r#"
    <section class="section" aria-labelledby="writing-heading">
      <h2 id="writing-heading" class="section-label">Writing</h2>
{list}    </section>
    "#
    )
}

/// Replace the region after the `sitegen:latest` marker (bare, or from an
/// earlier run) with `section`. Returns `None` when the marker is missing.
fn inject_latest(index_html: &str, section: &str) -> Option<String> {
    let start = index_html.find(LATEST_START)?;
    let after = start + LATEST_START.len();
    let end = index_html[after..]
        .find(LATEST_END)
        .map_or(after, |e| after + e + LATEST_END.len());
    Some(format!(
        "{}{LATEST_START}{section}{LATEST_END}{}",
        &index_html[..start],
        &index_html[end..]
    ))
}

fn atom_feed(posts: &[Post]) -> String {
    let base = SITE_URL;
    let updated = posts.first().map_or_else(today_utc, |p| p.date.clone());
    let mut entries = String::new();
    for p in posts {
        write!(
            entries,
            r#"  <entry>
    <title>{title}</title>
    <link href="{url}" />
    <id>{url}</id>
    <updated>{date}T00:00:00Z</updated>
    <summary>{desc}</summary>
  </entry>
"#,
            title = esc(&p.title),
            url = p.url(),
            date = p.date,
            desc = esc(&p.description),
        )
        .expect("writing to a String cannot fail");
    }
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Cole Leavitt</title>
  <link href="{base}/writing/" />
  <id>{base}/writing/</id>
  <updated>{updated}T00:00:00Z</updated>
{entries}</feed>
"#
    )
}

fn sitemap(posts: &[Post]) -> String {
    let mut urls = format!(
        "  <url>
    <loc>{SITE_URL}/</loc>
  </url>
  <url>
    <loc>{SITE_URL}/writing/</loc>
  </url>
"
    );
    for p in posts {
        write!(urls, "  <url>\n    <loc>{}</loc>\n  </url>\n", p.url())
            .expect("writing to a String cannot fail");
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset
  xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:schemaLocation="http://www.sitemaps.org/schemas/sitemap/0.9 https://www.sitemaps.org/schemas/sitemap/0.9/sitemap.xsd"
>
{urls}</urlset>
"#
    )
}

/// Load and render every non-draft post from `content_dir`.
fn load_posts(content_dir: &Path) -> Vec<Post> {
    let Ok(entries) = fs::read_dir(content_dir) else {
        return Vec::new();
    };
    let mut posts = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let Ok(src) = fs::read_to_string(&path) else {
            continue;
        };
        let (fields, body) = parse_frontmatter(&src);
        if field(&fields, "draft").is_some_and(|d| d == "true") {
            continue;
        }
        let Some(title) = field(&fields, "title").map(str::to_string) else {
            eprintln!(
                "sitegen: skipping {} (missing title)",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
            continue;
        };
        let Some(date) = field(&fields, "date").map(str::to_string) else {
            eprintln!("sitegen: skipping {title:?} (missing date)");
            continue;
        };
        posts.push(Post {
            slug: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            title,
            date,
            description: field(&fields, "description")
                .unwrap_or_default()
                .to_string(),
            body_html: render_markdown(body.trim_start()),
        });
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.slug.cmp(&a.slug)));
    posts
}

fn write_dist(dist: &Path, rel: &str, contents: &str) -> std::io::Result<()> {
    let path: PathBuf = dist.join(rel);
    fs::create_dir_all(path.parent().unwrap_or(dist))?;
    fs::write(path, contents)
}

fn main() -> std::io::Result<()> {
    let root = std::env::var("SITE_ROOT").unwrap_or_else(|_| ".".to_string());
    let root = PathBuf::from(root);
    let content_dir =
        root.join(std::env::var("CONTENT_DIR").unwrap_or_else(|_| "content/writing".to_string()));
    let dist = root.join(std::env::var("DIST_DIR").unwrap_or_else(|_| "dist".to_string()));

    let index_html = fs::read_to_string(dist.join("index.html")).unwrap_or_default();
    let css_href = stylesheet_href(&index_html);

    let posts = load_posts(&content_dir);

    // Remove stale post pages from earlier runs so deleted posts disappear.
    let _ = fs::remove_dir_all(dist.join("writing"));

    write_dist(
        &dist,
        "writing/index.html",
        &writing_index(&posts, &css_href),
    )?;
    for post in &posts {
        write_dist(
            &dist,
            &format!("writing/{}/index.html", post.slug),
            &post_page(post, &css_href),
        )?;
    }
    if let Some(home) = inject_latest(&index_html, &latest_section(&posts, &index_html)) {
        fs::write(dist.join("index.html"), home)?;
    }
    write_dist(&dist, "feed.xml", &atom_feed(&posts))?;
    write_dist(&dist, "sitemap.xml", &sitemap(&posts))?;

    println!("sitegen: {} post(s), css={css_href}", posts.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_is_split_from_body() {
        let src = "+++
title: Hi there
date: 2026-11-07
draft: true
+++

# Body
";
        let (fields, body) = parse_frontmatter(src);
        assert_eq!(field(&fields, "title"), Some("Hi there"));
        assert_eq!(field(&fields, "date"), Some("2026-11-07"));
        assert_eq!(field(&fields, "draft"), Some("true"));
        assert!(body.trim_start().starts_with("# Body"));
    }

    #[test]
    fn missing_frontmatter_returns_whole_source() {
        let (fields, body) = parse_frontmatter(
            "# Just markdown
",
        );
        assert_eq!(fields, Vec::<(String, String)>::new());
        assert_eq!(
            body,
            "# Just markdown
"
        );
    }

    #[test]
    fn mermaid_fence_becomes_inline_svg_figure() {
        let html = render_markdown(
            "before\n\n```mermaid\nflowchart TD\n A[Up] --> B{Ask}\n```\n\nafter\n",
        );
        assert!(html.contains("<figure class=\"diagram\" tabindex=\"0\"><svg"));
        assert!(html.contains("before"));
        assert!(html.contains("after"));
        assert!(!html.contains("```"));
    }

    #[test]
    fn escapes_html_entities() {
        let input = format!("a<b>&{q}c{q}", q = '"');
        assert_eq!(esc(&input), "a&lt;b&gt;&amp;&quot;c&quot;");
    }

    #[test]
    fn civil_date_matches_known_days() {
        assert_eq!(civil_date(0), "1970-01-01");
        assert_eq!(civil_date(19_722), "2023-12-31");
        assert_eq!(civil_date(19_723), "2024-01-01");
    }

    #[test]
    fn stylesheet_href_finds_hashed_css() {
        let html = r#"<link rel="stylesheet" href="/index-a1b2c3d4.css">"#;
        assert_eq!(stylesheet_href(html), "/index-a1b2c3d4.css");
        assert_eq!(stylesheet_href("<html></html>"), "index.css");
    }

    #[test]
    fn latest_injection_replaces_its_own_region() {
        let home = "<main>a<!-- sitegen:latest -->b</main>";
        let once = inject_latest(home, "X").expect("marker present");
        assert_eq!(
            once,
            "<main>a<!-- sitegen:latest -->X<!-- /sitegen:latest -->b</main>"
        );
        let twice = inject_latest(&once, "Y").expect("marker kept");
        assert_eq!(
            twice,
            "<main>a<!-- sitegen:latest -->Y<!-- /sitegen:latest -->b</main>"
        );
        assert_eq!(inject_latest("<main></main>", "X"), None);
    }

    #[test]
    fn latest_skips_posts_the_homepage_already_links() {
        let post = |slug: &str| Post {
            slug: slug.into(),
            title: slug.into(),
            date: "2026-11-07".into(),
            description: String::new(),
            body_html: String::new(),
        };
        let posts = [post("featured"), post("other")];
        let home = r#"<a href="/writing/featured/">x</a><!-- sitegen:latest -->"#;
        let section = latest_section(&posts, home);
        assert!(section.contains("/writing/other/") && !section.contains("/writing/featured/"));
        assert_eq!(latest_section(&posts[..1], home), "");
        // Links from an earlier run's injected list do not count as featured.
        let rerun = inject_latest(home, &section).unwrap();
        assert!(latest_section(&posts, &rerun).contains("/writing/other/"));
    }

    #[test]
    fn sitemap_lists_home_and_writing() {
        let map = sitemap(&[]);
        assert!(map.contains(&format!("<loc>{SITE_URL}/</loc>")));
        assert!(map.contains(&format!("<loc>{SITE_URL}/writing/</loc>")));
    }
}
