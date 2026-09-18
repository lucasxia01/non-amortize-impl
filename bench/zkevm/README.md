# zkEVM hash-share benchmark

Measures how much of a realistic hash-based SNARK prover is hashing, on Polygon's type-1 zkEVM
(`0xPolygonZero/zk_evm` @ `ef38861`, Plonky2/Starky FRI over Goldilocks, `KeccakGoldilocksConfig`:
Keccak-256 Merkle trees and challenger) proving a one-transaction block (an ERC-20 transfer via a
contract call, 56,499 gas; 64k zkEVM cycles across 10 STARK tables).

* `patches/plonky2-keccak-stats.patch` — instruments `KeccakHash` (leaf hash, node compression,
  challenger permutation incl. PoW grinding) with per-thread timers/counters; no functional change.
* `patches/zk_evm-bench.patch` — the `erc20` test reads `ZKEVM_RATE_BITS`, `ZKEVM_ARITY_BITS`,
  `ZKEVM_POW_BITS`, `ZKEVM_CAP_HEIGHT` (100-bit security, queries = ceil((100-pow)/rate_bits)),
  prints `KECCAK_WORK`, `KECCAK_PHASE`, `PHASE_WALL`, `PROOF_SIZE_BYTES`; the prover prints per-phase
  hash accounting.
* `setup.sh` — clone, vendor+patch plonky2 1.0.0, apply the bench patch, build (needs the pinned
  nightly toolchain; rustup installs it).
* `run.sh` — sweep configs × thread counts × reps; appends to `results/zkevm_hashshare.csv`.
* `summarize.py` — medians per config: wall, CPU, hash share of CPU, leaf/node/perm split,
  shareable (tag-independent) share, proof size.

```bash
bench/zkevm/setup.sh
CONFIGS="1:4:16 3:1:20" THREADS="1 14" REPS=3 bench/zkevm/run.sh
python3 bench/zkevm/summarize.py results/zkevm_hashshare.csv
```

Measured on an Apple M4 Pro (14 cores, 24 GB) under macOS 25.6 with Rust nightly-2024-09-24
(zk_evm's pinned toolchain); `results/zkevm_hashshare.csv` holds that machine's runs.

Hash share = (sum of in-situ timed Keccak calls, all threads) / (process user+sys CPU). Report the
single-threaded share (no contention inflation) and the multithreaded wall time. PoW grinding is
randomized (expected 2^pow permutations per STARK, 10 STARKs), so use medians over reps.
