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
* `check_tags.py` — cross-tag disjointness check on digest dumps (see below).
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

## Tagged (non-amortizable) prover

`ZKEVM_TAG=<hex, up to 32 bytes>` turns every Keccak call the prover and verifier make (Merkle
leaves, Merkle node compressions, challenger permutations, hence also PoW grinding) into
`Keccak256(tag || 0^104 || input)`: the tag padded to one full 136-byte sponge block. The
implementation clones the sponge state after absorbing that block ("midstate"), so a tagged call
costs exactly one clone of a 200-byte state more than an untagged one and no extra permutation.
Unset = plain Keccak-256. The construction lives in `plonky2/src/hash/keccak.rs::tagged` (patch).

Checks built into the `erc20` test when a tag is set:

* midstate equals the literal one-pass `Keccak256(tag || padding || input)` at 16 input lengths
  spanning block boundaries (asserted);
* the proof verifies under its tag and is rejected under a tag differing in one bit (asserted,
  `TAG_CHECK` line);
* with `ZKEVM_HASH_DUMP=<file>` the first 8 bytes of every digest produced are written out;
  `check_tags.py a.bin b.bin ...` reports shared digests across runs. Two tags and the untagged
  run share none (7.3M / 6.7M / 6.9M digests, 0 in common; the 515 within-run repeats are
  identical inputs such as padding rows). Dump mode serializes hashing through a mutex, so do not
  time it.

```bash
D=bench/zkevm/work; B=$(cat $D/erc20.bin)
for t in "" 00..01 deadbeef..; do ZKEVM_TAG=$t ZKEVM_HASH_DUMP=$D/dump_${t:0:8}.bin \
  ZKEVM_RATE_BITS=1 ZKEVM_ARITY_BITS=4 ZKEVM_POW_BITS=16 $B --nocapture --test-threads=1; done
python3 bench/zkevm/check_tags.py $D/dump_*.bin
ZKEVM_TAG=deadbeef OUT=results/zkevm_tagged.csv CONFIGS="3:1:20" bench/zkevm/run.sh   # overhead
# precise overhead: PoW off (identical call sequences) and the isolated microbenchmark
for t in "" deadbeef; do ZKEVM_TAG=$t OUT=results/zkevm_tagged_pow0.csv CONFIGS="1:4:0 3:1:0" THREADS=1 REPS=5 bench/zkevm/run.sh; done
(cd bench/zkevm/work/zk_evm && cargo test --release -p evm_arithmetization --test hash_overhead -- --nocapture)
```
