#!/usr/bin/env bash
# Builds the game into dist/ as a static site (any static host, e.g. GitHub Pages, can serve it).
# Needs: rustup target add wasm32-unknown-unknown, and wasm-bindgen-cli at the version pinned in
# Cargo.toml (cargo install wasm-bindgen-cli --version 0.2.129 --locked). wasm-opt is used if found.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown --lib
rm -rf dist
mkdir -p dist
wasm-bindgen --target web --no-typescript --out-dir dist target/wasm32-unknown-unknown/release/hanoi_rescue.wasm
if command -v wasm-opt >/dev/null; then
  wasm-opt -Os --enable-bulk-memory --enable-nontrapping-float-to-int -o dist/hanoi_rescue_bg.wasm dist/hanoi_rescue_bg.wasm
fi
cp web/index.html web/manifest.webmanifest web/icon.svg dist/
touch dist/.nojekyll
ls -la dist
