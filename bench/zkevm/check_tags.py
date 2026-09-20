#!/usr/bin/env python3
"""Cross-tag disjointness check. Each dump file holds the sorted first-8-byte prefixes of every
Keccak digest the prover produced (leaves, nodes, challenger permutations, grinding) in one run.
Reports, for every pair of runs, how many digest prefixes they share (expected: 0, up to the
2^-64 birthday noise), and how many repeats occur within a run (identical inputs hashed twice,
e.g. duplicated rows; informational)."""
import sys, numpy as np, itertools, os
runs = {os.path.basename(p): np.fromfile(p, dtype='<u8') for p in sys.argv[1:]}
for name, a in runs.items():
    u = np.unique(a)
    print(f"{name}: {a.size} digests, {a.size - u.size} within-run repeats")
for (n1, a), (n2, b) in itertools.combinations(runs.items(), 2):
    common = np.intersect1d(np.unique(a), np.unique(b))
    print(f"{n1} vs {n2}: {common.size} shared digest prefixes")
