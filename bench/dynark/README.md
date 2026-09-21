# Dynark benchmark: the trivial "tag as public input" Groth16 is amortizable

Toy tagged proof system: Groth16 (BLS12-381, arkworks 0.5) where the tag is public input #1,
bound by one constraint `tag * 0 = 0` (an unconstrained public input has a zero verifier-key
point and the proof would verify under any tag). A batch prover with the witness makes proofs
for T tags with Dynark's proof update (eprint 2025/1897, `OYJason08/DYNARK_` @ `c73a380`):
changing the tag changes one assignment entry and one constraint row, so an update is O(1)
group operations after a preprocessing step.

* `patches/dynark-tagged-bench.patch` — adds to `ark-groth16`:
  * `update_dynark_with_rng`: the upstream `update_dynark` draws its re-randomization from
    `ark_std::test_rng()` (deterministic); this takes a caller RNG.
  * `process_dynark_rows`: the projection terms `q_a[i], q_b[i]` for the given rows only, via the
    closed form (one size-n G1 MSM per row and vector) instead of upstream `process_dynark`'s
    O(n log n) FFT convolutions over all rows. Checked equal to the full version at the tag row.
  * `examples/tagged_bench.rs`: the circuits, the batch prover, the checks and the CSV writer.
* Circuits: `sha256:<blocks>` — SHA-256 preimage knowledge over `blocks` 64-byte blocks (arkworks
  gadget; ~40k constraints per block) with the tag as first public input; `synthetic:<log n>` —
  Dynark's random quadratic R1CS at 2^log n constraints.
* Checks in every run: a fresh proof verifies under its tag and is rejected under another (tag is
  bound); every updated proof verifies under its own tag; the first seven are also checked to be
  rejected under tag 0; with `CHECK=1` targeted preprocessing is compared to full preprocessing.
* Reported per (circuit, size, T): fresh-proof time (median of REPS), targeted and full
  preprocessing time, mean update time, honest cost `T x fresh`, batch cost
  `fresh + preprocessing + (T-1) x update`, and their ratio (amortization factor).

```bash
bench/dynark/setup.sh
CONFIGS="sha256:1 synthetic:16" TAGS="2 4 16 64 256" REPS=3 bench/dynark/run.sh
python3 bench/dynark/summarize.py results/dynark_tagged.csv
```

Measured on an Apple M4 Pro (14 threads; Groth16 proving and the MSMs are multithreaded).

## Groth16 MSM share (target 4)

`msm_share` (same patch) times each phase of the Groth16 prover used above: R1CS evaluation on
the assignment, the quotient FFTs, scalar conversions, and the five multi-scalar multiplications
(quotient over powers of tau, C over the witness key, A, B in G1, B in G2). This is the
group-based analogue of the hashing share: MSMs are the work an outsider cannot skip.

```bash
CONFIGS="sha256:1 synthetic:16" THREADS="1 14" REPS=3 bench/dynark/msm_share.sh
python3 bench/dynark/summarize_msm.py results/groth16_msm_share.csv
```
