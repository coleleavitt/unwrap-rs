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
"$bindgen" --target web --no-typescript --out-dir "$demo" \
    target/wasm32-unknown-unknown/release/xtensa_dis.wasm
wasm_opt=$(find "$tools" -path '*wasm-opt*' -name wasm-opt -type f 2>/dev/null | sort | tail -n 1)
if [ -n "$wasm_opt" ]; then
    "$wasm_opt" -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
        "$demo/xtensa_dis_bg.wasm" -o "$demo/xtensa_dis_bg.wasm"
fi
cp xtensa-dis/web/boot.js "$demo/"

SITE_ROOT=. DIST_DIR="$TRUNK_STAGING_DIR" exec cargo run --locked --release --quiet -p sitegen
