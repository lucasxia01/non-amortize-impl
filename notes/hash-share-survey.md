# Hashing share of the prover in hash-based SNARKs (survey, 2026-09-18)

Machine: Apple M4 Pro (14 cores, 24 GB). Method: sampling profiler (`samply record --save-only
--unstable-presymbolicate`) on each system's own prover; each sample is attributed to the innermost
frame whose crate is not a runtime crate (core/std/rayon/...), idle kernel-wait samples dropped,
and where possible restricted to samples under the prover's root function. "Hash" = samples owned
by the hash-primitive crates (p3_keccak, p3_symmetric sponge/compression wrappers, blake3, sha2,
blake2...); "Merkle" = tree-building overhead outside the primitive. Binius64 numbers come from its
own per-span tracing. The sampler-based numbers below (everything except the zkEVM section) are exploratory: on macOS
the sampler under-samples busy worker threads, so they were taken single-threaded where possible
and should be read as approximate. The zkEVM section is the authoritative, counter-based
measurement and is reproducible from `bench/zkevm/`. The sampler script and the Plonky3 probes
(narrow AIR, Ligero commit) are not in the repo.

## Full provers

| System (PCS) | Field | Hash | Statement | Hash share of prover |
|---|---|---|---|---|
| Plonky3 0.7 uni-stark FRI, **narrow synthetic AIR** (2 cols), rate 1/2, 2^20 rows | BabyBear | Keccak-256 | degree-2 chain | **51%** (+3% tree) |
| same, 8 cols, rate 1/2 | | | | **49%** (+2%) |
| same, 8 cols, **rate 1/8** | | | | **56%** (+3%); 64% when counting all threads |
| same, 32 cols, rate 1/2 | | | | 40% |
| same, 128 cols, rate 1/2 | | | | 34% |
| Plonky3 0.7 uni-stark FRI, Keccak-AIR (2633 cols), rate 1/2 / 1/4 / 1/8, 2^16 rows | BabyBear | Keccak-256 | 2730 keccak-f | 25% / 31% / 37% |
| Plonky3 0.7 uni-stark FRI, Blake3-AIR 2^16 / Poseidon2-AIR 2^18 | BabyBear | Keccak-256 | | 27% / 25% |
| Plonky3 HEAD binary-PCS (BaseFold-style), rate 1/4 | GF(2^128) tower | Keccak-256 | binary Keccak-AIR 2^16 | 32% (+2%) |
| Flock (Ligerito) | GF(2^128) | Blake3 (NEON) | 32k Blake3 compressions | ~12-20% (Merkle crate 12%, blake3 primitive partly inlined; setup hashing not fully separable) |
| Binius64 (FRI-Binius), rate 1/2 / 1/4 | GF(2^64/128) | SHA-256 (hw) | `ethsign`, 32 ECDSA sigs | 2% / 4% |
| Binius64, rate 1/2 | | SHA-256 | Keccak-256 of 64 KiB | 3% |

## PCS-only (commit / open)

| PCS | Field | Hash | Size | Hash share |
|---|---|---|---|---|
| arkworks 0.6 Brakedown (multilinear) | BN254 Fr | Blake2s leaves + SHA-256 nodes | 2^20 | commit 15%, open 9% (+18% tree) |
| arkworks 0.6 Ligero (multilinear) | BN254 Fr | same | 2^20 | commit 9%, open 8% (+5%) |
| Ligero-style probe on Plonky3: RS-LDE rows + Merkle columns | BabyBear | Keccak / SHA-256 / Blake3 | 2^20-2^22, rate 1/4 | commit 28-30% / 36-38% / 45-48% (multi-threaded; NEON DFT is ~1 ns/element) |

Not measurable: lcpc (2021 Ligero/Brakedown reference) no longer compiles; Blaze has no public
code; original Binius is archived and uses Groestl/Vision.

## Takeaways

- **Hashing dominates when the trace is narrow.** Merkle cost is per row (leaf + internal node,
  each ~1 Keccak-f for <=32-byte leaves), times blowup, times three trees (trace, quotient, FRI
  layers), while field work is per column. A 2-8 column AIR at 2^20 rows with Keccak-256 is 50-56%
  hashing single-threaded (64% at rate 1/8 counting all threads), and this is exactly the "simple
  AIR" setting. Wide AIRs (Keccak-AIR, 2633 columns) are 25-37%.
- Fast hashes kill the share: Blake3 (NEON) and hardware SHA-256 are 3-5x faster per byte than
  software Keccak, so Flock and Binius64 hash for <20% / <5% of the prover.
- Linear-code PCSs over big fields (arkworks Brakedown/Ligero on BN254) are encoding-bound (256-bit
  multiplications), not hash-bound; over BabyBear the NEON DFT is so cheap that even Ligero's
  commit is only ~30% Keccak.
- For the non-amortizability paper the attacker's shareable work is not (1 - hash share): after the
  first Merkle commitment all field work is challenge-dependent and hence tag-bound. Only witness
  generation and the first LDE/encoding are shareable (Plonky3 narrow AIR: ~10% of the prover).


## Realistic Ethereum-block statement: Polygon type-1 zkEVM (`0xPolygonZero/zk_evm`)

`evm_arithmetization` proves real EVM execution of a block across 10 STARK tables (CPU, Arithmetic,
Logic, Memory, MemBefore/After, BytePacking, Keccak, KeccakSponge, Poseidon/MPT) with cross-table
lookups, Goldilocks field, Plonky2/Starky FRI, and `KeccakGoldilocksConfig` (Keccak-256 Merkle
trees and challenger). Its tests prove one-transaction blocks: `erc20` (contract call doing an
ERC-20 transfer, 56k gas, MPT state-root update), `simple_transfer`, `erc721`, `log_opcode`, etc.
Reproducible from `bench/zkevm/` (pinned commit, patches, scripts).

Parameter sweep on the ERC-20 block, reproducible via `bench/zkevm/` (raw rows in
`results/zkevm_hashshare.csv`, 3 reps per cell, medians). Hash share = in-situ timed Keccak calls
(all threads, minus ~40 ns/call timer overhead) / process CPU time; queries = ceil((100-pow)/rate_bits);
proof size = sum of the ten per-table STARK proofs before recursive wrapping.

| Rate | FRI arity | PoW bits | Queries | Wall, 14 thr | CPU, 1 thr | Hash share, 1 thr | Hash share, 14 thr | Proof size |
|---|---|---|---|---|---|---|---|---|
| 1/2 | 16 | 16 | 84 | 3.5 s | 12.5 s | 19% | 21% | 4.08 MB |
| 1/2 | 2 | 16 | 84 | 3.6 s | 12.9 s | 19% | 20% | 4.61 MB |
| 1/4 | 2 | 16 | 42 | 4.4 s | 17.7 s | 27% | 29% | 2.49 MB |
| 1/8 | 2 | 16 | 28 | 5.7 s | 27.6 s | 34% | 36% | 1.78 MB |
| 1/16 | 2 | 16 | 21 | 10.2 s | 46.9 s | 40% | 45% | 1.43 MB |
| 1/8 | 2 | 20 | 27 | 6.4 s | 29.5 s | 38% | 47% | 1.72 MB |
| 1/16 | 2 | 20 | 20 | 10.6 s | 56.3 s | 47% | 49% | 1.37 MB |
| 1/4 | 16 | 24 | 38 | 19.3 s | 63.6 s | 74% | 93% | 1.99 MB |
| 1/8 | 16 | 24 | 26 | 13.3 s | 87.1 s | 74% | 84% | 1.44 MB |
| 1/16 | 16 | 24 | 19 | 17.6 s | 149 s | 76% | 77% | 1.11 MB |

Hashing by role (single-threaded, rate 1/8, 20 PoW bits): Merkle leaves 60%, Merkle nodes 22%,
challenger + PoW grinding 18%. The multithreaded share runs a few points higher because Merkle
hashing is memory-bandwidth bound and inflates more under contention than the field work.
Levers: rate is the honest one (more Merkle leaves per unit of field work, smaller proofs); FRI
arity is irrelevant; PoW bits are free up to ~20 and dominate the prover at 24 (2^24 Keccak per
STARK x 10 STARKs, grinding is then ~90% of all hashing). Recommended headline: rate 1/8, 20 PoW.

## Tagged prover (the non-amortizable construction on the zkEVM)

Every Keccak call in the Plonky2 hasher (Merkle leaves, node compressions, challenger
permutations and therefore PoW grinding) becomes `Keccak256(tag || 0^104 || input)`, implemented
by cloning the sponge state after absorbing the tag block ("midstate", `ZKEVM_TAG=<hex>`).
Verified: midstate == literal one-pass prefix at 16 lengths across block boundaries; a tagged
proof verifies under its tag and is rejected under a tag differing in one bit; digest dumps of
two tagged runs and the untagged run (7.3M / 6.7M / 6.9M digests) share 0 outputs.

Overhead, measured as ns per hash call by role (independent of the random PoW effort), ERC-20
block, 3 reps, `results/zkevm_tagged.csv`:

| Config | Threads | Mode | ns/leaf | ns/node | ns/perm | Prover CPU |
|---|---|---|---|---|---|---|
| rate 1/2, PoW 16 | 1 | untagged | 561 | 197 | 705 | 12.67 s |
| rate 1/2, PoW 16 | 1 | tagged | 566 | 196 | 707 | 12.56 s |
| rate 1/8, PoW 20 | 1 | untagged | 516 | 193 | 705 | 29.7 s (3.1M grinding perms) |
| rate 1/8, PoW 20 | 1 | tagged | 499 | 193 | 701 | 36.2 s (12.3M grinding perms) |

Per-call cost is unchanged (within +-2%): the construction is free. Total prover time differs only
through PoW grinding, which is deterministic per transcript and therefore per tag (expected 2^20
attempts per STARK, 10 STARKs; the two transcripts above needed 3.1M and 12.3M).



Plonky3 uni-stark + Keccak-256 Merkle tree and challenger, BabyBear, a narrow synthetic AIR
(8 columns, degree-2 transition), 2^20-2^22 rows, rate 1/8: ~56% hashing single-threaded, with
Keccak-AIR (wide) and rate sweeps as the secondary rows. (Plonky3 narrow-AIR probe; exploratory, not in the repo.)

## Benchmark suggestions for Ethereum / Zcash statements

1. Ethereum signatures: Binius64 `binius-examples ethsign prove -n N` (secp256k1 + Keccak). Ready
   made but only 2-4% hashing.
2. Ethereum hashing (RLP / Merkle-Patricia): a Keccak-f batch, Plonky3 Keccak-AIR (25-37%).
3. Bitcoin/Zcash: SHA-256d header chains (Binius64 `bitcoin_headers`) or batched SHA-256/Blake3
   compressions (Flock); depth-32 note-commitment Merkle membership maps to Flock's Merkle-path
   statements or Binius64 `hashsign`.
