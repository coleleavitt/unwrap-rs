# unwrap.rs Agent Instructions

This file exists for agents that read `AGENTS.md`. `CLAUDE.md` is the primary guide; read it first. If the two disagree, `CLAUDE.md` wins.

## Project

- Cole Leavitt's personal site at https://unwrap.rs: a static homepage with a Leptos 0.8 CSR animation (WebAssembly, Trunk), a Markdown writing section built by `sitegen`, and a small servant-rs server.
- `PRODUCT.md` defines audience, voice, anti-references, and accessibility targets (WCAG 2.2 AA).
- The `Result<T, E>.unwrap()` particle animation in `frontend/src/app/typing_animation/` is the signature interaction.

## Layout

- `frontend/index.html`: homepage shell (metadata, JSON-LD, Trunk directives, animation mount) with `sitegen:header`, `sitegen:sections` and `sitegen:latest` markers.
- `content/entries.toml`: every dated entry (homepage sections and `/cv/`), rendered by `sitegen/src/entries.rs`. Edit facts here only.
- `frontend/src/main.rs` + `frontend/src/app/typing_animation/`: the animation, mounted into `#animation-mount` (not under reduced motion).
- `content/writing/*.md` + `sitegen/`: writing pages, Atom feed, sitemap, homepage latest posts. Run by `deploy.sh` after `trunk build`.
- `frontend/index.css`, `frontend/fonts/`: one stylesheet for all pages; self-hosted JetBrains Mono.
- `server/src/main.rs`: one servant API description for `/api/health`, `/api/social-links`, and static files with directory indexes. Needs the sibling checkout `../servant-rs`.
- `deploy.sh`, `deploy/unwrap-rs.service`: release, status, rollback on the `linode` host.

## Rules

- Keep every visible claim true and specific. Never imply affiliation with JPL, NASA, or an employer.
- Keep the GitHub/LinkedIn/Email links identical in sitegen's `SITE_HEADER` (the one header for every page) and the server's `SOCIAL_LINKS`.
- Keep metadata, sitemap, and social card consistent with the page.
- Preserve keyboard focus, 44 px touch targets, and the reduced-motion path.
- Do not add trackers, third-party scripts, CDN fonts, SSR, databases, or Docker.
- Never publish research material that contains live credentials or undisclosed findings.
- Do not edit `../servant-rs`.
- Never run `./deploy.sh` (deploy) or `./deploy.sh rollback` unless Cole asks. `./deploy.sh check` is safe.

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
(cd frontend && env -u NO_COLOR trunk build --locked --release)
./deploy.sh check
```
