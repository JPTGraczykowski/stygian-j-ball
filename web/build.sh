#!/bin/sh
# Builds the browser version into target/web (or the directory given as $1).
# Serve that directory over HTTP, e.g.: python3 -m http.server -d target/web
set -eu
cd "$(dirname "$0")/.."
out="${1:-target/web}"

cargo build --release --target wasm32-unknown-unknown --bin stygian-j-ball

# Take the JS glue from the macroquad crate we actually built against, so the two
# always match.
macroquad=$(cargo metadata --format-version 1 |
  grep -o '"manifest_path":"[^"]*/macroquad-[0-9][^/"]*/Cargo.toml"' |
  head -n 1 | sed 's/^"manifest_path":"//; s/\/Cargo.toml"$//')
if [ ! -f "$macroquad/js/mq_js_bundle.js" ]; then
  echo "could not find macroquad's mq_js_bundle.js" >&2
  exit 1
fi

mkdir -p "$out"
cp target/wasm32-unknown-unknown/release/stygian-j-ball.wasm "$out/"
cp "$macroquad/js/mq_js_bundle.js" web/index.html web/storage.js "$out/"
echo "built $out"
