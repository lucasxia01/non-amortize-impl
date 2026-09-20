# Target 2: the trivial tagged Groth16 is amortizable (Dynark)

Toy tagged system: Groth16 (BLS12-381) with the tag as public input #1 bound by `tag * 0 = 0`.
Batch prover: one fresh proof, a preprocessing step, then one Dynark proof update per extra tag
(each update changes one assignment entry and one constraint row). Harness: `bench/dynark/`
(pinned Dynark commit + patch); raw rows in `results/dynark_tagged.csv`. Apple M4 Pro, 14
threads. Every updated proof verifies under its own tag and is rejected under another; the
targeted preprocessing equals Dynark's full preprocessing at the tag row (`CHECK=1`).

| Circuit | Constraints | Fresh proof | Targeted prep | Full prep (Dynark) | Update | Batch ratio T=2 / 16 / 64 / 256 |
|---|---|---|---|---|---|---|
| SHA-256, 1 block | 65,536 | 0.109 s | 0.045 s | 14.6 s | 1.0 ms | 1.4 / 10.3 / 31.8 / 67.6 |
| SHA-256, 4 blocks | 262,144 | 0.372 s | 0.168 s | 63.4 s | 1.0 ms | 1.4 / 10.7 / 39.3 / 118.5 |
| SHA-256, 16 blocks | 1,048,576 | 1.414 s | 0.730 s | 279.9 s | 1.0 ms | 1.3 / 10.5 / 41.0 / 150.8 |
| synthetic R1CS | 65,536 | 0.296 s | 0.062 s | 15.3 s | 1.0 ms | 1.7 / 12.7 / 44.9 / 123.4 |
| synthetic R1CS | 262,144 | 1.099 s | 0.271 s | 70.5 s | 1.0 ms | 1.6 / 12.7 / 49.1 / 173.0 |
| synthetic R1CS | 1,048,576 | 4.024 s | 1.070 s | 296.8 s | 1.0 ms | 1.6 / 12.6 / 49.9 / 192.6 |

Batch ratio = (T x fresh) / (fresh + preprocessing + (T-1) x update) with the targeted
preprocessing. Reading:

* A second tagged proof costs ~1 ms plus a one-time preprocessing of 25-50% of a proof, so the
  batch prover pays ~1.3-1.7 proofs for 2 tags, ~1.4 proofs for 16 tags, and the ratio grows
  almost linearly in T (the update cost is negligible against the proof at every size).
* Dynark's own full preprocessing (all n projection terms) is 100-200x a proof here and only
  pays off past ~T = 100-300 tags; the closed-form single-row preprocessing (two size-n MSMs)
  makes the attack cheap immediately. Both are the honest prover's own work (they need the
  witness), which is exactly what the definition permits.
* The constraint `tag * 0 = 0` binds the tag (a fresh proof under another tag is rejected) but
  does nothing else; any circuit in which the tag appears in few constraints behaves the same,
  since the update cost depends on the number of rows the tag touches, not on the circuit size.
