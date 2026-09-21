#!/usr/bin/env bash
# Clone Dynark (OYJason08/DYNARK_, arkworks-0.5 fork of ark-groth16 with proof updates) at a
# pinned commit, apply the tagged-bench patch, build the example.
# Usage: bench/dynark/setup.sh [workdir]   (default: bench/dynark/work)
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${1:-$HERE/work}"; mkdir -p "$WORK"; WORK="$(cd "$WORK" && pwd)"
REPO=https://github.com/OYJason08/DYNARK_.git
COMMIT=c73a380df86d8f5d195f1e20c8a1d86a7dce084b
cd "$WORK"
[ -d dynark ] || git clone "$REPO" dynark
cd dynark
git fetch -q origin "$COMMIT" || true
git checkout -q "$COMMIT"
git checkout -q -- . && git clean -fdq -e target
git apply "$HERE/patches/dynark-tagged-bench.patch"
cargo build --release --example tagged_bench --example msm_share 2>&1 | tail -1
echo "$WORK/dynark/target/release/examples/tagged_bench" > "$WORK/tagged_bench.bin"
echo "built: $(cat "$WORK/tagged_bench.bin")"
