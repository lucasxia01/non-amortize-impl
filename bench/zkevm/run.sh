#!/usr/bin/env bash
# Run the ERC-20 block prover over a grid of FRI configurations and thread counts, and append one
# CSV row per run to results/zkevm_hashshare.csv.
# Env knobs: CONFIGS="rate_bits:arity_bits:pow_bits ..." THREADS="1 14" REPS=3 OUT=<csv>
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
WORK="${WORK:-$HERE/work}"
BIN=$(cat "$WORK/erc20.bin")
OUT="${OUT:-$ROOT/results/zkevm_hashshare.csv}"
CONFIGS="${CONFIGS:-1:4:16 1:1:16 2:1:16 3:1:16 4:1:16 3:1:20 4:1:20 2:4:24 3:4:24 4:4:24}"
THREADS="${THREADS:-1 $(sysctl -n hw.ncpu 2>/dev/null || nproc)}"
REPS="${REPS:-3}"
mkdir -p "$(dirname "$OUT")"
if [ ! -f "$OUT" ]; then
  echo "config,rate_bits,arity_bits,pow_bits,queries,threads,rep,wall_s,cpu_user_s,cpu_sys_s,hash_cpu_s,hash_calls,hash_bytes,leaf_calls,leaf_ns,leaf_bytes,node_calls,node_ns,node_bytes,perm_calls,perm_ns,perm_bytes,phase_traces_s,phase_trace_commit_s,phase_trace_commit_hash_ns,phase_ctl_s,phase_proofs_s,phase_proofs_hash_ns,proof_bytes,hash_share_cpu,timer_overhead_ns" > "$OUT"
fi
for cfg in $CONFIGS; do
  IFS=: read -r R A P <<< "$cfg"
  for T in $THREADS; do
    for rep in $(seq 1 "$REPS"); do
      LOG=$(mktemp)
      RAYON_NUM_THREADS=$T ZKEVM_RATE_BITS=$R ZKEVM_ARITY_BITS=$A ZKEVM_POW_BITS=$P \
        /usr/bin/time -p "$BIN" --nocapture --test-threads=1 > "$LOG" 2>&1 || { echo "run failed: $cfg threads=$T"; tail -5 "$LOG"; exit 1; }
      python3 "$HERE/parse_log.py" "$LOG" "$cfg" "$R" "$A" "$P" "$T" "$rep" >> "$OUT"
      tail -1 "$OUT"
      rm -f "$LOG"
    done
  done
done
echo "wrote $OUT"
