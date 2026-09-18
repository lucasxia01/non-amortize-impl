#!/usr/bin/env bash
# Clone Polygon's type-1 zkEVM at a pinned commit, vendor plonky2 1.0.0 with the Keccak
# instrumentation patch, apply the zk_evm bench patch, and build the ERC-20 block test binary.
# Usage: bench/zkevm/setup.sh [workdir]   (default: bench/zkevm/work)
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${1:-$HERE/work}"
ZKEVM_REPO=https://github.com/0xPolygonZero/zk_evm.git
ZKEVM_COMMIT=ef388619ffbd5305209519a3a5bc0396185d68ac
PLONKY2_VER=1.0.0

mkdir -p "$WORK"; cd "$WORK"
if [ ! -d zk_evm ]; then
  git clone --filter=blob:none "$ZKEVM_REPO" zk_evm
fi
cd zk_evm
git fetch -q origin "$ZKEVM_COMMIT" || true
git checkout -q "$ZKEVM_COMMIT"
git checkout -q -- . && git clean -fdq -e vendor -e target
# rust-toolchain.toml pins nightly-2024-09-24; rustup installs it on first cargo call.
cargo fetch
REG=$(ls -d "${CARGO_HOME:-$HOME/.cargo}"/registry/src/*/plonky2-$PLONKY2_VER | head -1)
[ -d "$REG" ] || { echo "plonky2-$PLONKY2_VER not found in cargo registry"; exit 1; }
rm -rf vendor/plonky2 && mkdir -p vendor && cp -R "$REG" vendor/plonky2 && rm -rf vendor/plonky2/target
( cd vendor/plonky2 && patch -p1 < "$HERE/patches/plonky2-keccak-stats.patch" )
git apply "$HERE/patches/zk_evm-bench.patch"
cargo test --release -p evm_arithmetization --test erc20 --no-run 2>&1 | tail -2
BIN="$(pwd)/$(ls -t target/release/deps/erc20-* | grep -v '\.d$' | head -1)"
echo "$BIN" > "$WORK/erc20.bin"
echo "built: $BIN"
