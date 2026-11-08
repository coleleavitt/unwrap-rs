//! The site's dated entries (`content/entries.toml`), rendered the same way
//! on every page: the homepage sections, the CV, and the writing lists.

use std::fmt::Write as _;

use serde::Deserialize;

use crate::esc;

/// Where a section or entry appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Page {
    Home,
    Cv,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entries {
    pub cv: CvMeta,
    #[serde(rename = "section")]
    pub sections: Vec<Section>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CvMeta {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub id: String,
    pub label: String,
    pub pages: Vec<Page>,
    /// Trusted HTML shown under the list (it comes from this repository).
    pub more: Option<String>,
    #[serde(rename = "entry", default)]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub name: String,
    pub desc: Option<String>,
    pub date: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub href: Option<String>,
    #[serde(default)]
    pub refs: Vec<Ref>,
    pub pages: Option<Vec<Page>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ref {
    pub link: Option<String>,
    pub href: Option<String>,
    pub text: Option<String>,
}

/// `YYYY` or `YYYY-MM`.
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    match b.len() {
        4 => digits(0..4),
        7 => {
            digits(0..4)
                && b[4] == b'-'
                && digits(5..7)
                && (b"01".as_slice()..=b"12".as_slice()).contains(&&b[5..7])
        }
        _ => false,
    }
}

/// `end` is earlier than `start`, comparing at the coarser of the two
/// precisions (`2025` does not end before `2025-11`).
fn ends_before(start: &str, end: &str) -> bool {
    let n = start.len().min(end.len());
    end[..n] < start[..n]
}

impl Entries {
    /// Parse and check the file. Errors name the entry, so a bad date fails
    /// the build instead of reaching the page.
    pub fn parse(src: &str) -> Result<Self, String> {
        let entries: Self = toml::from_str(src).map_err(|e| e.to_string())?;
        for s in &entries.sections {
            for e in &s.entries {
                let bad = |what: &str| format!("{}: {:?}: {what}", s.id, e.name);
                match (&e.date, &e.start, &e.end) {
                    (Some(d), None, None) if is_date(d) => {}
                    (None, Some(a), Some(z))
                        if is_date(a) && (z == "now" || (is_date(z) && !ends_before(a, z))) => {}
                    (None, None, None) => {}
                    _ => {
                        return Err(bad(
                            "use `date`, or `start` with `end` (YYYY or YYYY-MM, end may be \"now\")",
                        ));
                    }
                }
                if e.refs.iter().any(|r| r.link.is_some() != r.href.is_some()) {
                    return Err(bad("a ref needs both `link` and `href`, or neither"));
                }
            }
        }
        Ok(entries)
    }
}

impl Entry {
    fn shown_on(&self, section: &Section, page: Page) -> bool {
        self.pages
            .as_ref()
            .unwrap_or(&section.pages)
            .contains(&page)
    }

    /// Sort key and `datetime` value: the date, or the start of a range.
    fn key(&self) -> &str {
        self.date
            .as_deref()
            .or(self.start.as_deref())
            .unwrap_or_default()
    }

    /// The right-hand column: `2026-05`, `2025-11 – now`, `2023 – 2024`.
    fn when(&self) -> Option<String> {
        match (&self.date, &self.start, &self.end) {
            (Some(d), _, _) => Some(d.clone()),
            (None, Some(a), Some(z)) => Some(format!("{a} – {z}")),
            _ => None,
        }
    }
}

/// One `<li>` in the shared entry layout. Without a description the name
/// spans the text columns (post titles, short rows).
pub fn entry_html(e: &Entry) -> String {
    let mut inner = e.desc.as_ref().map_or_else(
        || format!(r#"<span class="entry-title">{} </span>"#, esc(&e.name)),
        |desc| {
            format!(
                r#"<span class="entry-name">{} </span><span class="entry-desc">{} </span>"#,
                esc(&e.name),
                esc(desc)
            )
        },
    );
    if let Some(when) = e.when() {
        write!(
            inner,
            r#"<time class="entry-meta" datetime="{}">{when}</time>"#,
            e.key()
        )
        .expect("writing to a String cannot fail");
    }
    let head = e.href.as_ref().map_or_else(
        || format!(r#"<div class="entry-static">{inner}</div>"#),
        |href| format!(r#"<a href="{}">{inner}</a>"#, esc(href)),
    );
    if e.refs.is_empty() {
        return format!("        <li>{head}</li>\n");
    }
    let mut refs = String::new();
    for r in &e.refs {
        let link = match (&r.link, &r.href) {
            (Some(l), Some(h)) => format!(r#"<a href="{}">{}</a>"#, esc(h), esc(l)),
            _ => String::new(),
        };
        let text = r.text.as_deref().map(esc).unwrap_or_default();
        let sep = if link.is_empty() || text.is_empty() {
            ""
        } else {
            " "
        };
        writeln!(refs, "            <li>{link}{sep}{text}</li>")
            .expect("writing to a String cannot fail");
    }
    format!(
        "        <li>\n          {head}\n          <ul class=\"entry-refs\">\n{refs}          </ul>\n        </li>\n"
    )
}

/// A list of entries, newest first (entries without a date keep their order).
pub fn entries_list<'a>(entries: impl IntoIterator<Item = &'a Entry>) -> String {
    let mut sorted: Vec<&Entry> = entries.into_iter().collect();
    sorted.sort_by(|a, b| b.key().cmp(a.key()));
    let mut out = String::from("      <ul class=\"entries\">\n");
    for e in sorted {
        out.push_str(&entry_html(e));
    }
    out.push_str("      </ul>\n");
    out
}

/// Every section shown on `page`, in file order.
pub fn sections_html(entries: &Entries, page: Page) -> String {
    let mut out = String::new();
    for s in entries.sections.iter().filter(|s| s.pages.contains(&page)) {
        let shown: Vec<&Entry> = s.entries.iter().filter(|e| e.shown_on(s, page)).collect();
        if shown.is_empty() {
            continue;
        }
        write!(
            out,
            "    <section class=\"section\" aria-labelledby=\"{id}-heading\">\n      <h2 id=\"{id}-heading\" class=\"section-label\">{label}</h2>\n{list}",
            id = esc(&s.id),
            label = esc(&s.label),
            list = entries_list(shown),
        )
        .expect("writing to a String cannot fail");
        if let Some(more) = &s.more {
            writeln!(out, "      <p class=\"section-more\">{more}</p>")
                .expect("writing to a String cannot fail");
        }
        out.push_str("    </section>\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
[cv]
title = "CV"
description = "d"

[[section]]
id = "work"
label = "Work"
pages = ["home", "cv"]
more = 'More <a href="/x">here</a>'

[[section.entry]]
name = "Old"
desc = "first"
date = "2024-01"

[[section.entry]]
name = "Job"
desc = "a & b"
start = "2025-11"
end = "now"
refs = [{ link = "CVE-1", href = "https://e/1", text = "Vendor" }, { text = "plain" }]

[[section.entry]]
name = "CV only"
desc = "c"
date = "2025"
pages = ["cv"]

[[section]]
id = "edu"
label = "Education"
pages = ["cv"]

[[section.entry]]
name = "School"
start = "2021"
end = "2023"
"#;

    #[test]
    fn dates_and_ranges_render_in_one_style() {
        let e = Entries::parse(SRC).unwrap();
        let cv = sections_html(&e, Page::Cv);
        assert!(cv.contains(r#"<time class="entry-meta" datetime="2025-11">2025-11 – now</time>"#));
        assert!(cv.contains(r#"<time class="entry-meta" datetime="2024-01">2024-01</time>"#));
        assert!(cv.contains(r#"datetime="2021">2021 – 2023</time>"#));
    }

    #[test]
    fn sections_sort_newest_first_and_filter_by_page() {
        let e = Entries::parse(SRC).unwrap();
        let home = sections_html(&e, Page::Home);
        let cv = sections_html(&e, Page::Cv);
        assert!(home.find("Job").unwrap() < home.find("Old").unwrap());
        assert!(!home.contains("CV only") && !home.contains("Education"));
        assert!(cv.find("Job").unwrap() < cv.find("CV only").unwrap());
        assert!(cv.find("CV only").unwrap() < cv.find("Old").unwrap());
        assert!(home.contains(r#"<p class="section-more">More <a href="/x">here</a></p>"#));
    }

    #[test]
    fn text_is_escaped_and_refs_render() {
        let cv = sections_html(&Entries::parse(SRC).unwrap(), Page::Cv);
        assert!(cv.contains("a &amp; b"));
        assert!(cv.contains(r#"<li><a href="https://e/1">CVE-1</a> Vendor</li>"#));
        assert!(cv.contains("<li>plain</li>"));
        // no description: the name spans the text columns
        assert!(cv.contains(r#"<span class="entry-title">School </span>"#));
    }

    #[test]
    fn bad_dates_fail_with_the_entry_name() {
        let bad = SRC.replace(r#"date = "2024-01""#, r#"date = "Jan 2024""#);
        assert!(Entries::parse(&bad).unwrap_err().contains("\"Old\""));
        let both = SRC.replace(r#"date = "2025""#, "date = \"2025\"\nstart = \"2024\"");
        assert!(Entries::parse(&both).is_err());
        assert!(!is_date("2025-13"));
        let backwards = SRC.replace(r#"end = "2023""#, r#"end = "2019""#);
        assert!(Entries::parse(&backwards).is_err());
        assert!(!ends_before("2025-11", "2025") && ends_before("2025-11", "2025-02"));
    }

    #[test]
    fn the_real_file_parses() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../content/entries.toml"
        ))
        .unwrap();
        let e = Entries::parse(&src).unwrap();
        assert!(sections_html(&e, Page::Home).contains("research-heading"));
    }
}
