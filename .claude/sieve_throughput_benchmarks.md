# Sieve Throughput Benchmarks (2026-03-19)

Hardware: AMD Ryzen 9 5950X (32T), 62 GB RAM
500-digit 8-tuple, PTL=4B, SI=16

## rieMiner (c/s is Fermat-limited, not pure sieve throughput)

rieMiner's c/s reflects the combined sieve+Fermat pipeline. More sieve workers steal
cores from the fast ISPC Fermat test, so c/s drops above SW=8.

| SW | SB=23 c/s | SB=23 bpd | SB=24 c/s | SB=24 bpd | SB=25 c/s | SB=25 bpd |
|----|-----------|-----------|-----------|-----------|-----------|-----------|
| 4  | 10,762 | 0.001679 | 11,109 | 0.001741 | 11,411 | 0.001831 |
| 8  | 11,148 | 0.001759 | 11,146 | 0.001727 | 10,720 | 0.001852 |
| 12 | 10,458 | 0.001707 | 10,395 | 0.001744 | 9,160 | 0.001646 |
| 16 | 9,480 | 0.001593 | 8,498 | 0.001546 | 7,180 | 0.001308 |
| 24 | 6,727 | 0.001184 | 4,012 | 0.000664 | OOM | — |

**Best rieMiner:** SW=8 SB=25 → 0.001852 bpd (10,720 c/s, ratio 29.00)
**Auto-tuned (120s):** SW=9 SB=24 → 0.001987 bpd (12,392 c/s, ratio 29.27)

Auto-tuned is better because 120s run has more warmup. The 60s runs are init-penalized.

## primerug sieve-only (--sieve-benchmark, no Fermat)

Pure sieve candidate production rate. Test workers just drain the queue and count.

| SW | 1 offset c/s | 16 offsets c/s |
|----|-------------|----------------|
| 4  | 4,658 | 11,806 |
| 8  | 11,396 | 18,589 |
| 16 | 15,855 | 20,653 |
| 28 | OOM | OOM |

**Best measured:** SW=16 OFF=16 → 20,653 c/s (1.81x rieMiner's max throughput)

## Analysis

- rieMiner peaks at SW=4-8 due to Fermat being the bottleneck
- primerug's sieve scales well to SW=16 with multi-offsets
- Multi-offsets give ~2x throughput (amortized presieve cost)
- SW=28 OOMs at PTL=4B (28 × 3.3 GB > 62 GB RAM)
- Max practical SW at PTL=4B on 62 GB: ~18 workers

## Implication for GPU mode

GPU can test ~200-300K c/s. primerug's sieve produces ~20K c/s from one machine.
To saturate GPU:
- 1 machine: 20K c/s → GPU at 6-10%
- 4 machines: 80K c/s → GPU at 24-40%
- 15 machines: 300K c/s → GPU saturated

With GPU consuming 20K c/s at ratio ~29.4:
blocks/day = 86400 × 20,653 / 29.4^8 = 0.003267 (1.64x rieMiner)
