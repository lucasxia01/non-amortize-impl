//! Tagged sumcheck benchmark (target 3).
//!
//! Statement: S = sum_{x in {0,1}^l} prod_{i<d} f_i(x) for fixed multilinears f_i over Goldilocks.
//! Protocol: Fiat-Shamir (Keccak-256, tag pre-absorbed) sumcheck with a univariate skip of the
//! first k variables (Gruen 2024/108; Dao-Thaler 2024/1210): the prover sends s(Y) =
//! sum_{x''} prod_i f~_i(Y, x'') where f~_i(., x'') is the degree < 2^k univariate agreeing with
//! f_i(h, x'') on H = {0, .., 2^k - 1}; s has degree <= d(2^k - 1) and is sent as evaluations on
//! D = {0, .., d(2^k - 1)} which contains H. The verifier checks sum_{h in H} s(h) = S, samples r,
//! sets the claim to s(r), and the prover continues with l - k ordinary rounds on the folded
//! tables g_i(x'') = sum_h L_h(r) f_i(h, x''). k = 1 (H = {0,1}, D = {0..d}) is exactly the plain
//! protocol's first round, so k = 1 measures what a batch prover shares without any skip.
//!
//! Batch prover for T tags: s(Y) is tag-independent and computed once; per tag only the
//! transcript seeding, the fold at that tag's challenge, and the remaining rounds are redone.
//!
//! Usage: sumcheck-uniskip-bench [--l 16,20] [--d 2,3] [--k 1,2,3,4,5]  (k = 1 is the plain protocol: the first round is what gets shared) [--tags 1,2,4,8,16]
//!        [--reps 3] [--out results/sumcheck_uniskip.csv]
use p3_field::extension::BinomialExtensionField;
use p3_field::{BasedVectorSpace, Field, PrimeCharacteristicRing, PrimeField64};
use p3_goldilocks::Goldilocks;
use rand::{Rng, SeedableRng};
use std::io::Write;
use std::time::Instant;
use tiny_keccak::{Hasher, Keccak};

type F = Goldilocks;
type EF = BinomialExtensionField<Goldilocks, 2>;

// ---------------------------------------------------------------- transcript (tagged Keccak)

/// Fiat-Shamir transcript: every RO call is Keccak256(tag || 0-pad to 136 B || state || msg),
/// implemented by cloning the post-tag sponge state (midstate), as in the zkEVM benchmark.
#[derive(Clone)]
struct Transcript { midstate: Keccak, state: [u8; 32] }
impl Transcript {
    fn new(tag: &[u8; 32]) -> Self {
        let mut block = [0u8; 136]; block[..32].copy_from_slice(tag);
        let mut k = Keccak::v256(); k.update(&block);
        let mut t = Self { midstate: k, state: [0u8; 32] };
        t.absorb(b"sumcheck-uniskip-v1"); t
    }
    fn absorb(&mut self, msg: &[u8]) {
        let mut k = self.midstate.clone(); k.update(&self.state); k.update(msg); k.finalize(&mut self.state);
    }
    fn absorb_f(&mut self, xs: &[F]) { let mut b = Vec::with_capacity(xs.len() * 8); for x in xs { b.extend_from_slice(&x.as_canonical_u64().to_le_bytes()); } self.absorb(&b); }
    fn absorb_ef(&mut self, xs: &[EF]) { let mut b = Vec::with_capacity(xs.len() * 16); for x in xs { for c in <EF as BasedVectorSpace<F>>::as_basis_coefficients_slice(x) { b.extend_from_slice(&c.as_canonical_u64().to_le_bytes()); } } self.absorb(&b); }
    /// Sample a challenge in EF (two field elements from two RO calls, reduced mod p).
    fn challenge(&mut self) -> EF {
        let mut cs = [F::ZERO; 2];
        for c in cs.iter_mut() { self.absorb(b"challenge"); let w = u64::from_le_bytes(self.state[..8].try_into().unwrap()); *c = F::from_u64(w % F::ORDER_U64); }
        <EF as BasedVectorSpace<F>>::from_basis_coefficients_slice(&cs).unwrap()
    }
}

// ---------------------------------------------------------------- univariate skip helpers

/// Precomputed data for skipping k variables with a degree-d product.
struct Skip { hsize: usize, dsize: usize,
    /// lag[y][h] = L_h^H(y) for y in D \ H (row index y - hsize), h in H.
    lag: Vec<Vec<F>>,
    /// barycentric weights over D
    bary: Vec<F> }
impl Skip {
    fn new(k: usize, d: usize) -> Self {
        let hsize = 1 << k; let dsize = d * (hsize - 1) + 1;
        let pts: Vec<F> = (0..dsize as u64).map(F::from_u64).collect();
        let lag = (hsize..dsize).map(|y| (0..hsize).map(|h| {
            let mut num = F::ONE; let mut den = F::ONE;
            for j in 0..hsize { if j != h { num *= pts[y] - pts[j]; den *= pts[h] - pts[j]; } }
            num * den.inverse() }).collect()).collect();
        let bary = (0..dsize).map(|i| { let mut w = F::ONE; for j in 0..dsize { if j != i { w *= pts[i] - pts[j]; } } w.inverse() }).collect();
        Self { hsize, dsize, lag, bary }
    }
    /// Lagrange weights L_h^H(r) for h in H at an extension point r.
    fn weights_at(&self, r: EF) -> Vec<EF> {
        (0..self.hsize).map(|h| { let mut num = EF::ONE; let mut den = F::ONE;
            for j in 0..self.hsize { if j != h { num *= r - EF::from_u64(j as u64); den *= F::from_u64(h as u64) - F::from_u64(j as u64); } }
            num * den.inverse() }).collect()
    }
    /// Evaluate the degree < dsize polynomial given by values on D at r (barycentric).
    fn eval_on_d(&self, vals: &[EF], r: EF) -> EF {
        let mut num = EF::ZERO; let mut den = EF::ZERO;
        for (i, v) in vals.iter().enumerate() {
            let diff = r - EF::from_u64(i as u64);
            if diff == EF::ZERO { return *v; }
            let t = diff.inverse() * self.bary[i]; num += t * *v; den += t;
        }
        num * den.inverse()
    }
}

/// The skip-round message: s restricted to D. Tables are indexed x'' * 2^k + h (skipped variables
/// are the low bits). Cost ~ d * 2^l * (|D| - 2^k) base multiplications: this is the shareable work.
fn skip_message(tables: &[Vec<F>], sk: &Skip) -> Vec<F> {
    let hs = sk.hsize; let blocks = tables[0].len() / hs;
    let mut s = vec![F::ZERO; sk.dsize];
    let mut ext = vec![F::ZERO; sk.dsize]; // per-table extension buffer
    let mut prod = vec![F::ONE; sk.dsize];
    for b in 0..blocks {
        prod.iter_mut().for_each(|p| *p = F::ONE);
        for t in tables {
            let vals = &t[b * hs..(b + 1) * hs];
            ext[..hs].copy_from_slice(vals);
            for (y, row) in sk.lag.iter().enumerate() { let mut acc = F::ZERO; for (h, w) in row.iter().enumerate() { acc += *w * vals[h]; } ext[hs + y] = acc; }
            for (p, e) in prod.iter_mut().zip(&ext) { *p *= *e; }
        }
        for (si, p) in s.iter_mut().zip(&prod) { *si += *p; }
    }
    s
}

/// Fold the skipped variables at r: g(x'') = sum_h L_h(r) f(x'' * 2^k + h). Cost d * 2^l base x ext.
fn fold_skip(tables: &[Vec<F>], sk: &Skip, r: EF) -> Vec<Vec<EF>> {
    let w = sk.weights_at(r); let hs = sk.hsize;
    tables.iter().map(|t| t.chunks_exact(hs).map(|c| { let mut acc = EF::ZERO; for (v, wh) in c.iter().zip(&w) { acc += *wh * *v; } acc }).collect()).collect()
}

// ---------------------------------------------------------------- ordinary rounds

/// Round message: p(t) = sum_{x'} prod_i g_i(t, x') at t = 0..=d, folding the low bit.
fn round_message(tables: &[Vec<EF>], d: usize) -> Vec<EF> {
    let half = tables[0].len() / 2;
    let mut out = vec![EF::ZERO; d + 1];
    let mut vals = vec![EF::ZERO; tables.len()];
    for j in 0..half {
        for t in 0..=d {
            let tf = EF::from_u64(t as u64);
            for (vi, g) in vals.iter_mut().zip(tables) { let lo = g[2 * j]; let hi = g[2 * j + 1]; *vi = lo + tf * (hi - lo); }
            out[t] += vals.iter().copied().product::<EF>();
        }
    }
    out
}
fn fold_round(tables: &mut Vec<Vec<EF>>, r: EF) {
    for g in tables.iter_mut() { let half = g.len() / 2; for j in 0..half { let lo = g[2 * j]; let hi = g[2 * j + 1]; g[j] = lo + r * (hi - lo); } g.truncate(half); }
}
/// Evaluate the degree-d univariate given by values at 0..=d at r.
fn eval_univariate(vals: &[EF], r: EF) -> EF {
    let d = vals.len() - 1; let mut acc = EF::ZERO;
    for i in 0..=d { let mut num = EF::ONE; let mut den = F::ONE;
        for j in 0..=d { if j != i { num *= r - EF::from_u64(j as u64); den *= F::from_u64(i as u64) - F::from_u64(j as u64); } }
        acc += vals[i] * num * den.inverse(); }
    acc
}

// ---------------------------------------------------------------- prover / verifier

struct Proof { skip: Vec<F>, rounds: Vec<Vec<EF>>, finals: Vec<EF> }

/// The part after the skip message, which depends on the tag through the transcript.
fn prove_tail(tables: &[Vec<F>], sk: &Skip, claim: F, skip: &[F], tag: &[u8; 32], d: usize) -> Proof {
    let mut tr = Transcript::new(tag);
    tr.absorb_f(&[claim]); tr.absorb_f(skip);
    let r = tr.challenge();
    let mut g = fold_skip(tables, sk, r);
    let l_rem = g[0].len().trailing_zeros() as usize;
    let mut rounds = Vec::with_capacity(l_rem);
    for _ in 0..l_rem {
        let m = round_message(&g, d); tr.absorb_ef(&m);
        let rj = tr.challenge(); fold_round(&mut g, rj); rounds.push(m);
    }
    Proof { skip: skip.to_vec(), rounds, finals: g.iter().map(|t| t[0]).collect() }
}
fn prove(tables: &[Vec<F>], sk: &Skip, claim: F, tag: &[u8; 32], d: usize) -> Proof {
    let skip = skip_message(tables, sk); prove_tail(tables, sk, claim, &skip, tag, d)
}

/// Verifier with oracle access to the f_i (evaluates their MLEs at the final point, O(2^l)).
fn verify(tables: &[Vec<F>], sk: &Skip, claim: F, tag: &[u8; 32], d: usize, pf: &Proof) -> Result<(), String> {
    if pf.skip.len() != sk.dsize { return Err("skip message length".into()); }
    let sum: F = pf.skip[..sk.hsize].iter().copied().sum();
    if sum != claim { return Err("skip message does not sum to the claim".into()); }
    let mut tr = Transcript::new(tag);
    tr.absorb_f(&[claim]); tr.absorb_f(&pf.skip);
    let r = tr.challenge();
    let skip_ef: Vec<EF> = pf.skip.iter().map(|&v| EF::from(v)).collect();
    let mut cur = sk.eval_on_d(&skip_ef, r);
    let mut point = Vec::new();
    for m in &pf.rounds {
        if m.len() != d + 1 { return Err("round message length".into()); }
        if m[0] + m[1] != cur { return Err("round message inconsistent with the claim".into()); }
        tr.absorb_ef(m); let rj = tr.challenge(); cur = eval_univariate(m, rj); point.push(rj);
    }
    // oracle: f_i(r, point) = sum_h L_h(r) MLE_{x''}(f_i(h, .))(point)
    let w = sk.weights_at(r);
    let mut eq = vec![EF::ONE];
    // the prover folds the low bit first, so challenge j is bit j of the x'' index
    for &rj in &point { let mut nx: Vec<EF> = eq.iter().map(|e| *e * (EF::ONE - rj)).collect(); nx.extend(eq.iter().map(|e| *e * rj)); eq = nx; }
    let mut prod = EF::ONE;
    for (t, fin) in tables.iter().zip(&pf.finals) {
        let mut acc = EF::ZERO;
        for (b, e) in eq.iter().enumerate() { let mut inner = EF::ZERO; for h in 0..sk.hsize { inner += w[h] * t[b * sk.hsize + h]; } acc += *e * inner; }
        if acc != *fin { return Err("final evaluation mismatch".into()); }
        prod *= acc;
    }
    if prod != cur { return Err("final product mismatch".into()); }
    Ok(())
}

fn tag_of(i: u64) -> [u8; 32] { let mut k = Keccak::v256(); k.update(b"tag"); k.update(&i.to_le_bytes()); let mut o = [0u8; 32]; k.finalize(&mut o); o }

fn arg_list(args: &[String], name: &str, default: &str) -> Vec<usize> {
    let v = args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone()).unwrap_or(default.into());
    v.split(',').map(|s| s.trim().parse().unwrap()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ls = arg_list(&args, "--l", "16,20"); let ds = arg_list(&args, "--d", "2,3"); let ks = arg_list(&args, "--k", "1,2,3,4,5");
    let tags = arg_list(&args, "--tags", "1,2,4,8,16"); let reps = arg_list(&args, "--reps", "3")[0];
    let out = args.windows(2).find(|w| w[0] == "--out").map(|w| w[1].clone()).unwrap_or("results/sumcheck_uniskip.csv".into());
    let new = !std::path::Path::new(&out).exists();
    std::fs::create_dir_all(std::path::Path::new(&out).parent().unwrap()).ok();
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&out).unwrap();
    if new { writeln!(f, "l,d,k,dsize,T,t_single_s,t_skip_s,t_tail_s,t_batch_s,marginal_s,shareable_frac,t_verify_s,all_verified").unwrap(); }
    let tmax = *tags.iter().max().unwrap();
    for &l in &ls { for &d in &ds {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let tables: Vec<Vec<F>> = (0..d).map(|_| (0..1usize << l).map(|_| F::from_u64(rng.random::<u64>() % F::ORDER_U64)).collect()).collect();
        let claim: F = (0..1usize << l).map(|x| tables.iter().map(|t| t[x]).product::<F>()).sum();
        for &k in &ks {
            let sk = Skip::new(k, d);
            let med = |v: &mut Vec<f64>| { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] };
            let (mut t_single, mut t_skip, mut t_tail) = (vec![], vec![], vec![]);
            for _ in 0..reps {
                let t = Instant::now(); let skip = skip_message(&tables, &sk); t_skip.push(t.elapsed().as_secs_f64());
                let t = Instant::now(); let _ = prove_tail(&tables, &sk, claim, &skip, &tag_of(0), d); t_tail.push(t.elapsed().as_secs_f64());
                let t = Instant::now(); let _ = prove(&tables, &sk, claim, &tag_of(0), d); t_single.push(t.elapsed().as_secs_f64());
            }
            let (ts, tsk, ttl) = (med(&mut t_single), med(&mut t_skip), med(&mut t_tail));
            // batch prover for tmax tags: skip once, tails per tag; verify all; check cross-tag rejection
            let t = Instant::now();
            let skip = skip_message(&tables, &sk);
            let mut per_tag = Vec::with_capacity(tmax); let mut proofs = Vec::with_capacity(tmax);
            for i in 0..tmax { let t = Instant::now(); let p = prove_tail(&tables, &sk, claim, &skip, &tag_of(i as u64), d); per_tag.push(t.elapsed().as_secs_f64()); proofs.push(p); }
            let t_batch_all = t.elapsed().as_secs_f64();
            let tv = Instant::now();
            let mut ok = true;
            for (i, p) in proofs.iter().enumerate() { if let Err(e) = verify(&tables, &sk, claim, &tag_of(i as u64), d, p) { ok = false; eprintln!("VERIFY FAILED tag {i}: {e}"); } }
            let t_verify = tv.elapsed().as_secs_f64() / tmax as f64;
            // replay under another tag must fail; a batch proof must equal a single proof for the same tag
            if verify(&tables, &sk, claim, &tag_of(1_000_000), d, &proofs[0]).is_ok() { ok = false; eprintln!("proof accepted under a different tag"); }
            let single0 = prove(&tables, &sk, claim, &tag_of(0), d);
            if !(single0.skip == proofs[0].skip && single0.rounds == proofs[0].rounds && single0.finals == proofs[0].finals) { ok = false; eprintln!("batch proof differs from single proof"); }
            // wrong claim must be rejected
            if verify(&tables, &sk, claim + F::ONE, &tag_of(0), d, &proofs[0]).is_ok() { ok = false; eprintln!("wrong claim accepted"); }
            for &tt in &tags {
                let t_batch = tsk + per_tag[..tt].iter().sum::<f64>();
                let marginal = if tt > 1 { (t_batch - (tsk + per_tag[0])) / (tt - 1) as f64 } else { ttl };
                let share = 1.0 - marginal / ts;
                writeln!(f, "{l},{d},{k},{},{tt},{ts:.5},{tsk:.5},{ttl:.5},{t_batch:.5},{marginal:.5},{share:.4},{t_verify:.5},{ok}", sk.dsize).unwrap();
            }
            let _ = t_batch_all;
            eprintln!("l={l} d={d} k={k} |D|={:3}: single {ts:.3}s (skip {tsk:.3}s + tail {ttl:.3}s)  marginal/tag {:.3}s  shareable {:.1}%  verify {t_verify:.3}s  ok={ok}", sk.dsize, per_tag[1..].iter().sum::<f64>() / (tmax - 1) as f64, (1.0 - (per_tag[1..].iter().sum::<f64>() / (tmax - 1) as f64) / ts) * 100.0);
        }
    }}
}
