#!/bin/sh
# Trunk post_build hook: render the shared header, the entries and the
# writing pages into Trunk's staging directory, so `trunk serve` and
# `trunk build` both produce the whole site. Trunk.toml passes only this
# script's path; the cargo arguments live here.
set -eu
cd "$TRUNK_SOURCE_DIR/.."
SITE_ROOT=. DIST_DIR="$TRUNK_STAGING_DIR" exec cargo run --locked --release --quiet -p sitegen
