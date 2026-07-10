# unwrap.rs

The portfolio of Cole Leavitt, Principal Security Engineer and Security Researcher. The site pairs a Rust-native identity with a small typed backend and production-focused deployment flow.

## Stack

- Leptos 0.8 client-side rendering compiled to WebAssembly
- Trunk and Tailwind CSS for the frontend build
- servant-rs and Tokio for typed APIs and static asset serving
- nginx and a hardened systemd service in production

The frontend includes a phase-based `Result<T, E>.unwrap()` particle animation, semantic profile content, structured search metadata, a sitemap, and social preview assets.

## Development

Install the required targets and Trunk once:

```bash
rustup target add wasm32-unknown-unknown x86_64-unknown-linux-musl
cargo install trunk
```

Run the frontend development server from its directory:

```bash
cd frontend
trunk serve
```

The Trunk development server listens on `http://127.0.0.1:8080`.

## Build and check

```bash
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --locked --release --target x86_64-unknown-linux-musl -p server
(cd frontend && env -u NO_COLOR trunk build --locked --release)
```

Production artifacts are written to:

- `target/x86_64-unknown-linux-musl/release/server`
- `frontend/dist/`

Run the built application locally with:

```bash
ADDR=127.0.0.1:18080 \
DIST_DIR=frontend/dist \
./target/x86_64-unknown-linux-musl/release/server
```

The service exposes `/api/health`, `/api/social-links`, and the built frontend.

## Production deployment

The production host already uses nginx in front of a native systemd service. That is a better fit here than Docker: the deployable output is one static binary plus immutable frontend assets.

Routine releases are atomic and keep rollback history:

```bash
./deploy.sh check       # production builds and local smoke checks only
./deploy.sh             # build, smoke-test, and deploy to the `linode` SSH alias
./deploy.sh status      # inspect the current production release
./deploy.sh rollback    # activate the previous healthy release
```

Configuration can be overridden without editing the script:

```bash
DEPLOY_HOST=linode KEEP_RELEASES=5 PUBLIC_URL=https://unwrap.rs ./deploy.sh
```

The script never rewrites nginx or systemd during a routine release. For a new host, install [deploy/unwrap-rs.service](deploy/unwrap-rs.service) once, configure nginx to proxy `/` to `127.0.0.1:18080`, and then use `deploy.sh` for releases.

## License

MIT OR Apache-2.0
