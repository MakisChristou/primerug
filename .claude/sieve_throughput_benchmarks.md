# Sieve Throughput Benchmarks (2026-03-19)

Hardware: AMD Ryzen 9 5950X (16C/32T, 2x32MB L3), 62 GB RAM
500-digit 8-tuple, PTL=4B, SI=16

## Apples-to-Apples: Both tools with Fermat testing DISABLED

### rieMiner (SIEVE_BENCHMARK build, 92 offsets, T=32)

| SW | SB=23 | SB=24 | SB=25 |
|----|-------|-------|-------|
| 4 | — | 19,009 | 18,136 |
| 8 | — | 24,427 | 19,981 |
| 9 | 22,936 | 25,327 | — |
| 12 | 23,766 | **27,820** | 24,299 |
| 16 | — | 26,333 | 18,503 |

**Best: SW=12 SB=24 → 27,820 c/s**

rieMiner with fewer threads (T=13 SW=12 SB=24): 24,077 c/s (13% slower — needs extra
threads to drain internal task queue even in sieve-benchmark mode).

### primerug (--sieve-benchmark, T=32)

| SW | SB=24 OFF=16 | SB=24 OFF=92 | SB=25 OFF=16 | SB=25 OFF=92 |
|----|-------------|-------------|-------------|-------------|
| 4 | — | — | 12,300 | 12,146 |
| 8 | 17,259 | 17,382 | 19,104 | 19,255 |
| 12 | 23,692 | 23,753 | 24,915 | **24,981** |
| 16 | 24,595 | 24,594 | 20,412 | 21,596 |

**Best: SW=12 SB=25 OFF=92 → 24,981 c/s**

## Summary

| Tool | Best config | c/s | Relative |
|------|------------|-----|----------|
| **rieMiner** | SW=12 SB=24 T=32 | **27,820** | **1.00x** |
| primerug | SW=12 SB=25 OFF=92 | 24,981 | 0.90x |
| primerug | SW=16 SB=24 OFF=92 | 24,594 | 0.88x |

**primerug's sieve is at 90% of rieMiner's.** The 10% gap is:
- AVX2 presieve (rie_mod_1s_2p_8times: 8 mod reductions in parallel)
- AVX2 sieve marking (SIMD bit manipulation)

## Why SB=24 vs SB=25 matters differently for each tool

- SB=25 (33M sieve = 4MB): more candidates per iteration → better presieve amortization
- SB=24 (16M sieve = 2MB): fits L2 cache → less L3 pressure at high SW

rieMiner: SB=24 always wins because its AVX2 sieve exploits the smaller cache-friendly buffer.
primerug: SB=25 wins at SW≤12 (amortization dominates), SB=24 wins at SW=16 (cache dominates).

## Why throughput drops above SW=12

Both tools peak at SW=12 then decline. The bottleneck is **memory bandwidth**:
- Dense factor array: 8 × 28M × 4 = 896 MB per worker (random-ish access during marking)
- 12 workers × 896 MB = 10.7 GB of factor data competing for 64 MB L3 cache
- 16 workers saturates DRAM bandwidth (~50 GB/s on dual-channel DDR4)

## Can we hit 50K on one machine?

**No.** The memory bandwidth ceiling on the 5950X is ~28K c/s. To reach 50K:
- 2 sieve machines (2 × 25K) feeding 1 GPU over TCP
- Or a CPU with more L3/bandwidth (EPYC with 256MB L3, 8-channel DDR4)
