#!/usr/bin/env bash
# Sweep circuits/sizes; one CSV row per (circuit, size, T). Env: CONFIGS="sha256:1 sha256:4 synthetic:16 ..."
# (sha256:<64-byte blocks>, synthetic:<log2 constraints>), TAGS="2 4 16 64 256", REPS=3, FULL_PREP=1, OUT=<csv>
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; ROOT="$(cd "$HERE/../.." && pwd)"
WORK="${WORK:-$HERE/work}"; BIN=$(cat "$WORK/tagged_bench.bin")
OUT="${OUT:-$ROOT/results/dynark_tagged.csv}"
CONFIGS="${CONFIGS:-sha256:1 sha256:4 sha256:16 synthetic:16 synthetic:18 synthetic:20}"
mkdir -p "$(dirname "$OUT")"
for cfg in $CONFIGS; do
  IFS=: read -r C S <<< "$cfg"
  echo "== $cfg"
  CIRCUIT=$C SHA_BLOCKS=$S LOG_N=$S CHECK="${CHECK:-0}" FULL_PREP="${FULL_PREP:-1}" TAGS="${TAGS:-2 4 16 64 256}" REPS="${REPS:-3}" OUT="$OUT" "$BIN"
done
echo "wrote $OUT"
