#!/bin/sh
# The same entry point locally and in CI. Native mode requires a display + wgpu.
set -eu
cd "$(dirname "$0")/.."
case "${1:-}" in
  "") native=false ;;
  --native) native=true ;;
  *) echo 'Usage: scripts/check.sh [--native]' >&2; exit 2 ;;
esac
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
if "$native"; then
  cargo test --locked -- --include-ignored
  cargo build --locked
  python3 scripts/test-native.py
else
  cargo test --locked
fi
