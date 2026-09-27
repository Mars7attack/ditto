#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
case "$(uname -s)" in Linux) ;; *) echo 'Build this package on Linux.' >&2; exit 1 ;; esac
cargo build --release --locked
architecture="$(uname -m)"
stage="dist/ditto-linux-$architecture"
mkdir -p "$stage"
cp target/release/ditto README.md VALIDATION.md SPEC-TECHNIQUE.md CADRAGE.md "$stage/"
mkdir -p "$stage/assets" "$stage/design" "$stage/docs"
cp docs/CLAVIER-CHARSETS.md docs/SHADERS.md docs/OUTILS-ET-REGLAGES.md "$stage/docs/"
cp design/ditto-terminal.html design/ditto-terminal-preview.html "$stage/design/"
cp assets/README.md assets/LICENSE-PCFACE.txt "$stage/assets/"
cp assets/LICENSE-PCFACE.txt "$stage/Font-License.txt"
tar -czf "$stage.tar.gz" -C dist "ditto-linux-$architecture"
printf 'Package: %s.tar.gz\n' "$stage"
