# Target 3: sumcheck amortization with a univariate skip

Harness: `bench/sumcheck/` (standalone crate); raw rows in `results/sumcheck_uniskip.csv`.
Statement `sum_x prod_i f_i(x)` over `{0,1}^l`, Goldilocks, Keccak-256 transcript with the tag
pre-absorbed. The honest protocol skips the first `k` variables with one univariate message
(`k = 1` = plain sumcheck). A batch prover for `T` tags computes that message once and redoes only
the tag-dependent tail (fold at the tag's challenge + `l-k` rounds). Single-threaded, M4 Pro,
medians of 3; marginal cost measured over `T = 16` tags. All proofs verify; replays under another
tag and wrong claims are rejected; batch proofs equal single proofs.

| Size | d | k | \|D\| | Single prover | of which skip message | Marginal per extra tag | Shareable |
|---|---|---|---|---|---|---|---|
| 2^20 | 2 | 1 | 3 | 28 ms | 8 ms | 21.1 ms | 26% |
| 2^20 | 2 | 2 | 7 | 20 ms | 9 ms | 11.7 ms | 42% |
| 2^20 | 2 | 3 | 15 | 20 ms | 13 ms | 7.2 ms | 64% |
| 2^20 | 2 | 4 | 31 | 30 ms | 26 ms | 4.6 ms | 85% |
| 2^20 | 2 | 5 | 63 | 51 ms | 47 ms | 3.4 ms | 93% |
| 2^20 | 3 | 1 | 4 | 56 ms | 13 ms | 40.4 ms | 28% |
| 2^20 | 3 | 2 | 10 | 42 ms | 19 ms | 22.7 ms | 45% |
| 2^20 | 3 | 3 | 22 | 52 ms | 38 ms | 13.2 ms | 75% |
| 2^20 | 3 | 4 | 46 | 80 ms | 72 ms | 8.3 ms | 90% |
| 2^20 | 3 | 5 | 94 | 145 ms | 141 ms | 6.0 ms | 96% |

Reading:

* Plain sumcheck (`k = 1`) already lets a batch prover share its first round: 26-29% of the
  prover at 2^20. Each skipped variable moves work from the tag-dependent tail into the shared
  message: 64-75% at `k = 3`, 93-96% at `k = 5`, i.e. an extra tagged proof costs 4-7% of a
  proof.
* The skip message grows like `d 2^k`, so the honest prover's fastest choice is `k = 2-3`
  (42-52 ms at 2^20, d = 3) and `k = 5` is ~3x slower (145 ms). Even measured against the fastest
  honest configuration, the batch prover's marginal cost at `k = 5` (6 ms) is 7x below one honest
  proof, and at the honest optimum `k = 3` it is 13 ms vs 52 ms (74% shareable).
* This is the field-arithmetic side of a hash-based prover that the RO-query model does not see:
  the shared message is committed to under each tag separately (those hashes are not shared),
  but the work of computing it is. For a Spartan-style sumcheck whose polynomial carries a
  challenge-dependent `eq(tau, .)` factor, tau is drawn after the (tagged) commitment and nothing
  is shareable; the numbers above apply to sumchecks over tag-independent polynomials.
