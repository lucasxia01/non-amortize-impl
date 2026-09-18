#!/usr/bin/env python3
"""Usage: summarize.py [csv] [timer_overhead_ns]
Summarize results/zkevm_hashshare.csv: per config and thread count, median wall time, median
hashing share of CPU (hash_cpu / (user+sys)), per-role split, shareable (tag-independent) share,
and proof size."""
import csv, statistics as st, sys, collections
path = sys.argv[1] if len(sys.argv) > 1 else 'results/zkevm_hashshare.csv'
# Per-call overhead of the in-situ hash timer, subtracted from hash time (calls * overhead).
# Taken from the CSV's timer_overhead_ns column when present, else from argv[2], else 40 ns (M4 Pro).
rows = list(csv.DictReader(open(path)))
DEFAULT_OVH = float(sys.argv[2]) if len(sys.argv) > 2 else 40.0
def ovh(r):
    v = r.get('timer_overhead_ns') or ''
    return float(v) if v else DEFAULT_OVH
groups = collections.OrderedDict()
for r in rows: groups.setdefault((r['config'], r['threads']), []).append(r)
print(f"{'config':10} {'thr':>3} {'n':>2} {'wall s':>8} {'cpu s':>8} {'hash s':>8} {'hash%':>6} {'leaf%':>6} {'node%':>6} {'perm%':>6} {'shareable%':>10} {'proof MB':>8}")
for (cfg, thr), rs in groups.items():
    med = lambda k: st.median(float(r[k]) for r in rs)
    cpu = med('cpu_user_s') + med('cpu_sys_s')
    hs = st.median(float(r['hash_cpu_s']) - float(r['hash_calls']) * ovh(r) * 1e-9 for r in rs)  # overhead-corrected
    hns = med('leaf_ns') + med('node_ns') + med('perm_ns')  # role split uses raw timings (same bias per call)
    leaf, node, perm = (med(k) / hns * 100 if hns else 0 for k in ('leaf_ns', 'node_ns', 'perm_ns'))
    # shareable (tag-independent) work: trace generation + non-hash part of trace commitments,
    # meaningful for single-threaded runs where phase wall time == CPU time.
    shareable = (med('phase_traces_s') + med('phase_trace_commit_s') - (med('phase_trace_commit_hash_ns') * 1e-9 - (med('leaf_calls') + med('node_calls')) * 0.25 * ovh(rs[0]) * 1e-9)) / cpu * 100 if thr == '1' else float('nan')
    print(f"{cfg:10} {thr:>3} {len(rs):>2} {med('wall_s'):8.2f} {cpu:8.2f} {hs:8.2f} {hs/cpu*100:6.1f} {leaf:6.1f} {node:6.1f} {perm:6.1f} {shareable:10.1f} {med('proof_bytes')/1e6:8.2f}")
