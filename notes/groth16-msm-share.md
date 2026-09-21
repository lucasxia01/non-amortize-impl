# Target 4: MSM share of a group-based prover (Groth16)

Harness: `bench/dynark/msm_share.sh` (same patched Dynark/ark-groth16 prover as target 2, BLS12-381,
phase timers around the R1CS evaluation, the quotient FFTs, scalar conversions and the five MSMs);
raw rows in `results/groth16_msm_share.csv`. Apple M4 Pro, medians of 3, warm-up excluded.

| Circuit | Constraints | Threads | Prover | MSM | FFT (quotient) | R1CS eval + other | quotient MSM | C (witness) | A | B G1 | B G2 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| SHA-256, 1 block | 65,536 | 1 | 0.55 s | 84.4% | 15.0% | 0.6% | 75.8% | 1.6% | 3.8% | 1.1% | 2.1% |
| SHA-256, 4 blocks | 262,144 | 1 | 2.09 s | 81.7% | 17.7% | 0.6% | 73.1% | 1.9% | 3.2% | 1.4% | 2.3% |
| synthetic R1CS | 65,536 | 1 | 2.10 s | 95.8% | 4.0% | 0.2% | 20.1% | 20.1% | 11.4% | 11.2% | 33.1% |
| synthetic R1CS | 262,144 | 1 | 7.69 s | 94.9% | 5.0% | 0.1% | 20.3% | 20.3% | 11.1% | 11.1% | 32.0% |
| SHA-256, 1 block | 65,536 | 14 | 0.11 s | 72.8% | 25.3% | 1.8% | 51.3% | 5.5% | 6.0% | 3.0% | 6.5% |
| SHA-256, 4 blocks | 262,144 | 14 | 0.36 s | 77.4% | 20.9% | 1.0% | 54.2% | 6.3% | 6.5% | 3.1% | 7.5% |
| synthetic R1CS | 65,536 | 14 | 0.28 s | 90.3% | 9.7% | 0.8% | 19.6% | 19.4% | 10.1% | 10.1% | 30.5% |
| synthetic R1CS | 262,144 | 14 | 1.00 s | 92.3% | 7.3% | 0.3% | 20.1% | 20.2% | 10.8% | 10.2% | 30.7% |

Reading:

* Multi-scalar multiplication is 82-96% of the Groth16 prover single-threaded (73-92% on 14
  threads, since the FFTs parallelize worse); the quotient FFTs are the only other real cost.
  This is the group-based counterpart of the hashing share (19-47% for the zkEVM STARK).
* The witness shape matters: SHA-256's assignment is mostly bits, and Pippenger skips zero
  digits, so its four assignment MSMs are cheap and the prover is ~4x faster than the synthetic
  circuit of the same size; three quarters of it is the quotient MSM, whose scalars are
  full-width. The synthetic circuit (random full-width scalars) spreads evenly over the five MSMs.
* For the paper's group-based discussion: a batch prover cannot skip these MSMs across tags
  (each tagged proof needs its own group elements), except that, as target 2 shows, when the tag
  enters through a public input the second proof's MSMs collapse to O(1) scalar multiplications
  plus one preprocessing MSM.
