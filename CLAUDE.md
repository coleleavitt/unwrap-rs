# unwrap.rs Claude Instructions

## Mission

`unwrap.rs` is Cole Leavitt's personal site: the public identity of a principal security engineer and security researcher. It must stay small, truthful, fast, and accessible.

What must be preserved:

- Cole's name and identity lead the page. GitHub, LinkedIn, and email stay one click away.
- The `Result<T, E>.unwrap()` particle animation is the signature interaction. It shows craft; the page does not explain it.
- Every claim is specific, true, and visible. No keyword stuffing, no inflated titles, no wording that implies affiliation with JPL, NASA, or any employer.
- WCAG 2.2 AA: browser zoom, keyboard navigation, visible focus, readable contrast, 44 px touch targets, and a reduced-motion experience.

`PRODUCT.md` is the source of truth for audience, voice, anti-references, and design principles. Read it before any copy, layout, or visual change.

## Architecture First

The repo is a Cargo workspace (edition 2024) with three crates (`frontend`, `server`, `sitegen`) and a deploy layer. Decide which layer a change belongs to before writing code.

| Layer | Files | Owns |
|---|---|---|
| Page content | `frontend/index.html` | Static homepage HTML: name, sentence, link row, Work list, the `<!-- sitegen:latest -->` marker |
| Animation | `frontend/src/app/typing_animation/`, `frontend/src/main.rs` | The particle animation, mounted into `#animation-mount` (skipped under reduced motion) |
| CV | `content/cv.md`, `sitegen/` (`cv_page`) | `/cv/`: raw HTML in the homepage's entry layout, plus print styles in `index.css` (2 Letter pages). It repeats homepage facts; change both together |
| Writing | `content/writing/*.md`, `sitegen/` | Markdown posts (` ```mermaid ` fences become inline SVG; fences with a known language get build-time syntax highlighting via syntect, `hl-*` classes styled in `index.css`; Rust uses the vendored current grammar `sitegen/syntaxes/Rust.sublime-syntax`, since syntect's bundled grammars are from 2018) rendered to `dist/writing/`, `feed.xml`, `sitemap.xml`, and the homepage's latest-posts section (skips posts the homepage already links to) |
| Document and SEO | `frontend/index.html` head, `robots.txt`, `social-card.png`, `assets/social-card.svg`, `favicon.svg` (plus `favicon.ico` and `apple-touch-icon.png`, rendered from it) | `<title>`, Open Graph, JSON-LD, canonical URL, Trunk asset directives |
| Keys and disclosure | `frontend/pgp.asc`, `frontend/.well-known/` (`security.txt`, `openpgpkey/` WKD) | Cole's OpenPGP keys (`DEC2 3202 3B77…6193` primary, `2EFA…3402` older, cross-signed), RFC 9116 contact. Re-export both and the WKD file (`gpg --list-keys --with-wkd-hash`) when a key changes; renew `Expires` in `security.txt` yearly |
| Styling | `frontend/index.css`, `frontend/tailwind.config.js`, `frontend/fonts/` | One stylesheet for every page; self-hosted JetBrains Mono (Latin subset, OFL) |
| Server | `server/src/main.rs` | One servant-rs API description: `GET /api/health`, `GET /api/social-links`, and a `Raw` static-file catch-all for `dist/` with directory indexes and the security headers (CSP, Permissions-Policy, COOP; CORS on the WKD path) |
| Deploy | `deploy.sh`, `deploy/unwrap-rs.service` | Build (trunk, then sitegen), smoke test, atomic release, status, rollback on the `linode` host |

Animation module boundaries (keep them):

- `types.rs`: plain data (`Particle`, `Connection`, `AnimationPhase` and sub-phases, `ColorScheme`).
- `constants.rs`: layout units (`CHAR_ADVANCE`, `UNIT_EM`, field size, text position), target text, particle counts, the cipher alphabet.
- `state_machine.rs`: phase order (`next`) and phase durations as pure `const fn`s.
- `physics.rs`: per-phase particle updates. No DOM access.
- `mod.rs`: the Leptos `TypingAnimation` component, the `requestAnimationFrame` loop (delta time capped at 100 ms, paused while the tab is hidden), and rendering in `em` so `.unwrap()` sets flush against the static `Result<T, E>`.

Composition rules:

- The homepage is static HTML; only the animation is Leptos 0.8 CSR (`wasm32-unknown-unknown`, built by Trunk). No SSR, no hydration. Without JS or with reduced motion, the static `.unwrap()` span shows.
- Writing pages are plain HTML from `sitegen`, run by `deploy.sh` after `trunk build` (`SITE_ROOT=. DIST_DIR=frontend/dist cargo run -p sitegen`). `trunk serve` does not run it.
- The CSP allows Trunk's inline loader by hash, read from `dist/index.html` when the server starts. Restart the server after every rebuild, or the stale hash blocks the animation (deploys restart it).
- The server is not a Leptos server. It serves `DIST_DIR` and two typed endpoints. Add endpoints to the single `servant` API description, not as side routes.
- Every page shares one header (name, sentence, links, PGP line). It lives in `frontend/index.html` and, with the name as a home link, in `SITE_HEADER` in `sitegen/src/main.rs`; keep both in step. The GitHub/LinkedIn/Email links also appear in `SOCIAL_LINKS` in `server/src/main.rs` (the API; its `icon_path`s point at `src/app/social_links/icons/`, still copied by Trunk). Change them together.
- The server depends on the sibling checkout `../servant-rs` through path dependencies. That checkout must exist to build the server.
- `sitegen` depends on the sibling checkout `../mmd-svg` (Cole's crate, `coleleavitt/mmd-svg`): ` ```mermaid ` flowchart fences in posts render to inline SVG at build time, with no client JS. A diagram that fails to parse fails the build. Fix layout problems in `mmd-svg`, not in the site.

## Scope Boundaries

Do not add, unless Cole asks:

- A CMS, analytics, trackers, cookies, or third-party scripts.
- SSR, a database, auth, or new runtime services.
- Docker. Production is one static musl binary plus static assets behind nginx and systemd.
- Card-grid portfolio sections, long resume text, or extra subtitle rows. See `PRODUCT.md` anti-references.
- Posts built from `~/SiteResearch` material that contains live credentials or undisclosed findings.

## Current State

- Branch `dev`. Deployed 2026-11-08 (release `20261108T002116Z-d72ce4c`, seven posts, syntax highlighting).
- Production nginx (`/etc/nginx/sites-enabled/unwrap.rs` on `linode`, not in this repo) proxies everything to the app except `/dori/`. It adds only HSTS and `X-Frame-Options`; every other security header comes from the app, so do not add duplicates in nginx. The old `/.well-known` and `/proof` proxy to a Nebula peer was removed on 2026-11-07 (backups in `/root/unwrap.rs.nginx.bak-*`). TLS renews by DNS-01 through Cloudflare.
- Cloudflare injects its Web Analytics beacon; the CSP blocks it. Turn it off in the Cloudflare dashboard rather than allowing it.
- Layout: one left-aligned column (`--column: 56rem`) shared by the homepage and writing pages. One accent (`--accent: #dd68ff`).
- Posts: `aamva-pdf417.md` (the AAMVA CVEs; facts only from the public research repo), `thunderbolt-resume-race.md` (facts from the posted patches), `linux-hello-face-unlock.md`, `xtensa-decoder-apartment-wifi.md` (the vendor firmware finding stays out until disclosed), `keyoxide-gpg-agent.md`, `codeflow-mir.md`, `idb-rs-ida-95.md` (facts from the public PR only; nothing from decompiling IDA). `post-template.md` is a draft (`draft: true`). Posts use the technical write-up style, not a casual story voice.
- Unit tests: `server` (path resolution, content types) and `sitegen` (frontmatter, escaping, dates, feed/sitemap, homepage injection). `frontend/tests/typing_animation_test.rs` only runs on `wasm32` in a browser.

## Working Rules

- Read `PRODUCT.md` before changing copy or visuals. Keep the voice precise, technical, and quietly playful.
- Keep `index.html` metadata, JSON-LD, `sitemap.xml`, and `social-card.png` consistent with the visible page.
- Workspace lints are strict (`clippy::all`, `pedantic`, `nursery`, `cargo` at warn, `-D warnings` in checks). Fix lints; do not add `allow`s without a reason in `Cargo.toml`.
- Keep animation math in `physics.rs` and phase timing in `state_machine.rs`. Do not grow `mod.rs` into a god component.
- Any motion change must keep the `prefers-reduced-motion` path in `index.css` working.
- Do not edit `../servant-rs` from this repo. Report missing servant features instead.
- Never deploy (`./deploy.sh` or `./deploy.sh rollback`) unless Cole asks in this session. `./deploy.sh check` is safe; it does not contact production.
- Commit messages follow `.committed.toml`: conventional commits, subject at most 50 characters, body lines at most 72.

## Validation

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
(cd frontend && env -u NO_COLOR trunk build --locked --release)
./deploy.sh check        # full release build plus local smoke tests, no production contact
```

- `cargo test --workspace` on the host builds both crates but runs no tests today. The browser tests need `wasm-pack test --headless --firefox frontend` (not verified in this setup).
- Visual changes: run `cd frontend && trunk serve` (http://127.0.0.1:8080) and check 1440 px and 390 px widths, keyboard focus, and reduced motion.
- Local production run: `ADDR=127.0.0.1:18080 DIST_DIR=frontend/dist ./target/x86_64-unknown-linux-musl/release/server`.

## Project Claude Layout

- `CLAUDE.md` (this file): primary instructions. `AGENTS.md` mirrors the core rules for other agents.
- `MEMORY.md`: local scratch and handoff notes; ignored by git.
- `.claude/agents/`: `researcher` (read-only mapping) and `verifier` (checks after a change).
- `.claude/commands/`: `/audit` and `/repro`.
- `.claude/rules/`: always-on code style, security, and testing rules.
