# unwrap.rs

Personal website built with Leptos, WASM, and Tailwind CSS.

## Features

- **Particle Animation**: Phase-based animation system (scatter → converge → stable → dissolve) with unicode symbol morphing
- **Leptos 0.7**: Reactive signals, CSR rendering
- **Tailwind CSS**: Utility-first styling
- **WASM**: Compiled to WebAssembly via Trunk

## Development

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli
trunk serve
```

## Build

```bash
trunk build --release
```

Output in `dist/`.

## License

MIT OR Apache-2.0
