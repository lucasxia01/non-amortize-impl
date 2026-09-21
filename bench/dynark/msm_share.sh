#!/usr/bin/env bash
# Groth16 prover phase breakdown (MSM share). Env: CONFIGS="sha256:1 sha256:4 synthetic:16 synthetic:18"
# THREADS="1 14" REPS=3 OUT=<csv>
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; ROOT="$(cd "$HERE/../.." && pwd)"
WORK="${WORK:-$HERE/work}"; BIN="$(dirname "$(cat "$WORK/tagged_bench.bin")")/msm_share"
OUT="${OUT:-$ROOT/results/groth16_msm_share.csv}"
CONFIGS="${CONFIGS:-sha256:1 sha256:4 synthetic:16 synthetic:18}"
THREADS="${THREADS:-1 $(sysctl -n hw.ncpu 2>/dev/null || nproc)}"
mkdir -p "$(dirname "$OUT")"
for cfg in $CONFIGS; do IFS=: read -r C S <<< "$cfg"; for T in $THREADS; do
  RAYON_NUM_THREADS=$T CIRCUIT=$C SHA_BLOCKS=$S LOG_N=$S REPS="${REPS:-3}" OUT="$OUT" "$BIN" 2>&1 | grep GROTH16_PHASES | tail -1 | cut -c1-110
done; done
echo "wrote $OUT"
