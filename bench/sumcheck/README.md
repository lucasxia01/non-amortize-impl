# Sumcheck with a univariate skip: how much a batch prover shares across tags

Standalone crate (Plonky3 field crates, Goldilocks + quadratic extension, Keccak-256 transcript
with the tag pre-absorbed as in the zkEVM benchmark). Statement: `S = sum_x prod_{i<d} f_i(x)`
over `{0,1}^l` for fixed random multilinears. Protocol: Fiat-Shamir sumcheck whose first `k`
variables are handled by one univariate-skip message (Gruen 2024/108, Dao-Thaler 2024/1210) of
degree `d(2^k-1)` sent as evaluations on `D = {0..d(2^k-1)}`; then `l-k` ordinary rounds.
`k = 1` is the plain protocol (its first round is what a batch prover can share without a skip).

Batch prover for `T` tags: the skip message is tag-independent and computed once; per tag the
prover re-seeds the transcript, folds the tables at that tag's challenge and runs the remaining
rounds. Reported per `(l, d, k, T)`: single-prover time (skip + tail), batch time, marginal cost
per extra tag, shareable fraction `1 - marginal / single`, verifier time.

Checks on every configuration: all `T` proofs verify (including the skip message summing to the
claim over `H` and the final oracle check), a proof replayed under another tag is rejected, a wrong
claim is rejected, and the batch prover's proof for a tag equals the single prover's proof.

```bash
cargo build --release --manifest-path bench/sumcheck/Cargo.toml
bench/sumcheck/target/release/sumcheck-uniskip-bench --l 16,20 --d 2,3 --k 1,2,3,4,5 --tags 1,2,4,8,16 --reps 3 --out results/sumcheck_uniskip.csv
```
Single-threaded; the full sweep runs in under ten seconds on an M4 Pro.
