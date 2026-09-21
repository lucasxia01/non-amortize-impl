#!/usr/bin/env python3
"""Summarize results/groth16_msm_share.csv: median per (circuit, size, threads) of total prover
time and the share spent in MSMs, FFTs (quotient), R1CS evaluation and the rest."""
import csv, statistics as st, sys, collections
path = sys.argv[1] if len(sys.argv) > 1 else 'results/groth16_msm_share.csv'
rows = list(csv.DictReader(open(path))); g = collections.OrderedDict()
for r in rows: g.setdefault((r['circuit'], r['constraints'], r['threads']), []).append(r)
print(f"{'circuit':10} {'n':>8} {'thr':>4} {'total s':>8} {'MSM%':>6} {'FFT%':>6} {'R1CS%':>6} {'other%':>7} | {'msm_q%':>6} {'msm_c%':>6} {'msm_a%':>6} {'msm_b1%':>7} {'msm_b2%':>7}")
for (c, n, t), rs in g.items():
    m = lambda k: st.median(float(r[k]) for r in rs); tot = m('t_total_s')
    pct = lambda k: 100 * m(k) / tot
    print(f"{c:10} {int(n):8d} {t:>4} {tot:8.3f} {pct('msm_s'):6.1f} {pct('quotient_fft_s'):6.1f} {pct('r1cs_eval_s'):6.1f} {pct('other_s') + pct('scalar_conv_s'):7.1f} | {pct('msm_quotient_s'):6.1f} {pct('msm_c_witness_s'):6.1f} {pct('msm_a_s'):6.1f} {pct('msm_b_g1_s'):7.1f} {pct('msm_b_g2_s'):7.1f}")
