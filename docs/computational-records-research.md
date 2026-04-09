# Computational Record-Breaking: A Survey

*Last updated: April 2026*

A comprehensive overview of computational activities where individuals can compete for world records, with a focus on what's achievable on consumer/prosumer hardware.

## Table of Contents

- [Prime k-Tuple Searching](#prime-k-tuple-searching)
- [Mathematical Constant Computation (y-cruncher)](#mathematical-constant-computation-y-cruncher)
- [Mersenne Primes (GIMPS)](#mersenne-primes-gimps)
- [PrimeGrid](#primegrid)
- [Integer Factorization](#integer-factorization)
- [Other Activities](#other-activities)
- [Hardware Reference](#hardware-reference)

---

## Prime k-Tuple Searching

### Overview

A prime k-tuple is a set of k primes fitting a specific pattern (e.g., twin primes are 2-tuples with pattern {0, 2}). Searching for large k-tuples is a computational challenge that scales exponentially with both k and digit count.

The authoritative record tracker is [pzktupel.de](https://pzktupel.de/ktuplets.php), maintained by Norman Luhn (since 2021, previously by Jens Kruse Andersen and Dirk Augustin).

### Current Records (April 2026)

| k | Name | Digits | Date | Finder | Software |
|---|------|--------|------|--------|----------|
| 2 | Twin primes | 388,342 | Sep 2016 | Tom Greer | TwinGen, PrimeGrid, LLR |
| 3 | Triplets | 21,026 | Jan 2026 | Serge Batalov | PolySieve, CM |
| 4 | Quadruplets | 10,132 | Feb 2019 | Peter Kaiser | Primo |
| 5 | Quintuplets | 3,344 | Mar 2022 | Peter Kaiser | OpenPFGW, Primo |
| 6 | Sextuplets | 1,125 | Feb 2026 | Serge Batalov | EMSieve, CM |
| 7 | Septuplets | 1,002 | Jan 2021 | Peter Kaiser | Primo |
| **8** | **Octuplets** | **401** | **Sep 2023** | **Michalis Christou** | **rieMiner 0.93a** |
| 9 | Nonuplets | 314 | Nov 2025 | Riecoin #2468017 | Riecoin mining |
| 10 | 10-tuplets | 282 | Sep 2021 | Riecoin #1579367 | Riecoin mining |
| 11 | 11-tuplets | 108 | Sep 2019 | Kaiser, Stevens | Polysieve, OpenPFGW, Primo |
| 12 | 12-tuplets | 108 | Sep 2019 | Kaiser, Stevens | Polysieve, OpenPFGW, Primo |
| 13 | 13-tuplets | 66 | Oct 2021 | Roger Thompson | - |
| 14 | 14-tuplets | 50 | Feb 2013 | Roger Thompson | - |
| 15 | 15-tuplets | 40 | Jan 2017 | Norman Luhn | - |
| 16 | 16-tuplets | 35 | Nov 2016 | Roger Thompson | - |
| 17 | 17-tuplets | 33 | Feb 2021 | Roger Thompson | - |
| 18 | 18-tuplets | 28 | Mar 2014 | Chermoni & Wroblewski | - |
| 19 | 19-tuplets | 30 | Dec 2018 | Chermoni & Wroblewski | - |
| 20 | 20-tuplets | 31 | May 2021 | Chermoni & Wroblewski | - |

### 8-Tuple Top Entries

1. **401 digits** -- Michalis Christou (Sep 2023, rieMiner 0.93a)
2. 362 digits -- Michalis Christou (Jan 2023, rieMiner)
3. 343 digits -- Riecoin #2473980 (Dec 2025, Riecoin mining)
4. 338 digits -- Riecoin #2473988 (Dec 2025)
5. 334 digits -- Riecoin #2473882 (Dec 2025)
6. 333 digits -- Peter Kaiser (Sep 2022, Primo)

The Riecoin mining network continuously finds 8-tuples in the 333-343 digit range as a byproduct of 7-tuple mining, but dedicated searching is needed to push beyond 400 digits.

### Difficulty Scaling

The Hardy-Littlewood conjecture predicts the count of prime k-tuples below x is proportional to:

```
pi_k(x) ~ G_k * integral(2 to x) dt/(log t)^k
```

At 400 digits, log(10^400) ~ 921. Each additional prime in the tuple multiplies difficulty by roughly this factor. This explains the steep record dropoff:
- 7-tuple at 1,002 digits (algebraic forms + ECPP proving)
- 8-tuple at 401 digits (primorial sieving + Fermat PRP)
- 9-tuple at 314 digits (Riecoin mining network)
- 10-tuple at 282 digits (Riecoin mining)

### Software

| Tool | Language | Description | Speed |
|------|----------|-------------|-------|
| **rieMiner** | C++ | Reference Riecoin miner/constellation searcher. ISPC SIMD Fermat testing, AVX2 sieve. | Fastest |
| **primerug** | Rust | k-tuple finder using rug/GMP. Sieve-worker architecture, GPU support via primerug-gpu. | ~89% of rieMiner (CPU), ~50% faster with GPU |
| **Stella** | Rust | Experimental Rust port of rieMiner by Pttn. Early prototype. | Not benchmarked |
| **EMSieve** | - | By Serge Batalov. Used for 5-tuple and 6-tuple records. | Specialized |
| **PolySieve** | - | By Serge Batalov. Used for 3-tuple records. | Specialized |
| **Primo** | - | ECPP primality proving by Marcel Martin. Used for proven (not just PRP) records. | Proving only |

### Hardware Considerations for k-Tuple Searching

The pipeline is: **Presieve** (compute offsets) -> **Sieve** (mark composites) -> **Test** (Fermat/Miller-Rabin PRP).

| Factor | Impact | Notes |
|--------|--------|-------|
| **Core count** | High | More sieve workers = higher throughput. Linear scaling up to memory bandwidth limit. |
| **Clock speed** | Medium | Helps Fermat testing (single-threaded bignum math). |
| **DRAM bandwidth** | **Critical** | Each sieve worker randomly accesses ~900MB of factor data. 12 workers saturate dual-channel DDR4 (~50 GB/s). This is the primary bottleneck. |
| **L3 cache** | Medium | Sieve array (SieveBits=25 = 32MB) should fit in L3. |
| **SIMD (AVX2/512)** | Medium | rieMiner uses AVX2 for sieve marking and presieve. primerug currently scalar. |
| **GPU** | High | primerug-gpu (CGBN Miller-Rabin) achieves 60K c/s on RTX 3080 -- 2.4x the CPU sieve feed rate. Multiple sieve machines needed to saturate. |
| **RAM quantity** | Low | ~2.5 GB per sieve worker at PTL=4B. 64 GB is plenty. |

### Benchmarks (Ryzen 9 5950X, 32T, 64GB DDR4, RTX 3080)

**500-digit 8-tuple search:**

| Configuration | c/s | ratio | blocks/day | ETA |
|---------------|-----|-------|------------|-----|
| rieMiner (T=32, SW=9, 92 offsets, PTL=4B) | 12,392 | 29.27 | 0.00199 | 503 days |
| primerug CPU (T=32, SW=4, 16 offsets, PTL=4B) | 9,761 | 29.44 | 0.00176 | ~568 days |
| **primerug + RTX 3080** (T=32, SW=12, 16 offsets) | **23,836** | **29.42** | **0.0037** | **273 days** |

### Riecoin

Riecoin is a cryptocurrency (launched 2014) that uses prime constellation discovery as proof-of-work. Miners must find prime 7-tuples to validate blocks. Some 7-tuples turn out to be 8, 9, or 10-tuples, which are scored higher.

The [Constellation Explorer](https://riecoin.dev/Constellations/) tracks all finds. Bounties of 500-5000 RIC exist for record-breaking constellations.

### Active Competitors (2025-2026)

- **Serge Batalov** -- Extremely prolific. Holds triplet (21K digits) and sextuplet (1,125 digits) records using custom sieving + CM proving.
- **Peter Kaiser** -- Holds quadruplet, quintuplet, and septuplet records using Primo.
- **Riecoin mining network** -- Continuously produces 7-10 tuple records as a mining byproduct.
- **Roger Thompson** -- Dominant for k=13-17 range.
- **Chermoni & Wroblewski** -- Dominant for k=18-21 range.

---

## Mathematical Constant Computation (y-cruncher)

### Overview

[y-cruncher](https://www.numberworld.org/y-cruncher/) by Alexander Yee is the only software used for world-record computations of mathematical constants. Every pi record since 2009 has used it. It supports pi, e, sqrt(2), Golden Ratio, Log(2), Zeta(3), Catalan's, Euler-Mascheroni, Gamma functions, and many more.

Current version: v0.8.7 Build 9547 (November 2025).

### Current World Records

| Constant | Digits | Date | Who | Hardware |
|----------|--------|------|-----|----------|
| **Pi** | 314 trillion | Nov 2025 | StorageReview | 2x EPYC 9965, 1.5TB RAM, 2.5PB NVMe |
| **e** | 35 trillion | Dec 2023 | Jordan Ranous | 2x Xeon 8460H, 512 GB |
| **Sqrt(2)** | 28 trillion | Jun 2025 | Teck Por Lim | Xeon Gold 6230, 768 GB |
| **Golden Ratio** | 20 trillion | Nov 2023 | Jordan Ranous | EPYC 9654, 1.5 TB |
| **Sqrt(3)** | 4 trillion | May 2025 | DMAHJEFF | 2x EPYC 7401, 320 GB |
| **Log(2)** | 3.1 trillion | Oct 2025 | Mamdouh Barakat | TR 5965WX, 512 TB storage |
| **Log(10)** | 2 trillion | Jun 2025 | Lorenz Milla | Ryzen 7950X, 128 GB |
| **Lemniscate** | 2 trillion | May 2025 | Lorenz Milla | Ryzen 7950X, 64 GB |
| **Zeta(3)** | ~2 trillion | Dec 2023 | Andrew Sun | 2x Xeon 8347C, 505 GB |
| **Euler-Mascheroni** | 1.337 trillion | Sep 2023 | Andrew Sun | Xeon 8347C, 400 GB |
| **Catalan's** | 1.2 trillion | Mar 2022 | Seungmin Kim | 2x Xeon 6140 |
| **Gamma(1/4)** | 1.2 trillion | Jun 2025 | Dmitriy Grigoryev | Xeon W9-3595X, 2 TB RAM |
| **Gamma(1/3)** | 1.3 trillion | Aug 2025 | Mamdouh Barakat | TR 5965WX |
| **Zeta(5)** | 600 billion | Oct 2025 | Dmitriy Grigoryev | Xeon W7-3465X, 2 TB RAM |
| **Log(3)** | 600 billion | Jun 2025 | Dmitriy Grigoryev | Xeon W9-3595X, 2 TB RAM |
| **Log(5)** | 600 billion | Oct 2025 | Dmitriy Grigoryev | Xeon W7-3465X, 2 TB RAM |
| **Gamma(1/5)** | 270 billion | Oct 2025 | Dmitriy Grigoryev | Xeon W7-3465X, 2 TB RAM |

### Computational Complexity by Constant

| Class | Constants | Complexity | Relative Speed |
|-------|-----------|------------|----------------|
| Simplest | Sqrt(n), Golden Ratio | O(n log n) | Fastest |
| Moderate | e | O(n log(n)^2) | ~10x slower than sqrt |
| Hard | Pi, Log(n), Zeta(3), Catalan's, Gamma | O(n log(n)^3) | ~100x slower than sqrt |
| Hardest | Euler-Mascheroni | O(n log(n)^3) (huge constant) | Slowest of all |

### Hardware Requirements

**Storage is the primary bottleneck**, not CPU.

- ~4.7 TiB of swap space per trillion digits of pi
- RAM acts as a cache in swap mode; more RAM = dramatically less disk I/O
- NVMe SSDs essential for multi-trillion digit computations (enterprise drives recommended for endurance)
- ECC memory strongly recommended (bit flips corrupt months of work)

| Digits | RAM (in-memory) | Swap Space | Approx Time (modern HW) |
|--------|-----------------|------------|--------------------------|
| 1 billion | ~5 GB | N/A | Minutes |
| 10 billion | ~48 GB | N/A | Hours |
| 1 trillion | N/A | ~4.7 TiB | Days-weeks |
| 100 trillion | N/A | ~470 TiB | Months |
| 314 trillion | N/A | ~1.5 PiB | 110 days (best hardware) |

### Best Targets for Consumer Hardware

With a Ryzen 5950X, 64GB RAM, and ~6TB NVMe:
- **Gamma(1/5)** (270 billion) -- lowest record, potentially beatable
- Less-competed constants on y-cruncher's full list
- Simple constants (sqrt-type) can reach trillions of digits

---

## Mersenne Primes (GIMPS)

### Overview

The [Great Internet Mersenne Prime Search](https://www.mersenne.org/) tests numbers of the form 2^p - 1 for primality using PRP/Lucas-Lehmer tests. Software: Prime95/mprime (CPU), GpuOwl/PRPLL (GPU).

### Current Record

**M136279841** = 2^136,279,841 - 1 (41,024,320 digits), discovered October 12, 2024 by Luke Durant using ~$2M of cloud GPU infrastructure.

### Search Status (April 2026)

- All exponents below ~140M tested at least once
- All exponents below ~80M double-checked (verified)
- Probability of any single exponent being prime: ~1 in 300,000

### Prizes

- GIMPS: $3,000 per discovery
- EFF: **$150,000** for 100M-digit prime (unclaimed)
- EFF: **$250,000** for 1B-digit prime (unclaimed)

### Hardware and Feasibility

A PRP test at the wavefront (~140M exponents) takes ~1 month per core on a 5950X. Double-check work (exponents around 80M) takes days. With 16 cores, you could test 6-8 double-check exponents per month.

The chance of finding a prime is tiny per test, but non-zero. Several Mersenne primes have been discovered during verification runs.

---

## PrimeGrid

### Overview

[PrimeGrid](https://www.primegrid.com/) is a BOINC-based distributed computing project running ~15 prime search subprojects with competitive leaderboards and challenge events.

### Active Subprojects

| Subproject | Form | Current Range | Notable |
|------------|------|---------------|---------|
| 321 Prime Search | 3*2^n +/- 1 | ~7.3M digits | Record found on Ryzen 7900X |
| Cullen Primes | n*2^n + 1 | ~8.7M digits | |
| Woodall Primes | n*2^n - 1 | ~8.0M digits | |
| Seventeen or Bust | k*2^n + 1 | ~14.2M digits | 5 candidates remaining |
| Factorial Primes | n! +/- 1 | ~2.9M digits | |
| Primorial Primes | p# +/- 1 | ~4.4M digits | |
| **GFN-23** | **b^8388608 + 1** | **~43.6M digits** | **Would be the largest known prime** |

### GFN-23: The World Record Hunt

After M52 was found (Dec 2024), PrimeGrid began searching GFN-23 (generalized Fermat numbers b^2^23 + 1). Any prime found here would surpass the 41M-digit Mersenne record as the **largest known prime**. This search accepts CPU contributions.

### Feasibility

A Ryzen 5950X is competitive -- the 321 prime record was found on a comparable Ryzen 9 7900X. With 16 cores running 24/7, you'd test ~60+ candidates/day on LLR subprojects. This is one of the best opportunities for individual record-setting.

---

## Integer Factorization

### RSA Factoring

**RSA-250** (829 bits) was factored in Feb 2020 using CADO-NFS (~2,700 core-years). **RSA-260** (862 bits) remains unfactored. Individual contribution is limited to sieving work via NFS@Home (BOINC).

### Cunningham Project

Factoring numbers of the form b^n +/- 1 (ongoing since 1925). Individuals contribute via ECM (Elliptic Curve Method) using GMP-ECM or Yoyo@home (BOINC), hunting for 40-70 digit factors.

### Aliquot Sequences

Sum of proper divisors, iterated. The "Lehmer Five" (276, 552, 564, 660, 966) have thousands of terms with values exceeding 200 digits, unresolved. Extending them requires factoring 130-160 digit composites -- right in the sweet spot for a 16-core machine with 64GB RAM.

Tools: YAFU, CADO-NFS, YAFU@home (BOINC), FactorDB.

### Feasibility

| Number Size | Time on 5950X | Method |
|-------------|---------------|--------|
| < 100 digits | Minutes | QS |
| 100-120 digits | Hours-day | QS/NFS |
| 120-140 digits | Days-weeks | NFS |
| 140-160 digits | Weeks-months | NFS |

---

## Other Activities

### Collatz Conjecture Verification

Verified up to 2^71 (Jan 2025) by David Barina using GPU supercomputers. Not accessible to individuals -- dominated by massive GPU clusters.

### Goldbach Conjecture Verification

Verified up to 4*10^18 + 7*10^13 (Apr 2025). Custom sieve-based code (Gridbach). Possible to contribute but not a standard distributed project.

### Riemann Hypothesis: Zeta Zeros

Verified up to height 3*10^12 (Platt & Trudgian, 2021). Specialized academic work, no volunteer project.

### OEIS Sequence Extension

[oeis.org](https://oeis.org/) has 390,000+ sequences, many where the next term is a computational challenge. Individual, credit-based. Clever algorithms often matter more than raw hardware.

### Chess Endgame Tablebases

7-piece complete (Syzygy, ~140TB). 8-piece in progress but needs petabytes -- not feasible for individuals.

### Busy Beaver

BB(5) = 47,176,870, proved in 2024 via Coq (bbchallenge.org). BB(6) is theoretically intractable.

### Cryptographic Challenges

Bitcoin puzzle transaction: 79/160 solved. GPU-dominated (Kangaroo/BSGS algorithms). Not CPU-competitive.

---

## Hardware Reference

### Test Machine: AMD Ryzen 9 5950X

| Component | Specs |
|-----------|-------|
| CPU | AMD Ryzen 9 5950X (16C/32T, Zen 3, up to 5.09 GHz) |
| GPU | NVIDIA RTX 3080 |
| RAM | 64 GB DDR4 (dual-channel, ~50 GB/s bandwidth) |
| NVMe #1 | Fanxiang S501Q 4TB (boot, ~2.4 TB free) |
| NVMe #2 | Kingston KC3000 4TB |
| HDD | Seagate Exos 18TB |
| Total fast storage | ~6 TB NVMe + 18 TB slow HDD |

### Feasibility Summary

| Activity | Realistic? | Time Investment | Record Potential |
|----------|-----------|-----------------|------------------|
| **8-tuple records (primerug/rieMiner)** | Yes -- current record holder | Months | Already #1 at 401 digits |
| **PrimeGrid (LLR subprojects)** | Yes | Days per test | Could find record-holding prime |
| **PrimeGrid GFN-23** | Yes | Weeks per test | Could find largest known prime |
| **Aliquot sequences** | Yes | Days-weeks per extension | Meaningful frontier contribution |
| **GIMPS double-checking** | Yes | Days per test | Small chance of Mersenne prime |
| **Cunningham ECM** | Yes | Ongoing | Factor discoveries |
| **y-cruncher (minor constants)** | Maybe | Weeks-months | Gamma(1/5) potentially beatable |
| **OEIS extensions** | Yes | Varies | Credit for new terms |
| **y-cruncher (pi record)** | No | N/A | Needs petabytes of NVMe |
| **RSA-260** | No | N/A | Needs thousands of core-years |
| **Collatz frontier** | No | N/A | Needs GPU supercomputers |

---

## Key Resources

- **k-tuple records**: [pzktupel.de/ktuplets.php](https://pzktupel.de/ktuplets.php)
- **y-cruncher**: [numberworld.org/y-cruncher](https://www.numberworld.org/y-cruncher/)
- **GIMPS**: [mersenne.org](https://www.mersenne.org/)
- **PrimeGrid**: [primegrid.com](https://www.primegrid.com/)
- **Riecoin**: [riecoin.dev](https://riecoin.dev/)
- **Riecoin mining algorithm**: [riecoin.dev/en/Mining_Algorithm](https://riecoin.dev/en/Mining_Algorithm)
- **Prime database**: [t5k.org](https://t5k.org/)
- **FactorDB**: [factordb.com](http://factordb.com/)
- **BOINC**: [boinc.berkeley.edu](https://boinc.berkeley.edu/)
- **Cunningham Project**: [homes.cerias.purdue.edu/~ssw/cun/](https://homes.cerias.purdue.edu/~ssw/cun/)
- **Aliquot sequences**: [rechenkraft.net/aliquot/](http://rechenkraft.net/aliquot/)
- **primerug writeup**: [makischristou.gitbook.io/primes](https://makischristou.gitbook.io/primes/)
