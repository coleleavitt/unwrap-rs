#!/bin/sh
# Trunk post_build hook: render the shared header, the entries and the
# writing pages into Trunk's staging directory, so `trunk serve` and
# `trunk build` both produce the whole site. Trunk.toml passes only this
# script's path; the cargo arguments live here.
#
# It also builds the Xtensa post's live disassembler (xtensa-dis) to its
# own WebAssembly module under demos/, so only that post loads it.
set -eu
cd "$TRUNK_SOURCE_DIR/.."

# wasm-bindgen's CLI must match the library version in Cargo.lock; Trunk
# downloads exactly that version for the animation before this hook runs.
ver=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/version = |"/, ""); print; exit }' Cargo.lock)
tools="${XDG_CACHE_HOME:-$HOME/.cache}/trunk"
bindgen="$tools/wasm-bindgen-$ver/wasm-bindgen"
if [ ! -x "$bindgen" ]; then
    echo "trunk-sitegen: $bindgen is missing (Trunk fetches it on its first build)" >&2
    exit 1
fi
demo="$TRUNK_STAGING_DIR/demos/xtensa-dis"
cargo build --locked --release --quiet -p xtensa-dis --lib --target wasm32-unknown-unknown
wasm=target/wasm32-unknown-unknown/release/xtensa_dis.wasm
# Content-hashed names, like Trunk's: a CDN may cache these files for hours,
# and a cached old loader must never meet a new module.
hash=$(sha256sum "$wasm" | cut -c1-16)
name="xtensa_dis-$hash"
rm -rf "$demo"
"$bindgen" --target web --no-typescript --out-name "$name" --out-dir "$demo" "$wasm"
wasm_opt=$(find "$tools" -path '*wasm-opt*' -name wasm-opt -type f 2>/dev/null | sort | tail -n 1)
if [ -n "$wasm_opt" ]; then
    "$wasm_opt" -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
        "$demo/${name}_bg.wasm" -o "$demo/${name}_bg.wasm"
fi
# sitegen finds this file by its boot-*.js name
printf '%s\n' "// Loads the live Xtensa disassembler (Rust compiled to WebAssembly)." \
    "import init from \"./$name.js\";" "" "init();" > "$demo/boot-$hash.js"

SITE_ROOT=. DIST_DIR="$TRUNK_STAGING_DIR" exec cargo run --locked --release --quiet -p sitegen
