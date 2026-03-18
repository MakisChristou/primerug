# primerug — Architecture & Optimization Knowledge

## Pipeline Overview

Both primerug and rieMiner share a 3-phase pipeline:

```
Presieve (compute offsets) → Sieve (mark composites) → Test (Fermat PRP)
```

## Current primerug Implementation (verified from src/main.rs)

### Constants
- `SIEVE_BITS = 25`, `SIEVE_SIZE = 2^25 = 33554432` positions
- `SIEVE_WORDS = SIEVE_SIZE / 64` (u64 bitarray)
- L1 segment: `SEGMENT_BITS = 18` → 2^18 = 256K positions = 32KB sieve (fits L1)

### Presieve — `compute_factor_offsets()` (main.rs:107-138)
- For each prime p (starting from index m): calls `first_candidate.mod_u(p)` (GMP scalar)
- Computes `f_p = ((p - r) * inverses[i]) % p` — scalar, one prime at a time
- Uses `compute_mi()` for half-pattern offset multiples (doubling with conditional subtract)

### Sieve — `mark_composites()` (main.rs:145-237)
- **Small primes (p < 2^18):** L1-segmented loop — processes all segments, marking bits within each 32KB chunk. Good cache locality.
- **Large primes (p >= 2^18):** Write-combining cache (32 entries) with `_mm_prefetch` to hide latency of scattered writes.
- All bit marking is scalar: `*word |= 1u64 << (pos & 63)`
- No SIMD in sieve.

### Fermat Test — `fermat()` (main.rs:49-56)
- Uses GMP `pow_mod_mut`: `2^(n-1) mod n`, checks result == 1
- Processes **one candidate at a time**
- `test_candidates()` (main.rs:240-279) iterates sieve survivors sequentially

### Threading
- N independent worker threads, each with own `WorkBuffers`
- Each worker: pick random target → presieve → sieve+test for `sieve_iterations` rounds
- Stats via atomics (no locks)

---

## rieMiner Reference Implementation (verified from source)

### ISPC Fermat Test (ispc/primetest.ispc, ispc/fermat.cpp)

**Batching:**
- `JOB_SIZE = 16` candidates per ISPC dispatch
- `MAX_N_SIZE = 64` limbs (32-bit limbs) → max 2048-bit numbers
- Data layout: **AoS** — each candidate's N_Size limbs are contiguous at `offset = programIndex * N_Size`

**Montgomery Setup (fermat.cpp:69-109):**
- `mi = -M^{-1} mod 2^32` computed via `binvert_limb` lookup table + Newton's method
- REDCification: computes `R = 2 * B^n mod M` (Montgomery form of base 2)
- Uses GMP-derived `mpn_div_qr_invert` + `mpn_div_r_preinv_ns` for initial reduction
- `shift` = normalization shift of modulus (for conditional subtraction)

**Binary Exponentiation (primetest.ispc:278-340):**
- Left-to-right square-and-multiply
- Exponent = M itself (the candidate); line 283: `if (en == 0) E--` subtracts 1 for Fermat's n-1
- **Key optimization:** since base is 2, "multiply by base" = left-shift by 1 (line 314: `if (E & bit) a <<= 1`)
- No separate multiply routine needed — only squaring + conditional shift

**REDC (primetest.ispc:295-307):**
- Word-by-word Montgomery reduction inline in the squaring loop
- For each word j: `v = P[j] * mi`, then `P += M * v * B^j` via multiply-accumulate
- Carry propagated to `R[j]`

**Squaring:**
- `toom2SquareFull` (Karatsuba-style) used by default (line 291)
- Splits operand at midpoint: a0 (lower n words), a1 (upper s words)
- Computes 3 sub-squarings: v0=a0^2, vm1=|a0-a1|^2, vinf=a1^2
- `squareSimple` (schoolbook O(n^2)) used for sub-squarings within Toom-2

**Final Check (primetest.ispc:396-418):**
- DeREDCify: one more REDC pass with `T = [R | 0...0]`
- Result is 1 (prime) or M+1 (also prime, due to Montgomery representation wrapping)
- Both cases checked explicitly

**Variants:** `fermat_test` (SSE/AVX2) and `fermat_test512` (AVX-512)

### SIMD Sieve (Stella.cpp)

**Verified from Stella.cpp grep results:**
- SSE2: processes 2 primes × 4 lanes simultaneously using `__m128i`
- AVX2: processes 2 primes × 8 lanes using `__m256i`
- Uses `_mm_cmpgt_epi32` + `_mm_movemask_epi8` to find which offsets need marking
- Factor offsets stored in aligned `__m256i` arrays
- Pattern-specific: 6-tuple uses 3×SSE registers (2 primes × 3 offsets each)

### SIMD Presieve (external/mod_1_2_avx2.asm)
- Hand-written assembly: `rie_mod_1s_2p_8times`
- Computes 8 modular reductions in parallel using AVX2
- Precomputed divisor inverses

---

## Performance Gap Analysis (profiled 2026-03-18, AMD Ryzen 9 5950X, 1 thread)

### Benchmark: 500-digit 8-tuples, SieveIter=16, SieveBits=25

**Head-to-head (same settings: PrimeTableLimit=1M):**

| Metric | primerug | rieMiner | Gap |
|--------|----------|----------|-----|
| c/s | 991 | 1,454 | rieMiner **1.47x faster** |
| ratio | 47.2 | 45.5 | similar |
| us/fermat | 1,004 | ~690 (est.) | rieMiner **1.45x faster** |

**primerug phase breakdown (% of wall-clock time):**

| TableLimit | Presieve | Sieve | Fermat | c/s | ratio | us/fermat |
|------------|----------|-------|--------|-----|-------|-----------|
| 1M | 0.1% | 0.9% | **99.0%** | 991 | 47.2 | 1,004 |
| 10M | 0.4% | 1.3% | **98.3%** | 948 | 43.8 | 1,036 |
| 100M | 0.5% | 1.1% | **98.4%** | 921 | 35.8 | 1,989 |

**Fermat is 98-99% of all time. Sieving and presieve are negligible.**

**rieMiner with auto-tuned PrimeTableLimit (4.3B primes):**
- 895 c/s, ratio 28.8 — massive ratio improvement from deeper sieving.

### Effective search rate (tuples/sec ~ c/s / ratio^8)

| Config | c/s | ratio | Effective rate | vs best |
|--------|-----|-------|---------------|---------|
| primerug 1M | 991 | 47.2 | 3.4e-11 | 139x worse |
| primerug 10M | 948 | 43.8 | 6.7e-11 | 71x worse |
| primerug 100M | 921 | 35.8 | 3.8e-10 | 12x worse |
| rieMiner 1M | 1,454 | 45.5 | 8.6e-11 | 55x worse |
| **rieMiner auto (4.3B)** | **895** | **28.8** | **4.7e-9** | **baseline** |

### Key Insights

1. **Fermat dominance:** At 500 digits, Fermat testing is 98-99% of runtime regardless
   of table limit. Sieve/presieve optimization gives marginal returns.
2. **Deeper sieving is the biggest win for effective rate:** rieMiner's 4.3B table drops
   ratio from 45→29, which at the 8th power gives **140x** improvement. But us/fermat
   doubles at 100M table (1,004→1,989 us) because surviving candidates are "harder".
3. **rieMiner's per-candidate Fermat is 1.45x faster** due to Montgomery REDC +
   ISPC/AVX2 SIMD (2 candidates per instruction). primerug uses vanilla GMP `pow_mod_mut`.
4. **rieMiner uses 92 primorial offsets, primerug uses 1.** This is a massive search space
   multiplier at near-zero cost (same presieve, same sieve, 92x more candidates).

---

## Optimization Priorities (by impact, updated with profiling data)

### Priority 1: Multiple Primorial Offsets (biggest easy win — up to 92x search space)
- rieMiner precomputes 92 valid offsets for the 8-tuple pattern
- Each offset gives an independent candidate stream from the same primorial
- Near-zero additional cost: presieve/sieve is <2% of runtime
- Implementation: generate all valid offsets, loop over them in worker_loop

### Priority 2: Faster Fermat Test (1.5-3x speedup on 98% of runtime)
- Replace GMP `pow_mod_mut` with Montgomery REDC
- 32-bit limbs, binary exponentiation with REDC
- Key trick: base-2 Fermat means multiply = left-shift-by-1, no full multiply needed
- Toom-2 (Karatsuba) squaring for O(N^1.585) vs O(N^2)
- Further: ISPC or AVX2 intrinsics to test 2-16 candidates in parallel

### Priority 3: Deeper Sieving (ratio improvement — 5-10x effective rate)
- Support PrimeTableLimit up to 4B+ (rieMiner auto-tunes to this)
- At 100M table, us/fermat doubles — need faster Fermat first to compensate
- Presieve cost scales linearly with table size but is <1% of runtime

### Priority 4: SIMD Sieve (marginal — <2% of runtime)
- AVX2 sieve marking: process 2 primes per iteration, 8 offsets per register
- Only worth doing after Fermat is optimized

### Priority 5: Threading improvements
- Separate sieve producers from test consumers (producer-consumer pattern)
- Lock-free channels (crossbeam)

### Priority 6: Beyond rieMiner
- AVX-512 Fermat (16-wide)
- GPU offload for Fermat testing
- Adaptive sieve sizing based on L3 cache
