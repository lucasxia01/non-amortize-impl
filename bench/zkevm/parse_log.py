#!/usr/bin/env python3
"""Turn one erc20 test log (with PHASE_WALL / KECCAK_PHASE / KECCAK_WORK / PROOF_SIZE lines and
/usr/bin/time -p output) into one CSV row."""
import re, sys
log = open(sys.argv[1]).read(); cfg, R, A, P, T, rep = sys.argv[2:8]
def g(pat, default=''):
    m = re.search(pat, log, re.M); return m.group(1) if m else default
queries = g(r'STARK CONFIG:.*queries=(\d+)')
wall = g(r'KECCAK_WORK:.*prove_wall_s=([\d.]+)')
user = g(r'^user ([\d.]+)'); sysd = g(r'^sys ([\d.]+)')
hash_cpu = g(r'KECCAK_WORK:.*hash_cpu_s=([\d.]+)')
calls = g(r'KECCAK_WORK: calls=(\d+)'); nbytes = g(r'KECCAK_WORK:.*bytes=(\d+)')
roles = {k: g(rf'KECCAK_WORK:.*{k}=(\d+/\d+/\d+)').split('/') for k in ('leaf', 'node', 'perm')}
def phase(name):
    w = g(rf'PHASE_WALL: {name} ([\d.]+)')
    m = re.search(rf'KECCAK_PHASE: {name} leaf=\d+/(\d+)/\d+ node=\d+/(\d+)/\d+ perm=\d+/(\d+)/\d+', log)
    hns = sum(int(x) for x in m.groups()) if m else ''
    return w, hns
tr_w, _ = phase('traces'); tc_w, tc_h = phase('trace_commitments'); ctl_w, _ = phase('ctl'); pr_w, pr_h = phase('proofs')
proof = g(r'PROOF_SIZE_BYTES: (\d+)')
timer_ns = g(r'TIMER_OVERHEAD_NS: ([\d.]+)')
tag_mode = g(r'TAG_MODE: (\w+)', 'untagged')
share = f"{float(hash_cpu)/(float(user)+float(sysd)):.4f}" if hash_cpu and user else ''
row = [cfg, R, A, P, queries, T, rep, wall, user, sysd, hash_cpu, calls, nbytes,
       *roles['leaf'], *roles['node'], *roles['perm'], tr_w, tc_w, tc_h, ctl_w, pr_w, pr_h, proof, share, timer_ns, tag_mode]
print(','.join(str(x) for x in row))
