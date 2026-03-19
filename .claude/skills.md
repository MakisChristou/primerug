# primerug — Architecture & Optimization Knowledge

## Pipeline Overview

Both primerug and rieMiner share a 3-phase pipeline:

```
Presieve (compute offsets) → Sieve (mark composites) → Test (Fermat PRP)
```

## Current primerug Implementation (updated 2026-03-19)

### Architecture: Sieve-Worker Mode (`--sieve-workers N`)

```
Sieve Workers (K threads)  →  crossbeam MPMC queue  →  Test Workers (T-K threads)
 - own factors_to_eliminate                              - no sieve memory
 - own additional_factors (sparse)                       - Integer buffers only
 - own sieve bitarray                                    - Fermat PRP testing
 - push CandidateBatch {first_candidate, survivors}      - CPU or GPU
```

When `--sieve-workers 0` (default): monolithic mode (each thread does sieve + test).

### Sparse Large-Prime Elimination (rieMiner-style)

**factorMax** = `sieve_iterations × SIEVE_SIZE`

- **Dense primes** (< factorMax): stored in `factors_to_eliminate[tuple_size × i + f]`, iterated per sieve iteration (carried over with `-= SIEVE_SIZE`). Same as rieMiner.
- **Sparse primes** (≥ factorMax): each hits sieve at most once across ALL iterations. Pre-computed into `additional_factors[iteration] → Vec<u32>` of sieve positions during `compute_factor_offsets`. Marked in a separate `mark_composites_sparse()` pass.

**Memory per sieve worker at PTL=4B, 8-tuple, SI=16:**
- Dense: 8 × 28M × 4 bytes ≈ 864 MB
- Sparse: ~400M entries × 4 bytes ≈ 1.6 GB
- Total: ~2.5 GB per sieve worker

### u32 Prime Table

Primes and inverses stored as `Vec<u32>` (all primes up to 2^32 fit in u32).
Saves 50% memory: 203M primes × 4B = 812 MB (was 1.6 GB at u64).

### GPU Fermat Service (primerug-gpu)

- CUDA kernel using **CGBN** (Cooperative Groups BigNum) for big-integer arithmetic
- **Miller-Rabin base-2** test (stronger than Fermat) with Montgomery multiplication
- `BITS=2048`, `TPI=32` (one warp per candidate) — covers up to 617-digit numbers
- Batch pipeline: round 0 on all candidates → compact survivors → round 1 on survivors → ...
- Runs as standalone Unix socket service, primerug connects via `--gpu-socket`
- `compact_survivors` kernel uses atomicAdd for stream compaction between rounds

### Constants
- `SIEVE_BITS = 25`, `SIEVE_SIZE = 2^25 = 33554432` positions
- `SIEVE_WORDS = SIEVE_SIZE / 64` (u64 bitarray)
- L1 segment: `SEGMENT_BITS = 18` → 2^18 = 256K positions = 32KB sieve (fits L1)

### Key Source Files
- `primerug/src/main.rs` — WorkBuffers, TestBuffers, all worker loops, sieve functions
- `primerug/src/work_queue.rs` — CandidateBatch + WorkQueue (crossbeam bounded channel)
- `primerug/src/tools.rs` — prime table generation (u32), primorial, inverses
- `primerug/src/args.rs` — CLI args including `--sieve-workers`
- `primerug/src/gpu_client.rs` — GPU service client (Unix socket)
- `primerug-gpu/cuda/fermat_kernel.cu` — CGBN Miller-Rabin kernels
- `primerug-gpu/src/gpu.rs` — GpuPipeline: kernel launch, compaction, batch orchestration
- `primerug-gpu/src/protocol.rs` — Wire format for batch request/result

---

## rieMiner Reference Implementation

### ISPC Fermat Test (ispc/primetest.ispc, ispc/fermat.cpp)

- `JOB_SIZE = 16` candidates per ISPC dispatch
- Montgomery REDC: `mi = -M^{-1} mod 2^32`, REDCification for base-2
- Binary exponentiation with REDC; multiply-by-base = left-shift-by-1
- Toom-2 (Karatsuba) squaring for sub-quadratic performance
- AVX2 variant: 2 candidates per instruction

### SIMD Sieve (Stella.cpp)
- AVX2: processes 2 primes × 8 lanes using `__m256i`
- `_mm_cmpgt_epi32` + `_mm_movemask_epi8` for offset detection

### SIMD Presieve (external/mod_1_2_avx2.asm)
- `rie_mod_1s_2p_8times`: 8 modular reductions in parallel using AVX2

### 92 Primorial Offsets
- rieMiner precomputes 92 valid primorial offsets for 8-tuple
- Each offset gives independent candidate stream from same primorial
- primerug uses 1 offset — this is up to 92× search space multiplier at near-zero cost

---

## Benchmark Results (2026-03-19, AMD Ryzen 9 5950X 32T, RTX 3080, 62 GB RAM)

### 500-digit 8-tuple, SI=16, SieveBits=25

#### rieMiner (T=32 total, SW=4)

| PTL | c/s | ratio | blocks/day |
|-----|-----|-------|------------|
| 268M | 15,875 | 33.50 | 0.000864 |
| 536M | 14,758 | 32.33 | 0.001069 |
| 1B | 14,062 | 31.28 | 0.001326 |
| **4B** | **11,821** | **29.26** | **0.001901** |

#### primerug CPU-only (T=32, SW=4)

| PTL | c/s | ratio | blocks/day |
|-----|-----|-------|------------|
| 268M | 11,479 | 34.42 | 0.000503 |
| 536M | 10,400 | 33.72 | 0.000538 |
| 1B | 9,616 | 32.77 | 0.000625 |
| **4B** | **~5,900** | **30.34** | **~0.000710** |

#### primerug + RTX 3080 GPU

| PTL | SW | Test Workers | c/s | ratio | blocks/day |
|-----|----|----|-----|-------|------------|
| 268M | 4 | 1 GPU + 27 CPU | 41,117 | 35.55 | 0.001405 |
| 1B | 4 | 1 GPU + 27 CPU | 17,500 | 33.0 | 0.001075 |
| **4B** | **20** | **1 GPU + 11 CPU** | **15,807** | **30.34** | **0.001902** |

### Key Finding: primerug+GPU MATCHES rieMiner at PTL=4B

With 20 sieve workers feeding deep-sieved candidates (ratio ~30) to GPU + 11 CPU test workers:
- **0.001902 blocks/day vs rieMiner's 0.001901** — parity achieved!
- Memory: 20 × 2.5 GB + 2 GB shared ≈ 52 GB (fits in 62 GB)
- c/s still ramping at measurement end (12K→15.8K), steady-state likely higher

### Why GPU Changes the Optimal Operating Point

**Without GPU:** deep sieve (PTL=4B) is best because ratio^8 dominates, and Fermat is the
bottleneck regardless. But sieve-worker count is limited by memory.

**With GPU:** GPU replaces CPU Fermat workers, freeing CPU cores for sieve workers. The
optimal config shifts to MORE sieve workers + GPU:
- PTL=4B SW=20: GPU + 11 CPU test workers → 15.8K c/s, ratio 30 → **0.001902 bpd**
- PTL=268M SW=4: GPU + 27 CPU test workers → 41K c/s, ratio 35.5 → 0.001405 bpd
- PTL=4B SW=4: only 4 sieve workers → GPU starves, ~5.9K c/s → 0.000777 bpd

**The GPU doesn't help by being fast — it helps by freeing CPU for sieve workers.**

### Bottleneck Analysis at PTL=4B SW=20 GPU

1. **Sieve throughput**: 20 workers each presieve 203M primes (~15-20s per target) then
   produce 16 sieve iterations. Total: ~15-30K candidates/s (still ramping at measurement)
2. **GPU compute**: CGBN Miller-Rabin on 32K candidates per batch, ~100ms per batch.
   Theoretical GPU throughput: ~200K c/s. **GPU is NOT the bottleneck.**
3. **CPU-side serialization**: test_worker_loop_gpu reconstructs candidates
   (Integer multiply + add + to_digits + clone) before sending to GPU. ~2.5μs per candidate.
   At 32K batch: ~80ms CPU overhead per batch.
4. **Presieve warmup**: `compute_factor_offsets` for 203M primes is scalar GMP mod_u.
   Takes ~15s per target per sieve worker. First batches delayed.

### How to Run Benchmarks

**rieMiner:**
```bash
cat > /tmp/riebench.conf <<EOF
Mode = Benchmark
Difficulty = 1661
ConstellationPattern = 0, 2, 4, 2, 4, 6, 2, 6
Threads = 32
BenchmarkTimeLimit = 60
RefreshInterval = 60
PrimeTableLimit = 4294967296
SieveIterations = 16
SieveBits = 25
SieveWorkers = 4
LogDebug = No
EOF
cd rieMiner && ./rieMiner /tmp/riebench.conf
```

**primerug CPU-only:**
```bash
primerug -d 500 -p "0, 2, 6, 8, 12, 18, 20, 26" \
  -t 32 --sieve-workers 4 -l 4294967296 -i 16 -s 60
```

**primerug + GPU (optimal config):**
```bash
# Terminal 1: start GPU service
primerug-gpu /tmp/primerug-gpu.sock 32768 64

# Terminal 2: run with 20 sieve workers
primerug -d 500 -p "0, 2, 6, 8, 12, 18, 20, 26" \
  -t 32 --sieve-workers 20 -l 4294967296 -i 16 -s 60 \
  --gpu-socket /tmp/primerug-gpu.sock
```

**Computing blocks/day from primerug stats:**
```
blocks/day = 86400 × c/s / ratio^pattern_len
```

**Pattern formats:**
- rieMiner: DIFFERENTIAL `0, 2, 4, 2, 4, 6, 2, 6`
- primerug: ABSOLUTE `0, 2, 6, 8, 12, 18, 20, 26`

**Difficulty 1661 ≈ 500 decimal digits (1661-bit numbers).**

---

## Remaining Gaps vs rieMiner (ordered by impact)

### Gap 1: Presieve Speed (limits sieve throughput at deep PTL)
- `compute_factor_offsets` does scalar GMP `mod_u` for each of 203M primes
- Takes ~15-20s per target per sieve worker
- rieMiner uses AVX2 assembly (`rie_mod_1s_2p_8times`): 8 reductions in parallel
- **Impact:** faster presieve → more candidates/s → higher c/s at deep sieve
- **Estimated gain:** 4-8x faster presieve → sieve workers produce 2-3x more candidates

### Gap 2: Ratio (30.34 vs 29.26 — 1.34x at 8th power)
- primerug's ratio is consistently ~1 point higher than rieMiner at same PTL
- Likely cause: rieMiner uses 92 primorial offsets, primerug uses 1
- More offsets = more independent candidates per presieve = better statistical sampling
- **Impact:** (30.34/29.26)^8 = 1.34x blocks/day penalty
- **Fix:** implement multiple primorial offsets (near-zero compute cost)

### Gap 3: GPU Candidate Serialization Overhead
- test_worker_loop_gpu clones Integer + to_digits for every candidate
- ~2.5μs per candidate, 80ms for a 32K batch
- This is comparable to GPU compute time (~100ms), halving effective GPU throughput
- **Fix:** compute primorial×f + first_candidate directly in GPU limb format,
  avoiding Integer intermediary. Or: send (f, first_candidate_limbs) to GPU
  and have the GPU reconstruct candidates.

### Gap 4: CPU Fermat Speed (affects CPU test workers)
- GMP `pow_mod_mut`: ~1.67ms per Fermat test (500-digit)
- rieMiner ISPC SIMD: ~0.85ms per test (2x faster)
- Montgomery REDC + Karatsuba squaring + SIMD parallelism
- **Impact:** CPU test workers are 2x slower per candidate
- **Fix:** implement Montgomery REDC in Rust, optionally with SIMD

### Gap 5: Sieve SIMD (minor — <2% of runtime)
- rieMiner: AVX2 sieve marking (2 primes × 8 lanes)
- primerug: scalar with prefetch cache
- **Impact:** marginal, sieve is not the bottleneck
