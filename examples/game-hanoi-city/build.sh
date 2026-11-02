#!/usr/bin/env bash
# Builds the 3D game into dist/ as a static site.
# Needs: rustup target add wasm32-unknown-unknown; wasm-bindgen-cli at the version Cargo.lock
# resolves (cargo install wasm-bindgen-cli --version 0.2.130 --locked); wasm-opt is used if found.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown
rm -rf dist
mkdir -p dist
wasm-bindgen --target web --no-typescript --out-dir dist target/wasm32-unknown-unknown/release/hanoi-city.wasm
if [ -z "${SKIP_OPT:-}" ] && command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals -o dist/hanoi-city_bg.wasm dist/hanoi-city_bg.wasm
fi
mkdir -p dist/assets
cp -r assets/people assets/hair assets/tex assets/fonts assets/anim.glb assets/dog.glb dist/assets/
cp web/index.html web/icon.svg web/manifest.webmanifest dist/
touch dist/.nojekyll
du -sh dist dist/*.wasm
