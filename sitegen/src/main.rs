//! Static site generator for the /writing/ section.
//!
//! Reads Markdown posts from `content/writing/` (run from the repo root, or set
//! `CONTENT_DIR`/`DIST_DIR`) and writes plain HTML pages, an Atom feed and a
//! sitemap into the Trunk output directory. Posts are static HTML on purpose:
//! they are readable without JavaScript and crawlable, which the CSR app
//! cannot offer.

mod entries;

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use std::sync::LazyLock;

use entries::{Entries, Entry, Page, sections_html};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::{SyntaxDefinition, SyntaxSet};
use syntect::util::LinesWithEndings;

const SITE_URL: &str = "https://unwrap.rs";

/// One parsed post.
struct Post {
    slug: String,
    title: String,
    date: String,
    description: String,
    body_html: String,
    /// Estimated reading time, whole minutes (at least 1).
    reading_minutes: usize,
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

/// Render post Markdown to HTML. Fenced code blocks go through
/// [`render_code`]; everything else is plain pulldown-cmark.
fn render_markdown(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let mut events = Vec::new();
    // (language, text) of the code block being collected
    let mut code: Option<(String, String)> = None;
    for event in Parser::new_ext(md, options) {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split(|c: char| c == ',' || c.is_whitespace())
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code = Some((lang, String::new()));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((lang, text)) = code.take() {
                    events.push(Event::Html(render_code(&lang, &text).into()));
                }
            }
            Event::Text(text) => match &mut code {
                Some((_, buf)) => buf.push_str(&text),
                None => events.push(Event::Text(text)),
            },
            Event::Code(text) => events.push(Event::InlineHtml(inline_code(&text).into())),
            other => events.push(other),
        }
    }
    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    out
}

/// Inline code that may wrap only at its spaces: each word is unbreakable,
/// so a flag like `-march=x` never splits at its hyphen, while a long
/// fingerprint still wraps on a phone.
fn inline_code(text: &str) -> String {
    let words: Vec<String> = text
        .split(' ')
        .map(|w| format!("<span>{}</span>", esc(w)))
        .collect();
    format!("<code>{}</code>", words.join(" "))
}

/// The live Xtensa disassembler. The fence holds an optional `base <hex>`
/// line and the example bytes; the listing is rendered here, with the same
/// decoder the page later runs as WebAssembly (`/demos/xtensa-dis/`), so the
/// example reads the same without JavaScript. Inputs start read-only; the
/// WebAssembly module makes them editable when it loads.
fn xtensa_demo(src: &str) -> String {
    let mut base = "0x0".to_string();
    let mut hex = String::new();
    for line in src.lines() {
        if let Some(b) = line.trim().strip_prefix("base ") {
            base = format!("0x{}", b.trim().trim_start_matches("0x"));
        } else {
            hex.push_str(line.trim());
            hex.push('\n');
        }
    }

    let hex = hex.trim().to_string();
    let address =
        xtensa_dis::parse_address(&base).unwrap_or_else(|e| panic!("xtensa-demo base: {e}"));
    let code = xtensa_dis::parse_hex(&hex).unwrap_or_else(|e| panic!("xtensa-demo bytes: {e}"));
    let rows = hex.lines().count().clamp(2, 8);
    format!(
        r#"<figure class="xtensa-demo">
<div class="xtensa-inputs">
<label>Bytes (hex)<textarea class="xtensa-bytes" rows="{rows}" spellcheck="false" autocomplete="off" readonly>{hex}</textarea></label>
<label>Base address<input class="xtensa-base" value="{base}" spellcheck="false" autocomplete="off" readonly></label>
</div>
<pre class="xtensa-out" tabindex="0" aria-label="Disassembly">{listing}</pre>
</figure>
"#,
        hex = esc(&hex),
        base = esc(&base),
        listing = xtensa_dis::listing_html(&code, address),
    )
}

/// The content-hashed module that makes `.xtensa-demo` figures live
/// (`demos/xtensa-dis/boot-<hash>.js`, written by the Trunk hook), as a
/// site path.
fn xtensa_demo_boot(dist: &Path) -> Option<String> {
    fs::read_dir(dist.join("demos/xtensa-dis"))
        .ok()?
        .flatten()
        .find_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            (name.starts_with("boot-")
                && Path::new(&name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("js")))
            .then(|| format!("/demos/xtensa-dis/{name}"))
        })
}

/// Grammars for build-time highlighting: syntect's bundled Sublime syntaxes,
/// plus a current Rust grammar. The bundled ones date from Sublime Text 3
/// (sublimehq/Packages v3154); the vendored Rust grammar (sublimehq/Packages
/// 7f74b54, see THIRD-PARTY-NOTICES.md) scopes types such as `Result` or a
/// struct name as `storage.type`. Lookups prefer the last definition added.
static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(|| {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    let rust = SyntaxDefinition::load_from_str(
        include_str!("../syntaxes/Rust.sublime-syntax"),
        true,
        None,
    )
    .expect("the vendored Rust grammar is valid");
    builder.add(rust);
    builder.build()
});

/// One fenced code block as HTML, rendered at build time so pages need no
/// script: `mermaid` becomes an inline SVG diagram, a known language gets
/// `hl-*` classed spans (colors live in `index.css`), anything else, such
/// as logs and tool output, stays plain.
fn render_code(lang: &str, code: &str) -> String {
    if lang == "xtensa-demo" {
        return xtensa_demo(code);
    }
    if lang == "mermaid" {
        let svg =
            mmd_svg::render(code).unwrap_or_else(|e| panic!("mermaid block failed to render: {e}"));
        // focusable so keyboard users can scroll a wide diagram
        return format!("<figure class=\"diagram\" tabindex=\"0\">{svg}</figure>\n");
    }
    let syntax = SYNTAXES
        .find_syntax_by_token(lang)
        .filter(|s| !lang.is_empty() && s.name != "Plain Text");
    let Some(syntax) = syntax else {
        return format!("<pre><code>{}</code></pre>\n", esc(code));
    };
    let mut hl = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        &SYNTAXES,
        ClassStyle::SpacedPrefixed { prefix: "hl-" },
    );
    for line in LinesWithEndings::from(code) {
        hl.parse_html_for_line_which_includes_newline(line)
            .unwrap_or_else(|e| panic!("highlighting a {lang} block failed: {e}"));
    }
    format!(
        "<pre class=\"hl\"><code class=\"language-{}\">{}</code></pre>\n",
        esc(lang),
        hl.finalize()
    )
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
/// reads as one: the name, the sentence, the links and the PGP line. The
/// homepage gets the same header through its `sitegen:header` marker.
const SITE_HEADER: &str = r#"<header class="intro">
      <!-- name -->
      <p class="statement">I build and break systems software in <span class="accent">Rust</span>.</p>
      <nav class="links" aria-label="Elsewhere">
        <a href="/writing/"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M5 10.5a.5.5 0 0 1 .5-.5h2a.5.5 0 0 1 0 1h-2a.5.5 0 0 1-.5-.5m0-2a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m0-2a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m0-2a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5"/><path d="M3 0h10a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2v-1h1v1a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1H3a1 1 0 0 0-1 1v1H1V2a2 2 0 0 1 2-2"/><path d="M1 5v-.5a.5.5 0 0 1 1 0V5h.5a.5.5 0 0 1 0 1h-2a.5.5 0 0 1 0-1zm0 3v-.5a.5.5 0 0 1 1 0V8h.5a.5.5 0 0 1 0 1h-2a.5.5 0 0 1 0-1zm0 3v-.5a.5.5 0 0 1 1 0v.5h.5a.5.5 0 0 1 0 1h-2a.5.5 0 0 1 0-1z"/></svg>Writing</a>
        <a href="/cv/"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M5 8a2 2 0 1 0 0-4 2 2 0 0 0 0 4m4-2.5a.5.5 0 0 1 .5-.5h4a.5.5 0 0 1 0 1h-4a.5.5 0 0 1-.5-.5M9 8a.5.5 0 0 1 .5-.5h4a.5.5 0 0 1 0 1h-4A.5.5 0 0 1 9 8m1 2.5a.5.5 0 0 1 .5-.5h3a.5.5 0 0 1 0 1h-3a.5.5 0 0 1-.5-.5"/><path d="M2 2a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2zM1 4a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H8.96q.04-.245.04-.5C9 10.567 7.21 9 5 9c-2.086 0-3.8 1.398-3.984 3.181A1 1 0 0 1 1 12z"/></svg>CV</a>
        <a href="/uses/"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M13.5 3a.5.5 0 0 1 .5.5V11H2V3.5a.5.5 0 0 1 .5-.5zm-11-1A1.5 1.5 0 0 0 1 3.5V12h14V3.5A1.5 1.5 0 0 0 13.5 2zM0 12.5h16a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 0 12.5"/></svg>Uses</a>
        <a href="https://github.com/coleleavitt" rel="noopener noreferrer"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8"/></svg>GitHub</a>
        <a href="https://www.linkedin.com/in/coleleavitt/" rel="noopener noreferrer"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M0 1.146C0 .513.526 0 1.175 0h13.65C15.474 0 16 .513 16 1.146v13.708c0 .633-.526 1.146-1.175 1.146H1.175C.526 16 0 15.487 0 14.854zm4.943 12.248V6.169H2.542v7.225zm-1.2-8.212c.837 0 1.358-.554 1.358-1.248-.015-.709-.52-1.248-1.342-1.248S2.4 3.226 2.4 3.934c0 .694.521 1.248 1.327 1.248zm4.908 8.212V9.359c0-.216.016-.432.08-.586.173-.431.568-.878 1.232-.878.869 0 1.216.662 1.216 1.634v3.865h2.401V9.25c0-2.22-1.184-3.252-2.764-3.252-1.274 0-1.845.7-2.165 1.193v.025h-.016l.016-.025V6.169h-2.4c.03.678 0 7.225 0 7.225z"/></svg>LinkedIn</a>
        <a href="mailto:cole@unwrap.rs"><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M.05 3.555A2 2 0 0 1 2 2h12a2 2 0 0 1 1.95 1.555L8 8.414zM0 4.697v7.104l5.803-3.558zM6.761 8.83l-6.57 4.027A2 2 0 0 0 2 14h12a2 2 0 0 0 1.808-1.144l-6.57-4.027L8 9.586zm3.436-.586L16 11.801V4.697z"/></svg>Email</a>
      </nav>
      <p class="pgp"><a href="/pgp.asc" title="OpenPGP key"><span><svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M3.5 11.5a3.5 3.5 0 1 1 3.163-5H14L15.5 8 14 9.5l-1-1-1 1-1-1-1 1-1-1-1 1H6.663a3.5 3.5 0 0 1-3.163 2M2.5 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2"/></svg>PGP DEC2 3202 3B77 3F66 98EA</span> <span>54F1 1DFB 3167 910E 6193</span></a><a href="https://keyoxide.org/DEC232023B773F6698EA54F11DFB3167910E6193">verify</a></p>
      <!-- portrait -->
    </header>"#;

const LATEST_COUNT: usize = 5;

/// The shared header. The homepage's name is its h1; elsewhere the name
/// links home, since posts own the h1.
fn site_header(home: bool) -> String {
    let name = if home {
        r#"<h1 class="name">Cole Leavitt</h1>"#
    } else {
        r#"<p class="name"><a href="/">Cole Leavitt</a></p>"#
    };
    // The portrait appears on the homepage only, beside the name.
    let portrait = if home {
        r#"<img class="portrait" src="/cole.jpg" width="176" height="176" alt="Cole Leavitt" decoding="async" />"#
    } else {
        ""
    };
    SITE_HEADER
        .replace("<!-- name -->", name)
        .replace("<!-- portrait -->", portrait)
}

/// Page shell matching the homepage: same stylesheet, font and column.
fn page(
    title: &str,
    description: &str,
    url: &str,
    og_type: &str,
    css_href: &str,
    main: &str,
) -> String {
    let header = site_header(false);
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
    {header}
{main}
  </main>
</body>
</html>
"##
    )
}

/// The CV: every `cv` section of `content/entries.toml`, in the homepage's
/// entry layout.
fn cv_page(entries: &Entries, css_href: &str) -> String {
    let main = format!(
        "    <h1 class=\"post-title\">{}</h1>\n    <div class=\"cv\">\n{}    </div>",
        esc(&entries.cv.title),
        sections_html(entries, Page::Cv)
    );
    page(
        &format!("{} — Cole Leavitt", esc(&entries.cv.title)),
        &esc(&entries.cv.description),
        &format!("{SITE_URL}/cv/"),
        "profile",
        css_href,
        &main,
    )
}

/// Post list shared by /writing/ and the homepage, laid out like the other
/// homepage entries: the title on the left, the month on the right.
fn dated_entries<'a>(posts: impl IntoIterator<Item = &'a Post>) -> String {
    let rows: Vec<Entry> = posts
        .into_iter()
        .map(|p| Entry {
            name: p.title.clone(),
            date: Some(p.date.get(..7).unwrap_or(&p.date).to_string()),
            href: Some(format!("/writing/{}/", p.slug)),
            ..Entry::default()
        })
        .collect();
    entries::entries_list(&rows)
}

fn post_page(
    post: &Post,
    older: Option<&Post>,
    newer: Option<&Post>,
    css_href: &str,
    demo_boot: Option<&str>,
) -> String {
    let link = |p: &Post, rel: &str, label: &str| {
        format!(
            r#"      <a class="post-nav-{rel}" rel="{rel}" href="/writing/{slug}/"><span class="post-nav-label">{label}</span><span class="post-nav-title">{title}</span></a>
"#,
            slug = p.slug,
            title = esc(&p.title),
        )
    };
    let nav = if older.is_none() && newer.is_none() {
        String::new()
    } else {
        format!(
            "    <nav class=\"post-nav\" aria-label=\"More writing\">\n{}{}    </nav>\n",
            older.map(|p| link(p, "prev", "Older")).unwrap_or_default(),
            newer.map(|p| link(p, "next", "Newer")).unwrap_or_default(),
        )
    };
    let main = format!(
        r#"    <article>
      <h1 class="post-title">{title}</h1>
      <p class="post-meta"><time datetime="{date}">{date}</time> · {minutes} min read</p>
      <div class="post-body">
{body}      </div>
    </article>
{nav}"#,
        title = esc(&post.title),
        date = post.date,
        minutes = post.reading_minutes,
        body = post.body_html,
    );
    let html = page(
        &format!("{} — Cole Leavitt", esc(&post.title)),
        &esc(&post.description),
        &post.url(),
        "article",
        css_href,
        &main,
    );
    if !post.body_html.contains(r#"<figure class="xtensa-demo">"#) {
        return html;
    }
    let boot = demo_boot.unwrap_or_else(|| {
        panic!(
            "{} has a live demo, but demos/xtensa-dis/ was not built (run through trunk)",
            post.slug
        )
    });
    html.replacen(
        "</head>",
        &format!("  <script type=\"module\" src=\"{boot}\"></script>\n</head>"),
        1,
    )
}

/// A standalone page from `content/pages/<slug>.md`, served at `/<slug>/`.
struct StaticPage {
    slug: String,
    title: String,
    description: String,
    /// `YYYY-MM-DD` the page was last brought up to date.
    updated: String,
    body_html: String,
}

/// Load every page in `content/pages/`; a missing directory means no pages.
fn load_pages(dir: &Path) -> Result<Vec<StaticPage>, String> {
    let Ok(read) = fs::read_dir(dir) else {
        return Ok(Vec::new());
    };
    let mut pages = Vec::new();
    for entry in read {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let name = path.display().to_string();
        let src = fs::read_to_string(&path).map_err(|e| format!("{name}: {e}"))?;
        let (fields, body) = parse_frontmatter(&src);
        let slug = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        // these names already belong to other parts of the site
        if !is_slug(&slug) || matches!(slug.as_str(), "writing" | "cv" | "api" | "fonts" | "icons")
        {
            return Err(format!("{name}: file name is not a free [a-z0-9-] slug"));
        }
        let title = field(&fields, "title").ok_or_else(|| format!("{name}: missing title"))?;
        let updated =
            field(&fields, "updated").ok_or_else(|| format!("{name}: missing updated"))?;
        if !is_calendar_date(updated) {
            return Err(format!(
                "{name}: updated {updated:?} is not a YYYY-MM-DD calendar date"
            ));
        }
        pages.push(StaticPage {
            slug,
            title: title.to_string(),
            description: field(&fields, "description")
                .unwrap_or_default()
                .to_string(),
            updated: updated.to_string(),
            body_html: render_markdown(body.trim_start()),
        });
    }
    pages.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(pages)
}

fn static_page(p: &StaticPage, css_href: &str) -> String {
    let main = format!(
        r#"    <article>
      <h1 class="post-title">{title}</h1>
      <p class="post-meta">Updated <time datetime="{updated}">{updated}</time></p>
      <div class="post-body">
{body}      </div>
    </article>"#,
        title = esc(&p.title),
        updated = p.updated,
        body = p.body_html,
    );
    page(
        &format!("{} — Cole Leavitt", esc(&p.title)),
        &esc(&p.description),
        &format!("{SITE_URL}/{}/", p.slug),
        "website",
        css_href,
        &main,
    )
}

/// The page served for unknown paths (with status 404, by the server). Not
/// indexed and without a canonical URL, since it has no address of its own.
fn not_found_page(css_href: &str) -> String {
    let main = r#"    <h1 class="post-title">404</h1>
    <p class="post-meta">Not found</p>
    <div class="post-body">
      <p>This path returned <code>Err(NotFound)</code>. Calling <code>.unwrap()</code> on it would panic, so here is what does exist:</p>
      <ul>
        <li><a href="/">Home</a></li>
        <li><a href="/writing/">Writing</a></li>
        <li><a href="/cv/">CV</a></li>
        <li><a href="/uses/">Uses</a></li>
      </ul>
    </div>"#;
    let html = page(
        "Not found — Cole Leavitt",
        "No page at this address.",
        SITE_URL,
        "website",
        css_href,
        main,
    );
    html.replace(
        &format!("  <link rel=\"canonical\" href=\"{SITE_URL}\" />\n"),
        "  <meta name=\"robots\" content=\"noindex\" />\n",
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
    let page = inject(home_html, "latest", "").unwrap_or_else(|| home_html.to_string());
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

/// Fill the region after `<!-- sitegen:NAME -->` with `content`, replacing
/// what an earlier run put there (up to `<!-- /sitegen:NAME -->`). `None`
/// when the page has no such marker.
fn inject(html: &str, name: &str, content: &str) -> Option<String> {
    let start_marker = format!("<!-- sitegen:{name} -->");
    let end_marker = format!("<!-- /sitegen:{name} -->");
    let start = html.find(&start_marker)?;
    let after = start + start_marker.len();
    let end = html[after..]
        .find(&end_marker)
        .map_or(after, |e| after + e + end_marker.len());
    Some(format!(
        "{}{start_marker}{content}{end_marker}{}",
        &html[..start],
        &html[end..]
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
  <author>
    <name>Cole Leavitt</name>
    <uri>{base}/</uri>
  </author>
{entries}</feed>
"#
    )
}

fn sitemap(posts: &[Post], pages: &[StaticPage]) -> String {
    let mut urls = format!(
        "  <url>
    <loc>{SITE_URL}/</loc>
  </url>
  <url>
    <loc>{SITE_URL}/writing/</loc>
  </url>
"
    );
    write!(urls, "  <url>\n    <loc>{SITE_URL}/cv/</loc>\n  </url>\n")
        .expect("writing to a String cannot fail");
    for p in pages {
        write!(
            urls,
            "  <url>\n    <loc>{SITE_URL}/{}/</loc>\n  </url>\n",
            p.slug
        )
        .expect("writing to a String cannot fail");
    }
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

/// Load and render every non-draft post from `content_dir`. Any unreadable
/// file, missing field, invalid date or unsafe file name fails the build, so
/// a broken post never silently disappears from a release.
fn load_posts(content_dir: &Path) -> Result<Vec<Post>, String> {
    let dir_err = |e: std::io::Error| format!("{}: {e}", content_dir.display());
    let mut posts = Vec::new();
    for entry in fs::read_dir(content_dir).map_err(dir_err)? {
        let path = entry.map_err(dir_err)?.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let name = path.display().to_string();
        let src = fs::read_to_string(&path).map_err(|e| format!("{name}: {e}"))?;
        let (fields, body) = parse_frontmatter(&src);
        if field(&fields, "draft").is_some_and(|d| d == "true") {
            continue;
        }
        let slug = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !is_slug(&slug) {
            return Err(format!(
                "{name}: file name must be lowercase letters, digits and hyphens"
            ));
        }
        let title = field(&fields, "title").ok_or_else(|| format!("{name}: missing title"))?;
        let date = field(&fields, "date").ok_or_else(|| format!("{name}: missing date"))?;
        if !is_calendar_date(date) {
            return Err(format!(
                "{name}: date {date:?} is not a YYYY-MM-DD calendar date"
            ));
        }
        let body_html = render_markdown(body.trim_start());
        posts.push(Post {
            reading_minutes: reading_minutes(&body_html),
            body_html,
            slug,
            title: title.to_string(),
            date: date.to_string(),
            description: field(&fields, "description")
                .unwrap_or_default()
                .to_string(),
        });
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.slug.cmp(&a.slug)));
    Ok(posts)
}

/// Minutes to read rendered HTML at 200 words a minute (technical prose
/// reads slower than the usual 230-250), rounded up, at least 1.
fn reading_minutes(html: &str) -> usize {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    text.split_whitespace().count().div_ceil(200).max(1)
}

/// A post slug becomes a path segment and a URL, so keep it to `[a-z0-9-]`.
fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// `YYYY-MM-DD` naming a real day.
fn is_calendar_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |r: std::ops::Range<usize>| s.get(r).and_then(|t| t.parse::<u32>().ok());
    let (Some(y), Some(m), Some(d)) = (num(0..4), num(5..7), num(8..10)) else {
        return false;
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&d)
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

    let posts = load_posts(&content_dir).map_err(std::io::Error::other)?;
    let entries_path = root.join("content/entries.toml");
    let entries = Entries::parse(&fs::read_to_string(&entries_path)?)
        .map_err(|e| std::io::Error::other(format!("{}: {e}", entries_path.display())))?;

    // Remove stale post pages from earlier runs so deleted posts disappear.
    let _ = fs::remove_dir_all(dist.join("writing"));

    write_dist(
        &dist,
        "writing/index.html",
        &writing_index(&posts, &css_href),
    )?;
    let demo_boot = xtensa_demo_boot(&dist);
    // `posts` is newest first: the next element is the older post.
    for (i, post) in posts.iter().enumerate() {
        let newer = i.checked_sub(1).map(|n| &posts[n]);
        write_dist(
            &dist,
            &format!("writing/{}/index.html", post.slug),
            &post_page(
                post,
                posts.get(i + 1),
                newer,
                &css_href,
                demo_boot.as_deref(),
            ),
        )?;
    }
    let pages = load_pages(&root.join("content/pages")).map_err(std::io::Error::other)?;
    for p in &pages {
        let _ = fs::remove_dir_all(dist.join(&p.slug));
        write_dist(
            &dist,
            &format!("{}/index.html", p.slug),
            &static_page(p, &css_href),
        )?;
    }
    write_dist(&dist, "404.html", &not_found_page(&css_href))?;
    // Header and sections first: the latest-posts list skips posts the
    // finished page already links to.
    let missing = |name: &str| {
        std::io::Error::other(format!(
            "{}: no <!-- sitegen:{name} --> marker",
            dist.join("index.html").display()
        ))
    };
    let home =
        inject(&index_html, "header", &site_header(true)).ok_or_else(|| missing("header"))?;
    let home = inject(&home, "sections", &sections_html(&entries, Page::Home))
        .ok_or_else(|| missing("sections"))?;
    let latest = latest_section(&posts, &home);
    let home = inject(&home, "latest", &latest).ok_or_else(|| missing("latest"))?;
    fs::write(dist.join("index.html"), home)?;
    write_dist(&dist, "cv/index.html", &cv_page(&entries, &css_href))?;
    write_dist(&dist, "feed.xml", &atom_feed(&posts))?;
    write_dist(&dist, "sitemap.xml", &sitemap(&posts, &pages))?;

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
        // the diagram itself rendered: both labels and an edge
        assert!(html.contains(">Up<") && html.contains(">Ask<") && html.contains("<path"));
        assert!(html.contains("before"));
        assert!(html.contains("after"));
        assert!(!html.contains("```"));
    }

    #[test]
    fn known_languages_get_highlight_classes() {
        let html = render_markdown("```rust\nfn main() { let x = 1; }\n```\n");
        assert!(html.contains("<pre class=\"hl\"><code class=\"language-rust\">"));
        assert!(html.contains("hl-keyword") || html.contains("hl-storage"));
    }

    #[test]
    fn rust_types_use_the_vendored_grammar() {
        let html = render_markdown("```rust\nfn f(t: &Table) -> Result<u32, E> { todo!() }\n```\n");
        // the bundled 2018 grammar gives user types no storage.type scope
        assert!(html.contains("<span class=\"hl-storage hl-type hl-rust\">Table</span>"));
    }

    #[test]
    fn logs_and_unknown_languages_stay_plain_and_escaped() {
        let html = render_markdown("```\n<oops> & done\n```\n\n```txt\nplain\n```\n");
        assert!(html.contains("<pre><code>&lt;oops&gt; &amp; done\n</code></pre>"));
        assert!(html.contains("<pre><code>plain\n</code></pre>"));
        assert!(!html.contains("hl-"));
    }

    #[test]
    fn fence_info_splits_on_any_whitespace() {
        let html = render_markdown("```rust\tignore\nfn f() {}\n```\n");
        assert!(html.contains("language-rust"));
    }

    #[test]
    fn post_dates_and_slugs_are_validated() {
        assert!(is_calendar_date("2024-02-29") && !is_calendar_date("2023-02-29"));
        assert!(
            !is_calendar_date("2026-13-01")
                && !is_calendar_date("2026-1-01")
                && !is_calendar_date("26-11-07")
        );
        assert!(is_slug("codeflow-mir") && !is_slug("..") && !is_slug("a b") && !is_slug("Caps"));
    }

    #[test]
    fn unreadable_or_broken_posts_fail_the_build() {
        assert!(load_posts(Path::new("/nonexistent-content-dir")).is_err());
        let dir = std::env::temp_dir().join(format!("sitegen-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("ok.md"),
            "+++\ntitle: T\ndate: 2026-11-07\n+++\nbody\n",
        )
        .unwrap();
        assert_eq!(load_posts(&dir).unwrap().len(), 1);
        fs::write(dir.join("bad.md"), "+++\ntitle: T\ndate: 2026-02-30\n+++\n").unwrap();
        assert!(
            load_posts(&dir)
                .err()
                .is_some_and(|e| e.contains("calendar date"))
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn inline_code_wraps_only_between_words() {
        let html = render_markdown("Run `-march=a -O2` and `<x>`.");
        assert!(html.contains("<code><span>-march=a</span> <span>-O2</span></code>"));
        assert!(html.contains("<code><span>&lt;x&gt;</span></code>"));
    }

    #[test]
    fn xtensa_demo_renders_the_listing_at_build_time() {
        let html = render_markdown("```xtensa-demo\nbase 40080000\n36 41 00\n```\n");
        assert!(html.contains(r#"<figure class="xtensa-demo">"#));
        assert!(html.contains(r#"value="0x40080000""#));
        assert!(html.contains(r#"<span class="xt-mn">entry</span>      <span class="xt-reg">a1</span>, <span class="xt-imm">32</span>"#));
        assert!(html.contains("readonly"));
    }

    #[test]
    fn reading_time_counts_text_not_markup() {
        assert_eq!(reading_minutes("<p>short</p>"), 1);
        let long = format!("<pre class=\"x y z\">{}</pre>", "word ".repeat(401));
        assert_eq!(reading_minutes(&long), 3);
    }

    #[test]
    fn posts_link_their_neighbours() {
        let post = |slug: &str| Post {
            slug: slug.into(),
            title: slug.into(),
            date: "2026-11-07".into(),
            description: String::new(),
            body_html: String::new(),
            reading_minutes: 4,
        };
        let (old, mid, new) = (post("old"), post("mid"), post("new"));
        let html = post_page(&mid, Some(&old), Some(&new), "/a.css", None);
        assert!(html.contains(r#"rel="prev" href="/writing/old/""#));
        assert!(html.contains(r#"rel="next" href="/writing/new/""#));
        assert!(html.contains("· 4 min read"));
        assert!(!post_page(&mid, None, None, "/a.css", None).contains("post-nav"));
    }

    #[test]
    fn not_found_page_is_noindex_without_canonical() {
        let html = not_found_page("/a.css");
        assert!(html.contains(r#"<meta name="robots" content="noindex" />"#));
        assert!(!html.contains("rel=\"canonical\""));
        assert!(html.contains("Err(NotFound)"));
    }

    #[test]
    fn pages_need_a_free_slug_and_a_date() {
        let dir = std::env::temp_dir().join(format!("sitegen-pages-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("uses.md"),
            "+++\ntitle: Uses\nupdated: 2026-11-08\n+++\nhi\n",
        )
        .unwrap();
        let pages = load_pages(&dir).unwrap();
        assert_eq!((pages[0].slug.as_str(), pages.len()), ("uses", 1));
        assert!(static_page(&pages[0], "/a.css").contains("https://unwrap.rs/uses/"));
        fs::write(
            dir.join("cv.md"),
            "+++\ntitle: X\nupdated: 2026-11-08\n+++\n",
        )
        .unwrap();
        assert!(load_pages(&dir).is_err());
        fs::remove_dir_all(&dir).unwrap();
        assert!(load_pages(&dir).unwrap().is_empty());
    }

    #[test]
    fn feed_names_its_author() {
        assert!(atom_feed(&[]).contains("<author>\n    <name>Cole Leavitt</name>"));
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
        let once = inject(home, "latest", "X").expect("marker present");
        assert_eq!(
            once,
            "<main>a<!-- sitegen:latest -->X<!-- /sitegen:latest -->b</main>"
        );
        let twice = inject(&once, "latest", "Y").expect("marker kept");
        assert_eq!(
            twice,
            "<main>a<!-- sitegen:latest -->Y<!-- /sitegen:latest -->b</main>"
        );
        assert_eq!(inject("<main></main>", "latest", "X"), None);
    }

    #[test]
    fn latest_skips_posts_the_homepage_already_links() {
        let post = |slug: &str| Post {
            slug: slug.into(),
            title: slug.into(),
            date: "2026-11-07".into(),
            description: String::new(),
            body_html: String::new(),
            reading_minutes: 1,
        };
        let posts = [post("featured"), post("other")];
        let home = r#"<a href="/writing/featured/">x</a><!-- sitegen:latest -->"#;
        let section = latest_section(&posts, home);
        assert!(section.contains("/writing/other/") && !section.contains("/writing/featured/"));
        assert_eq!(latest_section(&posts[..1], home), "");
        // Links from an earlier run's injected list do not count as featured.
        let rerun = inject(home, "latest", &section).unwrap();
        assert!(latest_section(&posts, &rerun).contains("/writing/other/"));
    }

    #[test]
    fn sitemap_lists_home_and_writing() {
        let map = sitemap(&[], &[]);
        assert!(map.contains(&format!("<loc>{SITE_URL}/</loc>")));
        assert!(map.contains(&format!("<loc>{SITE_URL}/writing/</loc>")));
        assert!(map.contains(&format!("<loc>{SITE_URL}/cv/</loc>")));
    }

    #[test]
    fn cv_page_renders_cv_sections_with_the_shared_header() {
        let entries = Entries::parse(
            "[cv]\ntitle = \"CV\"\ndescription = \"d\"\n\n[[section]]\nid = \"edu\"\nlabel = \"Education\"\npages = [\"cv\"]\n\n[[section.entry]]\nname = \"School\"\nstart = \"2021\"\nend = \"2023\"\n",
        )
        .unwrap();
        let html = cv_page(&entries, "/a.css");
        assert!(html.contains("<title>CV — Cole Leavitt</title>"));
        assert!(html.contains("<link rel=\"canonical\" href=\"https://unwrap.rs/cv/\" />"));
        assert!(html.contains("2021 – 2023"));
        assert!(html.contains(r#"<p class="name"><a href="/">Cole Leavitt</a></p>"#));
    }

    #[test]
    fn homepage_header_uses_the_name_as_h1() {
        let home = site_header(true);
        assert!(home.contains(r#"<h1 class="name">Cole Leavitt</h1>"#));
        assert!(home.contains(r#"class="portrait""#) && !site_header(false).contains("portrait"));
        assert_eq!(
            home.replace(r#"<h1 class="name">Cole Leavitt</h1>"#, "")
                .replace(
                    r#"<img class="portrait" src="/cole.jpg" width="176" height="176" alt="Cole Leavitt" decoding="async" />"#,
                    ""
                ),
            site_header(false).replace(r#"<p class="name"><a href="/">Cole Leavitt</a></p>"#, "")
        );
    }
}
