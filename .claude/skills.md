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
- Each sieve worker iterates all offsets per target
- First offset: full GMP presieve. Subsequent: fast adjustment from stored remainders
- primerug now also supports `--primorial-offsets N` (default 1)

---

## Benchmark Results (2026-03-19, AMD Ryzen 9 5950X 32T, RTX 3080, 62 GB RAM)

### 500-digit 8-tuple

#### rieMiner auto-tuned (T=32, SW=9, 92 offsets, PTL=4B auto, SieveBits=24)

| Metric | Value |
|--------|-------|
| c/s | 12,392 |
| ratio | 29.27 |
| blocks/day | **0.001987** |
| **ETA for 8-tuple** | **503 days (1.4 years)** |

#### primerug CPU-only (T=32, SW=4, PTL=4B)

| Offsets | SI | c/s | ratio | blocks/day | vs rieMiner |
|---------|------|-----|-------|------------|-------------|
| 1 | 16 | 5,000 | 30.34 | 0.000710 | 36% |
| **16** | **16** | **9,761** | **29.44** | **0.001764** | **89%** |
| 16 | 32 | 9,688 | 29.32 | 0.001797 | 90% |

Multi-offsets nearly double c/s (amortized presieve) and ratio now matches rieMiner (~29.3).

#### primerug + RTX 3080 GPU (best measured config)

| Config | c/s | ratio | blocks/day | ETA |
|--------|-----|-------|------------|-----|
| SW=12 T=32 16-offsets PTL=4B | **23,836** | **29.42** | **0.0037** | **273 days** |

**GPU peak throughput: 60,000 c/s** (measured with synthetic benchmark).
Not 328K as originally estimated — CGBN Miller-Rabin at 2048 bits costs ~0.5s per 32K batch.

| Batch size | ms/batch | c/s |
|-----------|---------|-----|
| 4,096 | 97 ms | 42,131 |
| 8,192 | 159 ms | 51,469 |
| 32,768 | 546 ms | 59,989 |

GPU is at ~40% utilization with 1 sieve machine (25K feed vs 60K capacity).
2-3 sieve machines would saturate it.

### Key Findings

**Multi-offsets are the biggest win:**
- 16 offsets per target: 1.95× c/s improvement
- Ratio drops from 30.34 → 29.44 (broader residue class coverage)
- Fast adjustment avoids GMP mod_u — uses stored remainders + scalar delta

**Optimal config: SW=12 T=32 with GPU.**
- 12 sieve workers + 1 GPU worker + 19 CPU Fermat workers
- Tested SW=17 T=18 (pure GPU, no CPU Fermat): slightly worse (22.5K vs 23.8K)
- The 19 CPU workers add ~3-4K c/s and the sieve can't use those cores anyway

**Bottleneck is DRAM bandwidth, not RAM capacity or GPU:**
1. **Sieve peaks at SW=12** then drops. Each worker accesses ~900MB of factor data
   in random patterns. 12 workers saturate ~50 GB/s dual-channel DDR4.
2. **More RAM does NOT help.** Tested: SW=16 with enough RAM still drops 20%.
3. **GPU at 60K c/s is only 40% utilized** with 1 sieve machine producing 25K c/s.
4. **To go faster:** add sieve machines (horizontal scaling), not RAM or GPU.

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

**primerug CPU-only (best known config):**
```bash
primerug -d 500 -p "0, 2, 6, 8, 12, 18, 20, 26" \
  -t 32 --sieve-workers 4 -l 4294967296 -i 16 -s 60 \
  --primorial-offsets 16
```

**primerug + GPU (best known config):**
```bash
# Terminal 1: start GPU service
primerug-gpu /tmp/primerug-gpu.sock 32768 64

# Terminal 2: run with many sieve workers + multi-offsets
primerug -d 500 -p "0, 2, 6, 8, 12, 18, 20, 26" \
  -t 32 --sieve-workers 12 -l 4294967296 -i 16 -s 60 \
  --primorial-offsets 16 --gpu-socket /tmp/primerug-gpu.sock
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

### Gap 1: CPU Fermat Speed (21% c/s gap)
- GMP `pow_mod_mut`: ~1.67ms per Fermat test (500-digit)
- rieMiner ISPC SIMD: ~0.85ms per test (2x faster)
- Montgomery REDC + Karatsuba squaring + SIMD parallelism
- **Impact:** 9,761 c/s vs 12,392 c/s — the entire remaining CPU gap
- **Fix:** implement Montgomery REDC in Rust, optionally with SIMD

### Gap 2: GPU Candidate Serialization Overhead
- test_worker_loop_gpu clones Integer + to_digits for every candidate
- ~2.5μs per candidate, 80ms for a 32K batch
- This is comparable to GPU compute time (~100ms), halving effective GPU throughput
- **Fix:** send (f_values, first_candidate_limbs) to GPU and reconstruct there

### Gap 3: Presieve Speed (partially mitigated by multi-offsets)
- Full presieve: scalar GMP `mod_u` for 203M primes, ~15s per target
- Fast adjustment: ~4s per additional offset (no GMP)
- rieMiner uses AVX2 assembly: 8 reductions in parallel
- **Impact:** with 16 offsets, presieve is amortized but adjustment still ~4s each
- **Estimated gain from AVX2:** 2-4x faster adjustment → more offsets practical

### ~~Gap 4: Ratio~~ CLOSED
- Was 30.34 vs 29.26 — now 29.44 vs 29.27 with multi-offsets
- Remaining 0.17 difference is noise / different primorial number (p194# vs p185#)

### Gap 5: Sieve SIMD (minor — <2% of runtime)
- rieMiner: AVX2 sieve marking (2 primes × 8 lanes)
- primerug: scalar with prefetch cache
- **Impact:** marginal, sieve is not the bottleneck
