#!/usr/bin/env bash
# Builds the 3D game into dist/ as a static site: dist/gpu/ (WebGPU, the full renderer) and
# dist/gl/ (WebGL2, for browsers without WebGPU); index.html picks one, assets are shared.
# Needs: rustup target add wasm32-unknown-unknown; wasm-bindgen-cli at the version Cargo.lock
# resolves (cargo install wasm-bindgen-cli --version 0.2.130 --locked); wasm-opt is used if found.
# ONLY=gpu or ONLY=gl builds one of them (quicker while working).
set -euo pipefail
cd "$(dirname "$0")"
rm -rf dist
mkdir -p dist

build() {
  local feature=$1 out=$2
  cargo build --release --target wasm32-unknown-unknown --no-default-features --features "$feature"
  wasm-bindgen --target web --no-typescript --out-dir "dist/$out" target/wasm32-unknown-unknown/release/hanoi-city.wasm
  if [ -z "${SKIP_OPT:-}" ] && command -v wasm-opt >/dev/null; then
    wasm-opt -O2 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals \
      -o "dist/$out/hanoi-city_bg.wasm" "dist/$out/hanoi-city_bg.wasm"
  fi
}

if [ "${ONLY:-}" != gl ]; then build webgpu gpu; fi
if [ "${ONLY:-}" != gpu ]; then build webgl2 gl; fi

mkdir -p dist/assets
cp -r assets/people assets/hair assets/tex assets/fonts assets/anim.glb assets/dog.glb dist/assets/
if [ -d assets/hq ]; then cp -r assets/hq dist/assets/; fi
cp web/index.html web/icon.svg web/manifest.webmanifest dist/
touch dist/.nojekyll
du -sh dist dist/*/*.wasm
