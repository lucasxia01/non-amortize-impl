#!/usr/bin/env python3
"""Summarize results/dynark_tagged.csv: per circuit/size, fresh-proof time, preprocessing costs,
update cost, and the batch-prover ratio honest(T x fresh) / batch for each T."""
import csv, sys, collections
path = sys.argv[1] if len(sys.argv) > 1 else 'results/dynark_tagged.csv'
rows = list(csv.DictReader(open(path)))
g = collections.OrderedDict()
for r in rows: g.setdefault((r['circuit'], r['constraints']), []).append(r)
print(f"{'circuit':10} {'n':>8} {'fresh s':>8} {'prep_tgt s':>10} {'prep_full s':>11} {'update ms':>9} | ratio(T=2) ratio(4) ratio(16) ratio(64) ratio(256) [targeted]  | ratio(256) [full prep]")
for (c, n), rs in g.items():
    r0 = rs[0]; byT = {int(r['T']): r for r in rs}
    rat = lambda T: float(byT[T]['ratio_targeted']) if T in byT else float('nan')
    print(f"{c:10} {int(n):8d} {float(r0['t_fresh_s']):8.3f} {float(r0['t_prep_targeted_s']):10.3f} {float(r0['t_prep_full_s']):11.2f} {float(r0['t_update_ms']):9.2f} | {rat(2):9.2f} {rat(4):8.2f} {rat(16):9.2f} {rat(64):9.2f} {rat(256):10.2f}             | {float(byT[256]['ratio_full']) if 256 in byT else float('nan'):10.2f}")
